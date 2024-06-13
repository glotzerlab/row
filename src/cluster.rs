// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::fmt::Write as _;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};

use crate::builtin::BuiltIn;
use crate::workflow::Resources;
use crate::Error;

/// Cluster configuration
///
/// `Configuration` stores the cluster configuration for each defined
/// cluster.
///
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    /// The cluster configurations.
    #[serde(default)]
    pub(crate) cluster: Vec<Cluster>,
}

/// Cluster
///
/// `Cluster` stores everything needed to define a single cluster. It is read
/// from the `clusters.toml` file.
///
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Cluster {
    /// The cluster's name.
    pub name: String,

    /// The method used to automatically identify this cluster.
    pub identify: IdentificationMethod,

    /// The scheduler used on the cluster.
    pub scheduler: SchedulerType,

    /// The partitions in the cluster's queue.
    pub partition: Vec<Partition>,
}

/// Methods to identify clusters.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentificationMethod {
    /// Identify a cluster when an environment variable is equal to a value.
    ByEnvironment(String, String),
    /// Identify a cluster always (true) or never (false)
    Always(bool),
}

/// Types of schedulers.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerType {
    /// Submit jobs to run immediately in bash.
    Bash,
    /// Submit jobs to a Slurm queue.
    Slurm,
}

/// Partition parameters.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Partition {
    /// The partition's name.
    pub name: String,

    /// Maximum number of CPUs per job.
    pub maximum_cpus_per_job: Option<usize>,

    /// Require CPUs to be a multiple of this value.
    pub require_cpus_multiple_of: Option<usize>,

    /// Warn if CPUs are not a multiple of this value.
    pub warn_cpus_multiple_of: Option<usize>,

    /// Memory per CPU.
    pub memory_per_cpu: Option<String>,

    /// CPUs per node.
    pub cpus_per_node: Option<usize>,

    /// Minimum number of GPUs per job.
    pub minimum_gpus_per_job: Option<usize>,

    /// Maximum number of GPUs per job.
    pub maximum_gpus_per_job: Option<usize>,

    /// Require GPUs to be a multiple of this value.
    pub require_gpus_multiple_of: Option<usize>,

    /// Warn if GPUs are not a multiple of this value.
    pub warn_gpus_multiple_of: Option<usize>,

    /// Memory per GPU.
    pub memory_per_gpu: Option<String>,

    /// GPUs per node.
    pub gpus_per_node: Option<usize>,

    /// Prevent auto-selection
    #[serde(default)]
    pub prevent_auto_select: bool,

    /// Suffix the account name
    pub account_suffix: Option<String>,
}

impl Configuration {
    /// Identify the cluster.
    ///
    /// Identifying the current cluster consumes the `Configuration`.
    ///
    /// # Errors
    /// * `row::Error::ClusterNameNotFound` when a cluster by the given name
    ///   is not present in the configuration (when `name = Some(_)`).
    /// * `row::Error::ClusterNotFound` when the automatic identification
    ///   fails to find a cluster in the configuration.
    ///
    pub fn identify(self, name: Option<&str>) -> Result<Cluster, Error> {
        let cluster = if let Some(name) = name {
            self.cluster
                .into_iter()
                .find(|c| c.name == name)
                .ok_or_else(|| Error::ClusterNameNotFound(name.to_string()))?
        } else {
            self.cluster
                .into_iter()
                .find(Cluster::identity_matches)
                .ok_or_else(Error::ClusterNotFound)?
        };

        info!("Identified cluster '{}'.", cluster.name);
        Ok(cluster)
    }

