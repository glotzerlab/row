# Assigning values to directories

## Overview

This section shows how you can assign a **value** to each directory and use a command
**template** to access portions of that value when submitting **actions**.

## Directory values

So far, this tutorial has demonstrated small toy examples. In practice, any workflow
that you need to execute on a cluster likely has hundreds or thousands of directories -
each with different parameters. You could try to encode these parameters into the
directory names, but *please don't* - it quickly becomes unmanageable. Instead, you
should include a [JSON] file in each directory that identifies its **value**.

[JSON]: https://www.json.org

> Note: For pedagogical reasons, this next code block manually creates directory names
> and value files. In practice, you will likely find [signac] more
> convenient to work with - it will create the JSON files and directories for you with
> a cleaner syntax. This tutorial will cover **row** ↔ **signac** interoperation in a
> later section.

[signac]: ../python/signac.md

Create a new workflow project and place JSON files in each directory:
```bash
{{#include value.sh:init}}
```
The JSON files must all have the same name. Instruct **row** to read these files
with the `workspace.value_file` key in `workflow.toml`:

```toml
{{#include value-workflow.toml:workspace}}
```

Once you create a directory with a **value** file, that value **MUST NOT CHANGE**. Think
of it this way: The results of your computations (the final contents of the directory)
are a mathematical *function* of the **value**. When you want to know the results for
another value, *create a new directory with that value!*. **row** assumes this data
model and [caches] all value files so that it does not need to read thousands of files
every time you execute a **row** command.

[caches]: ../concepts/cache.md

## Passing values to commands

Now that your workspace directories have **values**, you can pass them to your
commands using **template parameters**. You have already used one template parameter:
`{directory}`. Each **template parameter** name is surrounded by curly braces.

[JSON] files store a (possibly nested) key/value mapping. Use a [*JSON pointer*] to
reference a portion of the directory's value by placing the [*JSON pointer*] between
curly braces. Add the following section to `workflow.toml` that uses **template
parameters** in the action's `command`:

```toml
{{#include value-workflow.toml:action}}
```

[*JSON pointer*]: ../concepts/json-pointers.md

Execute the following (and answer yes at the prompt):
```bash
{{#include value.sh:submit}}
```

You should see:
```plaintext
directory1, seed: 0, pressure: 1.5
directory2, seed: 1, pressure: 1.5
directory3, seed: 0, pressure: 2.1
directory4, seed: 1, pressure: 2.1
```

Consider how you would use this for your own workflows. For example:
```toml
command = './application -s {/seed} -p {/pressure} -o workspace/{directory}/out'
```

# Next steps

You have now assigned **values** to each **directory** in the workspace and learned
how you can use these **values** with **template parameters** in the **command**. The
next section will show you how to use **values** to form **groups**.
