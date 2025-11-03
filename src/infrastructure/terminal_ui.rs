use anyhow::Result;
use crossterm::{
    cursor,
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{self, Write};

use crate::domain::{SessionState, TimerSession};
use super::digit_display::DigitDisplay;

/// Terminal UI renderer
pub struct TerminalUI;

impl TerminalUI {
    pub fn render(session: &TimerSession) -> Result<()> {
        if session.state() == SessionState::Finished {
            return Self::render_finish_screen(session);
        }

        execute!(
            io::stdout(),
            cursor::MoveTo(0, 0),
            terminal::Clear(ClearType::All)
        )?;

        let mut stdout = io::stdout();
        let session_color = session.session_type().color();

        // Display large session type banner
        let banner_text = "╔═══════════════════════════════════════════╗";
        let session_type = session.session_type();
        let session_name = session_type.display_name();
        let session_line = format!("║{:^43}║", session_name);
        let bottom_line = "╚═══════════════════════════════════════════╝";

        execute!(
            stdout,
            cursor::MoveTo(2, 1),
            SetForegroundColor(session_color),
            Print(&banner_text),
            cursor::MoveTo(2, 2),
            Print(&session_line),
            cursor::MoveTo(2, 3),
            Print(&bottom_line),
            ResetColor
        )?;

        // Display state indicator
        let state_text = match session.state() {
            SessionState::Running => "▶ Running",
            SessionState::Paused => "⏸ PAUSED",
            SessionState::Completed => "✓ Completed",
            SessionState::Finished => "★ Finished",
        };
        let state_color = match session.state() {
            SessionState::Running => Color::Green,
            SessionState::Paused => Color::Yellow,
            SessionState::Completed => Color::Blue,
            SessionState::Finished => Color::Magenta,
        };
        execute!(
            stdout,
            cursor::MoveTo(16, 5),
            SetForegroundColor(state_color),
            Print(&format!("[{}]", state_text)),
            ResetColor
        )?;

        // Display large digital timer
        let total_secs = session.remaining().as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;

        let time_lines = DigitDisplay::render_time(minutes, seconds);

        for (i, line) in time_lines.iter().enumerate() {
            execute!(
                stdout,
                cursor::MoveTo(6, 7 + i as u16),
                SetForegroundColor(session_color),
                Print(line),
                ResetColor
            )?;
        }

        // Display cycle count
        let cycle_info = if let Some(max) = session.max_cycles() {
            format!("Cycle: {}/{}", session.cycle_count(), max)
        } else {
            format!("Cycle: {}", session.cycle_count())
        };
        execute!(
            stdout,
            cursor::MoveTo(18, 13),
            SetForegroundColor(Color::DarkGrey),
            Print(&cycle_info),
            ResetColor
        )?;

        // Display controls
        execute!(
            stdout,
            cursor::MoveTo(2, 16),
            SetForegroundColor(Color::DarkGrey),
            Print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"),
            cursor::MoveTo(2, 17),
            Print("  [SPACE] Pause/Resume  │  [ENTER] Skip  │  [Q] Quit"),
            ResetColor
        )?;

        stdout.flush()?;
        Ok(())
    }

    fn render_finish_screen(session: &TimerSession) -> Result<()> {
        execute!(
            io::stdout(),
            cursor::MoveTo(0, 0),
            terminal::Clear(ClearType::All)
        )?;

        let mut stdout = io::stdout();

        // Display large finish banner
        let banner_text = "╔═══════════════════════════════════════════╗";
        let finish_line = "║           🎉  FINISHED!  🎉               ║";
        let bottom_line = "╚═══════════════════════════════════════════╝";

        execute!(
            stdout,
            cursor::MoveTo(2, 1),
            SetForegroundColor(Color::Magenta),
            Print(&banner_text),
            cursor::MoveTo(2, 2),
            Print(finish_line),
            cursor::MoveTo(2, 3),
            Print(&bottom_line),
            ResetColor
        )?;

        // Display congratulations message
        let cycles_info = format!("Total cycles completed: {}", session.cycle_count());
        let congrats_lines = vec![
            "Great work! You've completed all cycles.",
            "",
            cycles_info.as_str(),
        ];

        for (i, line) in congrats_lines.iter().enumerate() {
            execute!(
                stdout,
                cursor::MoveTo(8, 7 + i as u16),
                SetForegroundColor(Color::White),
                Print(line),
                ResetColor
            )?;
        }

        // Display large "DONE" text
        let done_lines = vec![
            "████    ███   █   █ ████",
            "█   █  █   █  ██  █ █   ",
            "█   █  █   █  █ █ █ ███ ",
            "█   █  █   █  █  ██ █   ",
            "████    ███   █   █ ████",
        ];

        for (i, line) in done_lines.iter().enumerate() {
            execute!(
                stdout,
                cursor::MoveTo(10, 12 + i as u16),
                SetForegroundColor(Color::Cyan),
                Print(line),
                ResetColor
            )?;
        }

        // Display controls
        execute!(
            stdout,
            cursor::MoveTo(2, 19),
            SetForegroundColor(Color::DarkGrey),
            Print("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"),
            cursor::MoveTo(2, 20),
            Print("  [ENTER] or [Q] Exit  │  Auto-exit in 5 seconds..."),
            ResetColor
        )?;

        stdout.flush()?;
        Ok(())
    }
}
