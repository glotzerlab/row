# Best practices

Follow these guidelines to use **row** effectively.

## Exit actions early when products already exist.

There are some cases where **row** may fail to identify when your action completes:

* Software exits with an unrecoverable error.
* Your job exceeds its walltime and is killed.
* And many others...

To ensure that your action executes as intended, you should **check for the existence
of product files** when your action starts and **exit immediately** when they already
exist. This way, resubmitting an already completed job will not needlessly recompute
results or overwrite files you intended to keep.

## Write to temporary files and move them to the final product location.

For example, say `products = ["output.dat"]`. Write to `output.dat.in_progress`
while your calculation executes. Once the action is fully complete, *move*
`output.dat.in_progress` to `output.dat`.

If you wrote directly to `output.dat`, **row** might identify your computation as
**complete** right after it starts. This pattern also allows you to *continue* running
one calculation over several job submissions. Move the output file to its final location
only after the final submission completes the calculation.

## Group directories whenever possible, but not to an extreme degree.

The **scheduler** does an excellent job handling the queue. However, there is some
overhead and the scheduler can only process so many jobs at a time. Your cluster may
even limit how many jobs you are allowed to queue. So please don't submit thousands of
jobs at a time to your cluster. You can improve your workflow's throughput by grouping
directories together into a smaller number of jobs.

Group jobs that execute quickly in serial with `processes.per_submission` and
`walltime.per_directory`. After a given job has waited in the queue, it can process many
directories before exiting. Limit group sizes so that the total wall time of the job
remains reasonable.

Group jobs that take a longer time in parallel using MPI partitions,
`processes.per_directory` and `walltime.per_submission`. Limit the group sizes to a
relatively small fraction of the cluster (*except on Leadership class machines*).
Huge parallel jobs may wait a long time in queue before starting. Experiment with the
`group.maximum_size` value and find a good job size (in number of nodes) that balances
queue time vs. scheduler overhead.
