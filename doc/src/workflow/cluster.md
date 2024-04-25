# cluster

The `cluster` table sets the default cluster-specific parameters for all *actions*. You
can set action-specific cluster parameters in [`action.cluster`](action/cluster.md).
Keys in `cluster` must be one of the named clusters defined in
[`clusters.toml`](../clusters/index.md).

Example:
```toml
[cluster.delta]
account = "my_account"
setup = """
module reset
module load cuda
"""
options = ["--mem-per-cpu=1g"]
partition = "shared"

[cluster.anvil]
account = "other_account"
setup = "module load openmpi"
```

> Note: You may omit `[cluster]` entirely.

## account

`cluster.<name>.account`: **string** - Submit cluster jobs to this account on cluster
`<name>`. When you omit `account`, **row** does not add the `--account=` line to the
submission script.

## setup

`cluster.<name>.setup`: **string** - Lines to include in the submission script on
cluster `<name>`. The setup is executed *before* the action's command. You may omit
`setup` to leave this portion of the script blank.

## options

`cluster.<name>.options`: **array** of **strings** - List of additional command line
options to pass to the batch submission script on cluster `<name>`. For example.
`options = ["--mem-per-cpu=2g"]` will add the line
```
#SBATCH --mem-per-cpu=2g
```
to the top of a SLURM submission script. When you omit `options`,  it defaults to an
empty array.

## partition

`cluster.<name>.partition`: **string** - Force the use of a particular partition
when submitting jobs to the queue on cluster `<name`>. When omitted, **row**
will automatically determine the correct partition based on the configuration in
[`clusters.toml`](../clusters/index.md).

> Note: You should almost always omit `partition`. Set it *only* when you need a
> specialty partition that is not automatically selected.
