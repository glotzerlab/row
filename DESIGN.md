# Row

Row is yet another workflow engine that automates the process of executing **actions** on
*groups* of **directories**.

## Use-cases and implementation notes

* Operate entirely on the file system with no database backend.
* Define an arbitrary number of **actions**.
  * Track which **actions** have been **completed** on each **directory**.
  * Mark actions **complete** when all their product files are present.
  * Indicate the **previous actions** must be **completed** prior to executing another **action**.
* Define **resources** used by an **action**.
  * Make this information to the **job** schedulers and **launchers**.
* Generate a shell script (a **job**) that executes the **action's** **command** on a
  **group*.
  * Optionally use one or more **launchers**.
  * Allow the user to pass additional launcher arguments.
* Submit **jobs** to schedulers (e.g. SLURM).
  * Set typical options from convenience keys (account, walltime, partition).
  * Auto-detect partition based on resources (e.g. gpu vs cpu).
  * Set user-defined submission options.
  * Set user-defined setup commands.
  * Automatically submit **jobs** to the scheduler (e.g. **sbatch**).
  * Track which **actions** have been submitted on each **directory**.
* "Submit" **jobs** to the shell.
  * Run in the local process with a progress bar.
* Define **groups** by:
  * Start from all **directories** in the **workspace**.
  * When set, include only directories that match a list of pointer-operation-value expressions.
  * Sort by directory name first to provide a stable starting point.
  * Sort by a user-defined list of pointers (if set).
  * Optionally split the sorted sequence into groups where the sort keys are identical.
  * Further split groups down to a user-defined maximum size (if set).
* Define **submission groups** by additionally:
  * Start from the set of directories given to the command, or all directories if not provided.
  * After filtering but before sorting and grouping, determine **eligible**
    **directories** filtering out:
    * **Directories** where this **action** is **complete**.
    * **Directories** where the **previous actions** are not **complete**.
    * **Directories** where this **action** has been submitted to the
      scheduler.
  * Optionally require **whole groups**.
* Print the workflow **status**. For each **action**, note the number of directories:
  * Queued in SLURM.
  * Running in SLURM. (OR, just submitted).
  * **Eligible** to run.
  * **Waiting** on previous actions.
* List directories and show completion status, submitted job ID, and user-defined keys from the
  value.

Ideas:
* List scheduler jobs and show useful information.
* Cancel scheduler jobs specific to actions and/or directories.
* Command to uncomplete an action for a set of directories. This would remove the product files and
  update the cache.
* Some method to clear any cache (maybe this instead of uncomplete?). This would allow
  users to discover changed action names, changed products, manually uncompleted
  actions, and deal with corrupt cache files.

## Overview

Row is written in Rust to provide a fast and responsive interface for users with workflows of all
sizes. The tool will often be used on HPC resources where filesystem operations are expensive.
This design strives to avoid O(N) filesystem operations whenever possible. The Row library
implements the types and functions needed for the business logic. The binary parses arguments and
dispatches calls to the library.

### Library module structure

* `row`
  * `cli` - Top level command line commands.
  * `cluster` - Read the `clusters.toml` configuration file, determine the active
    cluster, and make the settings available.
  * `launcher` - Read the `launchers.toml` configuration file, provide code to construct
    commands with launcher prefixes.
  * `project` - Combine the workflow, state, and scheduler into one object and provide
    methods that work with the project as a whole.
  * `scheduler` - Generic scheduler interface and implementations for shell and SLURM.
  * `state` - Row's internal state of the workspace.
  * `workflow` - Type defining workflow and utility functions to find and read it.
  * `workspace` - Methods for working with workspace directories on the filesystem.


## Details

### Workflow definition

A directory that includes the file `workflow.toml` is the root of a Row **project**. When
executed, Row checks in the current working directory (and recursively in parent directories) until
`workflow.toml` is found. `workflow.toml` defines:

