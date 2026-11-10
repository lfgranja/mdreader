//! Prosodic plan for semantic markdown reading.

/// Prosodic plan variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProsodyPlan {
    /// Normal speech.
    Normal,
    /// Heading: slower.
    Heading,
    /// Emphasis: slight slowdown.
    Emphasis,
    /// Strong: slower.
    Strong,
    /// Block quote.
    Quote,
    /// Code: slow, deliberate.
    Code,
    /// List item.
    List,
    /// Question.
    Question,
    /// Exclamation.
    Exclamation,
    /// Thematic break: pause.
    ThematicBreak,
}

impl ProsodyPlan {
    /// Speed factor for this plan.
    #[must_use]
    pub const fn speed(self) -> f32 {
        match self {
            Self::Heading | Self::Strong => 0.9,
            Self::Emphasis | Self::Quote | Self::List => 0.95,
            Self::Code => 0.85,
            Self::Normal | Self::Question | Self::Exclamation | Self::ThematicBreak => 1.0,
        }
    }
}

/// Determine prosodic plan from segment content.
#[must_use]
pub fn plan_for_segment(seg: &crate::chunk::DocumentSegment) -> ProsodyPlan {
    let t = seg.content.trim();
    if t.is_empty() {
        return ProsodyPlan::Normal;
    }
    if t.chars()
        .all(|c| c == '*' || c == '-' || c == '_' || c.is_whitespace())
        && t.len() >= 3
    {
        return ProsodyPlan::ThematicBreak;
    }
    if t.starts_with('#') {
        return ProsodyPlan::Heading;
    }
    if t.starts_with('>') {
        return ProsodyPlan::Quote;
    }
    if t.starts_with("```") {
        return ProsodyPlan::Code;
    }
    if t.starts_with("* ") || t.starts_with("- ") || t.starts_with("+ ") {
        return ProsodyPlan::List;
    }
    if t.contains("**") {
        return ProsodyPlan::Strong;
    }
    if t.contains('*') || t.contains('_') {
        return ProsodyPlan::Emphasis;
    }
    if t.ends_with('?') {
        return ProsodyPlan::Question;
    }
    if t.ends_with('!') {
        return ProsodyPlan::Exclamation;
    }
    ProsodyPlan::Normal
}
