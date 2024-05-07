# show cluster

Usage:
```bash
row show cluster [OPTIONS]
```

Print the [current cluster configuration](../../clusters/index.md) (or for the cluster
given in `--cluster`).

Example output:
```
name = "none"
scheduler = "bash"

[identify]
always = true

[[partition]]
name = "none"
prevent_auto_select = false
```

## `[OPTIONS]`

### `--all`

Show the configuration of all clusters: both user-defined and built-in.

### `--name`

Show only the cluster's name.
