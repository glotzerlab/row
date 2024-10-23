// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use log::{debug, error, trace};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use regex::{Captures, Regex};
use serde_json::Value;
use shell_quote::{Quote, QuoteExt};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Write as _;
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::cluster::Cluster;
use crate::launcher::Launcher;
use crate::scheduler::{ActiveJobs, Scheduler};
use crate::workflow::{Action, Processes};
use crate::Error;

/// `BashScriptBuilder` builds `bash` scripts that execute row actions.
pub(crate) struct BashScriptBuilder<'a> {
    walltime_in_minutes: i64,
    total_processes: usize,
    cluster_name: &'a str,
    action: &'a Action,
    directories: &'a [PathBuf],
    workspace_path: &'a Path,
    directory_values: &'a HashMap<PathBuf, Value>,
    preamble: &'a str,
    launchers: &'a HashMap<String, Launcher>,
}

impl<'a> BashScriptBuilder<'a> {
    /// Construct a new bash script builder.
    pub(crate) fn new(
        cluster_name: &'a str,
        action: &'a Action,
        directories: &'a [PathBuf],
        workspace_path: &'a Path,
        directory_values: &'a HashMap<PathBuf, Value>,
        launchers: &'a HashMap<String, Launcher>,
    ) -> Self {
        let walltime_in_minutes = action
            .resources
            .total_walltime(directories.len())
            .signed_total_seconds()
            / 60;

        BashScriptBuilder {
            total_processes: action.resources.total_processes(directories.len()),
            walltime_in_minutes,
            cluster_name,
            action,
            directories,
            workspace_path,
            directory_values,
            preamble: "",
            launchers,
        }
    }

    /// Add a preamble.
    pub(crate) fn with_preamble(mut self, preamble: &'a str) -> Self {
        self.preamble = preamble;
        self
    }

    /// Create the bash script header.
    fn header(&self) -> String {
        let mut result = "#!/bin/bash\n".to_string();

        result.push_str(self.preamble);
        result.push('\n');

        result
    }

    /// Define the action's variables.
    fn variables(&self) -> Result<String, Error> {
        let mut result = "directories=(\n".to_string();
        for directory in self.directories {
            result.push_quoted(
                shell_quote::Bash,
                directory
                    .to_str()
                    .ok_or_else(|| Error::NonUTF8DirectoryName(directory.clone()))?,
            );
            result.push('\n');
        }
        result.push_str(")\n");

        let _ = write!(
            result,
            r#"
export ACTION_WORKSPACE_PATH={}
export ACTION_CLUSTER={}
export ACTION_NAME={}
export ACTION_PROCESSES={}
export ACTION_WALLTIME_IN_MINUTES={}
"#,
            <shell_quote::Bash as Quote<String>>::quote(
                self.workspace_path
                    .to_str()
                    .ok_or_else(|| Error::NonUTF8DirectoryName(self.workspace_path.into()))?
            ),
            <shell_quote::Bash as Quote<String>>::quote(self.cluster_name),
            <shell_quote::Bash as Quote<String>>::quote(self.action.name()),
            self.total_processes,
            self.walltime_in_minutes,
        );

        if let Processes::PerDirectory(processes_per_directory) = self.action.resources.processes()
        {
            let _ = writeln!(
                result,
                "export ACTION_PROCESSES_PER_DIRECTORY={processes_per_directory}",
            );
        }

        if let Some(threads_per_process) = self.action.resources.threads_per_process {
            let _ = writeln!(
                result,
                "export ACTION_THREADS_PER_PROCESS={threads_per_process}",
            );
        }

        if let Some(gpus_per_process) = self.action.resources.gpus_per_process {
            let _ = writeln!(result, "export ACTION_GPUS_PER_PROCESS={gpus_per_process}",);
        }

        Ok(result)
    }

    fn setup(&self) -> Result<String, Error> {
        let mut result = String::new();
        let user_setup = self
            .action
            .submit_options
            .get(self.cluster_name)
            .and_then(|c| c.setup.clone())
            .unwrap_or_default();

        if !user_setup.is_empty() {
            result.push('\n');
            result.push_str(&user_setup);
            result.push_str("\n\n");
            result.push_str(
                r#"test $? -eq 0 || { >&2 echo "[row] Error executing setup."; exit 1; }"#,
            );
        }

        let action_name = self.action.name();
        let row_executable = env::current_exe().map_err(Error::FindCurrentExecutable)?;
        let row_executable = row_executable.to_str().expect("UTF-8 path to executable.");
        let _ = write!(
            result,
            r#"
trap 'printf %s\\n "${{directories[@]}}" | {row_executable} scan --no-progress -a {action_name} - || exit 3' EXIT"#
        );

        Ok(result)
    }

