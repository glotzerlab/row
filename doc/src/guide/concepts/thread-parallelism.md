# Thread parallelism

In **row**, each **process** may execute many **threads** in parallel. The
**scheduler** gives each **thread** a dedicated **CPU core** on the *same physical*
**CPU node** as the host **process**.

If you are familiar with operating system concepts, **row**/**scheduler** **threads**
may be realized both by **OS threads** and **OS processes**. Request
`threads_per_process` to schedule resources when your command uses:

* OpenMP.
* a library that spawns threads, such as *some* **numpy** builds.
* operating system threads in general.
* the Python **[multiprocessing]** library.
* or otherwise executes many processes or threads (e.g. `make`, `ninja`).

## Passing the number of threads to your application/library

When launching OpenMP applications, set `launchers = ["openmp"]` and **row** will
set the `OMP_NUM_THREADS` environment variable accordingly.

For all other cases, refer to the documentation of your application or library. Most
provide some way to set the number of threads/processes. Use the environment variable
`ACTION_THREADS_PER_PROCESS` to ensure that the number of executed threads matches that
requested.

## Processing multiple directories in parallel with Python multiprocessing

You can execute serial actions on many directories in parallel with the
**[multiprocessing]** package in **Python**. Here is an example using **signac**:

```python
{{#include multiprocessing-example.py:action}}
```

Pair this with a workflow resource request like this to process many directories in
parallel.

```toml
[action.group]
maximum_size = 32
[action.resources]
threads_per_process = 8
walltime.per_directory = "01:00:00"
```

> Note: This action *always* requests the same number of CPU cores - even when there are
> **fewer directories**. You must ensure that you only submit this action on groups of
> directories larger than or equal to `threads_per_process`.
> [Process parallelism](process-parallelism.md) describes a method that automatically
> scales the resource request with the group size.

[multiprocessing]: https://docs.python.org/3/library/multiprocessing.html
