# Built-in launchers

**Row** includes built-in support for OpenMP and MPI via the launchers `"openmp"`
and `"mpi"`. These have been tested on the [built-in clusters](../clusters/built-in.md).
You may need to add new configurations for your specific cluster or adjust the `none`
launcher to match your system. Execute [`row show launchers`](../row/show/launchers.md)
to see the current launcher configuration.

## Hybrid OpenMP/MPI

When using OpenMP/MPI hybrid applications, place `"openmp"` first in the list of
launchers (`launchers = ["openmp", "mpi"]`) to generate the appropriate command:
```bash
OMP_NUM_THREADS=T srun --ntasks=N --cpus-per-task=T command $directory
```
