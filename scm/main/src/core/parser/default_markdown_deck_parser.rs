use crate::api::{
    AspectRatio, Deck, DeckParser, FactoryDeckParser, OverflowPolicy, ParseRequest, ParseResponse,
    PresentationError, ShapeKind, Slide, SlideElement, ValidateRequest,
};
use std::sync::Arc;

/// Deterministic Markdown implementation of [`DeckParser`].
pub(crate) struct DefaultMarkdownDeckParser;

impl DeckParser for DefaultMarkdownDeckParser {
    fn parse(&self, request: ParseRequest) -> Result<ParseResponse, PresentationError> {
        let deck = Self::parse_markdown(&request.source, request.default_aspect_ratio)?;
        Ok(ParseResponse {
            deck: Arc::new(deck),
        })
    }

    fn validate(&self, request: ValidateRequest) -> Result<(), PresentationError> {
        Self::validate_deck(&request.deck)
    }
}

impl DeckParser for FactoryDeckParser {
    fn parse(&self, request: ParseRequest) -> Result<ParseResponse, PresentationError> {
        DefaultMarkdownDeckParser.parse(request)
    }

    fn validate(&self, request: ValidateRequest) -> Result<(), PresentationError> {
        DefaultMarkdownDeckParser.validate(request)
    }
}

