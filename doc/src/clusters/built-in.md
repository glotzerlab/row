# Built-in clusters

**Row** includes built-in support for the following clusters.

## Andes (OLCF)

**Row** automatically selects from the following partitions on [Andes]:
* `batch`

> Note: Andes has no shared partition. All jobs must use 32 CPUs per node.

[Andes]: https://docs.olcf.ornl.gov/systems/andes_user_guide.html

## Anvil (Purdue)

**Row** automatically selects from the following partitions on [Anvil]:
* `shared`
* `wholenode`
* `gpu`

Other partitions may be selected manually.

There is no need to set `--mem-per-*` options on [Anvil] as the cluster automatically
chooses the largest amount of memory available per core by default.

> Note: The whole node partitions **require** that each job submitted request an
> integer multiple of 128 CPU cores.

[Anvil]: https://www.rcac.purdue.edu/knowledge/anvil

## Delta (NCSA)

**Row** automatically selects from the following partitions on [Delta]:
* `cpu`
* `gpuA100x4`

Other partitions may be selected manually.

[Delta] jobs default to a small amount of memory per core. **Row** inserts
`--mem-per-cpu` or `--mem-per-gpu` to select the maximum amount of memory possible that
allows full-node jobs and does not incur extra charges.

[Delta]: https://docs.ncsa.illinois.edu/systems/delta

## Frontier (OLCF)

**Row** automatically selects from the following partitions on [Frontier]:
* `batch`

> Note: Frontier has no shared partition. All jobs must use 8 GPUs per node.

[Frontier]: https://docs.olcf.ornl.gov/systems/frontier_user_guide.html#


## Great Lakes (University of Michigan)

**Row** automatically selects from the following partitions on [Great Lakes]:
* `standard`
* `gpu_mig40,gpu`
* `gpu`

Other partitions may be selected manually.

[Great Lakes] jobs default to a small amount of memory per core. **Row** inserts
`--mem-per-cpu` or `--mem-per-gpu` to select the maximum amount of memory possible that
allows full-node jobs and does not incur extra charges.

> Note: The `gpu_mig40,gpu` partition is selected only when there is one GPU per job.
> This is a combination of 2 partitions which decreases queue wait time due to the
> larger number of nodes that can run your job.

[Great Lakes]: https://its.umich.edu/advanced-research-computing/high-performance-computing/great-lakes
