#![allow(missing_docs)]

use md_parser::{
    AspectRatio, DeckParser, FactoryDeckParser, ParseRequest, PresentationError, SlideElement,
    ValidateRequest,
};

/// @covers: DeckParser
#[test]
fn test_paragraph_blocks_preserve_rich_markdown_source() {
    let response = FactoryDeckParser
        .parse(ParseRequest {
            source: "A paragraph\nwith **strong text** and a [link](https://example.com).\n\n- first item\n- second item\n\n> quoted text\n> continued quote".to_string(),
            default_aspect_ratio: AspectRatio::Widescreen16x9,
        })
        .unwrap_or_else(|error| panic!("rich Markdown fixture failed: {error}"));
    assert_eq!(
        response.deck.slides[0].elements,
        vec![
            SlideElement::Paragraph(
                "A paragraph\nwith **strong text** and a [link](https://example.com).".to_string()
            ),
            SlideElement::Paragraph("- first item\n- second item".to_string()),
            SlideElement::Paragraph("> quoted text\n> continued quote".to_string()),
        ]
    );
}

/// @covers: DeckParser
#[test]
fn test_all_common_heading_levels_are_preserved() {
    let response = FactoryDeckParser
        .parse(ParseRequest {
            source: "# One\n## Two\n### Three\n#### Four\n##### Five\n###### Six".to_string(),
            default_aspect_ratio: AspectRatio::Widescreen16x9,
        })
        .unwrap_or_else(|error| panic!("heading fixture failed: {error}"));
    let levels: Vec<_> = response.deck.slides[0]
        .elements
        .iter()
        .filter_map(|element| match element {
            SlideElement::Heading { level, .. } => Some(*level),
            _ => None,
        })
        .collect();
    assert_eq!(levels, vec![1, 2, 3, 4, 5, 6]);
}

/// @covers: DeckParser
#[test]
fn test_fenced_code_preserves_blocks_and_blank_lines() {
    let cases = [
        ("```rust\nhello\n```", vec!["hello"]),
        ("```\n```", vec![""]),
        ("```\n\nhello\n\n\n```", vec!["\nhello\n\n"]),
        ("```\nfirst\n```\n```\nsecond\n```", vec!["first", "second"]),
        (
            "```\ntitle: code\naspect_ratio: text\n---\n:::notes\n```",
            vec!["title: code\naspect_ratio: text\n---\n:::notes"],
        ),
    ];
    for (source, expected) in cases {
        let response = FactoryDeckParser
            .parse(ParseRequest {
                source: source.to_string(),
                default_aspect_ratio: AspectRatio::Widescreen16x9,
            })
            .unwrap_or_else(|error| panic!("code fixture failed: {error}"));
        let expected_elements: Vec<_> = expected
            .into_iter()
            .map(|text| SlideElement::Code(text.to_string()))
            .collect();
        assert_eq!(response.deck.slides.len(), 1);
        assert_eq!(
            response.deck.slides[0].elements, expected_elements,
            "{source:?}"
        );
        assert!(response.deck.title.is_empty());
    }
}

/// @covers: DeckParser
#[test]
fn test_code_line_budget_boundary() {
    for line_count in [34, 35] {
        let response = FactoryDeckParser
            .parse(ParseRequest {
                source: format!("```\n{}```", "code\n".repeat(line_count)),
                default_aspect_ratio: AspectRatio::Widescreen16x9,
            })
            .unwrap_or_else(|error| panic!("code fixture failed: {error}"));
        let result = FactoryDeckParser.validate(ValidateRequest {
            deck: response.deck,
        });
        if line_count == 34 {
            assert_eq!(result, Ok(()));
        } else {
            assert_eq!(
                result,
                Err(PresentationError::SlideOverflow {
                    slide: 1,
                    estimated_lines: 35
                })
            );
        }
    }
}

/// @covers: DeckParser
#[test]
fn test_notes_preserve_metadata_and_slide_markers() {
    let response = FactoryDeckParser
        .parse(ParseRequest {
            source: ":::notes\ntitle: secret\n---\n:::\n# Slide".to_string(),
            default_aspect_ratio: AspectRatio::Standard4x3,
        })
        .unwrap_or_else(|error| panic!("notes fixture failed: {error}"));
    assert!(response.deck.title.is_empty());
    assert_eq!(response.deck.slides.len(), 1);
    assert_eq!(
        response.deck.slides[0].elements,
        vec![
            SlideElement::Notes("title: secret".to_string()),
            SlideElement::Notes("---".to_string()),
            SlideElement::Heading {
                level: 1,
                text: "Slide".to_string()
            },
        ]
    );
    assert!(matches!(
        FactoryDeckParser.parse(ParseRequest {
            source: ":::notes\nsecret\n---\n# Next".to_string(),
            default_aspect_ratio: AspectRatio::Standard4x3,
        }),
        Err(PresentationError::MalformedSource(_))
    ));
}

