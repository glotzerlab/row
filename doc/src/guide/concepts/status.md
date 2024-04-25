# Directory status

For each action, each directory in the workspace that matches the action's
[include condition](../../workflow/action/group.md#include) has a single status:

* **Completed** directories are those where all
  [products](../../workflow/action/index.md#products) are present.
* **Submitted** directories have been submitted to the scheduler and currently remain
  queued or are running.
* **Eligible** directories are those where all
  [previous actions](../../workflow/action/index.md#previous_actions) have been
  completed.
* **Waiting** directories are none of the above.
