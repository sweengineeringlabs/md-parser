# Developer guide

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

The crate is not currently available on crates.io, and hosted docs.rs
documentation is not available yet. Installation examples use the Git
repository. The package's `version` is not evidence of a registry release.

```sh
cargo publish --dry-run   # verify packaging before every real publish
cargo publish
```

`Cargo.toml` sets `publish = true` explicitly and declares the package
version. For a subsequent release, bump the version and update `CHANGELOG.md`
before publishing. After a successful first publication, update installation
examples and documentation links to the actual published version.

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
