# Migrating from signac-flow

**Row** is a spiritual successor to
[signac-flow](https://docs.signac.io/projects/flow/en/latest/). Many concepts and
common usage patters map directly from **signac-flow** to **row**.

Concepts:
| flow | row |
|------|-----|
| *job* | *directory* |
| *cluster job* | *job* |
| *statepoint* | *value* |
| *operation* | [`action`](workflow/action/index.md) in `workflow.toml`|
| *group* | A command may perform multiple steps. |
| *label* | Not implemented. |
| *hooks* | Not implemented. |
| *environments* | [`clusters.toml`](clusters/index.md) |
| `project.py` | [`workflow.toml`](workflow/index.md) combined with [`actions.py`](guide/python/actions.md) |

Commands:
| flow | row |
|------|-----|
| `project.py status` | [`row show status`](row/show/status.md) |
| `project.py status --detailed` | [`row show directories --action action`](row/show/directories.md) |
| `project.py run` | [`row submit --cluster=none`](row/submit.md) |
| `project.py run --parallel` | A command *may* execute [group members][group] in [parallel]. |
| `project.py exec ...` | Execute your action's command in the shell. |
| `project.py submit` | [`row submit`](row/submit.md) |
| `project.py submit --partition <PARTITION>` | `row submit` *automatically* selects appropriate partitions. |
| `project.py submit -n <N>` | [`row submit -n <N>`](row/submit.md) |
| `project.py submit --pretend` | [`row submit --dry-run`](row/submit.md) |
| `project.py submit --bundle <N>` | [`group`][group] in `workflow.toml` |
| `project.py submit --bundle <N> --parallel` | A command *may* execute [group members][group] in [parallel]. |
| `project.py submit -o <PATTERN>` | [`row submit --action <PATTERN>`](row/submit.md) |
| `project.py <command> -j [JOB_ID1] [JOB_ID2] ...` | `row <command> [JOB_ID1] [JOB_ID2] ...` |
| `project.py <command> -j a1234` | `cd workspace; row <command> a1234*` |
| `project.py <command> -f <FILTER>` | `row <command> $(signac find <FILTER>)` |

Conditions:
| flow | row |
|------|-----|
| postcondition: `isfile` | [`products`](workflow/action/index.md#products) |
| postcondition: others | Not implemented. |
| precondition: `after` | [`previous_actions`](workflow/action/index.md#previous_actions) |
| precondition: state point comparison | [`include`](workflow/action/group.md#include) |
| precondition: others | Not implemented. |
| aggregation | [`group`][group] in `workflow.toml` |
| aggregation: `select` | [`include`](workflow/action/group.md#include) |
| aggregation: `sort_by` | [`sort_by`] |
| aggregation: `groupby` | [`sort_by`] and [`split_by_sort_key=true`](workflow/action/group.md#split_by_sort_key) |
| aggregation: `groupsof` | [`maximum_size`](workflow/action/group.md#maximum_size) |

Execution:
| flow | row |
|------|-----|
| `operation(cmd=...)` | [command](workflow/action/index.md#command) in `workflow.toml` |
| directives: `executable` | `command = "<executable> actions.py {directories}"` |
| directives: `np`, `ngpu`, `omp_num_threads`, `walltime` | [resources](workflow/action/resources.md) in `workflow.toml` |
| directives: Launch with MPI | [`launchers`](workflow/action/index.md#launchers) `= ["mpi"]` |
| directives: Launch with OpenMP | [`launchers`](workflow/action/index.md#launchers) `= ["openmp"]` |
| template job script: `script.sh` | [`submit_options`](workflow/action/submit-options.md) in `workflow.toml` |

[group]: workflow/action/group.md
[parallel]: guide/concepts/thread-parallelism.md
[`sort_by`]: workflow/action/group.md#sort_by
