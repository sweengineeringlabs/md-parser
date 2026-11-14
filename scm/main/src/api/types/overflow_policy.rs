/// Policy used when estimated slide content exceeds the fixed canvas.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OverflowPolicy {
    /// Reject the deck with an actionable validation error.
    Reject,
    /// Skip overflow rejection during validation. Rendering and clipping
    /// are the caller's responsibility; empty decks are still rejected.
    Clip,
}
