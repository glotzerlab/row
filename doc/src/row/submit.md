# submit

Usage:
```bash
row submit [OPTIONS] [DIRECTORIES]
```

`row submit` submits jobs to the scheduler. First it determines the
[status](../guide/concepts/status.md) of all the given directories for the selected
actions. Then it forms [groups](../workflow/action/group.md) and submits one job for
each group. Pass `--dry-run` to see what will be submitted.

## `[DIRECTORIES]`

Submit eligible jobs for these specific directories. By default, **row** submits
the entire workspace. For example:
```bash
row submit dir1 dir2 dir3
```

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
