# RFC-0001: Support section-oriented (non-slide) consumers

## Status

Draft. Requested by the RustSpeak project
(`sweengineeringlabs/rustspeak`, issue #6). Nothing here is implemented.

## Summary

A handful of small requests so `md-parser` can serve a consumer that wants
*sections of a document* (heading, prose, Mermaid diagram, optional notes)
rather than *slides*:

1. A non-breaking helper that groups a slide's elements into sections at
   headings.
2. ~~Lower the declared Rust version.~~ Withdrawn: see R2.
3. A decision on how consumers outside this organisation obtain the crate
   (the crates.io name is taken).

Plus one documentation request (R4) and an update of the Rust version to
1.100 (R5). One more idea, plain-text extraction, is raised only to be
declined (see Non-goals).

## Context

RustSpeak is generating narrated explainer videos from documents. Its
issue #6 asks for a pipeline: Markdown in, finished video out, with each
heading becoming a timed title card, its prose becoming narration, and its
`mermaid` fence becoming the diagram shown while that section is spoken.
Today those three things are copied between a `.md` file and a Rust
literal by hand.

`md-parser` already extracts most of what is needed: headings, paragraphs,
`mermaid` fences as `Diagram`, and `:::notes` as `Notes`, in source order.

### Evidence

A throwaway program parsed RustSpeak's real `docs/3-design/architecture.md`
with `FactoryDeckParser` (path dependency on this repository's `scm/`).

| Input | Result |
|---|---|
| `architecture.md` as written (no `---`, plain code fences) | 1 slide, 44 elements: 15 `Heading`, 26 `Paragraph`, 3 `Code`, 0 `Diagram`. `validate` fails: "slide 1 exceeds the fixed canvas (estimated 183 lines)". |
| Same shape authored with `---`, `:::notes` and a `mermaid` fence | 2 slides; `Heading`, `Paragraph`, `Diagram`, `Notes` all extracted correctly; `validate` passes. |

Toolchain check of this crate (`scm/`): on rustc 1.99.1 and on 1.100.0,
`cargo clippy --all-targets -- -D warnings` is clean and every test target
passes (1.99.1: 78 passed, 0 failed; 1.100.0: all test targets pass, 0
failed). `cargo fmt --check` is clean on 1.100.0.

## Requests

### R1. Section grouping helper

Slides are split only on `---` (see the rules definition), so an ordinary
document parses to one slide holding every heading. A consumer can walk the
flat element list and split at `Heading` itself, and the evidence shows the
order is preserved, so this is **convenience, not a blocker**.

Proposal: an additive, pure function, for example

```rust
impl Slide {
    /// Groups elements into sections, starting a new section at each Heading.
    /// Elements before the first Heading form a leading section with no heading.
    pub fn sections(&self) -> Vec<Section<'_>>;
}
pub struct Section<'a> {
    pub heading: Option<(u8, &'a str)>,
    pub elements: &'a [SlideElement],
}
```

Constraints: no change to `parse` output or `SlideElement`; no new
dependencies; keeps the crate's "returns data in memory, does not render"
contract. Open question: whether sections should nest by heading level or
stay flat (flat is enough for the requester and simpler).

### R2. Withdrawn

An earlier draft asked to lower the declared `rust-version` so a consumer
building at Rust 1.85 could use the crate. That is withdrawn: the RustSpeak
maintainers' direction is the opposite, that `md-parser` moves up to current
Rust (R5), and that the consumer moves up with it. The number is kept so the
other requests and their issues stay stable.

### R3. Obtainable by consumers

The README says to depend on the Git repository because the crates.io name
`md-parser` is registered to another project, and `Cargo.toml` has
`publish = false`. A Git dependency cannot be published to crates.io by the
consumer, and RustSpeak's `cargo-deny` configuration has
`unknown-git = "deny"`, so each consumer needs an explicit allowlist entry.

This is a question, not a demand: is a registry-published name planned?
The history mentions a `swe-md-parser` registry release being prepared and
then the identity being reset to `md-parser`. If a registry name is not
planned, consumers will keep using a Git dependency pinned to a tag, and
this RFC only asks that the tag policy be stated (the README pins
`v0.1.0`; the repository also carries `v1.9.x` tags from before the reset).

### R4. Documentation

State in the rules definition that `parse` is useful without calling
`validate`, and that an ordinary multi-heading document parses to a single
slide that `validate` will reject under `OverflowPolicy::Reject`. Both are
true today (see Evidence) but a reader has to infer them.

### R5. Update Rust to 1.100 (at least 1.99.1)

Requested by the RustSpeak maintainers: `md-parser` should be updated to
Rust 1.100 (current stable, released 2026-11-09), or at the very least to
1.99.1, the version installed on their machines. It is currently at 1.98.1.

Evidence (`scm/`): on both 1.99.1 and 1.100.0 clippy (`-D warnings`) is
clean and every test passes (see Evidence above), so no source change is
needed to move up. Places that state the old version:

- `scm/Cargo.toml`: `rust-version = "1.98.1"`.
- `docs/4-development/developer_guide.md`: "Install Rust 1.98.1 or later".
- `.github/workflows/ci.yml` uses `dtolnay/rust-toolchain@stable`, which
  floats implicitly and so does not pin or document the version.

Proposal: raise `rust-version` to 1.100 (or 1.99.1 if a smaller step is
preferred), update the developer guide to match, and make CI name the
version explicitly. Per the CHANGELOG, raising the declared version has
been marked **Breaking** before (1.95 -> 1.98.1), so this should be recorded
the same way.

Consequence for consumers: a consumer whose own minimum is below the new
`rust-version` cannot build against it. RustSpeak's CI currently has an
MSRV job at 1.85; moving `md-parser` up means RustSpeak must raise its own
floor, which is RustSpeak's change to make and is tracked on its side.

## Non-goals

- **Plain-text extraction of paragraphs.** `Paragraph` is raw Markdown (lists,
  tables, `**emphasis**`, links, inline code); the rules definition says
  CommonMark is deliberately not interpreted. Narration needs that text
  cleaned, but doing it here would turn a deliberately small crate into a
  CommonMark implementation. The requester will strip text itself (likely
  with a real Markdown parser over the paragraph source). Listed so the
  maintainers can confirm this stays out of scope.
- Rendering, TTS, video, or any change to validation behavior.
- Changing slide separators, metadata or the `:::notes` / `:::shape`
  conventions.

## Alternatives considered

- **Do nothing and use `pulldown-cmark` directly.** Removes the Git
  dependency problem (R3), and handles inline text. It loses the
  `:::notes` and `mermaid` conventions shared with the presentation tooling.
  RustSpeak has not yet chosen between the two; R3 largely decides it.
- **Consumer-side sectioning (no R1).** Works today with a loop over
  `slide.elements`; R1 only removes that boilerplate.

## Compatibility

R1 is additive. R5 raises the declared `rust-version`, which the CHANGELOG
has treated as a breaking change before, so it should be recorded as one.
R3 and R4 change no code.

## Unverified

- Whether any element of the public API is already intended to cover R1.
- RustSpeak's end-to-end pipeline: this RFC rests on a parse probe and
  reading RustSpeak's `Composition` / `ExplainerVideo` code, not on a
  working prototype.
