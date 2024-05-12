# Code style

## Rust

**Row's** rust code follows the [Rust style guide][1]. **Row's** [pre-commit][2]
configuration applies style fixes with `rustfmt` checks for common errors with `clippy`.

[1]: https://doc.rust-lang.org/style-guide/index.html
[2]: https://pre-commit.com/

## Python

**Row's** pre-commit configuration both formats and checks Python code with `ruff`.

## Markdown

Wrap **Markdown** files at 88 characters wide, except when not possible (e.g. when
formatting a table). Follow layout and design patterns established in existing markdown
files.

## Spelling/grammar

Contributors **must** configure their editors to perform spell checking (and preferably
grammar checking as well). **Row's** pre-commit runs
[typos](https://github.com/crate-ci/typos) which has a low rate of false positives.
Developers *should* also configure a more thorough checker of their choice to ensure
that code comments and documentation are free of errors. Suggested tools:
* [typos](https://github.com/crate-ci/typos)
* [ltex-ls](https://github.com/valentjn/ltex-ls)
* [cspell](https://cspell.org/)