    /// Open the cluster configuration
    ///
    /// Open `$HOME/.config/row/clusters.toml` if it exists and merge it with
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
        let clusters_toml_path = home.join(".config").join("row").join("clusters.toml");
        Self::open_from_path(clusters_toml_path)
    }

    fn open_from_path(clusters_toml_path: PathBuf) -> Result<Self, Error> {
        let mut clusters = Self::built_in();

        let clusters_file = match File::open(&clusters_toml_path) {
            Ok(file) => file,
            Err(error) => match error.kind() {
                io::ErrorKind::NotFound => {
                    trace!(
                        "'{}' does not exist, using built-in clusters.",
                        &clusters_toml_path.display()
                    );
                    return Ok(clusters);
                }
                _ => return Err(Error::FileRead(clusters_toml_path, error)),
            },
        };

        let mut buffer = BufReader::new(clusters_file);
        let mut clusters_string = String::new();
        buffer
            .read_to_string(&mut clusters_string)
            .map_err(|e| Error::FileRead(clusters_toml_path.clone(), e))?;

        trace!("Parsing '{}'.", &clusters_toml_path.display());
        let user_config = Self::parse_str(&clusters_toml_path, &clusters_string)?;
        clusters.merge(&user_config);
        Ok(clusters)
    }

    /// Parse a `Configuration` from a TOML string
    ///
    /// Does *NOT* merge with the built-in configuration.
    ///
    pub(crate) fn parse_str(path: &Path, toml: &str) -> Result<Self, Error> {
        let cluster: Configuration =
            toml::from_str(toml).map_err(|e| Error::TOMLParse(path.join("clusters.toml"), e))?;
        Ok(cluster)
    }

    /// Merge keys from another configuration into this one.
    ///
    /// Merging adds new keys from `b` into self. It also overrides any keys in
    /// both with the value in `b`.
    ///
    fn merge(&mut self, b: &Self) {
        let mut new_cluster = b.cluster.clone();
        new_cluster.extend(self.cluster.clone());
        self.cluster = new_cluster;
    }
}

impl Cluster {
    /// Check if the cluster's identity matches the current environment.
    fn identity_matches(&self) -> bool {
        trace!(
            "Checking cluster '{}' via '{:?}'.",
            self.name,
            self.identify
        );
        match &self.identify {
            IdentificationMethod::Always(condition) => *condition,
            IdentificationMethod::ByEnvironment(variable, value) => {
                env::var(variable).is_ok_and(|x| x == *value)
            }
        }
    }

    /// Find the partition to use for the given job.
    ///
    /// # Errors
    /// Returns `Err<row::Error>` when the partition is not found.
    ///
    pub fn find_partition(
        &self,
        partition_name: Option<&str>,
        resources: &Resources,
        n_directories: usize,
    ) -> Result<&Partition, Error> {
        debug!(
            "Finding partition for {} CPUs and {} GPUs.",
            resources.total_cpus(n_directories),
            resources.total_gpus(n_directories)
        );
        let mut reason = String::new();

        let partition = if let Some(partition_name) = partition_name {
            let named_partition = self
                .partition
                .iter()
                .find(|p| p.name == partition_name)
                .ok_or_else(|| Error::PartitionNameNotFound(partition_name.to_string()))?;

            if !named_partition.matches(resources, n_directories, &mut reason) {
                return Err(Error::PartitionNotFound(reason));
            }

            named_partition
        } else {
            self.partition
                .iter()
                .find(|p| p.matches(resources, n_directories, &mut reason))
                .ok_or_else(|| Error::PartitionNotFound(reason))?
        };

        Ok(partition)
    }
}

impl Partition {
    /// Check if a given job may use this partition.
    #[allow(clippy::similar_names)]
    fn matches(&self, resources: &Resources, n_directories: usize, reason: &mut String) -> bool {
        let total_cpus = resources.total_cpus(n_directories);
        let total_gpus = resources.total_gpus(n_directories);

        trace!("Checking partition '{}'.", self.name);

        if self.prevent_auto_select {
            let _ = writeln!(reason, "{}: Must be manually selected.", self.name);
            return false;
        }

        if self.maximum_cpus_per_job.map_or(false, |x| total_cpus > x) {
            let _ = writeln!(reason, "{}: Too many CPUs ({}).", self.name, total_cpus);
            return false;
        }

        if self
            .require_cpus_multiple_of
            .map_or(false, |x| total_cpus % x != 0)
        {
            let _ = writeln!(
                reason,
                "{}: CPUs ({}) not a required multiple.",
                self.name, total_cpus
            );
            return false;
        }

        if self
            .warn_cpus_multiple_of
            .map_or(false, |x| total_cpus % x != 0)
        {
            warn!(
                "{}: CPUs ({}) not a recommended multiple.",
                self.name, total_cpus
            );
            return true; // Issuing this warning does not prevent use of the partition.
        }

        if self.minimum_gpus_per_job.map_or(false, |x| total_gpus < x) {
            let _ = writeln!(reason, "{}: Not enough GPUs ({}).", self.name, total_gpus);
            return false;
        }

        if self.maximum_gpus_per_job.map_or(false, |x| total_gpus > x) {
            let _ = writeln!(reason, "{}: Too many GPUs ({}).", self.name, total_gpus);
            return false;
        }

        trace!("total_gpus {}", total_gpus);
        if let Some(v) = self.require_gpus_multiple_of {
            trace!("total_gpus % v = {}", total_gpus % v);
        }
        if self
            .require_gpus_multiple_of
            .map_or(false, |x| total_gpus == 0 || total_gpus % x != 0)
        {
            let _ = writeln!(
                reason,
                "{}: GPUs ({}) not a required multiple.",
                self.name, total_gpus
            );
            return false;
        }

        if self
            .warn_gpus_multiple_of
            .map_or(false, |x| total_gpus == 0 || total_gpus % x != 0)
        {
            warn!(
                "{}: GPUs ({}) not a recommended multiple. ",
                self.name, total_gpus
            );
            return true; // Issuing this warning does not prevent use of the partition.
        }

        true
    }
}

