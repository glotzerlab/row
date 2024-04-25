# Testing

## Executing unit tests

Run
```bash
cargo test
```
in the source directory to execute the unit and integration tests.

## Cluster-specific tests

TODO: Develop a strategy to test that both cluster auto-detection and the generated
jobs function correctly.

## Tutorial tests

Tutorial scripts should be testable. Write scripts using mdBook's anchor feature to
include [portions of files](https://rust-lang.github.io/mdBook/format/mdbook.html) in
the documentation as needed. This way, the tutorial can be tested by executing the
script. This type of testing validates that the script *runs*, not that it produces
the correct output.
