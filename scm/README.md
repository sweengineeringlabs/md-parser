# md-parser

A Rust library that parses Markdown into structured slide data and separately
validates estimated content limits. It uses only the Rust standard library
and returns an in-memory `Deck`; it does not render or export presentations.

## Quick start

Add the Git dependency to your application's `Cargo.toml`. The crates.io
package named `md-parser` is a different project.

```toml
[dependencies]
md-parser = { git = "https://github.com/sweengineeringlabs/md-parser" }
```

For a complete parse-and-validate snippet, see the
[project quick start](https://github.com/sweengineeringlabs/md-parser/blob/main/README.md#quick-start).

From this directory, run `cargo run --example basic`. It parses two slides,
prints `parsed 2 slides`, and creates no output file. Generate the API reference
with `cargo doc --no-deps --open`.

See the [rules definition](https://github.com/sweengineeringlabs/md-parser/blob/main/docs/3-design/rules_defination.md),
[architecture](https://github.com/sweengineeringlabs/md-parser/blob/main/docs/3-design/architecture.md), and
[developer guide](https://github.com/sweengineeringlabs/md-parser/blob/main/docs/4-development/developer_guide.md).

MIT licensed.