impl DefaultMarkdownDeckParser {
    /// Parses a deterministic Markdown deck. Slides are separated by `---`;
    /// speaker notes use a `:::notes` block terminated by `:::`.
    fn parse_markdown(source: &str, aspect_ratio: AspectRatio) -> Result<Deck, PresentationError> {
        let normalized = source.replace("\r\n", "\n").replace('\r', "\n");
        let mut slides = Vec::new();
        let mut title = String::new();
        let mut resolved_aspect_ratio = aspect_ratio;
        let mut content = normalized.as_str();
        if let Some(frontmatter) = normalized.strip_prefix("+++\n") {
            if let Some(end) = frontmatter.find("\n+++") {
                let metadata = &frontmatter[..end];
                for line in metadata.lines().filter(|line| !line.trim().is_empty()) {
                    Self::apply_toml_metadata_line(line, &mut title, &mut resolved_aspect_ratio)?;
                }
                content = &frontmatter[end + 4..];
            } else {
                return Err(PresentationError::MalformedSource(
                    "unterminated TOML presentation front matter".to_string(),
                ));
            }
        }
        if let Some(new_content) = Self::strip_yaml_frontmatter(
            normalized.as_str(),
            &mut title,
            &mut resolved_aspect_ratio,
        )? {
            content = new_content;
        }
        let mut current = Vec::new();
        let mut in_code = false;
        let mut code_lines = Vec::new();
        let mut code_language = String::new();
        let mut in_notes = false;
        let mut shape_kind = None;
        let mut shape_lines = Vec::new();
        let mut paragraph_lines = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(kind) = shape_kind {
                if trimmed == ":::" {
                    let label = shape_lines.join("\n").trim().to_string();
                    if label.is_empty() {
                        return Err(PresentationError::MalformedSource(
                            "shape blocks require a nonempty label".to_string(),
                        ));
                    }
                    current.push(SlideElement::Shape { kind, label });
                    shape_kind = None;
                    shape_lines.clear();
                } else {
                    shape_lines.push(line.to_string());
                }
                continue;
            }
            if slides.is_empty()
                && current.is_empty()
                && paragraph_lines.is_empty()
                && !in_code
                && !in_notes
            {
                if trimmed.starts_with("title:") {
                    Self::apply_metadata_line(trimmed, &mut title, &mut resolved_aspect_ratio)?;
                    continue;
                }
                if trimmed.starts_with("aspect_ratio:") {
                    Self::apply_metadata_line(trimmed, &mut title, &mut resolved_aspect_ratio)?;
                    continue;
                }
            }
            if trimmed == "---" && !in_code && !in_notes {
                Self::flush_paragraph(&mut paragraph_lines, &mut current);
                if !current.is_empty() {
                    slides.push(Slide {
                        elements: std::mem::take(&mut current),
                    });
                }
                continue;
            }
            if trimmed.eq_ignore_ascii_case(":::notes") && !in_code {
                Self::flush_paragraph(&mut paragraph_lines, &mut current);
                in_notes = true;
                continue;
            }
            if in_notes {
                if trimmed == ":::" {
                    in_notes = false;
                } else {
                    current.push(SlideElement::Notes(line.to_string()));
                }
                continue;
            }
            if trimmed.starts_with("```") {
                Self::flush_paragraph(&mut paragraph_lines, &mut current);
                if in_code {
                    in_code = false;
                    let code = code_lines.join("\n");
                    if code_language.eq_ignore_ascii_case("mermaid") {
                        current.push(SlideElement::Diagram(code));
                    } else {
                        current.push(SlideElement::Code(code));
                    }
                    code_lines.clear();
                    code_language.clear();
                } else {
                    in_code = true;
                    code_language = trimmed.trim_start_matches('`').trim().to_string();
                }
                continue;
            }
            if in_code {
                code_lines.push(line);
                continue;
            }
            if let Some(shape) = trimmed.strip_prefix(":::shape ") {
                Self::flush_paragraph(&mut paragraph_lines, &mut current);
                shape_kind = Some(Self::parse_shape_kind(shape)?);
                continue;
            }
            if let Some((level, text)) = Self::heading(trimmed) {
                Self::flush_paragraph(&mut paragraph_lines, &mut current);
                current.push(SlideElement::Heading {
                    level,
                    text: text.to_string(),
                });
            } else if trimmed.is_empty() {
                Self::flush_paragraph(&mut paragraph_lines, &mut current);
            } else {
                paragraph_lines.push(line.trim_end().to_string());
            }
        }
        if in_code || in_notes || shape_kind.is_some() {
            return Err(PresentationError::MalformedSource(
                "unterminated presentation code, notes, or shape block".to_string(),
            ));
        }
        Self::flush_paragraph(&mut paragraph_lines, &mut current);
        if !current.is_empty() {
            slides.push(Slide { elements: current });
        }
        if slides.is_empty() {
            return Err(PresentationError::EmptyDeck);
        }
        Ok(Deck {
            title,
            aspect_ratio: resolved_aspect_ratio,
            slides,
            overflow_policy: OverflowPolicy::Reject,
        })
    }

    fn heading(line: &str) -> Option<(u8, &str)> {
        let level = line
            .chars()
            .take_while(|character| *character == '#')
            .count();
        if !(1..=6).contains(&level) {
            return None;
        }
        let rest = line.get(level..)?;
        let text = rest.strip_prefix(' ')?;
        Some((level as u8, text.trim()))
    }

    fn flush_paragraph(lines: &mut Vec<String>, elements: &mut Vec<SlideElement>) {
        if !lines.is_empty() {
            elements.push(SlideElement::Paragraph(lines.join("\n")));
            lines.clear();
        }
    }

    fn parse_shape_kind(value: &str) -> Result<ShapeKind, PresentationError> {
        match value.trim().to_ascii_lowercase().as_str() {
            "rect" | "rectangle" => Ok(ShapeKind::Rectangle),
            "rounded" => Ok(ShapeKind::Rounded),
            "circle" => Ok(ShapeKind::Circle),
            "diamond" => Ok(ShapeKind::Diamond),
            _ => Err(PresentationError::MalformedSource(
                "shape kind must be rectangle, rounded, circle, or diamond".to_string(),
            )),
        }
    }

    /// Validates deterministic fixed-canvas constraints before rendering.
    fn validate_deck(deck: &Deck) -> Result<(), PresentationError> {
        if deck.slides.is_empty() {
            return Err(PresentationError::EmptyDeck);
        }
        if deck.overflow_policy == OverflowPolicy::Clip {
            return Ok(());
        }
        for (index, slide) in deck.slides.iter().enumerate() {
            let estimated_lines = slide
                .elements
                .iter()
                .map(|element| match element {
                    SlideElement::Heading { .. } => 2,
                    SlideElement::Paragraph(text) => {
                        let physical_lines =
                            text.lines().filter(|line| !line.trim().is_empty()).count();
                        let wrapped_lines = text.chars().count().div_ceil(90);
                        physical_lines.max(wrapped_lines).max(1)
                    }
                    SlideElement::Code(text) => text.lines().count().max(1),
                    SlideElement::Diagram(text) => text.lines().count().max(4),
                    SlideElement::Shape { .. } => 4,
                    SlideElement::Notes(_) => 0,
                })
                .sum::<usize>();
            if estimated_lines > 34 {
                return Err(PresentationError::SlideOverflow {
                    slide: index + 1,
                    estimated_lines,
                });
            }
        }
        Ok(())
    }

    /// Strips a recognized YAML-style front-matter block (`---` delimited,
    /// `title:`/`aspect_ratio:` lines only) from the start of `source`, applying
    /// its metadata. Returns `None` when `source` has no such block, so the
    /// caller can leave its own content slice untouched.
    fn strip_yaml_frontmatter<'a>(
        source: &'a str,
        title: &mut String,
        aspect_ratio: &mut AspectRatio,
    ) -> Result<Option<&'a str>, PresentationError> {
        let Some(frontmatter) = source.strip_prefix("---\n") else {
            return Ok(None);
        };
        let Some(end) = frontmatter.find("\n---") else {
            return Ok(None);
        };
        let metadata = &frontmatter[..end];
        let recognized = metadata.lines().all(|line| {
            let key = line.split_once(':').map(|(key, _)| key.trim());
            matches!(key, Some("title" | "aspect_ratio"))
        });
        if !recognized || !metadata.lines().any(|line| !line.trim().is_empty()) {
            return Ok(None);
        }
        for line in metadata.lines() {
            Self::apply_metadata_line(line, title, aspect_ratio)?;
        }
        Ok(Some(&frontmatter[end + 4..]))
    }

    fn apply_metadata_line(
        line: &str,
        title: &mut String,
        aspect_ratio: &mut AspectRatio,
    ) -> Result<(), PresentationError> {
        let Some((key, value)) = line.split_once(':') else {
            return Err(PresentationError::MalformedSource(
                "metadata line must use key: value syntax".to_string(),
            ));
        };
        let value = value.trim().trim_matches(['\"', '\'']);
        match key.trim() {
            "title" => *title = value.to_string(),
            "aspect_ratio" => {
                *aspect_ratio = match value {
                    "16:9" | "16x9" => AspectRatio::Widescreen16x9,
                    "4:3" | "4x3" => AspectRatio::Standard4x3,
                    _ => {
                        return Err(PresentationError::MalformedSource(
                            "aspect_ratio must be 16:9 or 4:3".to_string(),
                        ));
                    }
                }
            }
            _ => {
                return Err(PresentationError::MalformedSource(
                    "unsupported deck metadata key".to_string(),
                ))
            }
        }
        Ok(())
    }

    fn apply_toml_metadata_line(
        line: &str,
        title: &mut String,
        aspect_ratio: &mut AspectRatio,
    ) -> Result<(), PresentationError> {
        let Some((key, value)) = line.split_once('=') else {
            return Err(PresentationError::MalformedSource(
                "TOML metadata line must use key = value syntax".to_string(),
            ));
        };
        Self::apply_metadata_line(
            &format!("{}: {}", key.trim(), value.trim()),
            title,
            aspect_ratio,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_single_heading_happy() {
        let deck =
            DefaultMarkdownDeckParser::parse_markdown("# Slide", AspectRatio::Widescreen16x9)
                .unwrap_or_else(|error| panic!("valid source failed to parse: {error}"));
        assert_eq!(deck.slides.len(), 1);
    }

    #[test]
    fn test_parse_markdown_empty_source_error() {
        assert_eq!(
            DefaultMarkdownDeckParser::parse_markdown("", AspectRatio::Standard4x3),
            Err(PresentationError::EmptyDeck)
        );
    }

    #[test]
    fn test_validate_deck_overfull_slide_error() {
        let deck = DefaultMarkdownDeckParser::parse_markdown(
            &format!("# Slide\n{}", "line\n".repeat(40)),
            AspectRatio::Widescreen16x9,
        )
        .unwrap_or_else(|error| panic!("valid overflow fixture failed to parse: {error}"));
        assert!(matches!(
            DefaultMarkdownDeckParser::validate_deck(&deck),
            Err(PresentationError::SlideOverflow { .. })
        ));
    }

    #[test]
    fn test_validate_deck_within_budget_happy() {
        let deck =
            DefaultMarkdownDeckParser::parse_markdown("# Slide", AspectRatio::Widescreen16x9)
                .unwrap_or_else(|error| panic!("valid source failed to parse: {error}"));
        assert_eq!(DefaultMarkdownDeckParser::validate_deck(&deck), Ok(()));
    }

    #[test]
    fn test_strip_yaml_frontmatter_recognized_block_happy() {
        let mut title = String::new();
        let mut aspect_ratio = AspectRatio::Widescreen16x9;
        let remainder = DefaultMarkdownDeckParser::strip_yaml_frontmatter(
            "---\ntitle: Front Matter\naspect_ratio: 4:3\n---\n# Slide",
            &mut title,
            &mut aspect_ratio,
        )
        .unwrap_or_else(|error| panic!("recognized frontmatter failed: {error}"));
        assert_eq!(remainder, Some("\n# Slide"));
        assert_eq!(title, "Front Matter");
        assert_eq!(aspect_ratio, AspectRatio::Standard4x3);
    }

    #[test]
    fn test_strip_yaml_frontmatter_unrecognized_block_edge() {
        // A `---`-delimited block whose keys aren't title/aspect_ratio isn't
        // front matter at all -- it's the first slide's own content, so the
        // source is returned unconsumed via None rather than an error.
        let mut title = String::new();
        let mut aspect_ratio = AspectRatio::Widescreen16x9;
        let remainder = DefaultMarkdownDeckParser::strip_yaml_frontmatter(
            "---\nunexpected: value\n---\n# Slide",
            &mut title,
            &mut aspect_ratio,
        )
        .unwrap_or_else(|error| panic!("unrecognized-block fixture failed: {error}"));
        assert_eq!(remainder, None);
    }

    #[test]
    fn test_apply_metadata_line_title_happy() {
        let mut title = String::new();
        let mut aspect_ratio = AspectRatio::Widescreen16x9;
        DefaultMarkdownDeckParser::apply_metadata_line(
            "title: Quarterly Review",
            &mut title,
            &mut aspect_ratio,
        )
        .unwrap_or_else(|error| panic!("valid metadata line failed: {error}"));
        assert_eq!(title, "Quarterly Review");
    }

    #[test]
    fn test_apply_metadata_line_unsupported_key_error() {
        let mut title = String::new();
        let mut aspect_ratio = AspectRatio::Widescreen16x9;
        assert!(matches!(
            DefaultMarkdownDeckParser::apply_metadata_line(
                "unsupported: value",
                &mut title,
                &mut aspect_ratio
            ),
            Err(PresentationError::MalformedSource(_))
        ));
    }

    #[test]
    fn test_apply_toml_metadata_line_aspect_ratio_happy() {
        let mut title = String::new();
        let mut aspect_ratio = AspectRatio::Widescreen16x9;
        DefaultMarkdownDeckParser::apply_toml_metadata_line(
            "aspect_ratio = '4:3'",
            &mut title,
            &mut aspect_ratio,
        )
        .unwrap_or_else(|error| panic!("valid TOML metadata line failed: {error}"));
        assert_eq!(aspect_ratio, AspectRatio::Standard4x3);
    }

    #[test]
    fn test_apply_toml_metadata_line_missing_equals_error() {
        let mut title = String::new();
        let mut aspect_ratio = AspectRatio::Widescreen16x9;
        assert!(matches!(
            DefaultMarkdownDeckParser::apply_toml_metadata_line(
                "not a key value line",
                &mut title,
                &mut aspect_ratio
            ),
            Err(PresentationError::MalformedSource(_))
        ));
    }
}
