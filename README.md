# pdf-engine-presentation

> **TLDR:** Standalone, dependency-free, deterministic Markdown presentation-deck parser — headings, paragraphs, fenced code, speaker notes, slide breaks, TOML/YAML front matter. No runtime dependency.

## What

A Markdown-to-`Deck` parser and validator, extracted from the
[`pdf-engine`](https://github.com/sweengineeringlabs/pdf-engine) document
engine. The `DeckParser` trait turns a Markdown source string into a `Deck` of
`Slide`s made of `SlideElement`s (headings, paragraphs, fenced code, speaker
notes). A separate `validate()` call checks the result against a deterministic
fixed-canvas line estimate; `FactoryDeckParser` is the default implementation.
The crate has no external dependencies.

## Why

`pdf-engine` needed a small, deterministic Markdown subset parser for its
presentation-deck feature and it turned out to be generally useful independent
of that: no async runtime, no regex engine, no external crate at all — just
line-oriented parsing over `&str`. Identical input produces an equal `Deck`,
every time.

## When / How

Use it when you want a slide deck's structure (title, aspect ratio, per-slide
elements) out of a small Markdown subset without pulling in a general-purpose
Markdown/CommonMark implementation.

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

- `Deck`/`Slide` model slide *structure*, not slide *rendering* — this crate
  has no HTML/PDF export of its own (that lives in `pdf-engine`'s adapter
  layer, which depends on Chromium and is not part of this crate).
- Fenced code blocks are captured as plain text; the language tag after
  ` ``` ` (e.g. `rust`, `mermaid`) is not currently preserved.
- Only `#`/`##` headings, paragraphs, fenced code, and `:::notes` blocks are
  recognized — this is a deliberate subset, not a CommonMark implementation.

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
