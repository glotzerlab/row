// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use log::{debug, error, trace};
use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::io::Write;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{str, thread};

use crate::cluster::Cluster;
use crate::launcher::Launcher;
use crate::scheduler::bash::BashScriptBuilder;
use crate::scheduler::{ActiveJobs, Scheduler};
use crate::workflow::Action;
use crate::Error;

/// The `Slurm` scheduler constructs bash scripts and executes them with `sbatch`.
pub struct Slurm {
    cluster: Cluster,
    launchers: HashMap<String, Launcher>,
}

impl Slurm {
    /// Construct a new Slurm scheduler.
    pub fn new(cluster: Cluster, launchers: HashMap<String, Launcher>) -> Self {
        Self { cluster, launchers }
    }
}

/// Track the running squeue process
///
/// Or `None` when no process was launched.
pub struct ActiveSlurmJobs {
    squeue: Option<Child>,
    max_jobs: usize,
}

impl Scheduler for Slurm {
    fn make_script(&self, action: &Action, directories: &[PathBuf]) -> Result<String, Error> {
        let mut preamble = String::with_capacity(512);
        let mut user_partition = &None;

        write!(preamble, "#SBATCH --job-name={}", action.name()).expect("valid format");
        let _ = match directories.first() {
            Some(directory) => match directories.len() {
                0..=1 => writeln!(preamble, "-{}", directory.display()),
                _ => writeln!(
                    preamble,
                    "-{}+{}",
                    directory.display(),
                    directories.len() - 1
                ),
            },
            None => writeln!(preamble),
        };

        let _ = writeln!(preamble, "#SBATCH --output={}-%j.out", action.name());

        if let Some(submit_options) = action.submit_options.get(&self.cluster.name) {
            user_partition = &submit_options.partition;
        }

        // The partition
        let partition = self.cluster.find_partition(
            user_partition.as_deref(),
            &action.resources,
            directories.len(),
        )?;
        let _ = writeln!(preamble, "#SBATCH --partition={}", partition.name);

        // Resources
        let _ = writeln!(
            preamble,
            "#SBATCH --ntasks={}",
            action.resources.total_processes(directories.len())
        );

        if let Some(threads_per_process) = action.resources.threads_per_process {
            let _ = writeln!(preamble, "#SBATCH --cpus-per-task={threads_per_process}");
        }
        if let Some(gpus_per_process) = action.resources.gpus_per_process {
            let _ = writeln!(preamble, "#SBATCH --gpus-per-task={gpus_per_process}");

            if let Some(ref gpus_per_node) = partition.gpus_per_node {
                let n_nodes = (action.resources.total_gpus(directories.len()) + gpus_per_node - 1)
                    / gpus_per_node;
                let _ = writeln!(preamble, "#SBATCH --nodes={n_nodes}");
            }

            if let Some(ref mem_per_gpu) = partition.memory_per_gpu {
                let _ = writeln!(preamble, "#SBATCH --mem-per-gpu={mem_per_gpu}");
            }
        } else {
            if let Some(ref cpus_per_node) = partition.cpus_per_node {
                let n_nodes = (action.resources.total_cpus(directories.len()) + cpus_per_node - 1)
                    / cpus_per_node;
                let _ = writeln!(preamble, "#SBATCH --nodes={n_nodes}");
            }

            if let Some(ref mem_per_cpu) = partition.memory_per_cpu {
                let _ = writeln!(preamble, "#SBATCH --mem-per-cpu={mem_per_cpu}");
            }
        }

        // Slurm doesn't store times in seconds, so round up to the nearest minute.
        let total = action
            .resources
            .total_walltime(directories.len())
            .signed_total_seconds();
        let minutes = (total + 59) / 60;
        let _ = writeln!(preamble, "#SBATCH --time={minutes}");

        // Add global cluster submit options first so that users can override them.
        for option in &self.cluster.submit_options {
            let _ = writeln!(preamble, "#SBATCH {option}");
        }

        // Use provided submission options
        if let Some(submit_options) = action.submit_options.get(&self.cluster.name) {
            if let Some(ref account) = submit_options.account {
                if let Some(ref suffix) = partition.account_suffix {
                    let _ = writeln!(preamble, "#SBATCH --account={account}{suffix}");
                } else {
                    let _ = writeln!(preamble, "#SBATCH --account={account}");
                }
            }
            for option in &submit_options.custom {
                let _ = writeln!(preamble, "#SBATCH {option}");
            }
        }

        BashScriptBuilder::new(&self.cluster.name, action, directories, &self.launchers)
            .with_preamble(&preamble)
            .build()
    }

