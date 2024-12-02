# Grouping directories

## Overview

This section shows how you can use **values** to form **groups** of directories. Each
**job** executes an action's command on a **group** of directories.

## Initialize a workspace with values

To demonstrate the capabilities of **groups**, create a workspace with multiple types of
values. The following script follows the same process used in the previous section:

```bash
{{#include group.sh:init}}
```

As mentioned previously, `echo` is used here to create a _minimal_ script that you can
execute to follow along. For any serious production work you will likely find [signac]
more convenient to work with.

[signac]: ../python/signac.md

## Grouping by value

You can use **values** those to form **groups** of **directories**. Every action in
your workflow operates on **groups**. Add entries to the `action.group.include` array
in an action to select which directories to include by **value**. To see how this works,
replace the contents of `workflow.toml` with:

```toml
{{#include group-workflow2.toml}}
```

This workflow will apply the `process_point` action to the directories where
`value/type == "point"` and the `process_letter` action to the directories where
`value/type == "letter"`.

`action.group.include` is an array of conditions. A directory is included when *any*
condition is true. `condition` is a length 3 array with the contents: `[JSON pointer,
operator, operand]`. Think of the condition as an expression. The
[*JSON pointer*](../concepts/json-pointers.md) is a string that references a portion of
the directory's **value**. The *operator* is a comparison operator: `"<"`, `"<="`,
`"=="`, `">="`, or `">"`. The *operand* is the value to compare to. Together, these 3
elements make a *condition*.

**Row** applies each *condition* to all directories in the workspace. When a
*condition* is true, the directory is included in the action's **groups**.

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
{{#include group-workflow3.toml:sort}}
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
{{#include group-workflow4.toml:split}}
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
{{#include group-workflow5.toml:max}}
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
