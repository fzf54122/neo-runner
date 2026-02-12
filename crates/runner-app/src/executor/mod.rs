use runner_core::domain::TaskSpec;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

pub type TaskFuture<'a> = Pin<Box<dyn Future<Output = ExecutionResult> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub error: Option<String>,
    pub exit_code: Option<i32>,
    pub status_code: Option<u16>,
}

impl ExecutionResult {
    pub fn ok(exit_code: Option<i32>, status_code: Option<u16>) -> Self {
        Self {
            success: true,
            error: None,
            exit_code,
            status_code,
        }
    }

    pub fn err(message: impl Into<String>, exit_code: Option<i32>, status_code: Option<u16>) -> Self {
        Self {
            success: false,
            error: Some(message.into()),
            exit_code,
            status_code,
        }
    }
}

pub trait TaskExecutor: Send + Sync {
    fn task_type(&self) -> &'static str;
    fn execute<'a>(&'a self, task: &'a TaskSpec) -> TaskFuture<'a>;
}

pub struct ExecutorRegistry {
    executors: HashMap<String, Box<dyn TaskExecutor>>,
}

impl ExecutorRegistry {
    pub fn new() -> Self {
        Self {
            executors: HashMap::new(),
        }
    }

    pub fn register<E: TaskExecutor + 'static>(&mut self, executor: E) {
        self.executors
            .insert(executor.task_type().to_string(), Box::new(executor));
    }

    pub fn with_builtin() -> Self {
        let mut registry = Self::new();
        registry.register(ShellExecutor);
        registry.register(HttpExecutor);
        registry.register(SqlExecutor);
        registry
    }

    pub async fn execute(&self, task: &TaskSpec) -> ExecutionResult {
        let Some(executor) = self.executors.get(task.task_type.as_str()) else {
            return ExecutionResult::err(
                format!("unsupported task type: {}", task.task_type),
                None,
                None,
            );
        };
        executor.execute(task).await
    }
}

struct ShellExecutor;

impl TaskExecutor for ShellExecutor {
    fn task_type(&self) -> &'static str {
        runner_plugins::shell::plugin_name()
    }

    fn execute<'a>(&'a self, task: &'a TaskSpec) -> TaskFuture<'a> {
        Box::pin(async move {
            if let Err(err) = runner_plugins::shell::validate(task.cmd.as_deref()) {
                return ExecutionResult::err(format!("task '{}' {}", task.id, err), None, None);
            }
            let Some(cmd) = task.cmd.as_deref() else {
                return ExecutionResult::err(
                    format!("task '{}' missing shell cmd", task.id),
                    None,
                    None,
                );
            };
            match runner_infra::process::run_shell(cmd, task.timeout_ms).await {
                Ok(0) => ExecutionResult::ok(Some(0), None),
                Ok(code) => ExecutionResult::err(
                    format!("task '{}' exited with status {}", task.id, code),
                    Some(code),
                    None,
                ),
                Err(err) => ExecutionResult::err(
                    format!("task '{}' failed: {}", task.id, err),
                    None,
                    None,
                ),
            }
        })
    }
}

struct HttpExecutor;

impl TaskExecutor for HttpExecutor {
    fn task_type(&self) -> &'static str {
        runner_plugins::http::plugin_name()
    }

    fn execute<'a>(&'a self, task: &'a TaskSpec) -> TaskFuture<'a> {
        Box::pin(async move {
            if let Err(err) = runner_plugins::http::validate(task.method.as_deref(), task.url.as_deref()) {
                return ExecutionResult::err(format!("task '{}' {}", task.id, err), None, None);
            }
            let Some(method) = task.method.as_deref() else {
                return ExecutionResult::err(
                    format!("task '{}' missing http method", task.id),
                    None,
                    None,
                );
            };
            let Some(url) = task.url.as_deref() else {
                return ExecutionResult::err(
                    format!("task '{}' missing http url", task.id),
                    None,
                    None,
                );
            };

            let status = match runner_infra::http::request(method, url).await {
                Ok(status) => status,
                Err(err) => {
                    return ExecutionResult::err(
                        format!("task '{}' failed: {}", task.id, err),
                        None,
                        None,
                    );
                }
            };

            let accepted = task.expected_status.clone().unwrap_or_else(|| vec![200]);
            if accepted.contains(&status) {
                ExecutionResult::ok(None, Some(status))
            } else {
                ExecutionResult::err(
                    format!(
                        "task '{}' got unexpected status {}, expected {:?}",
                        task.id, status, accepted
                    ),
                    None,
                    Some(status),
                )
            }
        })
    }
}

struct SqlExecutor;

impl TaskExecutor for SqlExecutor {
    fn task_type(&self) -> &'static str {
        runner_plugins::sql::plugin_name()
    }

    fn execute<'a>(&'a self, task: &'a TaskSpec) -> TaskFuture<'a> {
        Box::pin(async move {
            if let Err(err) =
                runner_plugins::sql::validate(task.dsn.as_deref(), task.query.as_deref(), task.sql_file.as_deref())
            {
                return ExecutionResult::err(format!("task '{}' {}", task.id, err), None, None);
            }
            let Some(dsn) = task.dsn.as_deref() else {
                return ExecutionResult::err(
                    format!("task '{}' missing sql dsn", task.id),
                    None,
                    None,
                );
            };

            match runner_infra::sql::execute_batch(dsn, task.query.as_deref(), task.sql_file.as_deref())
                .await
            {
                Ok(()) => ExecutionResult::ok(None, None),
                Err(err) => ExecutionResult::err(
                    format!("task '{}' failed: {}", task.id, err),
                    None,
                    None,
                ),
            }
        })
    }
}
