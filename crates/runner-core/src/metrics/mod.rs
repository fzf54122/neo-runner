use crate::domain::RunResult;

#[derive(Debug, Clone, Default)]
pub struct RunMetrics {
    pub succeeded: usize,
    pub failed: usize,
    pub duration_ms: u128,
}

impl RunMetrics {
    pub fn from_result(result: &RunResult) -> Self {
        let failed = result.failed;
        let succeeded = result.total.saturating_sub(failed);
        let duration_ms = result.tasks.iter().map(|t| t.duration_ms).sum();
        Self {
            succeeded,
            failed,
            duration_ms,
        }
    }
}
