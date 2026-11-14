# Changelog

## [Unreleased]

## [1.9.13] — 2026-11-14

### Changed

- Renamed the Cargo package to `swe-md-parser` to resolve the crates.io name
  conflict while preserving the `md_parser` Rust library API.
- Published release 1.9.13 to crates.io as `swe-md-parser`.

## [1.9.12] — 2026-11-14

### Added

- Parses labeled rectangle, rounded, circle, and diamond shape directives.
- Preserves Mermaid fenced blocks as diagram slide elements for the renderer.

## [1.9.11] — 2026-11-14

### Changed

- Preserves contiguous Markdown lines as paragraph blocks for rich downstream
  rendering, recognizes all six ATX heading levels, and counts explicit
  paragraph lines in overflow estimates.

## [1.9.10] — 2026-11-14

### Changed

- **Breaking:** renamed the repository and Cargo package from
  `pdf-engine-presentation` to `md-parser`, and the Rust library import from
  `pdf_engine_presentation` to `md_parser`. Update Git dependencies to
  `md-parser = { git = "https://github.com/sweengineeringlabs/md-parser" }`
  and replace `use pdf_engine_presentation::...` with `use md_parser::...`.
  Parsing and validation behavior is unchanged. See the
  [rename migration guide](docs/4-development/developer_guide.md#rename-migration)
  for dependency/import changes, checkout remotes, and existing tag behavior.

## [1.9.9] — 2026-11-14

### Fixed

- Fenced code now produces exactly one `SlideElement::Code` per block,
  preserves blank lines, and keeps adjacent blocks separate. Metadata-like
  text inside code and notes is no longer consumed as deck metadata.
- Removed the obsolete `licenses.deny` setting from `scm/deny.toml`;
  the explicit license allowlist continues to reject unlisted licenses.
- Corrected Git installation instructions and publication status, separate
  parsing/validation examples, notes and overflow-policy semantics, API
  layout descriptions, and development/security documentation. Removed the
  unavailable hosted API reference in favor of local rustdoc instructions.

### Changed

- Centralized source and validation rules in
  `docs/3-design/rules_defination.md`, referenced from the architecture and
  developer guides. Reduced both READMEs to an introduction and quick start.
- Rewrote the project description and design rationale around the implemented
  input, output, parsing, and validation behavior. Removed unverified origin
  claims, external renderer references, and internal tool-audit notes.
- **Breaking:** `parse_markdown` and `validate_deck` free functions are
  replaced by the `DeckParser` trait (`parse`/`validate` methods, taking
  `ParseRequest`/`ValidateRequest` and returning `ParseResponse`/`()`),
  implemented directly by `FactoryDeckParser`. The crate is not currently
  available on crates.io; Git/path consumers must migrate to the new API.
- **Breaking:** the default entry-point types are named `FactoryDeckParser`
  and `FactoryValidator`, replacing `DeckParserFactory` / `ValidatorFactory`.
  They implement `DeckParser` / `Validator` directly.
- **Breaking:** MSRV raised from `1.95` to `1.98.1` (`rust-version` in
  `Cargo.toml`).

### Added

- `Validator` trait, implemented directly by `FactoryValidator` — a generic
  itemized validation contract (`ValidationError { violations: Vec<String> }`)
  alongside `DeckParser::validate`'s structured `PresentationError`.
- `ParseRequest`, `ParseResponse`, `ValidateRequest`, `GetValidatorRequest`,
  `GetValidatorResponse` DTOs; `FactoryDeckParser`, `FactoryValidator`
  marker/entry-point types.
- `DeckParser::validator()` default method — recovers a `Validator` from any
  `DeckParser` implementor.

## [1.9.7] — 2026-08-30

### Added

- Initial source version of `pdf-engine-presentation`. Deterministic
  Markdown deck parsing (`parse_markdown`) and content-limit validation
  (`validate_deck`) with `Deck`, `Slide`, `SlideElement`, `AspectRatio`,
  `OverflowPolicy`, and `PresentationError` contracts. Zero runtime
  dependencies.

This entry records the standalone source version, not a confirmed crates.io
publication.
