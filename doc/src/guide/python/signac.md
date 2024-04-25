# Working with signac projects

[signac](https://signac.io) is a Python library that helps you manage your workspace
directories. Define **values** as dictionaries, and **signac** will automatically create
directories for you. **signac** also offers many APIs to iterate, search, and perform
many other tasks.

To use **row** with **signac**, place `workflow.toml` at the root of your **signac**
project and add the lines:
```toml
[workspace]
value_file = "signac_statepoint.json"
```

That is all. Now you can use any values in your state points to form **groups**.

> Note: **signac** has a rich command line interface as well. You should consider using
> **signac** even if you are not a Python user.
