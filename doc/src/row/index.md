# row

`row` is a command line tool. Usage:
```bash
row [OPTIONS] <COMMAND>
```

`<COMMAND>` must be one of:
* [`init`](init.md)
* [`submit`](submit.md)
* [`show`](show/index.md)
* [`scan`](scan.md)
* [`clean`](clean.md)

<div class="warning">
You should execute at most <b>one</b> instance of <b>row</b> at a time for a given
project. <b>Row</b> maintains a cache and concurrent invocations may corrupt it. The
<code>scan</code> command is excepted from this rule.
</div>

## `[OPTIONS]`

`[OPTIONS]` may be one or more of the following. These top level options and may be
placed before or after the `<COMMAND>`. For example:
```bash
row -v show status
```
and
```bash
row show status -v
```
are equivalent.

### `--clear-progress`

When set, clear any progress bars when **row** exits. By default, progress bars are
left on the screen. Set the environment variable `ROW_CLEAR_PROGRESS` to change the
default.

### `--cluster`

Set the name of the cluster to check for submitted jobs. By default, **row** autodetects
the cluster based on the rules in [`clusters.toml`](../clusters/index.md). Set the
environment variable `ROW_CLUSTER` to change this default.

**Row** always shows the count of submitted jobs from *all* clusters known in the
submitted job cache. This option controls which cluster's cache will be updated. The
`none` cluster has no scheduler, so you can take advantage of this to show the workflow
status without running `squeue` or updating the submitted jobs.
```bash
row show status --cluster none
```
<div class="warning">
To avoid corrupting your submitted job cache, never set <code>--cluster</code> to one
that has a scheduler. If autodetection is not possible for your cluster, then you should
set the environment variable <code>ROW_CLUSTER</code> to ensure that all <b>row</b>
commands interact with the correct environment and cache.
</div>

### `--color`

Pass the option `--color <WHEN>` to control when **row** displays colored/styled
terminal output. `<WHEN>` may be `auto` (the default), `always`, or `never`.
Set the environment variable `ROW_COLOR` to change the default. You may also set
any of the [CLICOLORS](https://bixense.com/clicolors/) environment variables to
control the display of colored/styled output.

### `--io-threads`

Sets the number of threads that **row** spawns when scanning directories and files.
The default value of 8 is suitable for networked file systems. You may find that
`--io-threads=4` performs better on fast local drives. Set the environment variable
`ROW_IO_THREADS` to change the default.

### `--no-progress`

Hide all progress bars. By default, **row** shows progress bars. Set the environment
variable `ROW_NO_PROGRESS` to change the default.

### `--verbose`

(also: `-v`)

Increase the logging verbosity. By default, **row** shows errors and warnings. Repeat
this option to increase the verbosity up to 3 times to show information, debug, and
trace messages. For example:
```bash
row -vvv show status
```

### `--quiet`

(also: `-q`)

Decrease the logging verbosity. The first `--quiet` will hide warnings. Pass `--quiet`
a second time to also hide errors.

### `--help`

Print a help message and exit. Use `-h` to see a shorter help summary.

### `--version`

(also: `-V`)

Print the version number and exit.
