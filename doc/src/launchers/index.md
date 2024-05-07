# `launchers.toml`

**Row** includes [built-in launchers](built-in.md) to enable OpenMP and MPI on the
[built-in clusters](../clusters/built-in.md). You can override these configurations
and add new launchers in the file `$HOME/.config/row/launchers.toml`. It defines how
each **launcher** expands into a **command prefix**, with the possibility for specific
settings on each [**cluster**](../clusters/index.md). For example, an
[**action**](../workflow/action/index.md) with the configuration:
```toml
[[action]]
name = "action"
command = "command {directory}"
launchers = ["launcher1", "launcher2"]
```
will expand to:
```plaintext
<launcher1 prefix> <launcher2 prefix> command $directory || {{ ... handle errors }}
```
in the submission script.

The prefix is a function of the job's [**resources**](../workflow/action/resources.md)
and the size of the [**group**](../workflow/action/group.md) in the current submission.
The section [Launcher configuration](launcher.md) details how this prefix is
constructed.

## Default launcher configuration

The **default** configuration will be used when there is no cluster-specific
configuration for the currently active cluster. Every launcher **must** have a
default configuration. Create a new launcher by creating a table named ``<launcher
name>.default`` in `launchers.toml`. For example:
```toml
[launcher1.default]
# launcher1's default configuration
```

## Cluster-specific launcher configuration

Define a launcher configuration specific to a cluster in the table
`<launcher name>.<cluster>`, where `<cluster>` is one of the cluster names in
[`clusters.toml`](../clusters/index.md). For example:
```toml
[launcher1.none]
# launcher1's configuration for the cluster `none`.
```
