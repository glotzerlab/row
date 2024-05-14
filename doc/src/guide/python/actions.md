# Writing action commands in Python

In **row**, actions execute arbitrary **shell commands**. When your action is
**Python** code, you must structure that code so that it is a command line tool
that takes directories as arguments. There are many ways you can achieve this goal.

This guide will show you how to structure all of your actions in a single file:
`actions.py`. This layout is inspired by **row's** predecessor **signac-flow**
and its `project.py`.

> Note: If you are familiar with **signac-fow**, see [migrating from signac-flow][1]
> for many helpful tips.

[1]: ../../signac-flow.md

To demonstrate the structure of a project, let's build a workflow that computes the
sum of squares. The focus of this guide is on structure and best practices. You need to
think about how your simulation, analysis, data processing, or other code will fit into
this structure.

## Create the project

First, create the **row** project:
```bash
{{#include signac.sh:row_init}}
```

Then, create a file `populate_workspace.py` in the same directory as `workflow.toml`
with the contents:
```python
{{#include populate_workspace.py}}
```

Execute:
```bash
{{#include signac.sh:signac_init}}
```
to initialize the signac workspace and populate it with directories.

> Note: If you are not familiar with **signac**, then go read the [*basic* tutorial].
> Come back to the **row** documentation when you get to the section on *workflows*.
> For extra credit, reimplement the **signac** tutorial workflow in **row** after you
> finish reading this guide.

[*basic* tutorial]: https://docs.signac.io/en/latest/tutorial.html#basics

## Write actions.py

Now, create a file `actions.py` with the contents:
```python
{{#include actions.py}}
```

This file defines each **action** as a function with the same name. These functions take
an array of jobs as an argument: `def square(*jobs)` and `def compute_sum(*jobs)`. The
`if __name__ == "__main__":` block parses the command line arguments, forms an array of
signac jobs and calls the requested **action** function.

> Note: This example demonstrates looping over directories in **serial**. However, this
> structure also gives you the power to choose **serial** or **parallel** execution.
> Grouping many directories into a single cluster job submission will increase your
> workflow's throughput.

## Write workflow.toml

Next, replace the contents of `workflow.toml` with the corresponding workflow:
```toml
{{#include signac-workflow.toml}}
```

*Both* actions have the same **command**, set once by the
[**default action**](../../workflow/default.md):
```toml
{{#include signac-workflow.toml:5}}
```

`python actions.py` executes the `actions.py` file above. It is given the argument
`--action $ACTION_NAME` which selects the Python function to call. Here `$ACTION_NAME`
is an [environment variable](../../env.md) that **row** sets in job scripts. The
last arguments are given by `{directories}`. Unlike `{directory}` shown in previous
tutorials, `{directories}` expands to *ALL* directories in the submitted **group**. In
this way, `action.py` is executed once and is free to process the list of directories in
any way it chooses (e.g. in serial, with
[multiprocessing parallelism, multiple threads](../concepts/thread-parallelism.md),
using [MPI parallelism](../concepts/process-parallelism.md), ...).

## Execute the workflow

Now, submit the *square* action:
```bash
{{#include signac.sh:submit_square}}
```
and you should see:
```plaintext
Submitting 1 job that may cost up to 0 CPU-hours.
Proceed? [Y/n]: y
[1/1] Submitting action 'square' on directory 04bb77c1bbbb40e55ab9eb22d4c88447 and 9 more.
```

Next, submit the *compute_sum* action:
```bash
{{#include signac.sh:submit_sum}}
```
and you should see:
```plaintext
Submitting 1 job that may cost up to 0 CPU-hours.
Proceed? [Y/n]: y
[1/1] Submitting action 'compute_sum' on directory 04bb77c1bbbb40e55ab9eb22d4c88447 and 9 more.
285
```

It worked! `sum` printed the result `285`.

> Note: If you are on a cluster, use `--cluster=none` or wait for jobs to complete
> after submitting.

## Applying this structure to your workflows

With this structure in place, you can add new **actions** to your workflow following
these steps:
1) Write a function `def action(*jobs)` in `actions.py`.
2) Add:
    ```toml
    [[action]]
    name = "action"
    # And other relevant options
    ```
    to your `workflow.toml` file.

> Note: You may write functions that take only one job `def action(job)` without
> modifying the given implementation of `__main__`. However, you will need to set
> `action.group.maximum_size = 1` or use `{directory}` to ensure that `action.py` is
> given a single directory. If you implement your code using arrays, you can use
> **row's** grouping functionality to your benefit.

## Next steps

In this guide, you learned how to write workflow action commands in Python. Now, you
should know everything you need to build complex workflows with **row** and deploy them
on HPC resources.
