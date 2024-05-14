# Managing multiple actions

## Overview

This section explains that directories have a **status** for each action and how those
statuses are determined given the **products** and **previous_actions** properties.

## Summarizing the workflow's status

Each directory in the workspace has a [**status**](../concepts/status.md) for each
action in the workflow. Continuing with the `hello-workspace` example from the previous
section, execute:
```bash
{{#include hello.sh:status1}}
```
to see a summary. You should see:
```plaintext
Action Completed Submitted Eligible Waiting Remaining cost
hello          0         0        3       0    3 CPU-hours
```

## Products and previous actions

Wait. Why are there 3 eligible directories for `hello`? Didn't we already execute that
action? Yes, but the action defines no **products**. **Products** are files that an
action creates in a directory. **Row** considers an action complete when all
**products** are present in a given directory.

Replace the contents of `workflow.toml` with:
```toml
{{#include goodbye-workflow.toml}}
```

This file changes the `hello` action's command to: `echo "Hello, {directory}!" | tee
workspace/{directory}/hello.out` and sets its products to `["hello.out"]`. `tee` writes
its input both to the given file and the terminal, so these two changes together 1)
Write `hello.out` into the directory and 2) Instruct **row** that the action is complete
when `hello.out` is present.

> Note: Commands in **row** should be a single line. Multiple shell commands can be
> chained with `|` to transfer output to input or `&&` to ensure that each command in
> the sequence completes without error before starting the next.

The new `goodbye` action is much like the original `hello` except that it sets
`previous_actions = ["hello"]`. This line tells **row** that `hello` must be
**complete** before `goodbye` may be executed in a given directory.

Execute:
```bash
{{#include hello.sh:status2}}
```
again and you should now see that `hello` is still **eligible**:
```plaintext
Action  Completed Submitted Eligible Waiting Remaining cost
hello           0         0        3       0    3 CPU-hours
goodbye         0         0        0       3    3 CPU-hours
```
`goodbye` is **waiting** because its previous actions are not complete.

Now, submit eligible jobs on `directory1`:
Execute:
```bash
{{#include hello.sh:submit2}}
```

Run `row show status` and see if the output is what you expect.

## Getting more detailed information

`row show status` shows you the *number* of directories in each state. Sometimes you
want to know about *specific* directories. Execute:
```bash
{{#include hello.sh:directories_hello}}
```
to see the details of each directory with respect to the `hello` action. You should see:
```plaintext
Directory  Status
directory0 eligible
directory1 completed
directory2 eligible
```
`directory1` is complete while the others remain eligible. This means
that `directory1` should now be eligible for the `goodbye` action. You now know two
ways to find this out. Try them and see!

Next, submit the `goodbye` action:
```bash
{{#include hello.sh:submit3}}
```
and you should see:
```plaintext
[1/1] Submitting action 'goodbye' on directory directory1 (0 seconds).
Goodbye, directory1!
```

## Next steps

Now you know how to create workflows with multiple actions, control when **row**
considers an action complete on a directory, and prevent an action from running until
all previous actions have completed a directory first. The next section will explain
how to associate arbitrary data with each directory and use that to form groups.
