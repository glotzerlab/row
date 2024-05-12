# Summary

[Introduction](index.md)

# Guides

- [Installing row](guide/install.md)
- [Tutorial](guide/tutorial/index.md)
  - [Hello, workflow!](guide/tutorial/hello.md)
  - [Managing multiple actions](guide/tutorial/multiple.md)
  - [Grouping directories](guide/tutorial/group.md)
  - [Submitting jobs manually](guide/tutorial/scheduler.md)
  - [Requesting resources with row](guide/tutorial/resources.md)
  - [Submitting jobs with row](guide/tutorial/submit.md)
- [Using row with Python and signac](guide/python/index.md)
  - [Working with signac projects](guide/python/signac.md)
  - [Writing action commands in Python](guide/python/actions.md)
- [Concepts](guide/concepts/index.md)
  - [Best practices](guide/concepts/best-practices.md)
  - [Process parallelism](guide/concepts/process-parallelism.md)
  - [Thread parallelism](guide/concepts/thread-parallelism.md)
  - [Directory status](guide/concepts/status.md)
  - [JSON pointers](guide/concepts/json-pointers.md)
  - [Cache files](guide/concepts/cache.md)
# Reference

- [row](row/index.md)
  - [init](row/init.md)
  - [submit](row/submit.md)
  - [show](row/show/index.md)
    - [show status](row/show/status.md)
    - [show directories](row/show/directories.md)
    - [show cluster](row/show/cluster.md)
    - [show launchers](row/show/launchers.md)
  - [scan](row/scan.md)
  - [clean](row/clean.md)

- [`workflow.toml`](workflow/index.md)
  - [workspace](workflow/workspace.md)
  - [submit_options](workflow/submit-options.md)
  - [action](workflow/action/index.md)
    - [group](workflow/action/group.md)
    - [resources](workflow/action/resources.md)
    - [submit_options](workflow/action/submit-options.md)
- [`clusters.toml`](clusters/index.md)
  - [cluster](clusters/cluster.md)
  - [Built-in clusters](clusters/built-in.md)
- [`launchers.toml`](launchers/index.md)
  - [Launcher configuration](launchers/launcher.md)
  - [Built-in launchers](launchers/built-in.md)
- [Environment variables](env.md)

# Appendix

- [Release notes](release-notes.md)
- [Migrating from signac-flow](signac-flow.md)
- [For developers](developers/index.md)
  - [Contributing](developers/contributing.md)
  - [Code style](developers/style.md)
  - [Testing](developers/testing.md)
  - [Documentation](developers/documentation.md)
- [License](license.md)

-----
[Contributors](contributors.md)
