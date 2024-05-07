# Row

Row is a command line tool that helps you manage workflows on HPC resources. Define
**actions** in a workflow configuration file that apply to **groups** of **directories**
in your **workspace**. **Submit** actions to your HPC **scheduler**. Row tracks which
actions have been submitted on which directories so that you don't submit the same work
twice. Once a job completes, subsequent actions become eligible allowing you to process
your entire workflow to completion over many submissions.

The name is "row" as in "row, row, row your boat".

Notable features:
* Support both arbitrary directories and [signac](https://signac.io) workspaces.
* Execute actions via arbitrary shell commands.
* Flexible group definitions:
  * Select directories with conditions on their value.
  * Split directories by their value and/or into fixed size groups.
* Execute groups in serial or parallel.
* Schedule CPU and GPU resources.
* Automatically determine the partition based on the batch job size.
* Built-in configurations for many national and university HPC systems.
* Add custom cluster definitions for your resources.

TODO: better demo script to get output for README and row show documentation examples.

For example:
```bash
> row show status
Action Completed Submitted Eligible Waiting Remaining cost
one         1000       100      900       0  24K CPU-hours
two            0       200      800    1000   8K GPU-hours
```

```bash
> row show directories --value "/value"
Directory Status     Job ID /value
dir1      submitted 1432876    0.9
dir2      submitted 1432876    0.8
dir3      submitted 1432876    0.7

dir4      completed            0.5
dir5      completed            0.4
dir6      completed            0.3
```

**Row** is a spiritual successor to
[signac-flow](https://docs.signac.io/projects/flow/en/latest/).
