# default

The `default` table sets default options.

Example:

```toml
[default.action.submit_options.cluster1]
account = "my_account"
```

## action

`default.action`: **table** - accepts *any* key that is valid in
an [action array element](action/index.md). When an action array element omits a key,
the default key is used. When both the action **and** the default action omit a key,
the individually documented "when omitted" behavior takes effect.

> Note: This rule applies to all sub-keys as well. For example:
> ```toml
> [default.action.resources]
> processes.per_submission = 8
> walltime.per_directory = "02:00:00"
>
> [[action]]
> name = "action"
> command = "command {directory}"
> resources.processes.per_submission = 16
> ```
> Will result in an action that sets `processes.per_submission == 16` and
> `walltime.per_directory == "02:00:00"`.