/// @covers: DeckParser
#[test]
fn test_parse_notes_and_code_source_order_happy() {
    let response = FactoryDeckParser
        .parse(ParseRequest {
            source:
                "# First\nBody\n:::notes\nsecret\n:::\n---\n# Second\n```rust\nfn main() {}\n```"
                    .to_string(),
            default_aspect_ratio: AspectRatio::Widescreen16x9,
        })
        .unwrap_or_else(|error| panic!("valid fixture failed to parse: {error}"));
    assert_eq!(response.deck.slides.len(), 2);
    assert!(response.deck.slides[0]
        .elements
        .iter()
        .any(|element| matches!(element, SlideElement::Notes(_))));
    assert!(response.deck.slides[1]
        .elements
        .iter()
        .any(|element| matches!(element, SlideElement::Code(text) if text == "fn main() {}")));
}

/// @covers: DeckParser
#[test]
fn test_parse_unterminated_block_or_empty_source_error() {
    assert_eq!(
        FactoryDeckParser
            .parse(ParseRequest {
                source: String::new(),
                default_aspect_ratio: AspectRatio::Standard4x3,
            })
            .err(),
        Some(PresentationError::EmptyDeck)
    );
    assert!(matches!(
        FactoryDeckParser
            .parse(ParseRequest {
                source: "```\ncode".to_string(),
                default_aspect_ratio: AspectRatio::Standard4x3,
            })
            .err(),
        Some(PresentationError::MalformedSource(_))
    ));
}

/// @covers: DeckParser
#[test]
fn test_parse_bare_metadata_lines_preserve_slide_order_happy() {
    let response = FactoryDeckParser
        .parse(ParseRequest {
            source: "title: Quarterly Review\naspect_ratio: 4:3\n---\n# Slide".to_string(),
            default_aspect_ratio: AspectRatio::Widescreen16x9,
        })
        .unwrap_or_else(|error| panic!("valid metadata fixture failed to parse: {error}"));
    assert_eq!(response.deck.title, "Quarterly Review");
    assert_eq!(response.deck.aspect_ratio, AspectRatio::Standard4x3);
    assert_eq!(response.deck.slides.len(), 1);
}

/// @covers: DeckParser
#[test]
fn test_parse_toml_front_matter_happy() {
    let response = FactoryDeckParser
        .parse(ParseRequest {
            source: "+++\ntitle = 'Quarterly Review'\naspect_ratio = '4:3'\n+++\n# Slide"
                .to_string(),
            default_aspect_ratio: AspectRatio::Widescreen16x9,
        })
        .unwrap_or_else(|error| panic!("valid TOML metadata fixture failed: {error}"));
    assert_eq!(response.deck.title, "Quarterly Review");
    assert_eq!(response.deck.aspect_ratio, AspectRatio::Standard4x3);
}

/// @covers: DeckParser
#[test]
fn test_parse_yaml_front_matter_happy() {
    let response = FactoryDeckParser
        .parse(ParseRequest {
            source: "---\ntitle: Front Matter\naspect_ratio: 4:3\n---\n# Slide".to_string(),
            default_aspect_ratio: AspectRatio::Widescreen16x9,
        })
        .unwrap_or_else(|error| panic!("frontmatter fixture failed to parse: {error}"));
    assert_eq!(response.deck.title, "Front Matter");
    assert_eq!(response.deck.aspect_ratio, AspectRatio::Standard4x3);
    assert_eq!(response.deck.slides.len(), 1);
}

/// @covers: DeckParser
#[test]
fn test_validate_overfull_slide_error() {
    let response = FactoryDeckParser
        .parse(ParseRequest {
            source: format!("# Slide\n{}", "line\n".repeat(40)),
            default_aspect_ratio: AspectRatio::Widescreen16x9,
        })
        .unwrap_or_else(|error| panic!("valid overflow fixture failed to parse: {error}"));
    assert!(matches!(
        FactoryDeckParser.validate(ValidateRequest {
            deck: response.deck
        }),
        Err(PresentationError::SlideOverflow { .. })
    ));
}
