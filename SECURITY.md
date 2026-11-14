# Security Policy

## Supported Versions

| Version | Supported |
|---------|-----------|
| 1.9.x   | Yes       |

## Reporting a Vulnerability

Report security vulnerabilities to **engineers@swelabs.io**.

Please include:

- A description of the vulnerability
- Steps to reproduce
- Potential impact

We aim to acknowledge reports within 72 hours and provide a resolution timeline within 7 days.

Do **not** open a public GitHub issue for security vulnerabilities.

## Threat model

`pdf-engine-presentation` is a deterministic, dependency-free Markdown parser. It:

- Performs no file, network, or process I/O of its own — the parsing API takes
  an owned source `String` and transforms its text into a `Deck`.
- Denies `unsafe_code` at the crate level (`#![deny(unsafe_code)]`).
- Normalizes line endings, scans optional front matter, and parses content
  line by line without recursion. These scans and allocations scale linearly
  with input size; parsing is not a single pass over the original source.

The library has no third-party Rust crate dependencies. Callers embedding
untrusted Markdown should apply their own input-size limits at the boundary,
since this crate does not impose one.
