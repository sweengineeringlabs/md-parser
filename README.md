# pdf-engine-presentation

> Parses Markdown into structured slide data and validates estimated slide content limits. Rust library with zero external dependencies.

## What

`pdf-engine-presentation` is a Rust library. The `DeckParser` trait turns a
Markdown source string into a `Deck` of
`Slide`s made of `SlideElement`s (headings, paragraphs, fenced code, speaker
notes). A separate `validate()` call checks the result against a deterministic
fixed-canvas line estimate; `FactoryDeckParser` is the default implementation.
The output is Rust data held in memory. The library does not create a
presentation file, render slides, or export HTML/PDF.

## Why

Slide content has a simple structure: metadata, ordered slides, and ordered
elements within each slide. This library makes that structure available to
Rust applications so they can inspect or process it and check a content budget.
It parses a defined Markdown subset using the Rust standard library. Identical
source and aspect-ratio inputs produce an equal `Deck` every time.

## When / How

Use it when your application needs a deck's title, aspect ratio, slides, and
content elements from Markdown, with an optional content-limit check.

The crate is not currently available on crates.io. Use the Git repository;
Cargo discovers the package in its `scm/` subdirectory:

```toml
[dependencies]
pdf-engine-presentation = { git = "https://github.com/sweengineeringlabs/pdf-engine-presentation" }
```

```rust
use pdf_engine_presentation::{
    AspectRatio, DeckParser, FactoryDeckParser, ParseRequest, ValidateRequest,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = "# Quarterly review\n\nRevenue increased.\n---\n## Appendix";
    let response = FactoryDeckParser.parse(ParseRequest {
        source: source.to_string(),
        default_aspect_ratio: AspectRatio::Widescreen16x9,
    })?;
    println!("{} slide(s)", response.deck.slides.len());
    FactoryDeckParser.validate(ValidateRequest { deck: response.deck })?;
    Ok(())
}
```

Slides are separated by a `---` line. Speaker notes are wrapped in
`:::notes` / `:::` and ignored by the line estimate; callers control rendering.
Slide markers inside code or notes are treated as content. Optional front
matter sets `title` and `aspect_ratio` (`16:9` or `4:3`), either YAML-style
(`---` delimited, `key: value`) or TOML-style (`+++` delimited, `key = value`).
These are restricted metadata syntaxes, not full YAML/TOML parsers; see the
[source syntax](scm/README.md#source-syntax) for details.

Validation rejects estimates above 34 lines per slide: headings count as two,
paragraphs as one per 90 characters (rounded up, minimum one), code as its
line count (minimum one), and notes as zero. Both aspect ratios use this same
heuristic. `OverflowPolicy::Clip` skips overflow rejection; it does not clip
or render content. Empty decks are rejected under either policy.

## Known limitations

- The line budget is a heuristic. It does not measure fonts or visual layout.
- Fenced code blocks are captured as plain text; the language tag after
  ` ``` ` (e.g. `rust`, `mermaid`) is not currently preserved.
- Only `#`/`##` headings, paragraphs, fenced code, and `:::notes` blocks are
  recognized — this is a deliberate subset, not a CommonMark implementation.

## Run the example

From `scm/`, run:

```sh
cargo run --example basic
```

The example parses two slides and prints `parsed 2 slides` to the console.
It does not write a file or invoke validation. The usage snippet above shows
how to parse and then validate explicitly.

## Further reading

Full crate layout and the source-level API surface live in
[`scm/README.md`](scm/README.md) — the source-code documentation, which in
turn maps to:

| Document | Description |
|----------|-------------|
| [README](scm/README.md) | Crate layout and full usage examples |
| [Architecture](docs/3-design/architecture.md) | WHAT + WHY — module layout and design rationale |
| [Developer guide](docs/4-development/developer_guide.md) | Build, test, lint, and publish workflow |
| [Changelog](CHANGELOG.md) | Version history |

Generate the API reference locally with `cargo doc --no-deps --open` from
`scm/`. Hosted documentation on docs.rs is not currently available.

## License

MIT

---

A [Software Engineering Labs](https://swelabs.io) (SWE Labs) project.
