# Requesting resources with row

## Overview

This section shows how you can use **row** to automatically generate **job scripts**
that request the **resources** your actions need to execute. This guide cannot
anticipate what codes you use, so it demonstrates commonly used patterns without
providing fully working examples.

> Note: For a complete description, see
[resources in workflow.toml](../../workflow/action/resources.md).

## Execute directories on 1 CPU in serial

When you execute a script on a **group** of directories on 1 CPU in serial, request 1
task *per job submission* (`processes.per_submission`) and provide the total time needed
to process a single directory in *HH:MM:SS* format (`walltime.per_directory`).

```toml
[[action]]
name = "action"
command = serial_command {directory}

[action.resources]
processes.per_submission = 1
walltime.per_directory = "00:10:00"
```

When submitting a given **group**, **row** will compute the total `--time` request
from `walltime.per_submission * group_size`.

## Execute a threaded (or multiprocessing) computation on 8 CPU cores

For commands that execute with multiple threads (or multiple processes *on the same
node*), request `threads_per_process`.

```toml
[[action]]
name = "action"
command = threaded_command {directory}

[action.resources]
processes.per_submission = 1
threads_per_process = 8
walltime.per_directory = "00:10:00"
```

## Execute with OpenMP parallelism

The same as above, but this example will place `OMP_NUM_THREADS=<threads_per_process>`
before the command:

```toml
[[action]]
name = "action"
command = threaded_command {directory}
launchers = ["openmp"]

[action.resources]
processes.per_submission = 1
threads_per_process = 8
walltime.per_directory = "00:10:00"
```


# Execute MPI parallel calculations

To launch MPI enabled applications, request more than one *process* and the
`"mpi"` launcher. `launchers = ["mpi"]` will add the appropriate MPI launcher prefix
before your command (e.g. `srun --ntasks 16 parallel_application $directory`).

```toml
[[action]]
name = "action"
command = parallel_application {directory}
launchers = ["mpi"]

[action.resources]
processes.per_submission = 16
walltime.per_directory = "04:00:00"
```

> Note: You should **not** manually insert `srun`, `mpirun` or other launcher commands.
> Use `launchers = ["mpi"]`. Configure [`launchers.toml`](../../launchers/index.md)
> if the default does not function correctly on your system.

# Process many directories in parallel with MPI

Structure your action script to split the MPI communicator and execute on each directory
based on the partition index.

Unlike in previous examples, this one needs requests 4 ranks *per directory*
(`processes.per_directory`). These calculations run in parallel, so the walltime is
fixed *per submission* (`walltime.per_submission`).
```toml
[[action]]
name = "action"
command = partitioned_application {directories}
launchers = ["mpi"]

[action.resources]
processes.per_directory = 4
walltime.per_submission = "01:00:00"
```

## Execute a GPU accelerated application

Request `gpus_per_process` to allocate a GPU node.

```toml
[[action]]
name = "action"
command = gpu_application {directory}

[action.resources]
processes.per_submission = 1
gpus_per_process = 1
walltime.per_directory = "08:00:00"
```

> Note: You can of course combine processes, threads, and GPUs all in the same
> submission, provided you **know** that your application will make full use of all
> requested resources.

## Next steps

You now have some idea how to instruct **row** to generate resource requests, you are
ready to **submit** your jobs to the **cluster**.