    fn execution(&self) -> Result<String, Error> {
        let command = self.action.command();

        let contains_directory = command.contains("{directory}");
        let contains_directories = command.contains("{directories}");
        if contains_directory && contains_directories {
            return Err(Error::ActionContainsMultipleTemplates(
                self.action.name().into(),
            ));
        }
        if contains_directories && self.contains_json_pointer() {
            return Err(Error::DirectoriesUsedWithJSONPointer(
                self.action.name().into(),
            ));
        }

        // Build up launcher prefix
        let mut launcher_prefix = String::new();
        let mut process_launchers = 0;
        for launcher in self.action.launchers() {
            let launcher = self.launchers.get(launcher).ok_or_else(|| {
                Error::LauncherNotFound(launcher.clone(), self.action.name().into())
            })?;
            launcher_prefix
                .push_str(&launcher.prefix(&self.action.resources, self.directories.len()));
            if launcher.processes.is_some() {
                process_launchers += 1;
            }
        }

        if self.total_processes > 1 && process_launchers == 0 {
            return Err(Error::NoProcessLauncher(
                self.action.name().into(),
                self.total_processes,
            ));
        }
        if process_launchers > 1 {
            return Err(Error::TooManyProcessLaunchers(self.action.name().into()));
        }

        if contains_directory {
            if self.contains_json_pointer() {
                // When JSON pointers are present, produce one line per directory.
                let mut result = String::with_capacity(128 * self.directories.len());
                for directory in self.directories {
                    let current_command = self.substitute(command, directory)?;
                    let _ = writeln!(
                        result,
                        r#"
{launcher_prefix}{current_command} || {{ >&2 echo "[ERROR row::action] Error executing command."; exit 2; }}
"#
                    );
                }

                Ok(result)
            } else {
                // When there are no JSON pointers, use a compact for loop.
                let command = command.replace("{directory}", "$directory");
                let command = self.substitute(&command, Path::new(""))?;
                Ok(format!(
                    r#"
for directory in "${{directories[@]}}"
do
    {launcher_prefix}{command} || {{ >&2 echo "[ERROR row::action] Error executing command."; exit 2; }}
done
"#
                ))
            }
        } else if contains_directories {
            // {directories} is compatible with {workspace_path}, but not {/JSON pointer}
            let command = command.replace("{directories}", r#""${directories[@]}""#);
            let command = command.replace(
                "{workspace_path}",
                &<shell_quote::Bash as Quote<String>>::quote(
                    self.workspace_path
                        .to_str()
                        .ok_or_else(|| Error::NonUTF8DirectoryName(self.workspace_path.into()))?,
                ),
            );
            Ok(format!(
                r#"
{launcher_prefix}{command} || {{ >&2 echo "[row] Error executing command."; exit 1; }}
"#
            ))
        } else {
            Err(Error::ActionContainsNoTemplate(self.action.name().into()))
        }
    }

    pub(crate) fn build(&self) -> Result<String, Error> {
        Ok(self.header() + &self.variables()? + &self.setup()? + &self.execution()?)
    }

    /// Check if the command uses JSON pointers.
    fn contains_json_pointer(&self) -> bool {
        self.action.command().contains("{}") || self.action.command().contains("{/")
    }

    /** Substitute all template strings in a given command.

    Substitutes `{workspace_path}` with the value of `workspace_path`.
    Substitutes `{\JSON pointer}` with the value of the JSON pointer for the given directory.

    # Errors

    * `Err(row::JSONPointerNotFound)` when a JSON pointer named in `command` is not present
      in the values for the given directory.
    * `Err(row::InvalidTemplate)` when an unexpected name appears between `{` and `}`.
    */
    fn substitute(&self, command: &str, directory: &Path) -> Result<String, Error> {
        let replacement = |caps: &Captures| -> Result<String, Error> {
            match &caps[0] {
                "{workspace_path}" => Ok(shell_quote::Bash::quote(
                    self.workspace_path
                        .to_str()
                        .ok_or_else(|| Error::NonUTF8DirectoryName(self.workspace_path.into()))?,
                )),
                "{directory}" => {
                    Ok(shell_quote::Bash::quote(directory.to_str().ok_or_else(
                        || Error::NonUTF8DirectoryName(self.workspace_path.into()),
                    )?))
                }
                template if template.starts_with("{/") || template == "{}" => {
                    let pointer = caps[1].into();
                    let value = self
                        .directory_values
                        .get(directory)
                        .ok_or_else(|| Error::DirectoryNotFound(directory.into()))?
                        .pointer(pointer)
                        .ok_or_else(|| {
                            Error::JSONPointerNotFound(directory.into(), pointer.to_string())
                        })?;

                    match value {
                        // Value::to_string puts extra double quotes around JSON strings,
                        // extract the string itself.
                        Value::String(s) => Ok(<shell_quote::Bash as Quote<String>>::quote(s)),
                        _ => Ok(shell_quote::Bash::quote(&value.to_string())),
                    }
                }
                _ => Err(Error::InvalidTemplate(
                    self.action.name().into(),
                    caps[0].into(),
                )),
            }
        };

        let regex = Regex::new(r"\{([^\}]*)\}").expect("valid regular expression");
        replace_all(&regex, command, replacement)
    }
}

/// The `Bash` scheduler constructs bash scripts and executes them with `bash`.
pub struct Bash {
    cluster: Cluster,
    launchers: HashMap<String, Launcher>,
}

impl Bash {
    /// Construct a new Bash scheduler.
    pub fn new(cluster: Cluster, launchers: HashMap<String, Launcher>) -> Self {
        Self { cluster, launchers }
    }
}

pub struct ActiveBashJobs {}

impl Scheduler for Bash {
    fn make_script(
        &self,
        action: &Action,
        directories: &[PathBuf],
        workspace_path: &Path,
        directory_values: &HashMap<PathBuf, Value>,
    ) -> Result<String, Error> {
        BashScriptBuilder::new(
            &self.cluster.name,
            action,
            directories,
            workspace_path,
            directory_values,
            &self.launchers,
        )
        .build()
    }