- The **workspace**
  - path
  - A static **value file**
- cluster-specific `submit_options`
  - account
  - options
  - setup script
- The **actions**:
  - name
  - command
  - launchers
  - previous_actions
  - products
  - resources
    - processes (either per_submission or per_directory)
    - threads_per_process
    - gpus_per_process
    - walltime (either per_submission or per_directory)
  - Cluster- and action-specific `submit_options`
    - options
    - setup
    - partition
  - group
    - include
    - maximum_size
    - submit_whole
    - sort_by
    - split_by_sort_key

### Detecting completion

After executing an action, Row checks for the file that it **produces**. If that file exists, Row
records that in the state. In this way, the completion check is performed in parallel on the compute
nodes while the **job's** files are likely still in the filesystem cache. It also allows for
restartable **actions** to take require many executions before **completing**.

### Completion staging files

To mark an action complete, Row writes the list of completed **directories** to `.row/completed/
<random-filename>` in a binary file format. This gathers all the completion information without
generating conflicts when multiple jobs attempt to record completion information simultaneously.

Row generates the file in a temporary location and `rename` it to ensure that the entire set of
completed **directories** is read.

### The cache files

Row maintains the state of the workflow in several files:
* `values.json`
  * Cached copies of the user-provided static value file.
* `completed.postcard`
  * Completion status for each **action**.
* `submitted.postcard`
  * The last submitted job ID, referenced by action, directory, and cluster.

When Row updates the state, it collects the completion staging files and updates the entries in the
state accordingly. It also checks the scheduler for all known job IDs and removes any job IDs that
are no longer running. After writing the new state to disk, it removes the completion staging files.
When Row discovers new **directories**, it adds them to the state and populates the cached static
value files. It also removes any **directories** that are no longer present.

For consistency, Row performs these updates **every** time it computes the **status**, is asked
to **submit** jobs, or any command that queries the status. By design the `scan` command does not
update the caches.

### Status report

The status report gives an overview of all defined actions at the **directory** level. Status is
unaware of and does not provide information regarding individual **groups** or **jobs**. Each
time `status` is called, it updates the state.

In addition to the numbers of directories, the status report also summarizes the total number of CPU
(or GPU) hours that **eligible** and **waiting** actions would use.

### Submission

