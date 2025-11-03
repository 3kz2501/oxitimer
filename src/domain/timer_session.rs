use std::time::{Duration, Instant};

use super::session_state::SessionState;
use super::session_type::SessionType;

/// Core domain model representing a timer session
#[derive(Debug)]
pub struct TimerSession {
    session_type: SessionType,
    work_duration: Duration,
    break_duration: Duration,
    remaining: Duration,
    state: SessionState,
    start_time: Option<Instant>,
    cycle_count: usize,
    max_cycles: Option<usize>,
}

impl TimerSession {
    pub fn new(work_seconds: u64, break_seconds: u64, max_cycles: Option<usize>) -> Self {
        Self {
            session_type: SessionType::Work,
            work_duration: Duration::from_secs(work_seconds),
            break_duration: Duration::from_secs(break_seconds),
            remaining: Duration::from_secs(work_seconds),
            state: SessionState::Running,
            start_time: Some(Instant::now()),
            cycle_count: 1, // Start from 1 instead of 0 for better UX
            max_cycles,
        }
    }

    pub fn session_type(&self) -> SessionType {
        self.session_type
    }

    pub fn remaining(&self) -> Duration {
        self.remaining
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn cycle_count(&self) -> usize {
        self.cycle_count
    }

    pub fn max_cycles(&self) -> Option<usize> {
        self.max_cycles
    }

    pub fn is_finished(&self) -> bool {
        if let Some(max) = self.max_cycles {
            // During a Work session, check if we've exceeded max cycles
            // During a Break session, check if we've reached max cycles
            match self.session_type {
                SessionType::Work => self.cycle_count > max,
                SessionType::Break => self.cycle_count >= max,
            }
        } else {
            false
        }
    }

    fn current_duration(&self) -> Duration {
        match self.session_type {
            SessionType::Work => self.work_duration,
            SessionType::Break => self.break_duration,
        }
    }

    pub fn update(&mut self) -> bool {
        if self.state != SessionState::Running {
            return false;
        }

        if let Some(start) = self.start_time {
            let elapsed = start.elapsed();
            if elapsed >= self.current_duration() {
                // Ensure remaining is exactly zero when session completes
                self.remaining = Duration::ZERO;
                self.complete_session();
                return true;
            }
            self.remaining = self.current_duration() - elapsed;
        }
        false
    }

    pub fn complete_session(&mut self) {
        self.state = SessionState::Completed;
    }

    pub fn advance_to_next_session(&mut self) {
        // If we just completed a Work session and reached max cycles, finish immediately
        if self.session_type == SessionType::Work && self.is_finished() {
            self.state = SessionState::Finished;
            return;
        }

        // Otherwise, advance to next session type
        self.session_type = self.session_type.next();

        // Increment cycle count when transitioning from Break to Work
        if self.session_type == SessionType::Work {
            self.cycle_count += 1;
        }

        // Check again if we've reached the max cycles (after Break -> Work transition)
        if self.is_finished() {
            self.state = SessionState::Finished;
            return;
        }

        self.remaining = self.current_duration();
        self.state = SessionState::Running;
        self.start_time = Some(Instant::now());
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            SessionState::Running => {
                self.state = SessionState::Paused;
            }
            SessionState::Paused => {
                self.state = SessionState::Running;
                self.start_time = Some(Instant::now());
            }
            SessionState::Completed | SessionState::Finished => {}
        }
    }

    #[allow(dead_code)]
    pub fn format_remaining(&self) -> String {
        let total_secs = self.remaining.as_secs();
        let minutes = total_secs / 60;
        let seconds = total_secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_session_initialization() {
        let session = TimerSession::new(100, 50, None);
        assert_eq!(session.session_type(), SessionType::Work);
        assert_eq!(session.state(), SessionState::Running);
        assert_eq!(session.cycle_count(), 1); // Start from 1
        assert_eq!(session.remaining().as_secs(), 100);
        assert_eq!(session.max_cycles(), None);
    }

    #[test]
    fn test_timer_session_with_cycles() {
        let session = TimerSession::new(100, 50, Some(3));
        assert_eq!(session.max_cycles(), Some(3));
        assert!(!session.is_finished());
    }

    #[test]
    fn test_session_advancement() {
        let mut session = TimerSession::new(1, 1, None);

        // Start with work
        assert_eq!(session.session_type(), SessionType::Work);
        assert_eq!(session.cycle_count(), 1); // Start from 1

        // Complete work, advance to break
        session.complete_session();
        session.advance_to_next_session();
        assert_eq!(session.session_type(), SessionType::Break);
        assert_eq!(session.cycle_count(), 1); // Cycle count increases after break
        assert_eq!(session.state(), SessionState::Running);

        // Complete break, advance to work
        session.complete_session();
        session.advance_to_next_session();
        assert_eq!(session.session_type(), SessionType::Work);
        assert_eq!(session.cycle_count(), 2); // Cycle completed
        assert_eq!(session.state(), SessionState::Running);
    }

    #[test]
    fn test_cycle_limit() {
        let mut session = TimerSession::new(1, 1, Some(2));

        assert!(!session.is_finished());
        assert_eq!(session.state(), SessionState::Running);
        assert_eq!(session.cycle_count(), 1); // Start from 1

        // Complete first cycle (Work -> Break -> Work)
        session.complete_session();
        session.advance_to_next_session(); // Work -> Break
        assert_eq!(session.session_type(), SessionType::Break);
        assert_eq!(session.cycle_count(), 1);
        assert_eq!(session.state(), SessionState::Running);

        session.complete_session();
        session.advance_to_next_session(); // Break -> Work (cycle 2)
        assert_eq!(session.session_type(), SessionType::Work);
        assert_eq!(session.cycle_count(), 2);
        assert!(!session.is_finished()); // Not finished yet, still in cycle 2
        assert_eq!(session.state(), SessionState::Running);

        // Complete second work session -> should finish immediately without break
        session.complete_session();
        session.advance_to_next_session(); // Work -> Finished (no break)
        assert_eq!(session.cycle_count(), 2);
        assert!(session.is_finished());
        assert_eq!(session.state(), SessionState::Finished);
    }

    #[test]
    fn test_cycle_limit_exact() {
        // Test with 1 cycle - should finish immediately after first work session
        let mut session = TimerSession::new(1, 1, Some(1));
        assert_eq!(session.cycle_count(), 1); // Start from 1

        // Complete work - should finish immediately without break
        session.complete_session();
        session.advance_to_next_session(); // Work -> Finished (no break)
        assert_eq!(session.cycle_count(), 1);
        assert_eq!(session.state(), SessionState::Finished);
        assert!(session.is_finished());
    }

    #[test]
    fn test_pause_resume() {
        let mut session = TimerSession::new(100, 50, None);

        assert_eq!(session.state(), SessionState::Running);

        session.toggle_pause();
        assert_eq!(session.state(), SessionState::Paused);

        session.toggle_pause();
        assert_eq!(session.state(), SessionState::Running);
    }

    #[test]
    fn test_format_remaining() {
        let session = TimerSession::new(125, 50, None); // 2:05
        assert_eq!(session.format_remaining(), "02:05");

        let session = TimerSession::new(3661, 50, None); // 61:01
        assert_eq!(session.format_remaining(), "61:01");

        let session = TimerSession::new(0, 50, None); // 0:00
        assert_eq!(session.format_remaining(), "00:00");
    }
}
