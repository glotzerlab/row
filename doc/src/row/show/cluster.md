# show cluster

Usage:
```bash
row show cluster [OPTIONS]
```

Print the [current cluster configuration](../../clusters/index.md) in TOML format.

## `[OPTIONS]`

### `--all`

Show the configuration of all clusters: both user-defined and built-in.

### `--name`

Show only the cluster's name.

## Examples

* Show the autodetected cluster:
  ```bash
  row show cluster
  ```
* Show the configuration of a specific cluster:
  ```bash
  row show cluster --cluster=anvil
  ```
* Show all clusters:
  ```bash
  row show cluster --all
  ```
