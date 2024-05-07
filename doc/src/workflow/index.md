# workflow.toml

The file `workflow.toml` defines the [workspace](workspace.md),
[actions](action/index.md), and [submission options](submit-options.md). Place
`workflow.toml` in a directory to identify it as a **row** *project*.
The [`row` command line tool](../row/index.md) will identify the current project
by finding `workflow.toml` in the current working directory or any parent directory,
recursively.