    fn submit(
        &self,
        workflow_root: &Path,
        action: &Action,
        directories: &[PathBuf],
        workspace_path: &Path,
        directory_values: &HashMap<PathBuf, Value>,
        should_terminate: Arc<AtomicBool>,
    ) -> Result<Option<u32>, Error> {
        debug!("Executing '{}' in bash.", action.name());
        let script = self.make_script(action, directories, workspace_path, directory_values)?;

        let mut child = Command::new("bash")
            .stdin(Stdio::piped())
            .current_dir(workflow_root)
            .spawn()
            .map_err(|e| Error::SpawnProcess("bash".into(), e))?;

        let mut stdin = child.stdin.take().expect("Piped stdin");
        write!(stdin, "{script}")?;
        drop(stdin);

        trace!("Waiting for bash to complete.");
        let status = loop {
            if should_terminate.load(Ordering::Relaxed) {
                error!("Interrupted! Stopping the current execution and cleanly exiting.");
                signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGINT)?;
                break child
                    .wait()
                    .map_err(|e| Error::SpawnProcess("bash".into(), e))?;
            }

            thread::sleep(Duration::from_millis(1));

            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) => continue,
                Err(e) => return Err(Error::SpawnProcess("bash".into(), e)),
            }
        };

        if !status.success() {
            let message = match status.code() {
                None => match status.signal() {
                    None => "terminated by a unknown signal".to_string(),
                    Some(signal) => format!("terminated by signal {signal}"),
                },
                Some(code) => format!("exited with code {code}"),
            };
            return Err(Error::ExecuteAction(action.name().into(), message));
        }

        Ok(None)
    }

    /// Bash reports no active jobs.
    ///
    /// All jobs are executed immediately on submission.
    ///
    fn active_jobs(&self, _: &[u32]) -> Result<Box<dyn ActiveJobs>, Error> {
        Ok(Box::new(ActiveBashJobs {}))
    }
}

impl ActiveJobs for ActiveBashJobs {
    fn get(self: Box<Self>) -> Result<HashSet<u32>, Error> {
        Ok(HashSet::new())
    }
}

