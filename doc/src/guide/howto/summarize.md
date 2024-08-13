# Summarize directory groups with an action

Set [`submit_whole=true`] to ensure that an action is always submitted on the
*whole* group of included directories. For example, you could use this in an analysis
action that averages over replicates. Say your directories have values like
```json
{
  "temperature": 1.0,
  "pressure": 0.3,
  "replicate": 2
}
```
with many directories at the same *temperature* and *pressure* and different
values of *replicate*. You could average over all replicates at the same *temperature*
and *pressure* with an action like this:
```toml
[[action]]
name = "average"
[action.group]
sort_by = ["/temperature", "/pressure"]
split_by_sort_key = true
submit_whole = true
```

Actions that summarize output have no clear location to place output files (such as
plots). Many users will write summary output to the project root and omit `products`.
In this way, you can rerun the analysis whenever needed as **row** will never consider
the action **complete**.

[`submit_whole=true`]: ../../workflow/action/group.md#submit_whole