impl Default for Partition {
    fn default() -> Self {
        Partition {
            name: "partition".into(),
            maximum_cpus_per_job: None,
            memory_per_cpu: None,
            cpus_per_node: None,
            require_cpus_multiple_of: None,
            warn_cpus_multiple_of: None,
            minimum_gpus_per_job: None,
            maximum_gpus_per_job: None,
            memory_per_gpu: None,
            gpus_per_node: None,
            require_gpus_multiple_of: None,
            warn_gpus_multiple_of: None,
            prevent_auto_select: false,
            account_suffix: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_fs::prelude::*;
    use assert_fs::TempDir;
    use serial_test::{parallel, serial};

    use super::*;
    use crate::workflow::Processes;

    fn setup() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::max())
            .is_test(true)
            .try_init();
    }

    #[test]
    #[serial]
    fn identify() {
        setup();
        let clusters = vec![
            Cluster {
                name: "cluster0".into(),
                identify: IdentificationMethod::Always(false),
                scheduler: SchedulerType::Bash,
                partition: Vec::new(),
            },
            Cluster {
                name: "cluster1".into(),
                identify: IdentificationMethod::ByEnvironment("_row_select".into(), "a".into()),
                scheduler: SchedulerType::Bash,
                partition: Vec::new(),
            },
            Cluster {
                name: "cluster2".into(),
                identify: IdentificationMethod::ByEnvironment("_row_select".into(), "b".into()),
                scheduler: SchedulerType::Bash,
                partition: Vec::new(),
            },
            Cluster {
                name: "cluster3".into(),
                identify: IdentificationMethod::Always(true),
                scheduler: SchedulerType::Bash,
                partition: Vec::new(),
            },
            Cluster {
                name: "cluster4".into(),
                identify: IdentificationMethod::ByEnvironment("_row_Select".into(), "b".into()),
                scheduler: SchedulerType::Bash,
                partition: Vec::new(),
            },
        ];
        let cluster_configuration = Configuration { cluster: clusters };
        assert_eq!(
            cluster_configuration
                .clone()
                .identify(Some("cluster4"))
                .unwrap(),
            cluster_configuration.cluster[4]
        );
        assert!(matches!(
            cluster_configuration
                .clone()
                .identify(Some("not a cluster")),
            Err(Error::ClusterNameNotFound(_))
        ));

        env::remove_var("_row_select");
        assert_eq!(
            cluster_configuration.clone().identify(None).unwrap(),
            cluster_configuration.cluster[3]
        );

        env::set_var("_row_select", "b");
        assert_eq!(
            cluster_configuration.clone().identify(None).unwrap(),
            cluster_configuration.cluster[2]
        );

        env::set_var("_row_select", "a");
        assert_eq!(
            cluster_configuration.clone().identify(None).unwrap(),
            cluster_configuration.cluster[1]
        );

        assert_eq!(
            cluster_configuration
                .clone()
                .identify(Some("cluster0"))
                .unwrap(),
            cluster_configuration.cluster[0]
        );
    }

    #[test]
    #[parallel]
    fn empty_partition() {
        setup();

        let partition = Partition::default();

        let resources = Resources {
            processes: Some(Processes::PerDirectory(1)),
            threads_per_process: Some(2),
            gpus_per_process: Some(3),
            ..Resources::default()
        };
        let mut reason = String::new();
        assert!(partition.matches(&resources, 10, &mut reason));
    }

