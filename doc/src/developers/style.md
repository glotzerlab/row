# Code style

## Rust

**Row's** rust code follows the [Rust style guide][1]. **Row's** [prek][2]
configuration applies style fixes with `rustfmt` and checks for common errors with
`clippy`.

[1]: https://doc.rust-lang.org/style-guide/index.html
[2]: https://prek.j178.dev/

## Python

**Row's** `prek` configuration both formats and checks Python code with `ruff`.

## Markdown

Wrap **Markdown** files at 88 characters wide, except when not possible (e.g. when
formatting a table). Follow layout and design patterns established in existing markdown
files. Use reference-style links for long URLs.

## Spelling/grammar

Contributors **must** configure their editors to perform spell checking.
Suggested tools:
* [codebook](https://github.com/blopker/codebook)
* [cspell](https://cspell.org/)
* [typos](https://github.com/crate-ci/typos)
