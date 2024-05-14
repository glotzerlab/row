# Hello, workflow!

## Overview

This section introduces you the concepts of **row projects**, **workspaces**, and
**actions**. It also demonstrates how to **submit** actions.

## Initializing a project

All **row** commands operate on the current **project** directory which contains
the file `workflow.toml` and a **workspace** directory. Create a new directory
`hello-workflow` and initialize a project there:
```bash
{{#include hello.sh:init}}
```

**Row** helps you organize your workflow into distinct **actions** that each execute
on one (or more) directories in the **workspace**. `row init` created an empty
workspace. You must add directories so that your actions have something to work on:
```bash
{{#include hello.sh:create}}
```
These directories may have any name. **Row** will identify any **directory** at the top
level of `workspace` as a potential target for each **action**.

## Defining a workflow

Now that you have a **workspace**, you can define an **action** to execute. Replace the
empty `workflow.toml` file that `row init` created with:
```toml
{{#include hello-workflow.toml}}
```
`workflow.toml` is a [TOML](https://toml.io) file. In `workflow.toml`, `action` is an
array of tables and each `[[action]]` line adds a new element. Each element **requires**
the keys `name` and `command`. There are many optional keys you will learn about in
later tutorials, or you can skip ahead and
[read the action reference documentation](../../workflow/action/index.md).

`name` is a string that sets the name of the action. `command` is a template for a shell
command that **row** will execute on each directory. The `{directory}` in `command`
will be replaced with directory names.

## Submitting jobs

Execute:
```bash
{{#include hello.sh:submit}}
```
to submit jobs that execute the actions in the workflow.

You should see:
```plaintext
Submitting 1 job that may cost up to 3 CPU-hours.
Proceed? [Y/n]:
```

The cost is 3 CPU-hours because **action** defaults to 1 CPU-hour per directory
(later sections in this tutorial will cover resource costs in more detail).
`echo "Hello, {directory}!"` is certainly not going to take that long, so confirm
with `y` and then press enter. You should then see the action execute:
```plaintext
[1/1] Submitting action 'hello' on directory directory0 and 2 more.
Hello, directory0!
Hello, directory1!
Hello, directory2!
```

> Note: If you are following this tutorial on a cluster, `row submit` may submit the
> job to the queue instead! Later tutorial sections will cover clusters in more detail.
> For now, you can use `row submit --cluster=none` to execute actions directly in your
> current terminal session.

As an exercise, read the [`row submit`](../../row/submit.md) documentation and see if
you can find a way to submit this workflow only on `directory2`.

## Next steps

You have created your first **row** workflow and executed it! The next section of this
tutorial will show you how to configure one **action** that will execute after another
action **completes**.
