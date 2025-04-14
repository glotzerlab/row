# Release notes

## 0.5.0 (not yet released)

*Highlights:*

In **Row** 0.5, the subcommands `show directories`, `show jobs`, and `show status` now
report `No matches.` when there are no results to show.

Users can also request an amount of memory specific to an action:

```toml
[action.resources]
memory_per_cpu_mb = 1024
```

**Row** normally sets the memory request automatically to the maximum allowed on the
partition, so most users should omit the new action-specific memory request keys.
When set, the given value will be passed to the SLURM option `--mem-per-cpu` or
`--mem-per-gpu` if the partition has no memory request set or the action's value is
smaller than the partition's. **Row** returns an error when the user requests more
memory than the partition has available to prevent unexpected additional costs.

*Added:*

* Actions can request the following new resources: `memory_per_cpu_mb` or
  `memory_per_gpu_mb`.

*Changed:*

* Writing tabular output with no rows now results in the output `No matches.`.
* Build executables on Ubuntu 22.04.
* Report an error when the workflow requests 0 processes, GPUs, or threads.
* Require Rust 1.85 or newer to build.
* Memory requests in `clusters.toml` must now be set in MB. The `memory_per_*`
  keys are renamed to `memory_per_*_mb`.

## 0.4.0 (2024-12-06)

*Highlights:*

**Row** 0.4 expands the `command` templating functionality, adds shell autocompletion,
and the `show jobs` subcommand.

The new _template arguments_ allow direct use of command line applications as actions,
removing the need for _shim_ scripts that access the workspace path and/or directory
values before invoking a subprocess. `{workspace_path}` expands to the current project's
workspace path and `{/JSON pointer}` expands to the value of the given JSON pointer for
the directory acted on.

_Shell autocompletion_ allows users to autocomplete all parameter names and
workspace dependent values for `cluster`, `action`, and `directories`. To enable,
execute the appropriate command in your shell's profile:
* Bash: `source <(COMPLETE=bash row)`
* Fish: `source (COMPLETE=fish row | psub)`
* Zsh: `source <(COMPLETE=zsh row)`

`show jobs` prints a table summarizing all currently submitted jobs that match given
action and directory criteria.

*Added:*

* In job scripts, set the environment variable `ACTION_WORKSPACE_PATH` to the _relative_
  path to the current workspace.
* `{workspace_path}` template parameter in `action.command` - replaced with the
  _relative_ path to the current workspace.
* `{/JSON pointer}` template parameter in `action.command` - replaced with the portion
  of the directory's value referenced by the given JSON pointer.
* Shell autocomplete.
* `show jobs` subcommand.

*Fixed:*

* All user-provided content (directories, action names, cluster names, and values) are
  properly escaped in the bash script output.
* Typographical errors in the documentation.
* The documentation now builds correctly with _mdbook_ 0.4.43.
* Example code converts environment variables to integers where needed.

## 0.3.1 (2024-10-04)

*Changed:*

* Improved the documentation.

## 0.3.0 (2024-08-21)

*Highlights:*

**Row** 0.3 adds command line arguments to reduce the output from `show` commands,
including the `--short` option to most `show` subcommand and status filtering options
to `show status`. For example, `row show status --eligible` shows only actions that have
eligible directories and `row show cluster --all --short` lists the names of all the
available cluster configurations. **Row** 0.3 also makes a number of fixes and changes
when using `group.include` in a non-trivial manner.

*Added:*

* Edit links to documentation pages.
* New arguments to `show status` display actions that are in the requested states:
 `--completed`, `--eligible`, `--submitted`, and `--waiting`.
* `cluster.submit_options` configuration option in `clusters.toml`.
* `--short` option to `show launchers` and `show directories`.

*Changed:*

* Show `import` lines in Python examples.
* Improve the verbose output from `submit`.
* `show status` hides actions with 0 directories by default. Pass `--all` to show all
  actions.
* `clean` now cleans all caches by default.
* Submit jobs with `--constraint="scratch"` by default on Delta.
* Submit jobs with `--constraint="nvme"` by default on Frontier.
* `group.include.all` now employs short circuit evaluation.
* Change `--name` option of `show cluster` to `--short`.
* `show directories` now accepts an optional `--action` argument.

*Fixed:*

* `submit_whole = true` checks only directories that match `group.include`.
* Do not print trailing spaces after the final column in tabular output.

## 0.2.0 (2024-06-18)

**Row** 0.2 adds support for partial-node job submissions on clusters without shared
partitions and fixes GPU job submissions on some clusters.

*Added:*

* `warn_[cpus|gpus]_not_multiple_of` key in *clusters.toml*.

*Changed:*

* OLCF Frontier configuration now uses `warn_gpus_not_multiple_of` instead of `require_gpus_multiple_of`.
* OLCF Andes configuration now uses `warn_cpus_not_multiple_of` instead of `require_cpus_multiple_of`.

*Fixed:*

* Prevent `gpus-per-task is mutually exclusive with tres-per-task` error.
* Correctly set `--mem-per-gpu` on Great Lakes.
* Correct formatting in the documentation.
* Correct typos in the documentation.

## 0.1.3 (2024-05-30)

*Fixed:*

* Broken build.

## 0.1.2 (2024-05-30)

*Fixed:*

* Erroneous code examples in the *Grouping directories* tutorial.

## 0.1.1 (2024-05-29)

*Added:*

* `conda-forge` installation instructions.

## 0.1.0 (2024-05-22)

* Initial release.
