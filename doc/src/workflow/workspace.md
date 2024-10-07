# workspace

The `workspace` table describes the layout of your *workspace*.

Example:
```toml
[workspace]
path = "my_workspace"
value_file = "value.json"
```

> Note: You may omit `[workspace]` entirely.

## path

`workspace.path`: **string** - The location of your workspace directory *relative
to* the location of `workflow.toml`. When not set, `workspace.path` defaults to
`"workspace"`.

## value_file

`workspace.value_file`: **string** - The name of the JSON file **row** will read to
obtain the *value* of each directory. When you omit `value_file`, **row** assigns the
JSON *value* of `null` to each directory.

Set
```toml
workspace.value_file = "signac_statepoint.json"
```
to use **row** with [signac](https://signac.io) workspaces.
