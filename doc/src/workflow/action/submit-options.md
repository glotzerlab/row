# submit_options

`action.submit_options`: **table** - sets the cluster-specific submission options. Keys
in `submit_options` must be one of the named clusters defined in
 [`clusters.toml`](../../clusters/index.md).

Example:
```toml
[action.submit_options.cluster1]
account = "my_account"
setup = """
module reset
module load cuda
"""
custom = ["--mail-user=user@example.com"]
partition = "shared"

[action.submit_options.cluster2]
account = "other_account"
setup = "module load openmpi"
```

> Note: You may omit `[submit_options]` entirely.

## `<name>.account`

`action.submit_options.<name>.account`: **string** - Submit jobs to this account on
cluster `<name>`. When you omit `account`, **row** does not add the `--account=` line
to the submission script.

## `<name>.setup`

`action.submit_options.<name>.setup`: **string** - Lines to include in the submission
script on cluster `<name>`. The setup is executed *before* the action's command. You may
omit `setup` to leave this portion of the script blank.

## `<name>.custom`

`action.submit_options.<name>.custom`: **array** of **strings** - List of additional
command line options to pass to the batch submission script on cluster `<name>`. For
example. `custom = ["--mail-user=user@example.com"]` will add the line
```
#SBATCH --mail-user=user@example.com
```
to the top of a SLURM submission script. `custom` defaults to an empty array when
omitted.

## `<name>.partition`

`action.submit_options.<name>.partition`: **string** - Force the use of a particular
partition when submitting jobs to the queue on cluster `<name>`. When omitted, **row**
will automatically determine the correct partition based on the configuration in
[`clusters.toml`](../../clusters/index.md).

> Note: You should almost always omit `partition`. Set it *only* when your action
> **requires** a *specialty* partition that is not automatically selected.
