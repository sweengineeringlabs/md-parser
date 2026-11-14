# md-parser

A Rust library that parses Markdown into slide data: deck metadata, slides,
headings, paragraphs, code, and speaker notes. A separate validation call checks
estimated content limits. It has zero external dependencies and returns data
in memory; it does not render slides or generate presentation files.

## Quick start

The crate is published on crates.io as `swe-md-parser`. The dependency key
`md-parser` below is a local alias; the crates.io package named `md-parser` is
a different project.

```toml
[dependencies]
md-parser = { package = "swe-md-parser", version = "1.9.13" }
```

To build from the repository instead, use the same package alias with the
`sweengineeringlabs/md-parser` Git repository and the `v1.9.13` tag.

```toml
[dependencies]
md-parser = { package = "swe-md-parser", git = "https://github.com/sweengineeringlabs/md-parser", tag = "v1.9.13" }
```

```rust
use md_parser::{
    AspectRatio, DeckParser, FactoryDeckParser, ParseRequest, ValidateRequest,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let response = FactoryDeckParser.parse(ParseRequest {
        source: "# Review\nRevenue increased.\n---\n# Appendix".to_string(),
        default_aspect_ratio: AspectRatio::Widescreen16x9,
    })?;
    println!("{} slide(s)", response.deck.slides.len());
    FactoryDeckParser.validate(ValidateRequest { deck: response.deck })?;
    Ok(())
}
```

To run the repository's basic example, run `cargo run --example basic` from
`scm/`. It prints `parsed 2 slides` to the console and creates no file.

See the [rules definition](docs/3-design/rules_defination.md),
[architecture](docs/3-design/architecture.md), and
[developer guide](docs/4-development/developer_guide.md) for details.

MIT licensed. A [Software Engineering Labs](https://swelabs.io) project.
