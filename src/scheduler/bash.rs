use log::{debug, error, trace};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::env;
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::scheduler::Scheduler;
use crate::workflow::{Action, Processes};
use crate::Error;

/// `BashScriptBuilder` builds `bash` scripts that execute row actions.
struct BashScriptBuilder<'a> {
    walltime_in_minutes: i64,
    total_processes: usize,
    cluster_name: &'a str,
    action: &'a Action,
    directories: &'a [PathBuf],
    preamble: &'a str,
}

impl<'a> BashScriptBuilder<'a> {
    /// Construct a new bash script builder.
    pub(crate) fn new(
        cluster_name: &'a str,
        action: &'a Action,
        directories: &'a [PathBuf],
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
            preamble: "",
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
            result.push('\'');
            result.push_str(
                directory
                    .to_str()
                    .ok_or_else(|| Error::NonUTF8DirectoryName(directory.clone()))?,
            );
            result.push_str("'\n");
        }
        result.push_str(")\n");

        result.push_str(&format!(
            r#"
export ACTION_CLUSTER="{}"
export ACTION_NAME="{}"
export ACTION_PROCESSES="{}"
export ACTION_WALLTIME_IN_MINUTES="{}"
"#,
            self.cluster_name, self.action.name, self.total_processes, self.walltime_in_minutes,
        ));

        if let Processes::PerDirectory(processes_per_directory) = self.action.resources.processes {
            result.push_str(&format!(
                "export ACTION_PROCESSES_PER_DIRECTORY=\"{}\"\n",
                processes_per_directory
            ));
        }

        if let Some(threads_per_process) = self.action.resources.threads_per_process {
            result.push_str(&format!(
                "export ACTION_THREADS_PER_PROCESS=\"{}\"\n",
                threads_per_process
            ));
        }

        if let Some(gpus_per_process) = self.action.resources.gpus_per_process {
            result.push_str(&format!(
                "export ACTION_GPUS_PER_PROCESS=\"{}\"\n",
                gpus_per_process
            ));
        }

        Ok(result)
    }

    fn setup(&self) -> Result<String, Error> {
        let mut user_setup = self
            .action
            .cluster
            .get(self.cluster_name)
            .and_then(|c| c.setup.clone())
            .unwrap_or_default();
        if !user_setup.is_empty() {
            user_setup.push_str(
                r#"
test $? -eq 0 || { >&2 echo "[row] Error executing setup."; exit 1; }"#,
            );
        }

        let action_name = &self.action.name;
        let row_executable = env::current_exe().map_err(Error::FindCurrentExecutable)?;
        let row_executable = row_executable.to_str().expect("UTF-8 path to executable.");
        user_setup.push_str(&format!(
            r#"
trap 'printf %s\\n "${{directories[@]}}" | {row_executable} scan --no-progress -a {action_name} - || exit 3' EXIT"#
        ));

        Ok(user_setup)
    }

    fn execution(&self) -> Result<String, Error> {
        let contains_directory = self.action.command.contains("{directory}");
        let contains_directories = self.action.command.contains("{directories}");
        if contains_directory as u32 + contains_directories as u32 > 1 {
            return Err(Error::ActionContainsMultipleTemplates(
                self.action.name.clone(),
            ));
        }

        // TODO: Apply launcher.

        if contains_directory {
            let command = self.action.command.replace("{directory}", "$directory");
            Ok(format!(
                r#"
for directory in "${{directories[@]}}"
do
    {command} || {{ >&2 echo "[ERROR row::action] Error executing command."; exit 2; }}
done
"#
            ))
        } else if contains_directories {
            let command = self
                .action
                .command
                .replace("{directories}", r#""${directories[@]}""#);
            Ok(format!(
                r#"
{command} || {{ >&2 echo "[row] Error executing command."; exit 1; }}
"#
            ))
        } else {
            Err(Error::ActionContainsNoTemplate(self.action.name.clone()))
        }
    }

    pub(crate) fn build(&self) -> Result<String, Error> {
        Ok(self.header() + &self.variables()? + &self.setup()? + &self.execution()?)
    }
}

/// The `Bash` scheduler constructs bash scripts and executes them with `bash`.
pub struct Bash {
    /// The name of the cluster.
    cluster_name: String,
    // TODO: store partition and launcher (or maybe whole Cluster object reference?)
}

impl Bash {}

impl Scheduler for Bash {
    fn new(cluster_name: &str) -> Self {
        Self {
            cluster_name: cluster_name.to_string(),
        }
    }

    fn make_script(&self, action: &Action, directories: &[PathBuf]) -> Result<String, Error> {
        // TODO: Remove with_preamble when the slurm code is written.
        // it is here only to hide a dead code warning.
        BashScriptBuilder::new(&self.cluster_name, action, directories)
            .with_preamble("")
            .build()
    }

