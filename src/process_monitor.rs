/// Simple process state enum for polling-based monitoring
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ProcessEvent {
    /// Process is alive
    Alive,
    /// Process is not alive
    NotAlive,
}
