use crate::domain::JobSpec;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
}

#[derive(Debug, Clone)]
pub struct TimeoutPolicy {
    pub seconds: u64,
}

#[derive(Debug, Clone)]
pub struct ExecutionPolicy {
    pub fail_fast: bool,
    pub concurrency: usize,
    pub retry: RetryPolicy,
    pub timeout: TimeoutPolicy,
}

impl ExecutionPolicy {
    pub fn from_job(job: &JobSpec) -> Self {
        let max_attempts = job.default_retry.max_attempts.max(1);
        let max_retries = max_attempts.saturating_sub(1);
        let timeout_seconds = job.default_timeout_ms.unwrap_or(0) / 1000;

        Self {
            fail_fast: job.fail_fast,
            concurrency: job.max_concurrency.max(1),
            retry: RetryPolicy { max_retries },
            timeout: TimeoutPolicy {
                seconds: timeout_seconds,
            },
        }
    }
}
