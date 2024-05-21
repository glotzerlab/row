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
  For example, use **[mpi4py]** and execute Python functions on directories indexed by
  rank (example below).
* MPI-enable applications on many directories in parallel
  (`processes.per_directory = N`). Instruct your application to *partition* the MPI
  communicator (HOOMD-blue example below).

[mpi4py]: https://mpi4py.readthedocs.io

## Processing multiple directories in parallel with Python and **mpi4py**.

You can execute serial actions on many directories in parallel using **[mpi4py]**.
Use the communicators **rank** to index into the array of directories. Here is an
example using **signac**:

```python
{{#include mpi4py-example.py:action}}
```

Pair this with a workflow action like this to process many directories in parallel.

```toml
[[action]]
launchers = ["mpi"]
[action.group]
maximum_size = 128
[action.resources]
processes.per_directory = 1
walltime.per_submission = "08:00:00"
```

> Note: Adjust `maximum_size` to control how many directories are submitted per job.

## Executing multiple MPI decomposed simulations in parallel with **HOOMD-blue**.

Say your individual HOOMD-blue simulations scale well to 4 cores, and you have many
directories you want to execute in parallel. You can configure this action similar
to **mpi4py** above:

```toml
[[action]]
launchers = ["mpi"]
[action.group]
maximum_size = 128
[action.resources]
processes.per_directory = 4
walltime.per_submission = "08:00:00"
```

In your Python code, use the `ranks_per_partition` flag to HOOMD-blue's `Communicator`
to assign 4 ranks (processes) to each partition (directory). Then use the partition
index into the array of directories. Here is an example using **signac**:

```python
{{#include hoomd-example.py:action}}
```
