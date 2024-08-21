# Installing row

## Installing binaries with conda

**Row** is available on [conda-forge] for the *linux-64*, *linux-aarch64*, *osx-64*,
*osx-arm64* architectures. Install with [micromamba]:

```bash
micromamba install row
```

[conda-forge]: https://conda-forge.org/
[micromamba]: https://mamba.readthedocs.io/en/latest/user_guide/micromamba.html

## Installing binaries manually

Download binary from [latest row release] that matches your operating system and
hardware architecture.
* `x86_64-unknown-linux-gnu` - Linux x86_64 (*Intel/AMD 64-bit*).
* `aarch64-apple-darwin` - Mac arm64 (*Apple Silicon*).

Extract the file:
```bash
tar -xvf row-*.tar.zst
```

Place the executable `row` in a directory that is on your `$PATH`.

> Note: If you are unsure what your system architecture is, execute `uname -sm`.

[latest row release]: https://github.com/glotzerlab/row/releases

## Building the latest release from source

Install [Rust]. Then execute:

```bash
cargo install row --locked
```

Ensure that `$HOME/.cargo/bin` is on your `$PATH`.

> Note: You can keep your installation up to date with **[cargo-update]**.

[Rust]: https://doc.rust-lang.org/stable/book/
[cargo-update]: https://github.com/nabijaczleweli/cargo-update

## Building the latest development version

Clone the repository:
```bash
git clone git@github.com:glotzerlab/row.git
```

Install row:
```bash
cargo install --path row --locked
```

Ensure that `$HOME/.cargo/bin` is on your `$PATH`.
