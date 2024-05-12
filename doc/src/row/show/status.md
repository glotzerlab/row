# show status

Usage:
```bash
row show status [OPTIONS] [DIRECTORIES]
```

`row show status` prints a summary of all directories in the workspace.
The summary includes the number of directories in each
[status](../../guide/concepts/status.md) and an estimate of the remaining cost in either
CPU-hours or GPU-hours based on the number of submitted, eligible, and waiting jobs and
the [resources used by the action](../../workflow/action/resources.md).

## `[DIRECTORIES]`

Show the status of these specific directories. By default, **row** shows the status for
the entire workspace.

Pass a single `-` to read the directories from stdin (separated by newlines):
```bash
echo "dir1" | row show status -
```

## `[OPTIONS]`

### `--action`

(also: `-a`)

Set `--action <pattern>` to choose which actions to display by name. By default, **row**
shows the status of all actions. `<pattern>` is a wildcard pattern.

### `--no-header`

Hide the header in the output.

## Examples

* Show the status of the entire workspace:
  ```bash
  row show status
  ```
* Show the status of a specific action:
  ```bash
  row show status --action=action
  ```
* Show the status of all action names that match a wildcard pattern:
  ```bash
  row show status --action='project*'
  ```
* Show the status of specific directories in the workspace:
  ```bash
  row show status directory1 directory2
  ```
