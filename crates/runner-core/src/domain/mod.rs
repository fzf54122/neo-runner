use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: String,
    pub task_type: String,
    pub cmd: Option<String>,
    // yaml没写时，自动空数组
    #[serde(default)]
    pub depends_on: Vec<String>,
    pub timeout_ms: Option<u64>,
    pub retry: Option<RetrySpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobSpec {
    pub name: String,
    pub fail_fast: bool,
    pub max_concurrency: usize,
    pub tasks: Vec<TaskSpec>,
    pub default_timeout_ms: Option<u64>,
    pub default_retry: RetrySpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunResult {
    pub success: bool,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrySpec {
    pub max_attempts: u32,
}
