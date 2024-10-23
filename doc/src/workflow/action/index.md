# action

Each element in the `action` **array** defines one action in your workflow.

Examples:
```toml
[[action]]
name = "action_one"
command = "./action_one.sh workspace/{directory}"
products = ["one.data"]

[[action]]
name = "action_two"
command = "python action_two.py {directories}"
previous_actions = ["action_one"]
products = ["two.data", "log.txt"]
launchers = ["openmp", "mpi"]
[action.group]
maximum_size = 8
[action.resources]
processes.per_directory = 16
threads_per_process = 4
walltime.per_submission = "04:00:00"
```

## name

`action.name`: **string** - The action's name. You must set a name for each
action. The name may be set by [from](#from).

> Note: Two or more conceptually identical elements in the actions array *may* have
> the same name. All elements with the same name **must** have identical
> [`products`](#products) and [`previous_actions`](#previous_actions). All elements
> with the same name **must also** select non-intersecting subsets of directories with
> [`action.group.include`](group.md#include).

## command

`action.command`: **string** - The action's command template, which must
include either `{directory}` or `{directories}`.

**Row** will expand this template and insert it into the generated submission
script. When `{directory}` is present in `command`, row will execute it once
per directory:
```bash
./action_one.sh workspace/dir0 || exit 2
./action_one.sh workspace/dir1 || exit 2
./action_one.sh workspace/dir2 || exit 2
./action_one.sh workspace/dir3 || exit 2
./action_one.sh workspace/dir4 || exit 2
...
```

When `{directories}` is present, **row** executes the command once - passing
all directories as arguments:
```bash
python action_two.py dir0 dir1 dir2 dir3 dir4 dir5 || exit 2
```

In both cases, **row** appends error checking via `|| exit 2` to ensure
that the script exits at the first occurrence of an error. To chain multiple
steps together in a single *action*, you must either combine them in a script
or chain the steps with `&&`. For example:
```toml
command = "echo Message && python action.py {directory}"
```

### Template parameters

`action.command` will expand any template parameter contained within curly braces:
`{template_parameter}`.

* `{directory}` and `{directories}` are described above.
* `{workspace_path}` will be replaced with the _relative_ path from the project root
  (the directory containing `workflow.toml`) to the currently selected workspace.
* `{/JSON pointer}` will be replaced by a portion of the directory's value referenced
  by the given [JSON pointer]. Must be used with `{directory}`.
* `{}` will be replaced by the entire directory value formatted in JSON as a single
  command line argument. Must be used with `{directory}`
* All other template parameters are invalid.

For example:
```toml
command = "application -p {/pressure} -s {/seed} -o {workspace_path}/{directory}/out"
```

[JSON pointer]: ../../guide/concepts/json-pointers.md

## launchers

`action.launchers`: **array** of **strings** - The launchers to apply when executing a
command. A launcher is a prefix placed before the command in the submission script. The
launcher configuration [`lauchers.toml`](../../launchers/index.md) defines what launchers
are available on each cluster and how they are invoked. The example for `action_two`
above (`launchers = ["openmp", "mpi"]`) would expand into something like:
```bash
OM_NUM_THREADS=4 srun --ntasks=128 --cpus-per-task=4 python action_two.py ...
```
When omitted, `launchers` defaults to an empty array.

## previous_actions

`action.previous_actions`: **array** of **strings** - The previous actions that
must *all* be completed before this action may be executed. When omitted,
`previous_actions` defaults to an empty array.

## products

`action.products`: **array** of **strings** - The names of the files that the
action produces in the directory. When *all* products are present, that
directory has *completed* the action. When omitted, `products` defaults
to an empty array.

## `[group]`

See [group](group.md).

## `[resources]`

See [resources](resources.md).

## `[submit_options]`

See [submit_options](submit-options.md).

## from

`action.from`: **string** - Name of the **action** to copy settings from.

Every key in an `[[action]]` table (including sub-keys in `[action.group]`,
`[action.resources]`, and `[action.submit_options]`) may be set in one of 3 locations:

1. This action: `action.key[.sub_key]`.
2. The action named by `from`: `action_from.key[.sub_key]` (when `action.from` is set).
3. The default action: `default.action.key[.sub_key]`.

The action will take on the value set in the **first** location that does not omit
the key. When all 3 locations omit the key, the "when omitted" behavior takes effect
(documented separately for each key).

`from` is a convenient way to [submit the same action to different groups/resources].

> Note: `name` and `command` may be provided by `from` or `action.default` but may not
> be omitted entirely.

[submit the same action to different groups/resources]: ../../guide/howto/same.md
