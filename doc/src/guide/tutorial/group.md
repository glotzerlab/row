# Grouping directories

## Overview

This section shows how you can assign a **value** to each directory and use that
**value** to form **groups** of directories. Each **job** executes an action's command
on a **group** of directories.

## Directory values

So far, this tutorial has demonstrated small toy examples. In practice, any workflow
that you need to execute on a cluster likely has hundreds or thousands of directories -
each with different parameters. You could try to encode these parameters into the
directory names, but *please don't*. This quickly becomes unmanageable. Instead, you
should include a [JSON](https://www.json.org) file in each directory that identifies
its **value**.

> Note: For pedagogical reasons, this next code block manually creates directory names
> and value files. In practice, you will likely find [signac](../python/signac.md) more
> convenient to work with - it will create the JSON files and directories for you with
> a cleaner syntax. This tutorial will cover **row** ↔ **signac** interoperation in a
> later section.

Create a new workflow project and place JSON files in each directory:
```bash
{{#include group.sh:init}}
```
The JSON files must all have the same name. Instruct **row** to read these files
with the `workspace.value_file` key in `workflow.toml`:

```toml
{{#include group-workflow1.toml}}
```

Once you create a directory with a **value** file, that value **MUST NOT CHANGE**. Think
of it this way: The results of your computations (the final contents of the directory)
are a mathematical *function* of the **value**. When you want to know the results for
another value, *create a new directory with that value!*. **row** assumes this data
model and [caches](../concepts/cache.md) all value files so that it does not need to
read thousands of files every time you execute a **row** command.

## Grouping by value

Now that your workspace directories have **values**, you can use those to
form **groups**. Every action in your workflow operates on **groups**. Set the
`action.group.include` key in an action to select which directories to include by
**value**. To see how this works, replace the contents of `workflow.toml` with:
```toml
{{#include group-workflow2.toml}}
```

This workflow will apply the `process_point` action to the directories where
`value/type == "point"` and the `process_letter` action to the directories where
`value/type == "letter"`.

`include` is an array. Each element is a length 3 array with the contents: `[JSON
pointer, operator, operand]`. Think of each element as an expression. The [*JSON
pointer*](../concepts/json-pointers.md) is a string that reads a particular value
from the directory's **value**. The *operator* is a comparison operator: `"equal_to",
"greater_than", or "less_than"`. The *operand* is the value to compare to. Together,
these 3 elements make a *condition*.

**Row** applies these *conditions* to all directories in the workspace. When all
*conditions* are true, the directory is included in the action's **groups**.

> Note: This implies that every JSON pointer used in an `include` condition **MUST**
> be present in every value file.

## Showing values

Let's verify that **row** is grouping the directories as intended with `row show
directories`. The `--value` argument adds output columns with the directory **values**
(selected by JSON pointer). Execute:
```plaintext
{{#include group.sh:show_point1}}
```

You should see:
```plaintext
Directory  Status   /type   /x /y
directory1 eligible "point"  0 10
directory2 eligible "point"  3  8
directory3 eligible "point"  0  4
directory4 eligible "point"  3 11
directory5 eligible "point"  0 -3
directory6 eligible "point"  2  2
```
Only the directories with `type == "point"` are shown for the action `process_points`.

Show the directories for the `process_letter` action:

```plaintext
{{#include group.sh:show_letter}}
```

You should see:
```plaintext
Directory  Status   /type    /letter
directory7 eligible "letter" "alpha"
directory8 eligible "letter"  "beta"
directory9 eligible "letter" "gamma"
```

With `include`, you can limit an action to execute on a subset of directories. For
example, you could use this to store multiple subprojects in the same workspace and
apply actions to specific subprojects. Think about how you might employ this capability
in your own workflows.

## Sorting groups

**Row** can also sort directories in the **group**. You may have noticed that all `row
show directories` output so far has been sorted by directory name - that is the default
behavior. You can choose to instead sort **groups** by any number of **value** elements.

To demonstrate, add the line:
```toml
{{#include group-workflow3.toml:9}}
```
to the `[action.group]` table for the `"process_point"` action.

`sort_by` is an array of strings. Each element is a JSON pointer. **Row** sorts the
directories lexicographically by the values that these pointers refer to.

To see the results, execute:
```plaintext
{{#include group.sh:show_point2}}
```
again.

You should now see:
```plaintext
Directory  Status   /type   /x /y
directory1 eligible "point"  0 10
directory3 eligible "point"  0  4
directory5 eligible "point"  0 -3
directory6 eligible "point"  2  2
directory2 eligible "point"  3  8
directory4 eligible "point"  3 11
```

Notice how the directories are now sorted by `/x`. When `/x` is the same, the
directories are then sorted by directory name. If needed, you can reverse the sort order
with `action.group.reverse_sort = true`.

`row submit` processes directories in the sorted order, so you can use `sort_by` to
prioritize the directories you want to execute first.

> Note: You can sort by numbers, strings, and arrays of numbers and/or strings. You
> cannot sort by objects.

## Splitting into separate groups

In all examples so far, `row submit` would tell you that it is submitting one **job**.
That **job** would execute the action on **ALL** directories matched by the action's
`include` conditions, which is fine for small workspaces or quick actions.

You will need to break your workflow into multiple **job** submissions for large
workflows and long-running actions. If you do not, you will end up generating jobs that
take months to complete or ones that require more CPUs than your cluster has. Even when
you have the resources to run a massive job, you may still want to break it up so that
you can analyze your results in smaller chunks.

Each **job** executes an action's command on one **group**. To break your work into
multiple jobs, you need to split your directories into multiple **groups**. **Row**
provides two mechanisms to accomplish this. It can split by the `sort_key` and limit
groups to a `maximum_size`.

## Splitting by sort key

Add the line:
```toml
{{#include group-workflow4.toml:10}}
```
to the `[action.group]` table for the `"process_point"` action.

Execute:
```plaintext
{{#include group.sh:show_point3}}
```
and you should see:
```plaintext
directory1 eligible "point"  0 10
directory3 eligible "point"  0  4
directory5 eligible "point"  0 -3

directory6 eligible "point"  2  2

directory2 eligible "point"  3  8
directory4 eligible "point"  3 11
```

`row show directories` separates the groups with blank lines. Each group includes all
the jobs with identical sort keys `["/x"]`.

Now, when you execute:
```bash
{{#include group.sh:submit}}
```
you will see that `row submit` launches 3 jobs:
```plaintext
Submitting 3 jobs that may cost up to 6 CPU-hours.
Proceed? [Y/n]:
[1/3] Submitting action 'process_point' on directory directory1 and 2 more.
directory1
directory3
directory5
[2/3] Submitting action 'process_point' on directory directory6.
directory6
[3/3] Submitting action 'process_point' on directory directory2 and 1 more.
directory2
directory4
```

The **jobs** execute on the same **groups** that `show directories` printed.

You can use `split_by_sort_key = true` to execute related simulations at the same time.
Or, you could use it to average the results of all replicate simulations. Think
of other ways that you might utilize `split_by_sort_key` in your workflows.

> Note: You may find the `-n` option useful. It instructs `row submit` to launch only
> the first *N* jobs.

## Limiting the maximum group size

**Row** can also limit groups to a maximum size. To see how this works,
**REPLACE** the `split_by_sort_key = true` line with:
```toml
{{#include group-workflow5.toml:10}}
```

Now:
```bash
{{#include group.sh:show_point4}}
```
will show:
```plaintext
Directory  Status   /x /y
directory1 eligible  0 10
directory3 eligible  0  4
directory5 eligible  0 -3
directory6 eligible  2  2

directory2 eligible  3  8
directory4 eligible  3 11
```

Notice how the first group contains four directories (in general, the first *N*
groups will all have `maximum_size` directories). The last group gets the remaining two.

Use `maximum_size` to limit the amount of work done in one **job**. For example: if each
directory takes 1 hour to execute in serial, set `maximum_size = 4` to ensure that each
of your jobs will complete in 4 hours. Alternately, say your action uses 16 processes
in parallel for each directory. Set `maximum_size = 8` (or 16, 24, ...) to use
1 (or 2, 3, ...) whole nodes on a cluster with 128 CPU cores per node.
This tutorial will cover cluster job submissions and resources in the next section.

> Note: When you set both `maximum_size` and `split_by_sort_key = true`, **Row** first
> splits by the sort key, then splits the resulting groups into that are larger than
> the maximum size.

## Next steps

In this section, you learned how to assign **values** to directories and use those
**values** to form **groups** of **directories**. Now you are ready to move on to
working with schedulers in the next section.
