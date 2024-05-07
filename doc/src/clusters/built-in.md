# Built-in clusters

**Row** includes built-in support for the following clusters.

## Anvil (Purdue)

[Anvil documentation](https://www.rcac.purdue.edu/knowledge/anvil).

**Row** automatically selects from the following partitions:
* `shared`
* `wholenode`
* `gpu`

Other partitions may be selected manually.

There is no need to set `--mem-per-*` options on Anvil as the cluster automatically
chooses the largest amount of memory available per core by default.

## Delta (NCSA)

[Delta documentation](https://docs.ncsa.illinois.edu/systems/delta).

**Row** automatically selects from the following partitions:
* `cpu`
* `gpuA100x4`

Other partitions may be selected manually.

Delta jobs default to a small amount of memory per core. **Row** inserts `--mem-per-cpu`
or `--mem-per-gpu` to select the maximum amount of memory possible that allows full-node
jobs and does not incur extra charges.

## Great Lakes (University of Michigan)

[Great Lakes documentation](https://arc.umich.edu/greatlakes/).

**Row** automatically selects from the following partitions:
* `standard`
* `gpu_mig40,gpu`
* `gpu`

Other partitions may be selected manually.

Great Lakes jobs default to a small amount of memory per core. **Row** inserts
`--mem-per-cpu` or `--mem-per-gpu` to select the maximum amount of memory possible that
allows full-node jobs and does not incur extra charges.

> Note: The `gpu_mig40,gpu` partition is selected only when there is one GPU per job.
> This is a combination of 2 partitions which decreases queue wait time due to the
> larger number of nodes that can run your job.
