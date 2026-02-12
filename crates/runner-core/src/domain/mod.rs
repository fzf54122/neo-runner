use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: String,
    pub task_type: String,
    pub cmd: Option<String>,
    pub method: Option<String>,
    pub url: Option<String>,
    pub expected_status: Option<Vec<u16>>,
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
    pub failed: usize,
    pub tasks: Vec<TaskRunResult>,
    pub events: Vec<RunEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRunResult {
    pub id: String,
    pub success: bool,
    pub attempts: u32,
    pub error: Option<String>,
    pub duration_ms: u128,
    pub exit_code: Option<i32>,
    pub status_code: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunEvent {
    pub kind: String,
    pub task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrySpec {
    pub max_attempts: u32,
}
