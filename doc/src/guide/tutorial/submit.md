# Submitting jobs with row

## Overview

This section explains how to **submit** jobs to the **scheduler** with **row**.

## Preliminary steps

**Row** has built-in support for a number of [clusters](../../clusters/built-in.md):
* Andes (OLCF)
* Anvil (Purdue)
* Delta (NCSA)
* Great Lakes (University of Michigan)

You can skip to the [next heading](#checking-your-job-script) if you are using one of
these clusters.

If not, then you need to create a configuration files that describe your
cluster. You may also need to define launchers specific to your cluster.

* [`$HOME/.config/row/clusters.toml`](../../clusters/index.md) gives your cluster
  a name, instructions on how to identify it, and lists the partitions your cluster
  provides.
* [`$HOME/.config/row/launchers.toml`](../../launchers/index.md) defines how the
  launcher command prefixes (e.g. MPI, OpenMP) expand. The default for MPI is to use
  `srun`. If this doesn't work on your cluster, write `launchers.toml` to use a
  different command and/or options.

Many clusters have separate **partitions** for different resources (e.g. shared, whole
node, GPU, etc...). Your final script must request the correct `--partition` to execute
the command and charge accounts properly. `clusters.toml` describes rules by which
**row** automatically selects partitions when it generates job scripts.

> Note: Feel free to ask on the [discussion board][discussion] if you need help
> writing configuration files for your cluster.

[discussion]: https://github.com/glotzerlab/row/discussions

Check that the output of `row show cluster` and `row show launchers` is what you expect
before continuing.

## Checking your job script

For demonstration purposes, this guide will continue using the
[Hello, workflow](hello.md) example. In fact, you already learned how to submit jobs
in that section.

However, you should *always* check that the job script is correct before you **submit**
on a **cluster**. The `--dry-run` option prints the submission script (or scripts)
instead of submitting with `sbatch`:
```plaintext
row submit --dry-run
```
Remember, **YOU ARE RESPONSIBLE** for the content of the scripts that you submit.
Make sure that the script is requesting the correct resources and is routed to the
correct **partition**.

For example, the example workflow might generate a job script like this on Anvil:
```bash
#!/bin/bash
#SBATCH --job-name=hello-directory0+2
#SBATCH --partition=shared
#SBATCH --ntasks=1
#SBATCH --time=180

directories=(
'directory0'
'directory1'
'directory2'
)

export ACTION_CLUSTER="anvil"
export ACTION_NAME="hello"
export ACTION_PROCESSES="1"
export ACTION_WALLTIME_IN_MINUTES="180"

trap 'printf %s\\n "${directories[@]}" | /home/x-joaander/.cargo/bin/row scan --no-progress -a hello - || exit 3' EXIT
for directory in "${directories[@]}"
do
    echo "Hello, $directory!" || { >&2 echo "[ERROR row::action] Error executing command."; exit 2; }
done
```
Notice the selection of 1 task on the `shared` **partition**. This is correct for Anvil,
where the `shared` **partition** allows jobs smaller than one node and charges based
on the number of CPU cores quested.

> Note: If you are using **row** on one of the built-in clusters, then **row** should
> always select the correct partition for your jobs. If you find it does not, please
> open an [issue](https://github.com/glotzerlab/row/issues).

### Submitting jobs

When you are *sure* that the **job script** is correct, submit it with:
```bash
row submit
```

> If your cluster does not default to the correct account, you can set it in
> `workflow.toml`:
> ```toml
> [default.action.submit_options.<cluster name>]
> account = "<my account>"
> ```

### The submitted status

**Row** tracks the **Job IDs** that it submits. Every time you execute `row show status`
(or just about any `row` command), it will execute `squeue` in the background to see
which jobs are still **submitted**.

Use the `row show` family of commands to query details about submitted jobs.
For the `hello` workflow:
```bash
row show status
```
will show:
```plaintext
Action Completed Submitted Eligible Waiting Remaining cost
hello          0         3        0       0    3 CPU-hours
```

Similarly,
```bash
row show directories hello
```
will show something like:
```plaintext
Directory  Status    Job ID
directory0 submitted anvil/5044933
directory1 submitted anvil/5044933
directory2 submitted anvil/5044933
```

`row submit` is safe to use while submitted jobs remain in the queue. **Submitted**
directories are not eligible for execution, so `row submit` will not submit them again.

Wait a moment for the job to finish executing (you can verify with `squeue --me`).
Then `row show status` should indicate that the jobs are *eligible* once
more (recall that the hello example creates no products, so it will never *complete*).

## Next steps

Now you know how to use all the features of **row**. You are ready to deploy it and
*responsibly* execute jobs on thousands of directories in your workflows. Read on in the
next section if you would like to learn how to use **signac** to manage your workspace
and/or write your **action** commands in **Python**.
