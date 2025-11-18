![Row](row-horizontal.svg)

[![Crates.io Version](https://img.shields.io/crates/v/row?color=a44300)](https://crates.io/crates/row)
[![Read the Docs](https://img.shields.io/readthedocs/row/latest.svg)](https://row.readthedocs.io/)
[![GitHub Repository](https://img.shields.io/badge/github-repo-blue?logo=github)](https://github.com/glotzerlab/row)
[![GitHub Discussions](https://img.shields.io/github/discussions/glotzerlab/row)](https://github.com/glotzerlab/row/discussions/)
[![GitHub commit activity](https://img.shields.io/github/commit-activity/y/glotzerlab/row?label=commits)](https://github.com/glotzerlab/row)
[![Contributors](https://img.shields.io/github/contributors-anon/glotzerlab/row.svg?style=flat)](https://row.readthedocs.io/en/latest/contributors.html)
[![GitHub top language](https://img.shields.io/github/languages/top/glotzerlab/row?color=a44300)](https://rust-lang.org/)

Row is a command line tool that helps you manage workflows on HPC resources. Define
**actions** in a workflow configuration file that apply to **groups** of **directories**
in your **workspace**. **Submit** actions to your cluster's **scheduler**. Row tracks
which actions have been submitted on which directories so that you don't submit the same
work twice. Once a job completes, subsequent actions become eligible allowing you to
process your entire workflow to completion over many submissions.

The name is **row** as in *"row, row, row your boat"*.

Notable features:
* Support arbitrary directories and [signac](https://signac.readthedocs.io/) workspaces.
* Define your workflow in a configuration file.
* Execute actions via user-defined shell commands.
* Flexible group definitions:
  * Select directories with conditions on their value.
  * Split directories by their value and/or into fixed size groups.
  * Construct groups of any eligible directories or require that the entire matching
    group is submitted whole.
* Execute groups in serial or parallel.
* Schedule CPU and GPU resources.
* Multiple users can operate the same workflow.
* Automatically determine the partition based on the job's resources and size.
* Built-in configurations for many national and university HPC systems.
* Add custom cluster definitions for your resources.
* Row is **fast**.

## Demonstration

```bash
$ row submit --action=step1 -n 1
[1/1] Submitting action 'step1' on directory dir12 and 3 more (0ms).
Row submitted job 5095791.
```

```bash
$ row show status
Action     Completed Submitted Eligible Waiting Remaining cost
initialize        50         0       50       0    8 CPU-hours
step1              4         4       42      50   2K CPU-hours
step2              0         0        4      96  800 GPU-hours
```

```bash
$ row show directories --action step1 -n 3 --value="/value"
Directory Status    Job ID        /value
dir1      completed                  116
dir10     completed                  952
dir100    completed                  139
dir11     completed                  998

dir12     submitted anvil/5095791    950
dir13     submitted anvil/5095791    107
dir14     submitted anvil/5095791    127
dir15     submitted anvil/5095791    122

dir16     eligible                   682
dir17     eligible                   816
dir18     eligible                   803
dir19     eligible                   691
```

## Resources

- [Documentation](https://row.readthedocs.io/):
  Tutorial, command line interface documentation, and configuration file specifications.
- [Row discussion board](https://github.com/glotzerlab/row/discussions/):
  Ask the **row** user community for help.
- [signac](https://signac.readthedocs.io/):
  Python package to help you manage your workspace.

## History

**Row** is a spiritual successor to [signac-flow][flow].

[flow]: https://signac.readthedocs.io/projects/flow/en/latest/.
