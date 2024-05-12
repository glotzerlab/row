# scan

Usage
```bash
row scan [OPTIONS] [DIRECTORIES]
```

`row scan` scans the selected directories for action
[products](../workflow/action/index.md#products) and updates the cache
of completed directories accordingly.

Under normal usage, you should not need to execute `row scan` manually.
[`row submit`](submit.md) automatically scans the submitted directories after it
executes the action's command.

> Note: `row scan` only **adds** new completed directories. To mark directories
> as no longer completed, use [`row clean`](clean.md).

## `[DIRECTORIES]`

Scan these specific directories. By default, **row** scans the entire workspace.
Pass a single `-` to read the directories from stdin (separated by newlines).

## `[OPTIONS]`

### `--action`

(also: `-a`)

Set `--action <ACTION>` to choose which action to scan. By default, **row**
scans for products from all actions.

> Note: Unlike other commands, `--action` is **not** a wildcard.

## Examples

* Scan all directories for all actions:
  ```bash
  row scan
  ```
* Scan a specific action:
  ```bash
  row scan --action=action
  ```
* Scan specific directories:
  ```bash
  row scan directory1 directory2
  ```
