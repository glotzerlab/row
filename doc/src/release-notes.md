# Release notes

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
