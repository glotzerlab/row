# show directories

Usage:
```bash
row show directories [OPTIONS] <ACTION> [DIRECTORIES]
```

`row show directories` lists each selected directory with its
[status](../../guide/concepts/status.md) and scheduler job ID (when submitted) for the
given `<ACTION>`. You can also show elements from the directory's value, accessed by
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
echo "dir1" | row show directories action -
```

## `[OPTIONS]`

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
  row show directories one
  ```
* Show the directory value element `/value`:
  ```bash
  row show directories action --value=/value
  ```
* Show specific directories:
  ```bash
  row show directories action directory1 directory2
  ```
* Show eligible directories
  ```bash
  row show directories action --eligible
  ```
