// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use log::trace;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fmt::Write as _;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};

use crate::builtin::BuiltIn;
use crate::workflow::Resources;
use crate::Error;

/// Launcher configuration
///
/// `Configuration` stores the launcher configuration for each defined
/// launcher/cluster.
///
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Configuration {
    /// The launcher configurations.
    pub(crate) launchers: HashMap<String, HashMap<String, Launcher>>,
}

/// Launcher
///
/// `Launcher` is one element of the launcher configuration.
///
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Launcher {
    pub executable: Option<String>,
    pub gpus_per_process: Option<String>,
    pub processes: Option<String>,
    pub threads_per_process: Option<String>,
}

impl Launcher {
    /// Build the launcher prefix appropriate for the given resources
    pub fn prefix(&self, resources: &Resources, n_directories: usize) -> String {
        let mut result = String::new();
        let mut need_space = false;

        if let Some(executable) = &self.executable {
            result.push_str(executable);
            need_space = true;
        }

        if let Some(processes) = &self.processes {
            if need_space {
                result.push(' ');
            }
            let _ = write!(
                result,
                "{processes}{}",
                resources.total_processes(n_directories)
            );
            need_space = true;
        }

        if let (Some(self_threads), Some(resources_threads)) =
            (&self.threads_per_process, resources.threads_per_process)
        {
            if need_space {
                result.push(' ');
            }
            let _ = write!(result, "{self_threads}{resources_threads}");
            need_space = true;
        }

        if let (Some(self_gpus), Some(resources_gpus)) =
            (&self.gpus_per_process, resources.gpus_per_process)
        {
            if need_space {
                result.push(' ');
            }
            let _ = write!(result, "{self_gpus}{resources_gpus}");
            need_space = true;
        }

        if need_space {
            result.push(' ');
        }
        result
    }
}

impl Configuration {
    /// Open the launcher configuration
    ///
    /// Open `$HOME/.config/row/launchers.toml` if it exists and merge it with
    /// the built-in configuration.
    ///
    /// # Errors
    /// Returns `Err(row::Error)` when the file cannot be read or if there is
    /// as parse error.
    ///
    pub fn open() -> Result<Self, Error> {
        let home = match env::var("ROW_HOME") {
            Ok(row_home) => PathBuf::from(row_home),
            Err(_) => home::home_dir().ok_or_else(Error::NoHome)?,
        };
        let launchers_toml_path = home.join(".config").join("row").join("launchers.toml");
        Self::open_from_path(launchers_toml_path)
    }

    fn open_from_path(launchers_toml_path: PathBuf) -> Result<Self, Error> {
        let mut launchers = Self::built_in();

        let launchers_file = match File::open(&launchers_toml_path) {
            Ok(file) => file,
            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => {
                    trace!(
                        "'{}' does not exist, using built-in launchers.",
                        &launchers_toml_path.display()
                    );
                    return Ok(launchers);
                }
                _ => return Err(Error::FileRead(launchers_toml_path, error)),
            },
        };

        let mut buffer = BufReader::new(launchers_file);
        let mut launchers_string = String::new();
        buffer
            .read_to_string(&mut launchers_string)
            .map_err(|e| Error::FileRead(launchers_toml_path.clone(), e))?;

