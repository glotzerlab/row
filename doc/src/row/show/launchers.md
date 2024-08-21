# show launchers

Usage:
```bash
row show launchers [OPTIONS]
```

Print the [launchers](../../launchers/index.md) defined for the current cluster (or the
cluster given in `--cluster`). The output is TOML formatted.

This includes the user-provided launchers in [`launchers.toml`](../../launchers/index.md)
and the built-in launchers.

## `[OPTIONS]`

### `--all`

Show the launcher configurations for all clusters.

### `--short`

Show only the names of the launchers.

## Examples

* Show the launchers for the autodetected cluster:
  ```bash
  row show launchers
  ```
* Show the launchers for a specific cluster:
  ```bash
  row show launchers --cluster=anvil
  ```
* Show all launchers:
  ```bash
  row show launchers --all
  ```
* Show only names of all launchers:
  ```bash
  row show launchers --all --short
  ```
