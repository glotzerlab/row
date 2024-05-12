# init

Usage
```bash
row init [OPTIONS] <DIRECTORY>
```

`row init` creates `workflow.toml` and the workspace directory in the given DIRECTORY.
It creates the directory if needed. The default workspace path name is `workspace`. Use
the `--workspace` option to change this.

Set the `--signac` option to create a project compatible with signac. You must
separately initialize the signac project.

## `<DIRECTORY>`

The project directory to create. May be absolute or relative to the current directory.

## `[OPTIONS]`

### `--signac`

Create a signac compatible project.

* Sets workspace directory to `workspace`.
* Adds `value_file = "signac_statepoint.json"` to the `[workspace]`
  configuration.

### `--workspace`

(also: `-w`)

Set the name of the workspace directory. May not be used in combination with
`--signac`.

## Errors

`row init` returns an error when a row project already exists at the given DIRECTORY.

## Examples

* Create a project in the current directory:
  ```bash
  row init .
  ```
* Create a signac compatible project in the directory `project`:
  ```bash
  row init --signac project
  ```
* Create a project where the workspace is named `data`:
  ```bash
  row init --workspace data project
  ```