        trace!("Parsing '{}'.", &launchers_toml_path.display());
        let user_config = Self::parse_str(&launchers_toml_path, &launchers_string)?;
        launchers.merge(user_config);
        launchers.validate()?;
        Ok(launchers)
    }

    /// Parse a `Configuration` from a TOML string
    ///
    /// Does *NOT* merge with the built-in configuration.
    ///
    pub(crate) fn parse_str(path: &Path, toml: &str) -> Result<Self, Error> {
        Ok(Configuration {
            launchers: toml::from_str(toml)
                .map_err(|e| Error::TOMLParse(path.join("launchers.toml"), e))?,
        })
    }

    /// Merge keys from another configuration into this one.
    ///
    /// Merging adds new keys from `b` into self. It also overrides any keys in
    /// both with the value in `b`.
    ///
    fn merge(&mut self, b: Self) {
        for (launcher_name, launcher_clusters) in b.launchers {
            self.launchers
                .entry(launcher_name)
                .and_modify(|e| e.extend(launcher_clusters.clone()))
                .or_insert(launcher_clusters);
        }
    }

    /// Validate that the configuration is correct.
    ///
    /// Valid launcher configurations have a `default` cluster for all
    /// launchers.
    fn validate(&self) -> Result<(), Error> {
        for (launcher_name, launcher_clusters) in &self.launchers {
            if !launcher_clusters.contains_key("default") {
                return Err(Error::LauncherMissingDefault(launcher_name.clone()));
            }
        }

        Ok(())
    }

    /// Get all launchers for a specific cluster.
    ///
    /// # Panics
    /// When a given launcher has no default.
    ///
    pub fn by_cluster(&self, cluster_name: &str) -> HashMap<String, Launcher> {
        let mut result = HashMap::with_capacity(self.launchers.len());

        for (launcher_name, launcher_clusters) in &self.launchers {
            if let Some(launcher) = launcher_clusters.get(cluster_name) {
                result.insert(launcher_name.clone(), launcher.clone());
            } else {
                result.insert(
                    launcher_name.clone(),
                    launcher_clusters
                        .get("default")
                        .expect("launcher should have a default")
                        .clone(),
                );
            }
        }

        result
    }

    /// Get the complete launcher configuration.
    pub fn full_config(&self) -> &HashMap<String, HashMap<String, Launcher>> {
        &self.launchers
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use serial_test::parallel;

    use super::*;
    use crate::workflow::Processes;

    fn setup() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::max())
            .is_test(true)
            .try_init();
    }

    #[test]
    #[parallel]
    fn unset_launcher() {
        setup();
        let launchers = Configuration::built_in();
        let launchers_by_cluster = launchers.by_cluster("any_cluster");
        assert!(!launchers_by_cluster.contains_key("unset_launcher"));
    }

    #[test]
    #[parallel]
    fn openmp_prefix() {
        setup();
        let launchers = Configuration::built_in();
        let launchers_by_cluster = launchers.by_cluster("any_cluster");
        let openmp = launchers_by_cluster
            .get("openmp")
            .expect("a valid Launcher");

        let no_threads = Resources::default();
        assert_eq!(openmp.prefix(&no_threads, 10), "");
        assert_eq!(openmp.prefix(&no_threads, 1), "");

        let threads = Resources {
            threads_per_process: Some(5),
            ..Resources::default()
        };
        assert_eq!(openmp.prefix(&threads, 10), "OMP_NUM_THREADS=5 ");
        assert_eq!(openmp.prefix(&threads, 1), "OMP_NUM_THREADS=5 ");
    }

    #[test]
    #[parallel]
    fn mpi_prefix_none() {
        setup();
        let launchers = Configuration::built_in();
        let launchers_by_cluster = launchers.by_cluster("none");
        let mpi = launchers_by_cluster.get("mpi").expect("a valid Launcher");

        let one_proc = Resources::default();
        assert_eq!(mpi.prefix(&one_proc, 10), "mpirun -n 1 ");
        assert_eq!(mpi.prefix(&one_proc, 1), "mpirun -n 1 ");

        let procs_per_directory = Resources {
            processes: Some(Processes::PerDirectory(2)),
            ..Resources::default()
        };
        assert_eq!(mpi.prefix(&procs_per_directory, 11), "mpirun -n 22 ");
        assert_eq!(mpi.prefix(&procs_per_directory, 1), "mpirun -n 2 ");

        let all = Resources {
            processes: Some(Processes::PerDirectory(6)),
            threads_per_process: Some(3),
            gpus_per_process: Some(8),
            ..Resources::default()
        };
        assert_eq!(mpi.prefix(&all, 11), "mpirun -n 66 ");
        assert_eq!(mpi.prefix(&all, 1), "mpirun -n 6 ");
    }

    #[test]
    #[parallel]
    fn mpi_prefix_default() {
        setup();
        let launchers = Configuration::built_in();
        let launchers_by_cluster = launchers.by_cluster("any_cluster");
        let mpi = launchers_by_cluster.get("mpi").expect("a valid Launcher");

        let one_proc = Resources::default();
        assert_eq!(mpi.prefix(&one_proc, 10), "srun --ntasks=1 ");
        assert_eq!(mpi.prefix(&one_proc, 1), "srun --ntasks=1 ");

        let procs_per_directory = Resources {
            processes: Some(Processes::PerDirectory(2)),
            ..Resources::default()
        };
        assert_eq!(mpi.prefix(&procs_per_directory, 11), "srun --ntasks=22 ");
        assert_eq!(mpi.prefix(&procs_per_directory, 1), "srun --ntasks=2 ");

        let all = Resources {
            processes: Some(Processes::PerDirectory(6)),
            threads_per_process: Some(3),
            gpus_per_process: Some(8),
            ..Resources::default()
        };
        assert_eq!(
            mpi.prefix(&all, 11),
            "srun --ntasks=66 --cpus-per-task=3 --tres-per-task=gres/gpu:8 "
        );
        assert_eq!(
            mpi.prefix(&all, 1),
            "srun --ntasks=6 --cpus-per-task=3 --tres-per-task=gres/gpu:8 "
        );
    }

    #[test]
    #[parallel]
    fn open_no_file() {
        setup();
        let temp = TempDir::new().unwrap().child("launchers.json");
        let launchers = Configuration::open_from_path(temp.path().into()).expect("valid launchers");
        assert_eq!(launchers, Configuration::built_in());
    }

    #[test]
    #[parallel]
    fn open_empty_file() {
        setup();
        let temp = TempDir::new().unwrap().child("launchers.json");
        temp.write_str("").unwrap();
        let launchers = Configuration::open_from_path(temp.path().into()).expect("valid launchers");
        assert_eq!(launchers, Configuration::built_in());
    }

    #[test]
    #[parallel]
    fn no_default() {
        setup();
        let temp = TempDir::new().unwrap().child("launchers.json");
        temp.write_str(
            r"
[new_launcher.not_default]
",
        )
        .unwrap();
        let error = Configuration::open_from_path(temp.path().into());
        assert!(matches!(error, Err(Error::LauncherMissingDefault(_))));
    }

    #[test]
    #[parallel]
    fn new_launcher() {
        setup();
        let temp = TempDir::new().unwrap().child("launchers.json");
        temp.write_str(
            r#"
[new_launcher.default]
executable = "a"
processes = "b"
threads_per_process = "c"
gpus_per_process = "d"

[new_launcher.non_default]
executable = "e"
"#,
        )
        .unwrap();
        let launchers = Configuration::open_from_path(temp.path().into()).expect("valid launcher");

        let built_in = Configuration::built_in();
        assert_eq!(launchers.launchers.len(), 3);
        assert_eq!(launchers.launchers["openmp"], built_in.launchers["openmp"]);
        assert_eq!(launchers.launchers["mpi"], built_in.launchers["mpi"]);

        let launchers_by_cluster = launchers.by_cluster("non_default");
        let non_default = launchers_by_cluster.get("new_launcher").unwrap();
        assert_eq!(non_default.executable, Some("e".into()));
        assert_eq!(non_default.processes, None);
        assert_eq!(non_default.threads_per_process, None);
        assert_eq!(non_default.gpus_per_process, None);

        let launchers_by_cluster = launchers.by_cluster("any_cluster");
        let default = launchers_by_cluster.get("new_launcher").unwrap();
        assert_eq!(default.executable, Some("a".into()));
        assert_eq!(default.processes, Some("b".into()));
        assert_eq!(default.threads_per_process, Some("c".into()));
        assert_eq!(default.gpus_per_process, Some("d".into()));
    }
}
