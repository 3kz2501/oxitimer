/// Timer session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Running,
    Paused,
    Completed,
    Finished, // All cycles completed
}
