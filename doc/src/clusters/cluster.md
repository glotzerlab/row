# cluster

An element in `[[cluster]]` is a **table** that defines the configuration of a single
cluster.

For example:
```toml
[[cluster]]
name = "cluster1"
identify.by_environment = ["CLUSTER_NAME", "cluster1"]
scheduler = "slurm"
[[cluster.partition]]
name = "shared"
maximum_cpus_per_job = 127
maximum_gpus_per_job = 0
[[cluster.partition]]
name = "gpu-shared"
minimum_gpus_per_job = 1
[[cluster.partition]]
name = "compute"
require_cpus_multiple_of = 128
maximum_gpus_per_job = 0
[[cluster.partition]]
name = "debug"
maximum_gpus_per_job = 0
prevent_auto_select = true
```

## name

`cluster.name`: **string** - The name of the cluster.

## identify

`cluster.identify`: **table** - Set a condition to identify when **row** is executing
on this cluster. The table **must** have one of the following keys:

* `by_environment`: **array** of two strings - Identify the cluster when the environment
  variable `by_environment[0]` is set and equal to `by_environment[1]`.
* `always`: **bool** - Set to `true` to always identify this cluster. When `false`,
  this cluster may only be chosen by an explicit `--cluster` option.

> Note: The *first* cluster in the list that sets `identify.always = true` will prevent
> any later cluster from being identified (except by explicit `--cluster=name`).

## scheduler

`cluster.scheduler`: **string** - Set the job scheduler to use on this cluster. Must
be one of:

* `"slurm"`
* `"bash"`

## partition

`cluster.partition`: **array** of **tables** - Define the scheduler partitions that
**row** may select from when submitting jobs. **Row** will check the partitions in the
order provided and choose the *first* partition where the job matches all the
provided conditions. All conditions are optional.

### name

`cluster.partition.name`: **string** - The name of the partition as it should be passed
to the cluster batch submission command.

### maximum_cpus_per_job

`cluster.partition.maximum_cpus_per_job`: **integer** - The maximum number of CPUs that
can be used by a single job on this partition:
```plaintext
total_cpus <= maximum_cpus_per_job
```

### require_cpus_multiple_of

`cluster.partition.require_cpus_multiple_of`: **integer** - All jobs submitted to this
partition **must** use an integer multiple of the given number of cpus:
```plaintext
total_cpus % require_cpus_multiple_of == 0
```

### memory_per_cpu

`cluster.partition.memory_per_cpu`: **string** - CPU Jobs submitted to this partition
will pass this option to the scheduler. For example SLURM schedulers will set
`--mem-per-cpu=<memory_per_cpu>`.

### cpus_per_node

`cluster.partition.cpus_per_node`: **string** - Number of CPUs per node.

When `cpus_per_node` is not set, **row** will request `n_processes` tasks. In this case,
some schedulers are free to spread tasks among any number of nodes (for example, shared
partitions on Slurm schedulers).

When `cpus_per_node` is set, **row** will **also** request the minimal number of nodes
needed to satisfy `n_nodes * cpus_per_node >= total_cpus`. This may result in longer
queue times, but will lead to more stable performance for users.

> Note: Set `cpus_per_node` only when all nodes in the partition have the same number
> of CPUs.

### minimum_gpus_per_job

`cluster.partition.minimum_gpus_per_job`: **integer** - The minimum number of gpus that
must be used by a single job on this partition:
```plaintext
total_gpus >= minimum_gpus_per_job
```

### maximum_gpus_per_job

`cluster.partition.maximum_gpus_per_job`: **integer** - The maximum number of gpus that
can be used by a single job on this partition:
```plaintext
total_gpus <= maximum_gpus_per_job
```

### require_gpus_multiple_of

`cluster.partition.require_gpus_multiple_of`: **integer** - All jobs submitted to this
partition **must** use an integer multiple of the given number of gpus:
```plaintext
total_gpus % require_gpus_multiple_of == 0
```

### memory_per_gpu

`cluster.partition.memory_per_gpu`: **string** - GPU Jobs submitted to this partition
will pass this option to the scheduler. For example SLURM schedulers will set
`--mem-per-gpu=<memory_per_gpu>`.

### gpus_per_node

`cluster.partition.gpus_per_node`: **string** - Number of GPUs per node. Like
`cpus_per_node` but used when jobs request GPUs.

### prevent_auto_select

`cluster.partition.prevent_auto_select`: **boolean** - Set to true to prevent row from
automatically selecting this partition.

### account_suffix

`cluster.partition.account_suffix`: **string** - An account suffix when submitting jobs
to this partition. Useful when clusters define separate `account-cpu` and `account-gpu`
accounts.
