# Cache files

**Row** stores cache files in `<project root>/.row` to improve performance. In most
usage environments **row** will automatically update the cache and keep it synchronized
with the state of the workflow and workspace. The rest of this document describes
some scenarios where they cache may not be updated and how you fix the problem.

## Directory values

**Row** caches the **value** of every directory in the workspace. The cache will be
invalid when:
* *You change the contents* of any value file.
* *You change* `value_file` in `workflow.toml`.

> To recover from such a change, execute:
> ```bash
> row clean --directory
> ```

## Submitted jobs

**Row** caches the job ID, directory, and cluster name for every job it submits
to a cluster via `row submit`. **Row** will be unaware of any jobs that you manually
submit with `sbatch`.

> You should submit all jobs via:
> ```bash
> `row submit`
> ```

Copying a project directory (including `.row/`) from one cluster to another (or from
a cluster to a workstation) will preserve the submitted cache. The 2nd cluster cannot
access the job queue of the first, so all jobs will remain in the cache. *Submitting*
jobs on the 2nd cluster will inevitably lead to changes in the submitted cache on both
clusters that cannot be merged.

> Before you copy your project directory, wait for all jobs to finish, then execute
> ```bash
> row show status
> ```
> to update the cache.

## Completed directories

Jobs submitted by `row submit` check if they completed any directories on exit and
update the completed cache accordingly. A completed directory may not be discovered
if:
* *The job is killed* (e.g. due to walltime limits).
* *You execute an action manually* (e.g. `python action.py action directory`).
* *You change products* in `workflow.toml`.
* *You change the name of an action* in `workflow.toml`.

> To discover new completed directories, execute
> ```bash
> row scan
> ```
> This is safe to run any time, including at the same time as any running jobs.

`row scan` only discovers **completed** actions. It *does not* check if a currently
**complete** directory no longer contains an action's products. Therefore, **row** will
still consider directories complete even when:
* *You change products* in `workflow.toml`.
* *You delete product files* in a directory.

> To completely reset the completed cache, execute:
> ```bash
> row clean --completed
> row scan
> ```
> `row clean` will require that you wait until all submitted jobs have completed first.
