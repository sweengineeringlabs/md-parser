# Changelog

## [Unreleased]

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

- **Breaking:** `parse_markdown` and `validate_deck` free functions are
  replaced by the `DeckParser` trait (`parse`/`validate` methods, taking
  `ParseRequest`/`ValidateRequest` and returning `ParseResponse`/`()`),
  implemented directly by `FactoryDeckParser`. The crate is not currently
  available on crates.io; Git/path consumers must migrate to the new API.
- **Breaking:** the factory marker types are named `FactoryDeckParser` and
  `FactoryValidator` (trait-name suffix), not `DeckParserFactory` /
  `ValidatorFactory` — required by this org's `core_impl_name_has_trait_suffix`
  architecture rule, since these types implement `DeckParser`/`Validator`
  directly in `core/`.
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

### Historical internal architecture-audit notes

The following records an earlier run of the organization's internal `arch`
SEA-compliance tool. The tool version and report are not checked into this
repository, and CI does not run that audit. These notes have not been
revalidated and do not establish the current tool's behavior or open GitHub
issue status. That run reported two unresolved findings, described as
contradictions between two of the tool's own rules, or a documented gap
between a rule's stated behavior and its runtime behavior. A third,
previously listed here (`app_type_forbids_saf` vs `app_type_requires_saf`
reporting mutually exclusive application types for this crate in the same
audit run), was reported resolved by moving trait implementations from
`saf/` to `core/` in `ba7d9ea`.

- `api_impl_public_tests_external` fired despite its own documented edge case
  ("If api/ has no standalone pub fn ... this rule does not fire") — `api/`
  had zero standalone `pub fn` (trait declarations only), checked via
  `grep -rn "pub fn" main/src/api`.
- `saf_impl_public_tests_external` / `trait_svc_fn_scenario_coverage` vs
  `encapsulation.package_access_violation`: the latter banned re-exporting
  *concrete types* from `saf/` (only trait re-exports are permitted); the
  former two only recognized a re-exported *struct* as a valid "pub item" in a
  `*_svc_factory.rs` file, not a re-exported trait. No arrangement of
  `saf/deck_parser_svc_factory.rs` / `saf/validator_svc_factory.rs` was found to
  satisfy both rule families simultaneously — tested in both
  directions, including a `pub fn -> impl DeckParser` factory function
  (opaque return type, no concrete-type re-export): it did not satisfy
  `trait_svc_fn_scenario_coverage`, and its presence caused the audit run to
  silently drop 61 unrelated structural rules from the report rather than
  fail or skip them, so that workaround was rejected.

Separately, `test_coverage_threshold` and `test_mutation_score` both reported
`INTERNAL ERROR: builtin handler '<rule>' not registered` on that `arch`
build — described as a gap in the `arch` binary itself (missing `cargo-llvm-cov` /
`cargo-mutants` handler registration), not a finding about this crate.

## [1.9.7] — 2026-08-30

### Added

- Initial standalone extraction of `pdf-engine-presentation` from
  the `pdf-engine` monorepo (`scm/main/port/presentation`). Deterministic
  Markdown deck parsing (`parse_markdown`) and fixed-canvas validation
  (`validate_deck`) with `Deck`, `Slide`, `SlideElement`, `AspectRatio`,
  `OverflowPolicy`, and `PresentationError` contracts. Zero runtime
  dependencies.

This entry records the standalone source version, not a confirmed crates.io
publication.
