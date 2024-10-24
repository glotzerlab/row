// Copyright (c) 2024 The Regents of the University of Michigan.
// Part of row, released under the BSD 3-Clause License.

use std::collections::HashMap;

use crate::cluster::{self, Cluster, IdentificationMethod, Partition, SchedulerType};
use crate::launcher::{self, Launcher};

pub(crate) trait BuiltIn {
    fn built_in() -> Self;
}

impl BuiltIn for launcher::Configuration {
    /// Construct the built-in launchers
    ///
    fn built_in() -> Self {
        let mut result = Self {
            launchers: HashMap::with_capacity(2),
        };

        let mut openmp = HashMap::with_capacity(1);
        openmp.insert(
            "default".into(),
            Launcher {
                threads_per_process: Some("OMP_NUM_THREADS=".into()),
                ..Launcher::default()
            },
        );

        result.launchers.insert("openmp".into(), openmp);

        let mut mpi = HashMap::with_capacity(3);
        mpi.insert(
            "default".into(),
            Launcher {
                executable: Some("srun".into()),
                processes: Some("--ntasks=".into()),
                threads_per_process: Some("--cpus-per-task=".into()),
                gpus_per_process: Some("--tres-per-task=gres/gpu:".into()),
            },
        );

        mpi.insert(
            "anvil".into(),
            Launcher {
                executable: Some("srun --mpi=pmi2".into()),
                processes: Some("--ntasks=".into()),
                threads_per_process: Some("--cpus-per-task=".into()),
                gpus_per_process: Some("--tres-per-task=gres/gpu:".into()),
            },
        );

        mpi.insert(
            "none".into(),
            Launcher {
                executable: Some("mpirun".into()),
                processes: Some("-n ".into()),
                ..Launcher::default()
            },
        );

        result.launchers.insert("mpi".into(), mpi);

        result
    }
}

fn andes() -> Cluster {
    ////////////////////////////////////////////////////////////////////////////////////////
    // OLCF Andes
    Cluster {
        name: "andes".into(),
        identify: IdentificationMethod::ByEnvironment("LMOD_SYSTEM_NAME".into(), "andes".into()),
        scheduler: SchedulerType::Slurm,
        submit_options: Vec::new(),
        partition: vec![
            // Auto-detected partitions: batch
            Partition {
                name: "batch".into(),
                maximum_gpus_per_job: Some(0),
                warn_cpus_not_multiple_of: Some(32),
                cpus_per_node: Some(32),
                ..Partition::default()
            },
        ],
    }
}

fn anvil() -> Cluster {
    ////////////////////////////////////////////////////////////////////////////////////////
    // Purdue Anvil
    Cluster {
        name: "anvil".into(),
        identify: IdentificationMethod::ByEnvironment("RCAC_CLUSTER".into(), "anvil".into()),
        scheduler: SchedulerType::Slurm,
        submit_options: Vec::new(),
        partition: vec![
            // Auto-detected partitions: shared | wholenode | gpu
            Partition {
                name: "shared".into(),
                maximum_cpus_per_job: Some(127),
                maximum_gpus_per_job: Some(0),
                ..Partition::default()
            },
            Partition {
                name: "wholenode".into(),
                require_cpus_multiple_of: Some(128),
                maximum_gpus_per_job: Some(0),
                ..Partition::default()
            },
            Partition {
                name: "gpu".into(),
                minimum_gpus_per_job: Some(1),
                gpus_per_node: Some(4),
                ..Partition::default()
            },
            // The following partitions may only be selected manually.
            Partition {
                name: "wide".into(),
                require_cpus_multiple_of: Some(128),
                maximum_gpus_per_job: Some(0),
                prevent_auto_select: true,
                ..Partition::default()
            },
            Partition {
                name: "highmem".into(),
                maximum_gpus_per_job: Some(0),
                prevent_auto_select: true,
                ..Partition::default()
            },
            Partition {
                name: "debug".into(),
                maximum_gpus_per_job: Some(0),
                prevent_auto_select: true,
                ..Partition::default()
            },
            Partition {
                name: "gpu-debug".into(),
                minimum_gpus_per_job: Some(1),
                prevent_auto_select: true,
                ..Partition::default()
            },
        ],
    }
}

