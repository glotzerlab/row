# Launcher configuration

Each launcher configuration may set any (or none) of the following keys. The command
prefix constructed from this configuration will be:
```plaintext
{launcher.executable} [option1] [option2] ...
```

See [Built-in launchers](built-in.md) for examples.

## executable

`<launcher name>.<cluster>.executable`: **string** - Set the launcher's executable. May
be omitted.

## gpus_per_process

`<launcher name>.<cluster>.gpus_per_process`: **string**

When `launcher.gpus_per_process` *and* `resources.gpus_per_process` are both
set, add the following option to the launcher prefix:
```plaintext
{launcher.gpus_per_process}{resource.gpus_per_process}
```

## processes

`<launcher name>.<cluster>.processes`: **string**

When `launcher.processes` is set, add the following option to the launcher prefix:
```plaintext
{launcher.processes}{total_processes}
```
where `total_processes` is `n_directories * resources.processes.per_directory` or
`resources.processes.per_submission` depending on the resource configuration.

It is an error when `total_processes > 1` and the action requests *no* launchers that
set `processes`.

## threads_per_process

`<launcher name>.<cluster>.threads_per_process`: **string**

When `launcher.threads_per_process` *and* `resources.threads_per_process` are both
set, add the following option to the launcher prefix:
```plaintext
{launcher.threads_per_process}{resource.threads_per_process}
```