    #[test]
    #[parallel]
    fn partition_checks() {
        setup();

        let resources = Resources {
            processes: Some(Processes::PerDirectory(1)),
            threads_per_process: Some(2),
            gpus_per_process: Some(3),
            ..Resources::default()
        };
        let mut reason = String::new();

        let partition = Partition {
            maximum_cpus_per_job: Some(10),
            ..Partition::default()
        };

        assert!(!partition.matches(&resources, 6, &mut reason));
        assert!(partition.matches(&resources, 5, &mut reason));

        let partition = Partition {
            require_cpus_multiple_of: Some(10),
            ..Partition::default()
        };

        assert!(!partition.matches(&resources, 6, &mut reason));
        assert!(partition.matches(&resources, 5, &mut reason));
        assert!(partition.matches(&resources, 10, &mut reason));
        assert!(partition.matches(&resources, 15, &mut reason));

        let partition = Partition {
            minimum_gpus_per_job: Some(9),
            ..Partition::default()
        };

        assert!(!partition.matches(&resources, 1, &mut reason));
        assert!(!partition.matches(&resources, 2, &mut reason));
        assert!(partition.matches(&resources, 3, &mut reason));

        let partition = Partition {
            maximum_gpus_per_job: Some(9),
            ..Partition::default()
        };

        assert!(partition.matches(&resources, 1, &mut reason));
        assert!(partition.matches(&resources, 2, &mut reason));
        assert!(partition.matches(&resources, 3, &mut reason));
        assert!(!partition.matches(&resources, 4, &mut reason));

        let partition = Partition {
            require_gpus_multiple_of: Some(9),
            ..Partition::default()
        };

        assert!(!partition.matches(&resources, 1, &mut reason));
        assert!(!partition.matches(&resources, 2, &mut reason));
        assert!(partition.matches(&resources, 3, &mut reason));
        assert!(!partition.matches(&resources, 4, &mut reason));
        assert!(!partition.matches(&resources, 5, &mut reason));
        assert!(partition.matches(&resources, 6, &mut reason));

        let partition = Partition {
            prevent_auto_select: true,
            ..Partition::default()
        };

        assert!(!partition.matches(&resources, 1, &mut reason));
        assert!(!partition.matches(&resources, 2, &mut reason));
        assert!(!partition.matches(&resources, 3, &mut reason));
        assert!(!partition.matches(&resources, 4, &mut reason));
        assert!(!partition.matches(&resources, 5, &mut reason));
        assert!(!partition.matches(&resources, 6, &mut reason));
    }

    #[test]
    #[parallel]
    fn find_partition() {
        setup();

        let partitions = vec![
            Partition {
                name: "cpu".into(),
                maximum_cpus_per_job: Some(10),
                maximum_gpus_per_job: Some(0),
                ..Partition::default()
            },
            Partition {
                name: "gpu".into(),
                maximum_gpus_per_job: Some(10),
                minimum_gpus_per_job: Some(1),
                ..Partition::default()
            },
            Partition {
                name: "other".into(),
                maximum_cpus_per_job: Some(20),
                maximum_gpus_per_job: Some(20),
                ..Partition::default()
            },
        ];

        let cluster = Cluster {
            name: "cluster".into(),
            identify: IdentificationMethod::Always(true),
            scheduler: SchedulerType::Bash,
            partition: partitions,
        };

        let cpu_resources = Resources {
            processes: Some(Processes::PerDirectory(1)),
            ..Resources::default()
        };

        let gpu_resources = Resources {
            processes: Some(Processes::PerDirectory(1)),
            gpus_per_process: Some(1),
            ..Resources::default()
        };

        assert!(
            cluster
                .find_partition(None, &cpu_resources, 1)
                .unwrap()
                .name
                == "cpu"
        );
        assert!(
            cluster
                .find_partition(None, &cpu_resources, 10)
                .unwrap()
                .name
                == "cpu"
        );
        assert!(
            cluster
                .find_partition(None, &gpu_resources, 1)
                .unwrap()
                .name
                == "gpu"
        );
        assert!(
            cluster
                .find_partition(None, &gpu_resources, 10)
                .unwrap()
                .name
                == "gpu"
        );

        assert!(
            cluster
                .find_partition(None, &cpu_resources, 11)
                .unwrap()
                .name
                == "other"
        );
        assert!(
            cluster
                .find_partition(None, &gpu_resources, 11)
                .unwrap()
                .name
                == "other"
        );
        assert!(
            cluster
                .find_partition(None, &cpu_resources, 20)
                .unwrap()
                .name
                == "other"
        );
        assert!(
            cluster
                .find_partition(None, &gpu_resources, 20)
                .unwrap()
                .name
                == "other"
        );

        assert!(matches!(
            cluster.find_partition(None, &cpu_resources, 21),
            Err(Error::PartitionNotFound(_))
        ));
        assert!(matches!(
            cluster.find_partition(Some("not_a_partition"), &cpu_resources, 1),
            Err(Error::PartitionNameNotFound(_))
        ));

        assert!(
            cluster
                .find_partition(Some("other"), &gpu_resources, 20)
                .unwrap()
                .name
                == "other"
        );
        assert!(matches!(
            cluster.find_partition(Some("other"), &cpu_resources, 21),
            Err(Error::PartitionNotFound(_))
        ));
    }