fn delta() -> Cluster {
    ////////////////////////////////////////////////////////////////////////////////////////
    // NCSA delta
    Cluster {
        name: "delta".into(),
        identify: IdentificationMethod::ByEnvironment("LMOD_SYSTEM_NAME".into(), "Delta".into()),
        scheduler: SchedulerType::Slurm,
        submit_options: vec!["--constraint=\"scratch\"".to_string()],
        partition: vec![
            // Auto-detected partitions: cpu | gpuA100x4
            Partition {
                name: "cpu".into(),
                maximum_gpus_per_job: Some(0),
                cpus_per_node: Some(128),
                memory_per_cpu: Some("1970M".into()),
                account_suffix: Some("-cpu".into()),
                ..Partition::default()
            },
            Partition {
                name: "gpuA100x4".into(),
                minimum_gpus_per_job: Some(1),
                memory_per_gpu: Some("62200M".into()),
                gpus_per_node: Some(4),
                account_suffix: Some("-gpu".into()),
                ..Partition::default()
            },
            // The following partitions may only be selected manually.
            Partition {
                name: "gpuA100x8".into(),
                minimum_gpus_per_job: Some(1),
                memory_per_gpu: Some("256000M".into()),
                gpus_per_node: Some(8),
                account_suffix: Some("-gpu".into()),
                prevent_auto_select: true,
                ..Partition::default()
            },
            Partition {
                name: "gpuA40x4".into(),
                minimum_gpus_per_job: Some(1),
                memory_per_gpu: Some("62200M".into()),
                gpus_per_node: Some(4),
                account_suffix: Some("-gpu".into()),
                prevent_auto_select: true,
                ..Partition::default()
            },
            Partition {
                name: "gpuMI100x8".into(),
                minimum_gpus_per_job: Some(1),
                memory_per_gpu: Some("256000M".into()),
                gpus_per_node: Some(8),
                account_suffix: Some("-gpu".into()),
                prevent_auto_select: true,
                ..Partition::default()
            },
        ],
    }
}

fn frontier() -> Cluster {
    ////////////////////////////////////////////////////////////////////////////////////////
    // OLCF Frontier
    Cluster {
        name: "frontier".into(),
        identify: IdentificationMethod::ByEnvironment("LMOD_SYSTEM_NAME".into(), "frontier".into()),
        scheduler: SchedulerType::Slurm,
        submit_options: vec!["--constraint=\"nvme\"".to_string()],
        partition: vec![
            // Auto-detected partitions: batch
            Partition {
                name: "batch".into(),
                warn_gpus_not_multiple_of: Some(8),
                gpus_per_node: Some(8),
                ..Partition::default()
            },
        ],
    }
}

fn greatlakes() -> Cluster {
    ////////////////////////////////////////////////////////////////////////////////////////
    // Great Lakes
    Cluster {
        name: "greatlakes".into(),
        identify: IdentificationMethod::ByEnvironment("CLUSTER_NAME".into(), "greatlakes".into()),
        scheduler: SchedulerType::Slurm,
        submit_options: Vec::new(),
        partition: vec![
            // Auto-detected partitions: standard | gpu_mig40,gpu | gpu.
            Partition {
                name: "standard".into(),
                maximum_gpus_per_job: Some(0),
                cpus_per_node: Some(36),
                memory_per_cpu: Some("5G".into()),
                ..Partition::default()
            },
            Partition {
                name: "gpu_mig40,gpu".into(),
                minimum_gpus_per_job: Some(1),
                maximum_gpus_per_job: Some(1),
                memory_per_gpu: Some("60G".into()),
                ..Partition::default()
            },
            Partition {
                name: "gpu".into(),
                minimum_gpus_per_job: Some(1),
                memory_per_gpu: Some("60G".into()),
                // cannot set gpus_per_node, the partition is heterogeneous
                ..Partition::default()
            },
            // The following partitions may only be selected manually.
            Partition {
                name: "gpu_mig40".into(),
                minimum_gpus_per_job: Some(1),
                memory_per_gpu: Some("125G".into()),
                prevent_auto_select: true,
                ..Partition::default()
            },
            Partition {
                name: "spgpu".into(),
                minimum_gpus_per_job: Some(1),
                memory_per_gpu: Some("47000M".into()),
                prevent_auto_select: true,
                ..Partition::default()
            },
            Partition {
                name: "largemem".into(),
                maximum_gpus_per_job: Some(0),
                prevent_auto_select: true,
                ..Partition::default()
            },
            Partition {
                name: "standard-oc".into(),
                maximum_gpus_per_job: Some(0),
                cpus_per_node: Some(36),
                memory_per_cpu: Some("5G".into()),
                prevent_auto_select: true,
                ..Partition::default()
            },
            Partition {
                name: "debug".into(),
                maximum_gpus_per_job: Some(0),
                cpus_per_node: Some(36),
                memory_per_cpu: Some("5G".into()),
                prevent_auto_select: true,
                ..Partition::default()
            },
        ],
    }
}

fn none() -> Cluster {
    // Fallback none cluster.
    Cluster {
        name: "none".into(),
        identify: IdentificationMethod::Always(true),
        scheduler: SchedulerType::Bash,
        submit_options: Vec::new(),
        partition: vec![Partition {
            name: "none".into(),
            ..Partition::default()
        }],
    }
}

impl BuiltIn for cluster::Configuration {
    fn built_in() -> Self {
        let cluster = vec![andes(), anvil(), delta(), frontier(), greatlakes(), none()];

        cluster::Configuration { cluster }
    }
}
