# show jobs

Usage:
```bash
row show jobs [OPTIONS] [DIRECTORIES]
```

`row show jobs` lists submitted jobs that execute a matching action on any of the
provided directories.

## `[DIRECTORIES]`

List jobs that execute an action on one or more of the given directories. By default,
**row** shows jobs executing an action on any directory.

Pass a single `-` to read the directories from stdin (separated by newlines):
```bash
echo "dir1" | row show jobs [OPTIONS] -
```

## `[OPTIONS]`

### `--action`

(also: `-a`)

Set `--action <pattern>` to show only jobs that mach the given pattern by name.
By default, **row** shows jobs executing any action. `<pattern>` is a wildcard pattern.

### `--no-header`

Hide the header in the output.

### `--short`

Show only the job IDs.

## Examples

* Show all jobs:
  ```bash
  row show jobs
  ```
* Show jobs that execute actions on any of the given directories:
  ```bash
  row show jobs directory1 directory
  ```
* Show jobs that execute the action 'one':
  ```bash
  row show jobs --action one
  ```
* Show jobs that execute an action starting with 'analyze':
  ```bash
  row show jobs --action 'analyze*'
  ```
* Cancel SLURM jobs executing action 'two':
  ```bash
  row show jobs --action two --short | xargs scancel
  ```
* Show SLURM job status for the current workspace:
  ```bash
  squeue --me -j $(row show jobs --short | paste -sd,)
  ```
