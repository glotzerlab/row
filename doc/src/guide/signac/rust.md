# Writing actions in Rust

In Rust, you use [serde-json] to read signac state points. One way to organize
your actions is to build a single binary and use [clap] to parse action
subcommands. See the [hoomd-rs workflow tutorial] for a complete simulation
workflow with actions implemented in Rust.

[serde-json]: https://docs.rs/serde_json
[clap]: https://docs.rs/clap
[hoomd-rs workflow tutorial]: https://hoomd-rs.readthedocs.io/en/stable/workflow-tutorial