/** Fallible `replace_all`.

From [the regex documentation].

[the regex documentation]: https://docs.rs/regex/latest/regex/struct.Regex.html#fallibility
*/
fn replace_all<E>(
    re: &Regex,
    haystack: &str,
    replacement: impl Fn(&Captures) -> Result<String, E>,
) -> Result<String, E> {
    let mut new = String::with_capacity(haystack.len());
    let mut last_match = 0;
    for caps in re.captures_iter(haystack) {
        let m = caps.get(0).unwrap();
        new.push_str(&haystack[last_match..m.start()]);
        new.push_str(&replacement(&caps)?);
        last_match = m.end();
    }
    new.push_str(&haystack[last_match..]);
    Ok(new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use serial_test::parallel;
    use speedate::Duration;

    use crate::builtin::BuiltIn;
    use crate::cluster::{IdentificationMethod, SchedulerType};
    use crate::launcher;
    use crate::workflow::Walltime;
    use crate::workflow::{Resources, SubmitOptions};

    fn setup() -> (Action, Vec<PathBuf>, HashMap<String, Launcher>) {
        let resources = Resources {
            processes: Some(Processes::PerDirectory(2)),
            threads_per_process: Some(4),
            gpus_per_process: Some(1),
            walltime: Some(Walltime::PerSubmission(
                Duration::new(true, 0, 240, 0).expect("Valid duration."),
            )),
        };

        let action = Action {
            name: Some("action".to_string()),
            command: Some("command {directory}".to_string()),
            launchers: Some(vec!["mpi".into()]),
            resources,
            ..Action::default()
        };

        let directories = vec![PathBuf::from("a"), PathBuf::from("b"), PathBuf::from("c")];
        let launchers = launcher::Configuration::built_in();
        (action, directories, launchers.by_cluster("cluster"))
    }

    #[test]
    #[parallel]
    fn header() {
        let (action, directories, launchers) = setup();
        let script = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build()
        .expect("Valid script.");
        println!("{script}");

        assert!(script.starts_with("#!/bin/bash"));
    }

    #[test]
    #[parallel]
    fn preamble() {
        let (action, directories, launchers) = setup();
        let script = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .with_preamble("#preamble")
        .build()
        .expect("Valid script.");
        println!("{script}");

        assert!(script.contains("#preamble\n"));
    }

    #[test]
    #[parallel]
    fn no_setup() {
        let (action, directories, launchers) = setup();
        let script = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build()
        .expect("Valid script.");
        println!("{script}");

        assert!(!script.contains("test $? -eq 0 ||"));
    }

    #[test]
    #[parallel]
    fn with_setup() {
        let (mut action, directories, launchers) = setup();
        action
            .submit_options
            .insert("cluster".to_string(), SubmitOptions::default());

        let script = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build()
        .expect("Valid script.");
        println!("{script}");
        assert!(!script.contains("test $? -eq 0 ||"));

        action.submit_options.get_mut("cluster").unwrap().setup = Some("my setup".to_string());
        let script = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build()
        .expect("Valid script.");
        println!("{script}");
        assert!(script.contains("my setup"));
        assert!(script.contains("test $? -eq 0 ||"));
    }

    #[test]
    #[parallel]
    fn execution_directory() {
        let (action, directories, launchers) = setup();
        let script = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build()
        .expect("Valid script.");
        println!("{script}");

        assert!(script.contains("command $directory"));
    }

    #[test]
    #[parallel]
    fn execution_directories() {
        let (mut action, directories, launchers) = setup();
        action.command = Some("command {directories}".to_string());

        let script = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build()
        .expect("Valid script.");
        println!("{script}");

        assert!(script.contains("command \"${directories[@]}\""));
    }

    #[test]
    #[parallel]
    fn execution_openmp() {
        let (mut action, directories, launchers) = setup();
        action.resources.processes = Some(Processes::PerSubmission(1));
        action.launchers = Some(vec!["openmp".into()]);
        action.command = Some("command {directories}".to_string());

        let script = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build()
        .expect("Valid script.");
        println!("{script}");

        assert!(script.contains("OMP_NUM_THREADS=4 command \"${directories[@]}\""));
    }

    #[test]
    #[parallel]
    fn execution_mpi() {
        let (mut action, directories, launchers) = setup();
        action.launchers = Some(vec!["mpi".into()]);
        action.command = Some("command {directories}".to_string());

        let script = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build()
        .expect("Valid script.");
        println!("{script}");

        assert!(script.contains(
            "srun --ntasks=6 --cpus-per-task=4 --tres-per-task=gres/gpu:1 command \"${directories[@]}\""
        ));
    }

    #[test]
    #[parallel]
    fn command_errors() {
        let (mut action, directories, launchers) = setup();
        action.command = Some("command {directory} {directories}".to_string());

        let result = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build();

        assert!(matches!(
            result,
            Err(Error::ActionContainsMultipleTemplates { .. })
        ));

        action.command = Some("command".to_string());

        let result = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build();

        assert!(matches!(
            result,
            Err(Error::ActionContainsNoTemplate { .. })
        ));
    }

    #[test]
    #[parallel]
    fn variables() {
        let (action, directories, launchers) = setup();
        let script = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build()
        .expect("Valid script.");

        println!("{script}");

        assert!(script.contains("export ACTION_CLUSTER=cluster\n"));
        assert!(script.contains("export ACTION_NAME=action\n"));
        assert!(script.contains("export ACTION_PROCESSES=6\n"));
        assert!(script.contains("export ACTION_WALLTIME_IN_MINUTES=4\n"));
        assert!(script.contains("export ACTION_PROCESSES_PER_DIRECTORY=2\n"));
        assert!(script.contains("export ACTION_THREADS_PER_PROCESS=4\n"));
        assert!(script.contains("export ACTION_GPUS_PER_PROCESS=1\n"));
    }

    #[test]
    #[parallel]
    fn more_variables() {
        let (mut action, directories, launchers) = setup();
        action.resources.processes = Some(Processes::PerSubmission(10));
        action.resources.walltime = Some(Walltime::PerDirectory(
            Duration::new(true, 0, 60, 0).expect("Valid duration."),
        ));
        action.resources.threads_per_process = None;
        action.resources.gpus_per_process = None;

        let script = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build()
        .expect("Valid script.");

        println!("{script}");

        assert!(script.contains("export ACTION_CLUSTER=cluster\n"));
        assert!(script.contains("export ACTION_NAME=action\n"));
        assert!(script.contains("export ACTION_PROCESSES=10\n"));
        assert!(script.contains("export ACTION_WALLTIME_IN_MINUTES=3\n"));
        assert!(!script.contains("export ACTION_PROCESSES_PER_DIRECTORY"));
        assert!(!script.contains("export ACTION_THREADS_PER_PROCESS"));
        assert!(!script.contains("export ACTION_GPUS_PER_PROCESS"));
    }

    #[test]
    #[parallel]
    fn scheduler() {
        let (action, directories, launchers) = setup();
        let cluster = Cluster {
            name: "cluster".into(),
            scheduler: SchedulerType::Bash,
            identify: IdentificationMethod::Always(false),
            partition: Vec::new(),
            submit_options: Vec::new(),
        };
        let script = Bash::new(cluster, launchers)
            .make_script(&action, &directories, &PathBuf::default(), &HashMap::new())
            .expect("Valid script");
        println!("{script}");

        assert!(script.contains("command $directory"));
    }

    #[test]
    #[parallel]
    fn launcher_required() {
        let (mut action, directories, launchers) = setup();
        action.launchers = Some(vec![]);
        action.command = Some("command {directories}".to_string());

        let result = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build();

        assert!(matches!(result, Err(Error::NoProcessLauncher(_, _))));
    }

    #[test]
    #[parallel]
    fn too_many_launchers() {
        let (mut action, directories, launchers) = setup();
        action.resources.processes = Some(Processes::PerSubmission(1));
        action.launchers = Some(vec!["mpi".into(), "mpi".into()]);
        action.command = Some("command {directories}".to_string());

        let result = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &PathBuf::default(),
            &HashMap::new(),
            &launchers,
        )
        .build();

        assert!(matches!(result, Err(Error::TooManyProcessLaunchers(_))));
    }

    #[test]
    #[parallel]
    fn invalid_template_without_pointer() {
        let (mut action, directories, launchers) = setup();
        action.command = Some(r"command {directory} {invalid}".to_string());
        let workspace_path = PathBuf::from("workspace_path/test");
        let directory_values = HashMap::new();

        let builder = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &workspace_path,
            &directory_values,
            &launchers,
        );

        assert!(!builder.contains_json_pointer());

        let result = builder.build();

        assert!(matches!(result, Err(Error::InvalidTemplate(_, _))));
    }

    #[test]
    #[parallel]
    fn invalid_template_with_pointer() {
        let (mut action, directories, launchers) = setup();
        action.command = Some(r"command {directory} {invalid} {/pointer}".to_string());
        let workspace_path = PathBuf::from("workspace_path/test");
        let directory_values = HashMap::new();

        let builder = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &workspace_path,
            &directory_values,
            &launchers,
        );

        assert!(builder.contains_json_pointer());

        let result = builder.build();

        assert!(matches!(result, Err(Error::InvalidTemplate(_, _))));
    }

    #[test]
    #[parallel]
    fn workspace_path_without_pointer() {
        let (mut action, directories, launchers) = setup();
        action.command = Some(r"command {directory} {workspace_path}".to_string());
        let workspace_path = PathBuf::from("test/path");
        let directory_values = HashMap::new();

        let builder = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &workspace_path,
            &directory_values,
            &launchers,
        );

        assert!(!builder.contains_json_pointer());

        let script = builder.build().expect("valid script");

        println!("{script}");

        assert!(script.contains("export ACTION_WORKSPACE_PATH=test/path\n"));
        assert!(script.contains("command $directory test/path"));

        // Test again with a path that requires escaping
        let workspace_path = PathBuf::from("test $path");

        let builder = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &workspace_path,
            &directory_values,
            &launchers,
        );

        assert!(!builder.contains_json_pointer());

        let script = builder.build().expect("valid script");

        println!("{script}");

        assert!(script.contains("export ACTION_WORKSPACE_PATH=$'test $path'\n"));
        assert!(script.contains("command $directory $'test $path'"));
    }

    #[test]
    #[parallel]
    fn workspace_path_with_directories() {
        let (mut action, directories, launchers) = setup();
        action.command = Some(r"command {directories} {workspace_path}".to_string());
        let workspace_path = PathBuf::from("test_path");
        let directory_values = HashMap::new();

        let builder = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &workspace_path,
            &directory_values,
            &launchers,
        );

        assert!(!builder.contains_json_pointer());

        let script = builder.build().expect("valid script");

        println!("{script}");

        assert!(script.contains("export ACTION_WORKSPACE_PATH=test_path\n"));
        assert!(script.contains(r#"command "${directories[@]}" test_path"#));

        // Test again with a path that requires escaping
        let workspace_path = PathBuf::from("test $path");

        let builder = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &workspace_path,
            &directory_values,
            &launchers,
        );

        assert!(!builder.contains_json_pointer());

        let script = builder.build().expect("valid script");

        println!("{script}");

        assert!(script.contains("export ACTION_WORKSPACE_PATH=$'test $path'\n"));
        assert!(script.contains(r#"command "${directories[@]}" $'test $path'"#));
    }

    #[test]
    #[parallel]
    fn workspace_path_with_json_pointers() {
        let (mut action, directories, launchers) = setup();
        action.command = Some(
            r"command {directory} {workspace_path} {/value} {/name} {/valid} {/array} {}"
                .to_string(),
        );
        let workspace_path = PathBuf::from("test $path");
        let mut directory_values = HashMap::new();
        directory_values.insert(
            PathBuf::from("a"),
            json!({"value": 1, "name": "directory_a", "valid": true, "array": [1,2,3]}),
        );
        directory_values.insert(
            PathBuf::from("b"),
            json!({"value": 5, "name": "directory_b", "valid": false, "array": [4,5,6]}),
        );
        directory_values.insert(
            PathBuf::from("c"),
            json!({"value": 7, "name": "directory_c", "valid": null, "array": [7,8,9]}),
        );

        let builder = BashScriptBuilder::new(
            "cluster",
            &action,
            &directories,
            &workspace_path,
            &directory_values,
            &launchers,
        );

        assert!(builder.contains_json_pointer());

        let script = builder.build().expect("valid script");

        println!("{script}");

        assert!(script.contains("export ACTION_WORKSPACE_PATH=$'test $path'\n"));
        assert!(script.contains("command a $'test $path' 1 directory_a true $'[1,2,3]'"));
        assert!(script.contains("command b $'test $path' 5 directory_b false $'[4,5,6]'"));
        assert!(script.contains("command c $'test $path' 7 directory_c null $'[7,8,9]'"));
    }
}
