# Process parallelism

In **row**, a *process* is one of many copies of an executable program. Copies may
(or may not) execute on different physical **compute nodes**.

Neither **row** nor the **job scheduler** can execute more than **one process per
job**. When you request more than one **process** (via `processes.per_directory` or
`processes.per_submission`), you must pair it with a **launcher** that can execute those
processes: e.g. `launcher = ["mpi"]`.

> In other words: The **scheduler** reserves enough **compute nodes** to satisfy
> the requested resources, but the **launcher** is responsible for executing those
> **processes**.

At this time **MPI** is the only **process** launcher that **row** supports. You can
configure additional launchers in [`launchers.toml`](../../launchers/index.md).

Use **MPI** parallelism to launch:
* MPI-enabled applications on one directory (`processes.per_submission = N`,
  `group.maximum_size = 1`).
* MPI-enabled applications on many directories in serial
  (`processes.per_submission = N`).
* Serial applications on many directories in parallel (`processes.per_directory = 1`).
  For example, use `mpi4py` and execute Python functions on directories indexed by rank.
* MPI-enable applications on many directories in parallel
  (`processes.per_directory = N`). Instruct your application to *partition* the MPI
  communicator.

TODO: Provide a concrete example using HOOMD

TODO: Provide a concrete example using mpi4py
