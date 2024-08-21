# Set the cluster account

Use the default action to conveniently set the account (or accounts) once in your
`workflow.toml`. It will apply to all actions that do not override the account

```toml
[default.action.submit_options.cluster1]
account = "cluster1-account"
[default.action.submit_options.cluster2]
account = "cluster2-account"

[[action]]
# Will use the defaults above.

[[action]]
# Will use the defaults above.

[[action]]
submit_options.cluster1.account = "alternate-account"
# Will use the "alternate-account" on cluster1 and "cluster2-account" on cluster2.
```

> Note: NCSA Delta assigns `<prefix>-cpu` and `<prefix>-gpu` accounts. Set
> `submit_options.delta.account = "<prefix>"`. **Row** will automatically append the
> `-cpu` or `-gpu` when submitting to the CPU or GPU partitions respectively.
