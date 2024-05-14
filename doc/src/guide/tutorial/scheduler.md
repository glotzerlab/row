# Submitting jobs manually

## Overview

This section gives background information on **clusters**, **nodes**, and
**job schedulers**. It also outlines your responsibilities when using **row** on a
shared resource.

## Clusters

If you are interested in using **row**, you probably have access to a **cluster** where
you can execute the **jobs** in your workflows. **Row** is a tool that makes it easy to
generate *thousands* of **jobs**. Please use it responsibly.

<div class="warning">
<b>DO NOT ASSUME</b> that the jobs that <b>row</b> generates are always correct. It is
<b>YOUR RESPONSIBILITY</b> to <i>understand</i> the contents of a <b>proper job</b>
and <i>validate</i> what <b>row</b> generates before submitting a large number of them.
If not, you could easily burn through your entire allocation with jobs that cost 100
times what you expected them to.
</div>

With that warning out of the way, let's cover some of the basics you need to know.

> Note: This guide is generic and covers only the topics directly related to **row**.
> You can find more information in your **cluster's** documentation.

## Login and compute nodes

**Clusters** are large groups of computers called **nodes**. When you use log in to
a **cluster**, you are given direct access to a **login node**. A typical cluster
might have 2-4 **login nodes**. Login nodes are **SHARED RESOURCES** that many others
actively use. You should use **login nodes** to edit text files, submit jobs, check
on job status, and *maybe* compile source code. In general, you should restrict your
**login node** usage to commands that will execute and complete *immediately* (or within
a minute).

You should execute everything that takes longer than a minute or otherwise uses
extensive resources on one or more **compute nodes**. Typical clusters have *thousands*
of compute nodes.

## Job scheduler

The **job scheduler** controls access to the **compute nodes**. It ensures that each
**job** gets *exclusive* access to the resources it needs to execute. To see what jobs
are currently scheduled, run
```bash
squeue
```
on a **login node**.

> Note: This guide assumes your cluster uses Slurm. Refer to your cluster's
> documentation for equivalent commands if it uses a different scheduler.

You will likely see some **PENDING** and **RUNNING** jobs. **RUNNING** jobs have been
assigned a number of (possibly fractional) **compute nodes** and are currently executing
on those resources. **PENDING** jobs are waiting for the **resources** that they request
to become available.

## Submitting a job

You should understand how to submit a job manually before you use **row** to automate
the process. Start with a "Hello, world" job. Place this text in a file called `job.sh`:
```bash
#!/bin/bash
#SBATCH --ntasks=1
#SBATCH --time=1

echo "Hello, World!"
taskset -cp $$
```

The first line of the script tells Slurm that this is a bash script. The next two are
options that will be processed by the scheduler:
* `--ntasks=1` requests that the **job scheduler** allocate *at least* 1 CPU core (it
  *may* allocate and charge your account for more, see below).
* `--time=1` indicates that the script will execute in 1 minute or less.

The last two lines are the body of our script. This example prints "Hello, World!"
and then the list of CPU cores the **job** is allowed to execute on.

To submit the **job** to the **scheduler**, execute:
```bash
sbatch job.sh
```

> Note: Check the documentation for your cluster before submitting this job. If
> `sbatch` reported an error, you may also need to set `--account`, `--partition`, or
> other options.

When `sbatch` successfully submits, it will inform you of the **job's ID**. You can
monitor the status of the **job** with:
```bash
squeue --me
```

The **job** will show first in the `PENDING` state. Once there is a **compute node**
available with the requested resources, the **scheduler** will start the job executing
on that **node**. `squeue` will then report that the job is `RUNNING`. It should
complete after a few moments, at which point `squeue` will no longer list the job.

At this time, you should see a file `slurm-<Job ID>.out` appear in your current
directory. Inspect its contents to see the output of the script. For example:
```
Hello, World!
pid 830675's current affinity list: 99
```

> Note: If you see more than one number in the affinity list (e.g. 0-127), then the
> **scheduler** gave your job access to more CPU cores than `--ntasks=1` asks for.
> This may be because your **cluster** allocates **whole nodes** to jobs. Refer to
> your **cluster's** documentation to see specific details on how jobs are allocated
> to nodes and charged for resource usage. Remember, it is **YOUR RESPONSIBILITY** (not
> **row's**) to understand whether `--ntasks=1` costs 1 CPU-hour per hour or more (e.g.
> 128 CPU-hours per hour). If your cluster lacks a *shared* partition, then you need to
> structure your **actions** and **groups** in such a way to use all the cores you are
> given or else the resources are wasted.

## Requesting resources

There are many types of resources that you can request in a job script. One is time.
The above example requested 1 minute (`--time=1`). The `--time` option is a promise
to the **scheduler** that your job will complete in less than the given time. The
**scheduler** will use this information to efficiently plan other jobs to run after
yours. If your job is still running after the specified time limit, the **scheduler**
will terminate your job.

Another resource you can request is more CPU cores. For example, add `--cpus-per-task=4`
to the above script:
```bash
#!/bin/bash
#SBATCH --ntasks=1
#SBATCH --cpus-per-task=4
#SBATCH --time=1

echo "Hello, World!"
taskset -cp $$
```

Submit this script and see if the output is what you expect.

You can also request GPUs, memory, licenses, and others. In the next section, you will
learn how to use **row** to automatically generate job scripts that request **CPUs**,
**GPUs**, and **time**. You can set
[`custom` submit options](../../workflow/action/submit-options.md) to request others.

Most **clusters** also have separate **partitions** (requested with
`--partition=<partition>` for certain resources (e.g. GPU). See your **cluster's**
documentation for details.

## Next steps

Now that you know all about **compute nodes** and **job schedulers**, you can now learn
how to define these resource requests in `workflow.toml` so that **row** can generate
appropriate **job scripts**.