    fn submit(
        &self,
        working_directory: &Path,
        action: &Action,
        directories: &[PathBuf],
        should_terminate: Arc<AtomicBool>,
    ) -> Result<Option<u32>, Error> {
        let script = BashScriptBuilder::new(&self.cluster_name, action, directories).build()?;
        debug!("Executing '{}' in bash.", action.name);

        let mut child = Command::new("bash")
            .stdin(Stdio::piped())
            .current_dir(working_directory)
            .spawn()
            .map_err(|e| Error::SpawnProcess("bash".into(), e))?;

        let mut stdin = child.stdin.take().expect("Piped stdin");
        write!(stdin, "{}", script)?;
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
            return Err(Error::ExecuteAction(action.name.clone(), message));
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use speedate::Duration;

    use crate::workflow::Walltime;
    use crate::workflow::{ClusterParameters, Resources};

    fn setup() -> (Action, Vec<PathBuf>) {
        let resources = Resources {
            processes: Processes::PerDirectory(2),
            threads_per_process: Some(4),
            gpus_per_process: Some(1),
            walltime: Walltime::PerSubmission(
                Duration::new(true, 0, 240, 0).expect("Valid duration."),
            ),
            ..Resources::default()
        };

        let action = Action {
            name: "action".to_string(),
            command: "command {directory}".to_string(),
            resources,
            ..Action::default()
        };

        let directories = vec![PathBuf::from("a"), PathBuf::from("b"), PathBuf::from("c")];
        (action, directories)
    }

    #[test]
    fn test_header() {
        let (action, directories) = setup();
        let script = BashScriptBuilder::new("cluster", &action, &directories)
            .build()
            .expect("Valid script.");
        println!("{script}");

        assert!(script.starts_with("#!/bin/bash"));
    }

    #[test]
    fn test_preamble() {
        let (action, directories) = setup();
        let script = BashScriptBuilder::new("cluster", &action, &directories)
            .with_preamble("#preamble")
            .build()
            .expect("Valid script.");
        println!("{script}");

        assert!(script.contains("#preamble\n"));
    }

    #[test]
    fn test_no_setup() {
        let (action, directories) = setup();
        let script = BashScriptBuilder::new("cluster", &action, &directories)
            .build()
            .expect("Valid script.");
        println!("{script}");

        assert!(!script.contains("test $? -eq 0 ||"));
    }

    #[test]
    fn test_setup() {
        let (mut action, directories) = setup();
        action
            .cluster
            .insert("cluster".to_string(), ClusterParameters::default());

        let script = BashScriptBuilder::new("cluster", &action, &directories)
            .build()
            .expect("Valid script.");
        println!("{script}");
        assert!(!script.contains("test $? -eq 0 ||"));

        action.cluster.get_mut("cluster").unwrap().setup = Some("my setup".to_string());
        let script = BashScriptBuilder::new("cluster", &action, &directories)
            .build()
            .expect("Valid script.");
        println!("{script}");
        assert!(script.contains("my setup"));
        assert!(script.contains("test $? -eq 0 ||"));
    }

    #[test]
    fn test_execution_directory() {
        let (action, directories) = setup();
        let script = BashScriptBuilder::new("cluster", &action, &directories)
            .build()
            .expect("Valid script.");
        println!("{script}");

        assert!(script.contains("command $directory"));
    }

    #[test]
    fn test_execution_directories() {
        let (mut action, directories) = setup();
        action.command = "command {directories}".to_string();

        let script = BashScriptBuilder::new("cluster", &action, &directories)
            .build()
            .expect("Valid script.");
        println!("{script}");

        assert!(script.contains("command \"${directories[@]}\""));
    }

    #[test]
    fn test_command_errors() {
        let (mut action, directories) = setup();
        action.command = "command {directory} {directories}".to_string();

        let result = BashScriptBuilder::new("cluster", &action, &directories).build();

        assert!(matches!(
            result,
            Err(Error::ActionContainsMultipleTemplates { .. })
        ));

        action.command = "command".to_string();

        let result = BashScriptBuilder::new("cluster", &action, &directories).build();

        assert!(matches!(
            result,
            Err(Error::ActionContainsNoTemplate { .. })
        ));
    }

    #[test]
    fn test_variables() {
        let (action, directories) = setup();
        let script = BashScriptBuilder::new("cluster", &action, &directories)
            .build()
            .expect("Valid script.");

        println!("{script}");

        assert!(script.contains("export ACTION_CLUSTER=\"cluster\"\n"));
        assert!(script.contains("export ACTION_NAME=\"action\"\n"));
        assert!(script.contains("export ACTION_PROCESSES=\"6\"\n"));
        assert!(script.contains("export ACTION_WALLTIME_IN_MINUTES=\"4\"\n"));
        assert!(script.contains("export ACTION_PROCESSES_PER_DIRECTORY=\"2\"\n"));
        assert!(script.contains("export ACTION_THREADS_PER_PROCESS=\"4\"\n"));
        assert!(script.contains("export ACTION_GPUS_PER_PROCESS=\"1\"\n"));
    }

    #[test]
    fn test_more_variables() {
        let (mut action, directories) = setup();
        action.resources.processes = Processes::PerSubmission(10);
        action.resources.walltime =
            Walltime::PerDirectory(Duration::new(true, 0, 60, 0).expect("Valid duration."));
        action.resources.threads_per_process = None;
        action.resources.gpus_per_process = None;

        let script = BashScriptBuilder::new("cluster", &action, &directories)
            .build()
            .expect("Valid script.");

        println!("{script}");

        assert!(script.contains("export ACTION_CLUSTER=\"cluster\"\n"));
        assert!(script.contains("export ACTION_NAME=\"action\"\n"));
        assert!(script.contains("export ACTION_PROCESSES=\"10\"\n"));
        assert!(script.contains("export ACTION_WALLTIME_IN_MINUTES=\"3\"\n"));
        assert!(!script.contains("export ACTION_PROCESSES_PER_DIRECTORY"));
        assert!(!script.contains("export ACTION_THREADS_PER_PROCESS"));
        assert!(!script.contains("export ACTION_GPUS_PER_PROCESS"));
    }

    #[test]
    fn test_scheduler() {
        let (action, directories) = setup();
        let script = Bash::new("cluster")
            .make_script(&action, &directories)
            .expect("Valid script");
        println!("{script}");

        assert!(script.contains("command $directory"));
    }
}
