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
* the Python **multiprocessing** library.
* or otherwise executes many processes or threads (e.g. `make`, `ninja`).

## Passing the number of threads to your application/library

When launching OpenMP applications, set `launchers = ["openmp"]` and **row** will
set the `OMP_NUM_THREADS` environment variable accordingly.

For all other cases, refer to the documentation of your application or library. Most
provide some way to set the number of threads/processes. Use the environment variable
`ACTION_THREADS_PER_PROCESS` to ensure that the number of executed threads matches that
requested.

TODO: Provide a concrete example using the Python multiprocessing library