    fn submit(
        &self,
        workflow_root: &Path,
        action: &Action,
        directories: &[PathBuf],
        should_terminate: Arc<AtomicBool>,
    ) -> Result<Option<u32>, Error> {
        debug!("Submtitting '{}' with sbatch.", action.name());

        // output() below is blocking with no convenient way to interrupt it.
        // If the user pressed ctrl-C, let the current call to submit() finish
        // and update the cache. Assuming that there will be a next call to
        // submit(), that next call will return with an Interrupted error before
        // submitting the next job.
        if should_terminate.load(Ordering::Relaxed) {
            error!("Interrupted! Cancelling further job submissions.");
            return Err(Error::Interrupted);
        }

        let script = self.make_script(action, directories)?;

        let mut child = Command::new("sbatch")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("--parsable")
            .current_dir(workflow_root)
            .spawn()
            .map_err(|e| Error::SpawnProcess("sbatch".into(), e))?;

        let mut stdin = child.stdin.take().expect("Piped stdin");
        let input_thread = thread::spawn(move || {
            let _ = write!(stdin, "{script}");
        });

        trace!("Waiting for sbatch to complete.");
        let output = child
            .wait_with_output()
            .map_err(|e| Error::SpawnProcess("sbatch".into(), e))?;

        input_thread.join().expect("The thread should not panic");

        if output.status.success() {
            let job_id_string = str::from_utf8(&output.stdout).expect("Valid UTF-8 output");
            let job_id = job_id_string
                .trim_end_matches(char::is_whitespace)
                .parse::<u32>()
                .map_err(|_| Error::UnexpectedOutput("sbatch".into(), job_id_string.into()))?;
            Ok(Some(job_id))
        } else {
            let message = match output.status.code() {
                None => match output.status.signal() {
                    None => "sbatch was terminated by a unknown signal".to_string(),
                    Some(signal) => format!("sbatch was terminated by signal {signal}"),
                },
                Some(code) => format!("sbatch exited with code {code}"),
            };
            Err(Error::SubmitAction(action.name().into(), message))
        }
    }

    /// Use `squeue` to determine the jobs that are still present in the queue.
    ///
    /// Launch `squeue --jobs job0,job1,job2 -o "%A" --noheader` to determine which of
    /// these jobs are still in the queue.
    ///
    fn active_jobs(&self, jobs: &[u32]) -> Result<Box<dyn ActiveJobs>, Error> {
        if jobs.is_empty() {
            return Ok(Box::new(ActiveSlurmJobs {
                squeue: None,
                max_jobs: 0,
            }));
        }

        debug!("Checking job status with squeue.");

        let mut jobs_string = String::with_capacity(9 * jobs.len());
        // Prefix the --jobs argument with "1,". Otherwise, squeue reports an
        // error when a single job is not in the queue.
        if jobs.len() == 1 {
            jobs_string.push_str("1,");
        }
        for job in jobs {
            let _ = write!(jobs_string, "{job},");
        }

        let squeue = Command::new("squeue")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg("--jobs")
            .arg(&jobs_string)
            .args(["-o", "%A"])
            .arg("--noheader")
            .spawn()
            .map_err(|e| Error::SpawnProcess("squeue".into(), e))?;

        Ok(Box::new(ActiveSlurmJobs {
            squeue: Some(squeue),
            max_jobs: jobs.len(),
        }))
    }
}

