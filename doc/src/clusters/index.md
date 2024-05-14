# `clusters.toml`

**Row** includes [built-in cluster configurations](built-in.md) for a variety of
national and university HPC resources. You can override these and add new clusters in
the file `$HOME/.config/row/clusters.toml`. Each cluster includes a *name*, a method to
*identify* the cluster, the type of *scheduler*, and details on the *partitions*.
See [cluster configuration](cluster.md) for the full specification.

The configuration defines the clusters in an *array*:
```toml
[[cluster]]
name = "cluster1"
# ...

[[cluster]]
name = "cluster2"
# ...
```

User-provided clusters in `$HOME/.config/row/clusters.toml` are placed first in the
array. Execute [`row show cluster --all`](../row/show/cluster.md) to see the complete
cluster configuration.

## Cluster identification

On startup, **row** iterates over the array of clusters in order. If `--cluster` is not
set, **row** checks the `identify` condition in the configuration. If `--cluster` is
set, **row** checks to see if the name matches. **Row** selects the *first* cluster
that matches.

> To override a built-in, your configuration should include a cluster by the same name
> and `identify` condition.
