# Contributing

Contributions are welcomed via [pull requests on GitHub][github]. Contact the **row**
developers before starting work to ensure it meshes well with the planned development
direction and follows standards set for the project.

[github]: https://github.com/glotzerlab/gsd/row

## Features

### Implement functionality in a general and flexible fashion

New features should be applicable to a variety of use-cases. The **row** developers can
assist you in designing flexible interfaces.

### Maintain performance of existing code paths

Expensive code paths should only execute when requested.

### Maintain compatibility

New features should be opt-in and *preserve the behavior* of all existing user scripts.

## Version control

### Base your work off the correct branch

Base all bug fixes and new features on `trunk`.

### Propose a minimal set of related changes

All changes in a pull request should be *closely related*. Multiple change sets that are
loosely coupled should be proposed in *separate pull requests*.

### Agree to the Contributor Agreement

All contributors must agree to the **Contributor Agreement** before their pull request
can be merged.

### Set your git identity

Git identifies every commit you make with your name and e-mail. [Set your identity][id]
to correctly identify your work and set it *identically on all systems* and accounts
where you make commits.

[id]: http://www.git-scm.com/book/en/v2/Getting-Started-First-Time-Git-Setup

## Source code

### Use a consistent style

Follow all guidelines outlined in the [Code style](style.md) section of the
documentation.

### Document code with comments

Write Rust documentation comments for traits, functions, etc. Also comment complex
sections of code so that other developers can understand them.

### Compile without warnings

Your changes should compile without warnings.

## Tests

### Write unit tests

Add unit tests for all new functionality and bug fixes.

### Test validity

Run research-scale simulations using new functionality and ensure that it behaves as
intended.

## User documentation

### Write user documentation

Document all new configuration keys, command line options, command line tools,
and any important user-facing change in the mdBook documentation.

### Tutorial

When applicable, update or write a new tutorial or how-to guide.

### Add developer to the credits

Update the contributors documentation to name each developer that has contributed to the
code.

### Add a change log entry

Add a short concise entry describing the change in `doc/src/release-notes.md`.
