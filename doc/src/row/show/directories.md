# show directories

Usage:
```bash
row show directories [OPTIONS] [DIRECTORIES]
```

`row show directories` lists each selected directory.

When provided an action, `row show directories` also shows each directory's
[status](../../guide/concepts/status.md) and scheduler job ID (when submitted) for the
given action. You can also show elements from the directory's value, accessed by
[JSON pointer](../../guide/concepts/json-pointers.md). Blank lines separate
[groups](../../workflow/action/group.md).

By default, `row show status` displays directories with any status. Set one or more
of `--completed`, `--submitted`, `--eligible`, and `--waiting` to show specific
directories that have specific statuses.

## `[DIRECTORIES]`

List these specific directories. By default, **row** shows all directories that match
the action's [include condition](../../workflow/action/group.md#include)

Pass a single `-` to read the directories from stdin (separated by newlines):
```bash
echo "dir1" | row show directories [OPTIONS] -
```

## `[OPTIONS]`

### `--action`

Select directories that are included by the provided action.

### `--completed`

Show directories with the *completed* status.

### `--eligible`

Show directories with the *eligible* status.

### `--n-groups`

(also: `-n`)

Limit the number of groups displayed.

### `--no-header`

Hide the header in the output.

### `--no-separate-groups`

Do not write blank lines between groups.

### `--short`

Show only the directory names.

### `--submitted`

Show directories with the *submitted* status.

### `--value`

Pass `--value <JSON POINTER>` to add a column of output that shows an element of the
directory's value as a JSON string. You may pass `--value` multiple times to include
additional columns.

### `--waiting`

Show directories with the *waiting* status.

## Examples

* Show all the directories for action `one`:
  ```bash
  row show directories --action one
  ```
* Show the directory value element `/value`:
  ```bash
  row show directories --action action --value=/value
  ```
* Show specific directories:
  ```bash
  row show directories --action action directory1 directory2
  ```
* Show eligible directories
  ```bash
  row show directories --action action --eligible
  ```
* Show the names of all directories
  ```bash
  row show directories
  ```
* Show the names of eligible directories
  ```bash
  row show directories --action action --eligible --short
  ```
