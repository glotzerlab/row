# submit_options

`action.submit_options`: **table** - Override the global cluster-specific
submission options with values specific to this action. Any key that can be set
in the global [`submit_options.<name>`](../submit-options.md) can be overridden in
`action.submit_options.<name>`.

Example:
```toml
[action.submit_options.cluster1]
setup = "echo Executing action on cluster1..."
```
