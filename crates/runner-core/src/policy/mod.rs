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
