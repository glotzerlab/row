# resources

`action.resources`: **table** - Defines the resources that the action will consume.
**Row** passes this information to the scheduler when you submit a job.

Example:
```toml
[action.resources]
processes.per_submission = 10
threads_per_process = 4
walltime.per_submission = "04:00:00"
```
> Note: You may omit `[action.resources]` entirely.

## processes

`action.resources.processes`: **table** - Set the number of processes this action
will execute on (launched by `mpi` or similarly capable launcher). The table **must**
have one of two keys: `per_submission` or `per_directory`.

Examples:
```toml
processes.per_submission = 16
```
```toml
processes.per_directory = 8
```

When set to `per_submission`, **row** asks the scheduler to allocate the given
number of processes for each job. When set to `per_directory`, **row** requests the
given value multiplied by the number of directories in the submission group. Use
`per_submission` when your action loops over directories and reuses the same processes
to perform computations. Use `per_directory` when your action parallelizes over the
directories and therefore requires `processes.per_directory * num_directories` total
processes.

When omitted, `processes` defaults to `per_submission = 1`.

## threads_per_process

`action.resources.threads_per_process`: **integer** - The number of CPU threads your
action utilizes per process. When omitted, **row** does not make any specific request
for threads from the scheduler. Most schedulers default to 1 thread per process in this
case.

## gpus_per_process

`action.resources.gpus_per_process`: **integer** - The number of GPUs your action
utilizes per process. When omitted, **row** does not make any specific request for GPUs
from the scheduler. Most schedulers default to 0 GPUs per process in this case.

## walltime

`action.resources.walltime`: **table** - Set the walltime that this action takes to
execute. The table **must** have one of two keys: `per_submission` or `per_directory`.
Valid walltime strings include `"HH:MM:SS"`, `"D days, HH:MM:SS"`, and all other valid
`Duration` formats parsed by [speedate](https://docs.rs/speedate/latest/speedate/).

Examples:
```toml
walltime.per_submission = "4 days, 12:00:00"
```
```toml
walltime.per_directory = "00:10:00"
```

When set to `per_submission`, **row** asks the scheduler to allocate the given walltime
for each job. When set to `per_directory`, **row** requests the given value multiplied
by the number of directories in the submission group. Use `per_submission` when your
action parallelizes over directories and therefore takes the same amount of time
independent of the submission group size. Use `per_directory` when your action loops
over the directories and therefore the walltime scales with the number of directories.

When omitted, `walltime` defaults to `per_directories = 01:00:00`.
