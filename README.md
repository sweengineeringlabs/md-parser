# md-parser

A Rust library that parses Markdown into slide data: deck metadata, slides,
headings, paragraphs, code, and speaker notes. A separate validation call checks
estimated content limits. It has zero external dependencies and returns data
in memory; it does not render slides or generate presentation files.

## Quick start

Add the Git dependency to your application's `Cargo.toml`. The crates.io
package named `md-parser` is a different project.

```toml
[dependencies]
md-parser = { git = "https://github.com/sweengineeringlabs/md-parser" }
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
