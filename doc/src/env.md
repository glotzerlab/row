# Environment variables

## In job scripts

**Row** sets the following environment variables in generated job scripts:

| Environment variable | Value |
|----------------------|-------|
| `ACTION_CLUSTER` | Name of the cluster the action is executing on. |
| `ACTION_NAME` | The name of the action that is executing. |
| `ACTION_PROCESSES` | The total number of processes that this action uses. |
| `ACTION_WALLTIME_IN_MINUTES` | The requested job walltime in minutes. |
| `ACTION_PROCESSES_PER_DIRECTORY` | Set to the value of `action.resources.processes_per_directory`. Unset when `processes_per_submission`.|
| `ACTION_THREADS_PER_PROCESS` | Set to the value of `action.resources.threads_per_process`. Unset when `threads_per_process` is omitted. |
| `ACTION_GPUS_PER_PROCESS` | Set to the value of `action.resources.gpus_per_process`. Unset when `gpus_per_process` is omitted. |

# Set row options

Set any of these environment variables to provide default values for
[command line options].

| Environment variable | Option |
|----------------------|-------------|
| `ROW_CLEAR_PROGRESS`| --clear-progress |
| `ROW_CLUSTER` | --cluster |
| `ROW_COLOR` | --color |
| `ROW_IO_THREADS` | --io-threads |
| `ROW_NO_PROGRESS` | --no-progress |

[command line options]: row/index.md
