# cargo-public-api

List and diff the public API of Rust library crates between releases and commits. Allows you to detect breaking API changes and semver violations. Relies on and automatically builds rustdoc JSON, for which a recent version of the Rust nightly toolchain must be installed.

# Installation

```bash
# Install cargo-public-api with a recent regular stable Rust toolchain
cargo install cargo-public-api

# Ensure nightly-2022-05-19 or later is installed so cargo-public-api can build rustdoc JSON for you
rustup install nightly
```

# Usage

## List the public API

This example lists the public API of the ubiquitous `regex` crate. First let's clone the repo:

```bash
git clone https://github.com/rust-lang/regex
cd regex
```

Now we can list the public API of `regex` by doing

```bash
cargo public-api
```

which will print the public API of `regex` with one line per public item in the API:

<img src="doc/img/list-public-api.jpg" width="550" alt="colored output of listing public api">

## Diff the public API

To diff the API between say **1.3.0** and **1.4.0** of `regex`, use `--diff-git-checkouts` while standing in the git repo. Like this:

```bash
cargo public-api --diff-git-checkouts 1.3.0 1.4.0
```

and the API diff will be printed:

```
Removed items from the public API
=================================
(none)

Changed items in the public API
===============================
(none)

Added items to the public API
=============================
pub fn regex::Match::range(&self) -> Range<usize>
pub fn regex::RegexSet::empty() -> RegexSet
pub fn regex::RegexSet::is_empty(&self) -> bool
pub fn regex::SubCaptureMatches::clone(&self) -> SubCaptureMatches<'c, 't>
pub fn regex::bytes::Match::range(&self) -> Range<usize>
pub fn regex::bytes::RegexSet::empty() -> RegexSet
pub fn regex::bytes::RegexSet::is_empty(&self) -> bool
pub fn regex::bytes::SubCaptureMatches::clone(&self) -> SubCaptureMatches<'c, 't>
```

You can also manually do a diff by writing the full list of items to a file for two different versions of your library and then do a regular `diff` between the files.

# Expected output

In general, output aims to be character-by-character identical to the textual parts of the regular `cargo doc` HTML output. For example, [this item](https://docs.rs/bat/0.20.0/bat/struct.PrettyPrinter.html#method.input_files) has the following textual representation in the rendered HTML:

```
pub fn input_files<I, P>(&mut self, paths: I) -> &mut Self
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
```

and `cargo public-api` represents this item in the following manner:

```
pub fn bat::PrettyPrinter::input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>
```

If we normalize by removing newline characters and adding some whitespace padding to get the alignment right for side-by-side comparison, we can see that they are exactly the same, except an irrelevant trailing comma:

```
pub fn                     input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>,
pub fn bat::PrettyPrinter::input_files<I, P>(&mut self, paths: I) -> &mut Self where I: IntoIterator<Item = P>, P: AsRef<Path>
```

# Blanket implementations

By default, blanket implementations such as `impl<T> Any for T`, `impl<T> Borrow<T> for T`, and `impl<T, U> Into<U> for T where U: From<T>` are omitted from the list of public items of a crate. For the vast majority of use cases, blanket implementations are not of interest, and just creates noise.

Use `--with-blanket-implementations` if you want to include items of blanket implementations in the output:
```bash
cargo public-api --with-blanket-implementations
```

# Output formats

Currently there are two output formats. You can choose between `--output-format plain` (default, supports syntax highlighting) and `--output-format markdown`.

# Target audience

Maintainers of Rust libraries that want to keep track of changes to their public API.

# Limitations

See [`[limitation]`](https://github.com/Enselic/cargo-public-api/labels/limitation)
labeled issues.

# Compatibility matrix

| cargo-public-api | Understands the rustdoc JSON output of  |
| ---------------- | --------------------------------------- |
| v0.11.x          | nightly-2022-05-19                      |
| v0.10.x          | nightly-2022-03-14 — nightly-2022-05-18 |
| v0.5.x           | nightly-2022-02-23 — nightly-2022-03-13 |
| v0.2.x           | nightly-2022-01-19 — nightly-2022-02-22 |
| v0.0.5           | nightly-2021-10-11 — nightly-2022-01-18 |

# Implementation details

This utility is implemented with and adds conveniences on top of the [public-api](https://crates.io/crates/public-api) library (https://github.com/Enselic/cargo-public-api/tree/main/public-api).

# Development tips

See [development.md](./doc/development.md).

# Maintainers

- [Enselic](https://github.com/Enselic)
- [douweschulte](https://github.com/douweschulte)
