# show launchers

Usage:
```bash
row show launchers [OPTIONS]
```

Print the [launchers](../../launchers/index.md) defined for the current cluster (or the
cluster given in `--cluster`).

Example output:
```
[mpi]
executable = "mpirun"
processes = "-n "

[openmp]
threads_per_process = "OMP_NUM_THREADS="
```

This includes the user-provided launchers in [`launchers.toml`](../../launchers/index.md)
and the built-in launchers (or the user-provided overrides).

## `[OPTIONS]`

### `--all`

Show the launcher configurations for all clusters.