    #[test]
    #[parallel]
    fn open_no_file() {
        setup();
        let temp = TempDir::new().unwrap().child("clusters.json");
        let clusters = Configuration::open_from_path(temp.path().into()).expect("valid clusters");
        assert_eq!(clusters, Configuration::built_in());
    }

    #[test]
    #[parallel]
    fn open_empty_file() {
        setup();
        let temp = TempDir::new().unwrap().child("clusters.json");
        temp.write_str("").unwrap();
        let clusters = Configuration::open_from_path(temp.path().into()).expect("valid clusters");
        assert_eq!(clusters, Configuration::built_in());
    }

    #[test]
    #[parallel]
    fn minimal_cluster() {
        setup();
        let temp = TempDir::new().unwrap().child("clusters.json");
        temp.write_str(
            r#"
[[cluster]]
name = "a"
identify.always = true
scheduler = "bash"

[[cluster.partition]]
name = "b"
"#,
        )
        .unwrap();
        let clusters = Configuration::open_from_path(temp.path().into()).unwrap();
        let built_in_clusters = Configuration::built_in();
        assert_eq!(clusters.cluster.len(), 1 + built_in_clusters.cluster.len());

        let cluster = clusters.cluster.first().unwrap();
        assert_eq!(cluster.name, "a");
        assert_eq!(cluster.identify, IdentificationMethod::Always(true));
        assert_eq!(cluster.scheduler, SchedulerType::Bash);
        assert_eq!(
            cluster.partition,
            vec![Partition {
                name: "b".into(),
                ..Partition::default()
            }]
        );
    }

    #[test]
    #[parallel]
    fn maximal_cluster() {
        setup();
        let temp = TempDir::new().unwrap().child("clusters.json");
        temp.write_str(
            r#"
[[cluster]]
name = "a"
identify.by_environment = ["b", "c"]
scheduler = "slurm"

[[cluster.partition]]
name = "d"
maximum_cpus_per_job = 2
require_cpus_multiple_of = 4
warn_cpus_multiple_of = 4
memory_per_cpu = "e"
minimum_gpus_per_job = 8
maximum_gpus_per_job = 16
require_gpus_multiple_of = 32
warn_gpus_multiple_of = 32
memory_per_gpu = "f"
cpus_per_node = 10
gpus_per_node = 11
account_suffix = "-gpu"
"#,
        )
        .unwrap();
        let clusters = Configuration::open_from_path(temp.path().into()).unwrap();
        let built_in_clusters = Configuration::built_in();
        assert_eq!(clusters.cluster.len(), 1 + built_in_clusters.cluster.len());

        let cluster = clusters.cluster.first().unwrap();
        assert_eq!(cluster.name, "a");
        assert_eq!(
            cluster.identify,
            IdentificationMethod::ByEnvironment("b".into(), "c".into())
        );
        assert_eq!(cluster.scheduler, SchedulerType::Slurm);
        assert_eq!(
            cluster.partition,
            vec![Partition {
                name: "d".into(),

                maximum_cpus_per_job: Some(2),
                require_cpus_multiple_of: Some(4),
                warn_cpus_multiple_of: Some(4),
                memory_per_cpu: Some("e".into()),
                minimum_gpus_per_job: Some(8),
                maximum_gpus_per_job: Some(16),
                require_gpus_multiple_of: Some(32),
                warn_gpus_multiple_of: Some(32),
                memory_per_gpu: Some("f".into()),
                prevent_auto_select: false,
                cpus_per_node: Some(10),
                gpus_per_node: Some(11),
                account_suffix: Some("-gpu".into()),
            }]
        );
    }
}