impl ActiveJobs for ActiveSlurmJobs {
    fn get(self: Box<Self>) -> Result<HashSet<u32>, Error> {
        let mut result = HashSet::with_capacity(self.max_jobs);

        if let Some(squeue) = self.squeue {
            trace!("Waiting for squeue to complete.");
            let output = squeue
                .wait_with_output()
                .map_err(|e| Error::SpawnProcess("sbatch".into(), e))?;

            if !output.status.success() {
                let message = match output.status.code() {
                    None => match output.status.signal() {
                        None => "squeue was terminated by a unknown signal".to_string(),
                        Some(signal) => format!("squeue was terminated by signal {signal}"),
                    },
                    Some(code) => format!("squeue exited with code {code}"),
                };
                return Err(Error::ExecuteSqueue(
                    message,
                    str::from_utf8(&output.stderr).expect("Valid UTF-8").into(),
                ));
            }

            let jobs = str::from_utf8(&output.stdout).expect("Valid UTF-8");
            for job in jobs.lines() {
                result.insert(
                    job.parse()
                        .map_err(|_| Error::UnexpectedOutput("squeue".into(), job.into()))?,
                );
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::parallel;

    use crate::builtin::BuiltIn;
    use crate::cluster::{Cluster, IdentificationMethod, Partition, SchedulerType};
    use crate::launcher;
    use crate::workflow::{Processes, SubmitOptions};

    fn setup() -> (Action, Vec<PathBuf>, Slurm) {
        let action = Action {
            name: Some("action".to_string()),
            command: Some("command {directory}".to_string()),
            launchers: Some(vec!["mpi".into()]),
            ..Action::default()
        };

        let directories = vec![PathBuf::from("a"), PathBuf::from("b"), PathBuf::from("c")];
        let launchers = launcher::Configuration::built_in();
        let cluster = Cluster {
            name: "cluster".into(),
            identify: IdentificationMethod::Always(false),
            scheduler: SchedulerType::Slurm,
            partition: vec![Partition::default()],
            submit_options: Vec::new(),
        };

        let slurm = Slurm::new(cluster, launchers.by_cluster("cluster"));
        (action, directories, slurm)
    }

    #[test]
    #[parallel]
    fn default() {
        let (action, directories, slurm) = setup();
        let script = slurm
            .make_script(&action, &directories)
            .expect("valid script");
        println!("{script}");

        assert!(script.contains("#SBATCH --job-name=action"));
        assert!(script.contains("#SBATCH --ntasks=1"));
        assert!(!script.contains("#SBATCH --account"));
        assert!(script.contains("#SBATCH --partition=partition"));
        assert!(!script.contains("#SBATCH --cpus-per-task"));
        assert!(!script.contains("#SBATCH --gpus-per-task"));
        assert!(script.contains("#SBATCH --time=180"));
    }

    #[test]
    #[parallel]
    fn cluster_submit_options() {
        let (action, directories, mut slurm) = setup();
        slurm.cluster.submit_options = vec!["--option=value".to_string()];

        let script = slurm
            .make_script(&action, &directories)
            .expect("valid script");
        println!("{script}");

        assert!(script.contains("#SBATCH --job-name=action"));
        assert!(script.contains("#SBATCH --ntasks=1"));
        assert!(!script.contains("#SBATCH --account"));
        assert!(script.contains("#SBATCH --partition=partition"));
        assert!(!script.contains("#SBATCH --cpus-per-task"));
        assert!(!script.contains("#SBATCH --gpus-per-task"));
        assert!(script.contains("#SBATCH --time=180"));
        assert!(script.contains("#SBATCH --option=value"));
    }

    #[test]
    #[parallel]
    fn ntasks() {
        let (mut action, directories, slurm) = setup();

        action.resources.processes = Some(Processes::PerDirectory(3));

        let script = slurm
            .make_script(&action, &directories)
            .expect("valid script");
        println!("{script}");

        assert!(script.contains("#SBATCH --ntasks=9"));
    }

    #[test]
    #[parallel]
    fn account() {
        let (mut action, directories, slurm) = setup();

        action.submit_options.insert(
            "cluster".into(),
            SubmitOptions {
                account: Some("c".into()),
                ..SubmitOptions::default()
            },
        );

        let script = slurm
            .make_script(&action, &directories)
            .expect("valid script");
        println!("{script}");

        assert!(script.contains("#SBATCH --account=c"));
    }

    #[test]
    #[parallel]
    fn custom() {
        let (mut action, directories, slurm) = setup();

        action.submit_options.insert(
            "cluster".into(),
            SubmitOptions {
                custom: vec!["custom0".into(), "custom1".into()],
                ..SubmitOptions::default()
            },
        );

        let script = slurm
            .make_script(&action, &directories)
            .expect("valid script");
        println!("{script}");

        assert!(script.contains("#SBATCH custom0"));
        assert!(script.contains("#SBATCH custom1"));
    }

    #[test]
    #[parallel]
    fn cpus_per_task() {
        let (mut action, directories, slurm) = setup();

        action.resources.threads_per_process = Some(5);

        let script = slurm
            .make_script(&action, &directories)
            .expect("valid script");
        println!("{script}");

        assert!(script.contains("#SBATCH --cpus-per-task=5"));
    }

    #[test]
    #[parallel]
    fn gpus_per_task() {
        let (mut action, directories, slurm) = setup();

        action.resources.gpus_per_process = Some(5);

        let script = slurm
            .make_script(&action, &directories)
            .expect("valid script");
        println!("{script}");

        assert!(script.contains("#SBATCH --gpus-per-task=5"));
    }

    #[test]
    #[parallel]
    fn mem_per_cpu() {
        let (action, directories, _) = setup();

        let launchers = launcher::Configuration::built_in();
        let cluster = Cluster {
            name: "cluster".into(),
            identify: IdentificationMethod::Always(false),
            scheduler: SchedulerType::Slurm,
            submit_options: Vec::new(),
            partition: vec![Partition {
                memory_per_cpu: Some("a".into()),
                ..Partition::default()
            }],
        };

        let slurm = Slurm::new(cluster, launchers.by_cluster("cluster"));

        let script = slurm
            .make_script(&action, &directories)
            .expect("valid script");
        println!("{script}");

        assert!(script.contains("#SBATCH --mem-per-cpu=a"));
    }

    #[test]
    #[parallel]
    fn mem_per_gpu() {
        let (mut action, directories, _) = setup();

        let launchers = launcher::Configuration::built_in();
        let cluster = Cluster {
            name: "cluster".into(),
            identify: IdentificationMethod::Always(false),
            scheduler: SchedulerType::Slurm,
            submit_options: Vec::new(),
            partition: vec![Partition {
                memory_per_gpu: Some("b".into()),
                ..Partition::default()
            }],
        };

        let slurm = Slurm::new(cluster, launchers.by_cluster("cluster"));

        action.resources.gpus_per_process = Some(1);

        let script = slurm
            .make_script(&action, &directories)
            .expect("valid script");
        println!("{script}");

        assert!(script.contains("#SBATCH --mem-per-gpu=b"));
    }

    #[test]
    #[parallel]
    fn cpus_per_node() {
        let (mut action, directories, _) = setup();

        let launchers = launcher::Configuration::built_in();
        let cluster = Cluster {
            name: "cluster".into(),
            identify: IdentificationMethod::Always(false),
            scheduler: SchedulerType::Slurm,
            submit_options: Vec::new(),
            partition: vec![Partition {
                cpus_per_node: Some(10),
                ..Partition::default()
            }],
        };

        let slurm = Slurm::new(cluster, launchers.by_cluster("cluster"));

        action.resources.processes = Some(Processes::PerSubmission(81));

        let script = slurm
            .make_script(&action, &directories)
            .expect("valid script");
        println!("{script}");

        assert!(script.contains("#SBATCH --nodes=9"));
    }

    #[test]
    #[parallel]
    fn gpus_per_node() {
        let (mut action, directories, _) = setup();

        let launchers = launcher::Configuration::built_in();
        let cluster = Cluster {
            name: "cluster".into(),
            identify: IdentificationMethod::Always(false),
            scheduler: SchedulerType::Slurm,
            submit_options: Vec::new(),
            partition: vec![Partition {
                gpus_per_node: Some(5),
                ..Partition::default()
            }],
        };

        let slurm = Slurm::new(cluster, launchers.by_cluster("cluster"));

        action.resources.processes = Some(Processes::PerSubmission(81));
        action.resources.gpus_per_process = Some(1);

        let script = slurm
            .make_script(&action, &directories)
            .expect("valid script");
        println!("{script}");

        assert!(script.contains("#SBATCH --nodes=17"));
    }
}
