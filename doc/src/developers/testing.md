# Testing

## Executing unit tests

Run
```bash
cargo test
```
in the source directory to execute the unit and integration tests.

## Writing unit tests

Write tests using standard Rust conventions. All tests must be marked either `#[serial]`
or `#[parallel]` explicitly. Some serial tests set environment variables and/or the
current working directory, which may conflict with any test that is automatically run
concurrently. Check for this with:
```bash
rg --multiline "#\[test\]\n *fn"
```
(see the [safety discussion][1] in `std::env` for details).

[1]: https://doc.rust-lang.org/std/env/fn.set_var.html

## Cluster-specific tests

The file `validate/validate.py` in the source code repository provides a full suite of
tests to ensure that jobs are submitted correctly on clusters. The file docstring
describes how to run the tests.

## Tutorial tests

The tutorial scripts in `doc/src/guide/*.sh` are runnable. These are described in the
documentation using mdBook's anchor feature to include [portions of files][2] in the
documentation as needed. This way, the tutorial can be tested by executing the script.
This type of testing validates that the script *runs*, not that it produces the correct
output. Developers should manually check the tutorial script output as needed.

[2]: https://rust-lang.github.io/mdBook/format/mdbook.html
