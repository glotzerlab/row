# Submit the same action to different groups/resources

You can submit the same action to different groups and resources. To do so,
create multiple elements in the action array *with the same name*. Each must use
[`action.group.include`] to select *non-overlapping subsets*. You can use
[`action.from`] to copy all fields from one action and selectively override others.

For example, this `workflow.toml` uses 4 processors on directories with small *N* and 8
those with a large *N*.

```toml
[default.action]
command = "python actions.py --action $ACTION_NAME {directories}"

[[action]]
name = "compute"
products = ["results.out"]
[action.resources]
walltime.per_submission = "12:00:00"
processes.per_directory = 4
[action.group]
maximum_size = 32
[[action.group.include]]
condition = ["/N", "<=", "4096"]

[[action]]
from = "compute"
resources.processes.per_directory = 8
[[action.group.include]]
condition = ["/N", ">", "4096"]
```

[`action.group.include`]: ../../workflow/action/group.md#include
[`action.from`]: ../../workflow/action/index.md#from
