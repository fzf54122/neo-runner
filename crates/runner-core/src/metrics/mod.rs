#[derive(Debug, Clone, Default)]
pub struct RunMetrics {
    pub succeeded: usize,
    pub failed: usize,
    pub duration_ms: u128,
}
