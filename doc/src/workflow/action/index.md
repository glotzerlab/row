# action

`action` is an **array** where each element defines one action in your workflow.

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
action.group.maximum_size = 8
[action.resources]
processes.per_directory = 16
threads_per_process = 4
walltime.per_submission = "04:00:00"
```

## name

`action.name`: **string** - The action's name. You must set a name for each
action.

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

## launchers

`action.launchers`: **array** of **strings** - The launchers to apply when executing a
command. A launcher is a prefix placed before the command in the submission script. The
cluster configuration [`clusters.toml`](../../clusters/index.md) defines what launchers
are available on each cluster and how they are invoked. The example for `action_two`
above (`launchers = ["openmpi", "mpi"]`) would expand into something like:
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
