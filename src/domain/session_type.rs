use crossterm::style::Color;

/// Represents the type of session (work or break)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    Work,
    Break,
}

impl SessionType {
    pub fn display_name(&self) -> &str {
        match self {
            SessionType::Work => "WORK",
            SessionType::Break => "BREAK",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            SessionType::Work => Color::Green,
            SessionType::Break => Color::Cyan,
        }
    }

    pub fn next(&self) -> Self {
        match self {
            SessionType::Work => SessionType::Break,
            SessionType::Break => SessionType::Work,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_type_transition() {
        assert_eq!(SessionType::Work.next(), SessionType::Break);
        assert_eq!(SessionType::Break.next(), SessionType::Work);
    }

    #[test]
    fn test_session_colors() {
        assert_eq!(SessionType::Work.color(), Color::Green);
        assert_eq!(SessionType::Break.color(), Color::Cyan);
    }

    #[test]
    fn test_session_display_names() {
        assert_eq!(SessionType::Work.display_name(), "WORK");
        assert_eq!(SessionType::Break.display_name(), "BREAK");
    }
}
