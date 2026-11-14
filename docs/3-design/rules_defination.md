# Rules definition

This document defines source conventions and validation behavior for the
default `FactoryDeckParser` and `FactoryValidator` implementations. Markdown
has no slide model: `---` slide separators and `:::notes` blocks are conventions
defined by this library. This is a restricted text format, not a full
CommonMark, YAML, or TOML implementation.

The [architecture](architecture.md) describes how the code implements these
rules. Update this document and the corresponding tests together when changing
parsing or validation behavior.

## Input and output

- Parsing accepts a source `String` and a default `AspectRatio`.
- CRLF and bare CR line endings are normalized to LF before parsing.
- The result is a `ParseResponse` containing an `Arc<Deck>`: title, aspect
  ratio, ordered slides, and overflow policy. Each slide holds ordered elements.
- The default title is empty. Headings do not set it. Metadata may override
  the title and default aspect ratio.
- Parsed decks use `OverflowPolicy::Reject`. Only callers, not source metadata,
  can change that policy in the data model.
- Parsing does not invoke validation. Neither operation renders, clips,
  or writes files.

## Deck metadata

Only the case-sensitive keys `title` and `aspect_ratio` are supported.

| Key | Value |
|-----|-------|
| `title` | Text stored as the deck title |
| `aspect_ratio` | `16:9` or `16x9` for widescreen; `4:3` or `4x3` for standard |

Keys and values are trimmed. Leading and trailing single/double quote
characters are stripped from values. There is no general escaping, multiline
string, or comment syntax. Repeated keys overwrite earlier values. Unsupported
aspect-ratio values return `MalformedSource`.

### YAML-style front matter

```text
---
title: Review
aspect_ratio: 16:9
---
# First slide
```

- The opening must be `---` followed by a newline at the start of the
  normalized source.
- A block is metadata only if it has at least one nonempty line and every
  line contains a supported key followed by `:`.
- Blank lines, unsupported keys, or a missing closing delimiter cause fallback
  to ordinary source parsing, rather than a front-matter error.

### TOML-style front matter

```text
+++
title = "Review"
aspect_ratio = "16:9"
+++
# First slide
```

- The opening must be `+++` followed by a newline at the start of the
  normalized source.
- Each nonblank metadata line must contain a supported key followed by `=`.
  Blank lines are skipped.
- A missing closing delimiter, missing `=`, or unsupported key returns
  `MalformedSource`.

For both forms, the current closing-delimiter search matches a newline followed
by the marker prefix; it does not require a delimiter-only line. Write
delimiter-only lines for unambiguous input.

### Bare metadata

```text
title: Review
aspect_ratio: 4:3
# First slide
```

Outside code and notes, trimmed lines starting with `title:` or `aspect_ratio:`
are metadata while no slide has been completed and the current slide has no
elements. Blank lines and leading slide separators do not end that opportunity.
After an element is added, metadata-like text is content. Bare metadata may
override front-matter values.

## Slides and elements

Outside code and notes, recognition uses trimmed lines in this order:

1. Eligible bare metadata.
2. A line equal to `---`: end the current slide if it has elements.
3. A line equal to `:::notes`, case-insensitively: enter notes mode.
4. A line starting with three backticks: enter code mode.
5. A line starting with `# ` or `## `: create a level 1 or 2 heading.
6. Any other nonempty line: create one paragraph element.

Blank lines outside blocks are ignored. Adjacent paragraph lines remain separate
elements. Heading text is the trimmed line after its prefix; inline formatting
and trailing heading markers are not interpreted. Leading, trailing, or repeated
slide separators do not create empty slides. Remaining elements form the last
slide at end of input.

In ordinary content, `---` separates slides even where Markdown would interpret
it as a horizontal rule or another construct. Lists, links, emphasis, tables,
images, and `###` or deeper headings have no special model; their source text
becomes paragraph text.

### Fenced code

- A trimmed line starting with three backticks toggles code mode. Fence
  lengths and language labels are not matched or validated; tilde fences
  are not recognized.
- Each closed block produces one `Code(String)`, including an empty block.
  Fence lines and language labels are discarded.
- Content lines retain whitespace and are joined with LF. Blank lines are
  retained. Separate blocks remain separate elements.
- Slide separators, headings, metadata, and notes directives inside code are
  content. A backtick fence still closes the block.
- End of input while in code mode returns `MalformedSource`.

### Speaker notes

- A trimmed `:::notes` line opens notes mode, case-insensitively.
- Each content line creates a separate `Notes(String)` retaining its
  whitespace, including blank lines.
- A trimmed `:::` closes notes. Repeated opening directives are ignored;
  notes blocks do not nest.
- Slide separators, headings, metadata, and code fences inside notes are
  content. `---` does not close notes or start a slide.
- Outside notes, `:::` is ordinary paragraph text.
- End of input while in notes mode returns `MalformedSource`.

## Validation

Call `DeckParser::validate` or `Validator::validate` explicitly. Both default
implementations apply the same rules and leave the deck unchanged.

- A deck with no slides returns an error under either overflow policy.
- `OverflowPolicy::Clip` accepts a nonempty deck without estimating overflow.
  It does not clip content or measure a visual boundary.
- `OverflowPolicy::Reject` sums these estimates for each slide:

| Element | Estimated lines |
|---------|-----------------|
| Heading | 2 |
| Paragraph | Unicode scalar-value count divided by 90, rounded up; minimum 1 |
| Code | Rust `str::lines().count()`; minimum 1 |
| Notes | 0 |

At most 34 estimated lines are allowed per slide, under either aspect ratio.
The first slide exceeding the budget returns `SlideOverflow` with a one-based
slide number and estimated line count. A manually constructed slide with no
elements is valid; a deck with no slides is not.

This estimate does not account for fonts, word wrapping, rendered Markdown,
or physical dimensions. It does not establish that content visually fits.

## Errors and repeatability

Parsing returns `EmptyDeck` when no elements were produced. Notes-only slides
and empty code blocks do contain elements and can form a deck. Unclosed code
or notes and invalid recognized metadata return `MalformedSource`.

`DeckParser::validate` returns structured `PresentationError` variants. The
default `Validator` converts the first failure into one message in
`ValidationError::violations`; it does not collect all failures.

The same source and default aspect ratio produce an equal deck. Validating
the same deck produces the same result. Neither operation uses I/O or a clock.
