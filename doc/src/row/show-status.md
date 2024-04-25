# show status

Usage:
```bash
row show status [OPTIONS] [DIRECTORIES]
```

Example output:
```plaintext
Action Completed Submitted Eligible Waiting Remaining cost
one         1000       100      900       0  24K CPU-hours
two            0       200      800    1000   8K GPU-hours
```

For each action, the summary details the number of directories in each
[status](../guide/concepts/status.md).
`row show status` also estimates the remaining cost in either CPU-hours or GPU-hours
based on the number of submitted, eligible, and waiting jobs and the
[resources used by the action](../workflow/action/resources.md).

## `[DIRECTORIES]`

Show the status of these specific directories. By default, **row** shows the status for
the entire workspace. For example:
```bash
row show status dir1 dir2 dir3
```

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
