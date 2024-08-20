# submit

Usage:
```bash
row submit [OPTIONS] [DIRECTORIES]
```

`row submit` submits jobs to the scheduler. First it determines the
[status](../guide/concepts/status.md) of all the given directories for the selected
actions. Then it forms [groups](../workflow/action/group.md) and submits one job for
each group. Pass `--dry-run` to see the script(s) that will be submitted. Execute
```
row show directories --action action --eligible
```
to see the specific directory groups that will be submitted.

## `[DIRECTORIES]`

Submit eligible jobs for these specific directories. By default, **row** submits
the entire workspace.
## `[OPTIONS]`

### `--action`

(also: `-a`)

Set `--action <pattern>` to choose which actions to display by name. By default, **row**
submits the eligible jobs of all actions. `<pattern>` is a wildcard pattern.

### `--dry-run`

Print the scripts that would be submitted instead of submitting them.

### `-n`

Set `-n <N>` to limit the number of submitted jobs. **Row** will submit up to the first
`N` jobs.

### `--yes`

Skip the interactive confirmation.

## Examples

* Print the job script(s) that will be submitted:
  ```bash
  row submit --dry-run
  ```
* Submit jobs for all eligible directories:
  ```bash
  row submit
  ```
* Submit the first eligible job:
  ```bash
  row submit -n 1
  ```
* Submit jobs for a specific action:
  ```bash
  row submit --action=action
  ```
* Submit jobs for all actions that match a wildcard pattern:
  ```bash
  row submit --action='project*'
  ```
* Submit jobs on specific directories:
  ```bash
  row submit directory1 directory2
  ```