A submission examines all **actions** and **directories** and determines which **groups**
are **eligible** to submit to the **job** scheduler. It then prints a status of the expected total
resource usage and asks for confirmation before submitting the **job(s)** to the scheduler. After
submitting, Row updates the **state** with a record of which scheduler **job** IDs were submitted
for each **action**/**directory** combination.

Provide a --dry-run option that shows the user what **job** script(s) would be submitted.

End the remaining submission sequence on an error return value from the scheduler. Save the cache
to capture the state of any successfully submitted jobs.

### Shell "submissions"

Row is **NOT** a scheduler itself. It will submit and run whatever the user asks it to. For shell
submissions, Row ignores the resource request and assumes the users knows that what they are
submitting is reasonable. For scheduler submissions, Row may make some basic sanity checks
(e.g. underpopulated whole node submissions) but will otherwise pass through whatever script as
generated by user input.

### Group definition

The group defining options **include** and **sort_by** use JSON pointer
syntax. This allows users to select any element of their value when defining groups.

### Launcher configuration

Launchers define prefixes that go in front of commands. These prefixes (e.g.
OMP_NUM_THREADS, srun) take arguments when the user requests certain resources. **Row**
provides built in support for OpenMP and MPI on the built-in clusters that **row
supports. Users can override these and provide new launchers in `launchers.toml`.

Each launcher optionally emits an `executable`, and arguments for
* total number of processes
* threads per process
* gpus per process
when both the launcher defines such an argument and the user requests the relevant
resource.

### Cluster configuration

Row provides default configurations for many national HPC systems. Users can override these defaults
or define new systems in `$HOME/.config/row/clusters.toml`.

A single cluster defines:
* name
* identify: one of
  * by_environment: [string, string]
  * always: bool
* scheduler
* partition (listed in priority order)
  * name
  * maximum_cpus_per_job
  * require_cpus_multiple_of
  * maximum_gpus_per_job
  * require_gpus_multiple_of

The list of partitions provides a mechanism by which the scheduler can determine which partition to
execute on. It will check the partitions in order and return the first partition where a given job
is under both the maximum CPUs and GPUs. Users can override the partition selection in the
**action** definition.

The `require_[c/g]pus_multiple_of` keys are optional. When set, `submit` will ensure that each job
submitted to this partition has an exact multiple of the given number of CPUs or GPUs. This ensures
that users make full use of whole node partitions. Shared partitions should not set these keys.

On creation, a `Cluster` object instantiates the appropriate `Scheduler` and passes it the details
of the launchers and the partitions. Users can submit to `bash` directly with the `none` cluster.

### Environment variables

When executing actions, set environment variables so that the action script is aware of the context
in which it is running:

* `ACTION_CLUSTER` - current cluster name.
* `ACTION_NAME` - the action's name.
* `ACTION_PROCESSES_PER_DIRECTORY` - the number of processes requested per directory. Only set
  when `action.resources.processes` is `per_directory`.
* `ACTION_PROCESSES` - the total number of processes in this submission.
* `ACTION_THREADS_PER_PROCESS` - the number of threads per process. Not set when
  `action.resources.threads_per_process` is missing from the action's definition.
* `ACTION_GPUS_PER_PROCESS` - the number of GPUs per process. Not set when
  `action.resources.gpus_per_process` is missing from the action's definition.
* `ACTION_WALLTIME_IN_MINUTES` - the total walltime in minutes.

### Designing actions for resiliency.

Users should implement idempotent actions. For example, an action that generates a file should
first check if that file exists. When it does, return immediately. If a user loses or corrupts
the **state** file, then rerunning parts of the workflow will not corrupt the workspace.

Users can manually recheck **action completion** whenever needed. For example, if the state file is
lost, corrupted, or a job is killed before completing successfully. Users will use the same command
that row uses internally to check completion status. In large workspaces, checking the completion
status may take a long time, so it should display a progress bar.

## Subcommands

* `init` - create `workflow.toml` and `workspace` if they do not yet exist. (TODO: write init)
* `scan` - scan the workspace for directories that have completed actions.
* `show` - show properties of the workflow:
  * `status` - summarize the status of the workflow.
  * `directories` - list directories in the workflow.

Ideas for other commands, `uncomplete`

## Definitions

- **action**: A user-defined script.
- **complete**: An **action** is **complete** after its **command** has been executed on the
  **directory**.
- **command**: A shell command.
- **condition**: An expression that determines whether a given **directory** is a member of a
- **directory**: A directory on the file system inside the **workspace**.
- **eligible**: An **action** is **eligible** to execute on a **directory** when all **previous
  actions** are **complete**, the **action** is not yet submitted to the scheduler, and the
  **action** itself is not yet **complete**.
- **group**: A ordered collection of **workspace directories**.
- **job**: A shell script that executes an **action** on a **group*.
- **launcher**: A tool that facilitates executing **commands** (e.g. `srun`).
- **value file**: A JSON, file that stores the value of this directory.
- **previous actions**: The **actions** that must all be completed prior to starting a given
  **action**.
- **product** - A file present in the **directory** after the **action** is complete.
- **project**: A filesystem directory that contains a workflow definition and a **workspace**.
- **resources**: The number of processes, cpus per process, gpus per process, and walltime needed.
- **status**: The overall state of the workflow.
- **submission group**: A **group** that is ready for a given **action** to be applied.
- **whole group**: A **submission group** that is identical to the **group** found
  without applying the additional submission filters.
- **workspace**: The location on the file system that contains **directories**.
