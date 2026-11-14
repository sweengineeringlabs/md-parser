# Developer guide

Repository and Cargo package: `md-parser`. Rust library import: `md_parser`.

## Rename migration

The project was renamed from `pdf-engine-presentation` to `md-parser`.

| Item | Previous name | Current name |
|------|---------------|--------------|
| GitHub repository | `sweengineeringlabs/pdf-engine-presentation` | `sweengineeringlabs/md-parser` |
| Cargo dependency | `pdf-engine-presentation` | `md-parser` |
| Rust import | `pdf_engine_presentation` | `md_parser` |

For an existing checkout, update its remote from the repository root:

```sh
git remote set-url origin https://github.com/sweengineeringlabs/md-parser.git
```

Update consuming applications' `Cargo.toml` entries:

```toml
[dependencies]
md-parser = { git = "https://github.com/sweengineeringlabs/md-parser", tag = "v1.9.10" }
```

Replace `use pdf_engine_presentation::...` with `use md_parser::...` and
update any fully qualified references. Parsing and validation behavior is
unchanged. An existing local directory does not need to be renamed.

The rename is implemented in commit `a3ed7d7`. Existing tags retain their
original commits: `v1.9.9` predates the rename and still uses the former
package/import names. `v1.9.10` is the first tag using `md-parser` / `md_parser`.
Pin that tag as shown above, or use the current `main`/`dev` branch when
installing the renamed package from Git.
No crates.io publication was performed as part of the rename.

## Setup

Install Rust 1.98.1 or later, including the `rustfmt` and `clippy` components.
Run bootstrap from the repository root. Git hooks require Bash (provided by
Git for Windows on Windows).

```sh
./scm/bootstrap.sh     # or: pwsh scm/bootstrap.ps1 on Windows
```

Installs git hooks (`core.hooksPath` → `scm/scripts/hooks`) and fetches
dependencies. The `pre-commit` hook runs `cargo fmt --check`, `cargo clippy
-D warnings`, and `cargo test`; the `commit-msg` hook rejects commits that
carry AI-assistant attribution (see [Commit policy](#commit-policy) below).

## Build, test, lint

All commands run from `scm/`, the package root:

```sh
cargo build --all-targets
cargo test --all-targets
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo deny check --config deny.toml
cargo doc --no-deps
```

Install the optional dependency-policy checker with `cargo install cargo-deny`
before running `cargo deny`. The configuration uses an explicit license
allowlist; licenses outside it are rejected. CI currently runs formatting,
Clippy, tests, and a publish dry run; the dependency-policy and documentation
commands above are additional local checks.

## Publishing

This implementation is distributed from Git. The crates.io name `md-parser`
is registered to a different project (`parsing-tech/md-rs`, owner `xieyuheng`);
its existing release is not this library. Registry publication is blocked on
resolving package naming or obtaining appropriate ownership/access, tracked in
[pdf-engine issue #15](https://github.com/sweengineeringlabs/pdf-engine/issues/15).
Do not substitute a registry-only dependency for the documented Git dependency.
The package version and successful dry runs are not evidence of publication.

```sh
cargo publish --dry-run   # verify packaging before every real publish
cargo publish
```

`Cargo.toml` sets `publish = true` explicitly and declares the package
version. For a subsequent release, bump the version and update `CHANGELOG.md`
before publishing. Only publish after the registry-name decision and relevant
credentials are resolved. After a successful first publication, update
installation examples and documentation links to the actual published package.

## Public API and versioning

From `v1.9.10`, the supported public contract consists of the crate-root exports
and the documented source/validation behavior in the
[rules definition](../3-design/rules_defination.md). The public contract includes
the parser/validator traits, default entry points, request/response types,
deck/slide/element types, and errors. Internal `api/`, `core/`, and `saf/`
organization is not a consumer-facing API.

For the 1.x Git releases, use the following versioning policy:

| Change | Required version change |
|--------|-------------------------|
| Incompatible public type, trait, signature, or documented parsing/validation behavior | Major |
| Backward-compatible public functionality | Minor |
| Bug fix restoring documented behavior, or documentation-only correction | Patch |
| Increased minimum supported Rust version | At least minor, with release notes |

Renames and ownership changes must include explicit consumer migration notes.
The earlier rename and API history remain recorded in `CHANGELOG.md`; this
policy does not retroactively assert compatibility across older tags. Document
behavioral changes and update the rules and tests in the same change.

## Commit policy

No commit, PR description, or code comment may credit an AI assistant
(`Co-Authored-By: <ai>`, "Generated with ...", a 🤖 footer, etc.). This is
checked for matching attribution patterns in commit messages and PR bodies:

- Locally, by the `commit-msg` git hook installed via bootstrap — it rejects
  the commit outright, before it's even made.
- On GitHub, by the `No AI Attribution` workflow
  (`.github/workflows/no-ai-attribution.yml`) — it scans both the PR
  description and every commit message in the PR, and cannot be bypassed with
  `git commit --no-verify`.

These checks do not scan source comments or recognize every possible wording;
the policy is broader than the automated pattern checks.

## Adding a new public API item

1. New data types go in their own file under `main/src/api/types/`,
   `main/src/api/dto/` (request/response types), or `main/src/api/error/` —
   one public type per file, named after the type in snake_case (sibling
   files sharing a prefix, e.g. `factory_*`, get grouped into a same-named
   subdirectory) — and are re-exported by name (no globs) from `api/mod.rs`.
   The existing `saf/contract_svc.rs` and `saf/mod.rs` wildcard re-exports
   expose these items through the crate root. A new trait goes in
   `main/src/api/traits/`. New
   parsing/validation logic goes in `main/src/core/` (as methods on a
   `Default*`-prefixed struct, in its own subdirectory alongside a matching
   `api/<domain>/` module). A factory/marker type that implements its own
   trait directly (so it's usable without an existing implementor to call
   `factory()` on) must carry the trait name as a suffix, and that impl block
   lives in `core/` alongside the `Default*` implementation it delegates to —
   `saf/` remains a re-export facade: `contract_svc.rs` re-exports the API,
   and `*_svc_factory.rs` files re-export the relevant port traits.
2. Document every public item — `#![warn(missing_docs)]` is set crate-wide.
3. `api/` and `saf/` source files must not contain `#[cfg(test)]` blocks —
   add a test file under `tests/` instead (e.g. `tests/<type>_int_test.rs`
   for a new api/ type, or `tests/<trait>_int_test.rs` with happy/error/edge
   coverage via a hand-written test double for a new trait method — name it
   `Test<Trait>`, not `Fake*`/`Mock*`, following the project's test naming
   convention). `core/` source files are
   the opposite: private functions there need an inline `#[cfg(test)] mod
   tests` block in the same file, and public ones still need an external
   `tests/` file. Every test needs an `/// @covers: <item_name>` doc comment.
4. Update the item's API documentation. For changes to source syntax or
   validation behavior, update the [rules definition](../3-design/rules_defination.md)
   and its corresponding tests together. Update
   [architecture.md](../3-design/architecture.md) when module structure or
   design rationale changes. Keep READMEs focused on the introduction and
   quick start.
5. Add a `CHANGELOG.md` entry under `[Unreleased]`.
