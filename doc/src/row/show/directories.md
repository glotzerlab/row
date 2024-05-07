# show directories

Usage:
```bash
row show directories [OPTIONS] <ACTION> [DIRECTORIES]
```

Example output:
```plaintext
Directory Status        Job /value
dir1      submitted 1432876    0.9
dir2      submitted 1432876    0.8
dir3      submitted 1432876    0.7

dir4      completed            0.5
dir5      completed            0.4
dir6      completed            0.3
```

`row show directories` lists each selected directory with its
[status](../../guide/concepts/status.md) and scheduler job ID (when submitted) for the
given `<ACTION>`. You can also show elements from the directory's value, accessed by
[JSON pointer](../../guide/concepts/json-pointers.md). Blank lines separate
[groups](../../workflow/action/group.md).

## `[DIRECTORIES]`

List these specific directories. By default, **row** shows all directories that match
the action's [include condition](../../workflow/action/group.md#include)
For example:
```bash
row show directories action dir1 dir2 dir3
```

Pass a single `-` to read the directories from stdin (separated by newlines):
```bash
echo "dir1" | row show directories action -
```

## `[OPTIONS]`

### `--no-header`

Hide the header in the output.

### `--no-separate-groups`

Do not write blank lines between groups.

### `--value`

Pass `--value <JSON POINTER>` to add a column of output that shows an element of the
directory's value as a JSON string. You may pass `--value` multiple times to include
additional columns.
