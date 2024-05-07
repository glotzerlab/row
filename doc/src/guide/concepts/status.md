# Directory status

For each action, each directory in the workspace that matches the action's
[include condition](../../workflow/action/group.md#include) has a single status:

| Status | Description |
|--------|-------------|
| **Completed** | Directories where all [products](../../workflow/action/index.md#products) are present. |
| **Submitted** | Directories that been submitted to the scheduler and currently remain queued or are running. |
| **Eligible** | Directories where all [previous actions](../../workflow/action/index.md#previous_actions) are **completed**. |
| **Waiting** | None of the above. |

Each directory may have only **one** status, evaluated in the order listed above.
For example, a directory will be **completed** if all of its products are present,
*even when a submitted job is still in queue*.
