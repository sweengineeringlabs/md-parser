/// A supported slide element.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SlideElement {
    /// Plain text content.
    Paragraph(String),
    /// Heading content and level.
    Heading {
        /// Heading level.
        level: u8,
        /// Heading text.
        text: String,
    },
    /// Fenced code content.
    Code(String),
    /// Mermaid source from a fenced `mermaid` code block.
    Diagram(String),
    /// A labeled built-in presentation shape.
    Shape {
        /// Shape geometry.
        kind: ShapeKind,
        /// Markdown label displayed inside the shape.
        label: String,
    },
    /// Speaker notes retained in the deck and ignored by the line estimate.
    /// The caller decides whether to render them.
    Notes(String),
}

/// Geometry for a built-in presentation shape directive.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ShapeKind {
    /// Rectangular box.
    Rectangle,
    /// Box with rounded corners.
    Rounded,
    /// Circle or ellipse.
    Circle,
    /// Diamond used for a decision node.
    Diamond,
}
