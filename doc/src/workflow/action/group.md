# group

`action.group`: **table** - Control how **row** forms the groups of directories
that it submits.

Example:
```toml
[action.group]
include = [["/subproject", "equal_to", "project_one"]]
sort_by = ["/value"]
split_by_sort_key = true
maximum_size = 16
submit_whole = true
reverse_sort = true
```

> Note: You may omit `[action.group]` entirely.

Execute [`row show directories <ACTION>`](../../row/show/directories.md) to display the
groups of directories included in a given action.

## include

`action.group.include`: **array** of **arrays** - Define a set of conditions that must
all be true for a directory to be included in this group. Each condition is an **array**
of three elements: The *JSON pointer*, *the operator*, and the *operand*. The [JSON
pointer](../../guide/concepts/json-pointers.md) points to a specific element
from the directory's value. The operator may be `"less_than"`, `"greater_than"`, or
`"equal_to"`.

For example, select all directories where a value is in the given range:
```toml
include = [["/value, "less_than", 0.9], ["/value", "greater_than", 0.2]]
```
Choose directories where an array element is equal to a specific value:
```toml
include = [["/array/1", "equal_to", 12]]
```
Match against strings:
```toml
include = [["/map/name", "equal_to", "string"]]
```
Compare by array:
```toml
include = [["/array", "eqal_to", [1, "string", 14.0]]]
```

Both operands **must** have the same data type. The JSON pointer must be present in the
value of **every** directory.

When you omit `include`, **row** includes **all** directories in the workspace.

> Note: **Row** compares arrays *lexicographically*.

<div class="warning">
JSON Objects (also known as maps or dictionaries) are not comparable. You must use
pointers to specific keys in objects.
</div>

## sort_by

`action.group.sort_by`: **array** of **strings** - An array of
[JSON pointers](../../guide/concepts/json-pointers.md) to elements of each directory's
value. **Row** will sort directories matched by `include` by these quantities
*lexicographically*. For example,
```toml
action.group.sort_by = ["/a", "/b"]
```
sorts by `"/a"` first, then by `"/b"` when `"/a"` is equal.

Each JSON pointer must be present in the value of **every** directory matched by
`include`. While each array element may be a different type (e.g. `"/a"` could be a
string and `"/b"` a number), a given array element **must** be the same type across all
matched directories.

When you omit `sort_by`, **row** sorts the directories by name.

## reverse_sort

`action.group.reverse_sort`: **boolean** - Set to `true` to sort in *descending* order.
By default, **row** sorts in *ascending* order.

## split_by_sort_key

`action.group.split_by_sort_key`: **boolean** - Set to `true` to split the sorted
directories into groups where the sort keys are identical.

For example, the (directory name, value) pairs: `[(dir1, 1), (dir2, 1), (dir3, 1),
(dir4, 2), (dir5, 3), (dir6, 3)]`

would split into the groups:
* `[(dir1, 1), (dir2, 1), (dir3, 1)]`
* `[(dir4, 2)]`
* `[(dir5, 3), (dir6, 3)]`

When omitted, `split_by_sort_key` defaults to `false` and *all* directories matched
by `include` are placed in a single group.

## maximum_size

`action.group.maximum_size`: **integer** - Maximum size of a group.

**Row** further splits the groups into smaller groups up to the given `maximum_size`.
When the number of directories is not evenly divisible by `maximum_size`, **row**
creates the first **n** groups with `maximum_size` elements and places one remainder
group at the end.

For example, with `maximum_size = 2` the directories: `[dir1, dir2, dir3, dir4, dir5]`

would split into the groups:
* `[dir1, dir2]`
* `[dir3, dir4]`
* `[dir5]`

When omitted, there is no maximum group size.

When `maximum_size` is set **and** `split_by_sort_key` is `true`, **row** first splits
by the sort key, then splits the resulting groups according to `maximum_size`.

## submit_whole

`action.group.submit_whole`: **boolean** - Set to `true` to require that
[`row submit`](../../row/submit.md) must always submit *whole* groups. Normally,
*submit* forms groups from the *eligible* directories (those not already completed and
not submitted). When `submit_whole` is `true`, *submit* will issue an error if such an
eligible group is not present in the set of all groups matched by this directory.

For example, set `submit_whole` to `true` when your action computes an average or
otherwise summarizes the group. This will prevent you from accidentally averaging
only a portion of the group.

When omitted, `submit_whole` defaults to `false`.
