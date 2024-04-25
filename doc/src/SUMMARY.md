# Summary

[Introduction](index.md)

# Guides

- [Installing row](guide/install.md)
- [Tutorial](guide/tutorial/index.md)
  - [Hello, workflow!](guide/tutorial/hello.md)
  - [Managing multiple actions](guide/tutorial/multiple.md)
  - [Grouping directories](guide/tutorial/group.md)
  - [Submitting jobs to a scheduler]()
  - [Best practices for actions]()
- [Using row with Python and signac](guide/python/index.md)
  - [Working with signac projects](guide/python/signac.md)
  - [Writing action commands in Python](guide/python/actions.md)
- [Concepts](guide/concepts/index.md)
  - [Directory status](guide/concepts/status.md)
  - [The row cache](guide/concepts/cache.md)
  - [JSON pointers](guide/concepts/json-pointers.md)
# Reference

- [row](row/index.md)
  - [init](row/init.md)
  - [show status](row/show-status.md)
  - [show directories](row/show-directories.md)
  - [submit](row/submit.md)
  - [scan](row/scan.md)
  - [uncomplete](row/uncomplete.md)

- [`workflow.toml`](workflow/index.md)
  - [workspace](workflow/workspace.md)
  - [cluster](workflow/cluster.md)
  - [action](workflow/action/index.md)
    - [group](workflow/action/group.md)
    - [resources](workflow/action/resources.md)
    - [cluster](workflow/action/cluster.md)
- [`clusters.toml`](clusters/index.md)
  - [Built-in clusters](clusters/built-in.md)
- [Environment variables](env.md)

# Appendix

- [Change log]()
- [Migrating from signac-flow](signac-flow.md)
- [For developers](developers/index.md)
  - [Contributing]()
  - [Code style](developers/style.md)
  - [Testing](developers/testing.md)
  - [Documentation](developers/documentation.md)
- [License]()

-----
[Contributors]()
