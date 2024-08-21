# Release notes

## 0.3.0 (not yet released)

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
