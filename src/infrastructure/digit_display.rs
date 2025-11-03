/// ASCII art for large digit display (7-segment style)
pub struct DigitDisplay;

impl DigitDisplay {
    // Each digit is 5 lines tall and 5 characters wide
    const DIGITS: [&'static [&'static str; 5]; 10] = [
        // 0
        &[
            " ███ ",
            "█   █",
            "█   █",
            "█   █",
            " ███ ",
        ],
        // 1
        &[
            "  █  ",
            " ██  ",
            "  █  ",
            "  █  ",
            " ███ ",
        ],
        // 2
        &[
            " ███ ",
            "    █",
            " ███ ",
            "█    ",
            "█████",
        ],
        // 3
        &[
            "████ ",
            "    █",
            " ███ ",
            "    █",
            "████ ",
        ],
        // 4
        &[
            "█   █",
            "█   █",
            "█████",
            "    █",
            "    █",
        ],
        // 5
        &[
            "█████",
            "█    ",
            "████ ",
            "    █",
            "████ ",
        ],
        // 6
        &[
            " ███ ",
            "█    ",
            "████ ",
            "█   █",
            " ███ ",
        ],
        // 7
        &[
            "█████",
            "    █",
            "   █ ",
            "  █  ",
            "  █  ",
        ],
        // 8
        &[
            " ███ ",
            "█   █",
            " ███ ",
            "█   █",
            " ███ ",
        ],
        // 9
        &[
            " ███ ",
            "█   █",
            " ████",
            "    █",
            " ███ ",
        ],
    ];

    const COLON: [&'static str; 5] = [
        "  ",
        "██",
        "  ",
        "██",
        "  ",
    ];

    /// Render time in large ASCII art format (MM:SS)
    pub fn render_time(minutes: u64, seconds: u64) -> Vec<String> {
        let min_tens = (minutes / 10) as usize;
        let min_ones = (minutes % 10) as usize;
        let sec_tens = (seconds / 10) as usize;
        let sec_ones = (seconds % 10) as usize;

        let mut lines = vec![String::new(); 5];

        for line_idx in 0..5 {
            lines[line_idx].push_str(Self::DIGITS[min_tens][line_idx]);
            lines[line_idx].push_str("  ");
            lines[line_idx].push_str(Self::DIGITS[min_ones][line_idx]);
            lines[line_idx].push_str("  ");
            lines[line_idx].push_str(Self::COLON[line_idx]);
            lines[line_idx].push_str("  ");
            lines[line_idx].push_str(Self::DIGITS[sec_tens][line_idx]);
            lines[line_idx].push_str("  ");
            lines[line_idx].push_str(Self::DIGITS[sec_ones][line_idx]);
        }

        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digit_display_rendering() {
        let lines = DigitDisplay::render_time(12, 34);

        // Should produce 5 lines
        assert_eq!(lines.len(), 5);

        // Each line should be non-empty
        for line in &lines {
            assert!(!line.is_empty());
        }

        // Basic sanity check for format
        // Lines should contain digit characters and spaces
        for line in &lines {
            assert!(line.chars().all(|c| c == '█' || c == ' '));
        }
    }
}
