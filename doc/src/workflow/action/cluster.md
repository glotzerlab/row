# cluster

`action.cluster`: **table** - Override the global cluster-specific parameters with
values specific to this action. Any key that can be set in the global
[`cluster.<name>`](../cluster.md) can be overridden in `action.cluster.<name>`.

Example:
```toml
[action.cluster.anvil]
setup = "echo Executing action three on anvil..."
```
