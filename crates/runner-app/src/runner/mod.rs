use crate::scheduler::build_batches;
use runner_core::domain::{JobSpec, RetrySpec, RunEvent, RunResult, TaskRunResult, TaskSpec};
use std::time::Instant;
use tokio::task::JoinSet;

fn resolve_max_attempts(job: &JobSpec, task: &runner_core::domain::TaskSpec) -> u32 {
    task.retry
        .as_ref()
        .map(|r| r.max_attempts)
        .unwrap_or(job.default_retry.max_attempts)
}

#[derive(Debug, Clone)]
struct RunnableTask {
    id: String,
    task_type: String,
    cmd: Option<String>,
    method: Option<String>,
    url: Option<String>,
    expected_status: Option<Vec<u16>>,
    timeout_ms: Option<u64>,
    attempts: u32,
}

#[derive(Debug, Clone)]
struct AttemptResult {
    success: bool,
    error: Option<String>,
    duration_ms: u128,
    exit_code: Option<i32>,
    status_code: Option<u16>,
}

impl RunnableTask {
    fn from_task(job: &JobSpec, task: &TaskSpec) -> Self {
        Self {
            id: task.id.clone(),
            task_type: task.task_type.clone(),
            cmd: task.cmd.clone(),
            method: task.method.clone(),
            url: task.url.clone(),
            expected_status: task.expected_status.clone(),
            timeout_ms: task.timeout_ms,
            attempts: resolve_max_attempts(job, task),
        }
    }
}

async fn execute_task(task: RunnableTask) -> AttemptResult {
    let started = Instant::now();

    match task.task_type.as_str() {
        "shell" => {
            let cmd = task
                .cmd
                .as_deref();

            let Some(cmd) = cmd else {
                return AttemptResult {
                    success: false,
                    error: Some(format!("task '{}' missing shell cmd", task.id)),
                    duration_ms: started.elapsed().as_millis(),
                    exit_code: None,
                    status_code: None,
                };
            };

            match runner_infra::process::run_shell(cmd, task.timeout_ms).await {
                Ok(0) => AttemptResult {
                    success: true,
                    error: None,
                    duration_ms: started.elapsed().as_millis(),
                    exit_code: Some(0),
                    status_code: None,
                },
                Ok(code) => AttemptResult {
                    success: false,
                    error: Some(format!("task '{}' exited with status {}", task.id, code)),
                    duration_ms: started.elapsed().as_millis(),
                    exit_code: Some(code),
                    status_code: None,
                },
                Err(err) => AttemptResult {
                    success: false,
                    error: Some(format!("task '{}' failed: {}", task.id, err)),
                    duration_ms: started.elapsed().as_millis(),
                    exit_code: None,
                    status_code: None,
                },
            }
        }
        "http" => {
            let Some(method) = task.method.as_deref() else {
                return AttemptResult {
                    success: false,
                    error: Some(format!("task '{}' missing http method", task.id)),
                    duration_ms: started.elapsed().as_millis(),
                    exit_code: None,
                    status_code: None,
                };
            };

            let Some(url) = task.url.as_deref() else {
                return AttemptResult {
                    success: false,
                    error: Some(format!("task '{}' missing http url", task.id)),
                    duration_ms: started.elapsed().as_millis(),
                    exit_code: None,
                    status_code: None,
                };
            };

            let status = match runner_infra::http::request(method, url).await {
                Ok(status) => status,
                Err(err) => {
                    return AttemptResult {
                        success: false,
                        error: Some(format!("task '{}' failed: {}", task.id, err)),
                        duration_ms: started.elapsed().as_millis(),
                        exit_code: None,
                        status_code: None,
                    };
                }
            };

            let accepted: Vec<u16> = task
                .expected_status
                .clone()
                .unwrap_or_else(|| vec![200]);
            if accepted.contains(&status) {
                AttemptResult {
                    success: true,
                    error: None,
                    duration_ms: started.elapsed().as_millis(),
                    exit_code: None,
                    status_code: Some(status),
                }
            } else {
                AttemptResult {
                    success: false,
                    error: Some(format!(
                        "task '{}' got unexpected status {}, expected {:?}",
                        task.id, status, accepted
                    )),
                    duration_ms: started.elapsed().as_millis(),
                    exit_code: None,
                    status_code: Some(status),
                }
            }
        }
        other => AttemptResult {
            success: false,
            error: Some(format!("unsupported task type: {}", other)),
            duration_ms: started.elapsed().as_millis(),
            exit_code: None,
            status_code: None,
        },
    }
}

async fn execute_task_with_retry(task: RunnableTask) -> TaskRunResult {
    let mut last: Option<AttemptResult> = None;
    for attempt in 1..=task.attempts {
        let once = RunnableTask {
            attempts: 1,
            ..task.clone()
        };
        let result = execute_task(once).await;
        if result.success {
            return TaskRunResult {
                id: task.id,
                success: true,
                attempts: attempt,
                error: None,
                duration_ms: result.duration_ms,
                exit_code: result.exit_code,
                status_code: result.status_code,
            };
        }
        last = Some(result);
    }

    let fallback = last.unwrap_or(AttemptResult {
        success: false,
        error: Some("task failed without attempt result".to_string()),
        duration_ms: 0,
        exit_code: None,
        status_code: None,
    });

    TaskRunResult {
        id: task.id,
        success: false,
        attempts: task.attempts,
        error: fallback.error,
        duration_ms: fallback.duration_ms,
        exit_code: fallback.exit_code,
        status_code: fallback.status_code,
    }
}

async fn execute_batch(tasks: Vec<RunnableTask>, max_concurrency: usize) -> Vec<TaskRunResult> {
    let mut results: Vec<TaskRunResult> = Vec::new();
    let mut set: JoinSet<TaskRunResult> = JoinSet::new();
    let mut idx = 0usize;
    let concurrency = max_concurrency.max(1);

    while idx < tasks.len() || !set.is_empty() {
        while set.len() < concurrency && idx < tasks.len() {
            let task = tasks[idx].clone();
            idx += 1;
            set.spawn(async move { execute_task_with_retry(task).await });
        }

        if let Some(joined) = set.join_next().await {
            match joined {
                Ok(result) => results.push(result),
                Err(err) => results.push(TaskRunResult {
                    id: "<join-error>".to_string(),
                    success: false,
                    attempts: 1,
                    error: Some(format!("task join error: {}", err)),
                    duration_ms: 0,
                    exit_code: None,
                    status_code: None,
                }),
            }
        }
    }

    results
}

pub async fn run() -> RunResult {
    let job = JobSpec {
        name: "default".to_string(),
        fail_fast: true,
        max_concurrency: 1,
        tasks: Vec::new(),
        default_timeout_ms: None,
        default_retry: RetrySpec { max_attempts: 1 },
    };

    run_job(&job).await.unwrap_or_else(|_| RunResult {
        success: false,
        total: 0,
        failed: 0,
        tasks: Vec::new(),
        events: Vec::new(),
    })
}

pub async fn run_job(job: &JobSpec) -> Result<RunResult, String> {
    let batches = build_batches(job)?;
    let mut failed = 0usize;
    let mut task_results: Vec<TaskRunResult> = Vec::new();
    let mut events: Vec<RunEvent> = Vec::new();
    events.push(RunEvent {
        kind: "run_started".to_string(),
        task_id: None,
    });

    for batch in batches {
        for task in &batch {
            events.push(RunEvent {
                kind: "task_started".to_string(),
                task_id: Some(task.id.clone()),
            });
        }

        let runnable: Vec<RunnableTask> = batch
            .into_iter()
            .map(|task| RunnableTask::from_task(job, task))
            .collect();

        let outcomes = execute_batch(runnable, job.max_concurrency).await;
        for outcome in outcomes {
            events.push(RunEvent {
                kind: "task_finished".to_string(),
                task_id: Some(outcome.id.clone()),
            });

            if !outcome.success {
                if job.fail_fast {
                    events.push(RunEvent {
                        kind: "run_finished".to_string(),
                        task_id: None,
                    });
                    return Err(
                        outcome
                            .error
                            .clone()
                            .unwrap_or_else(|| "task failed without error".to_string()),
                    );
                }
                if let Some(err) = &outcome.error {
                    eprintln!("{err}");
                }
                failed += 1;
            }
            task_results.push(outcome);
        }
    }

    events.push(RunEvent {
        kind: "run_finished".to_string(),
        task_id: None,
    });

    Ok(RunResult {
        success: failed == 0,
        total: job.tasks.len(),
        failed,
        tasks: task_results,
        events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_job(fail_fast: bool, tasks: Vec<TaskSpec>) -> JobSpec {
        JobSpec {
            name: "demo".to_string(),
            fail_fast,
            max_concurrency: 2,
            tasks,
            default_timeout_ms: None,
            default_retry: RetrySpec { max_attempts: 1 },
        }
    }

    fn mk_task(id: &str, task_type: &str, deps: &[&str]) -> TaskSpec {
        TaskSpec {
            id: id.to_string(),
            task_type: task_type.to_string(),
            cmd: None,
            method: None,
            url: None,
            expected_status: None,
            depends_on: deps.iter().map(|v| v.to_string()).collect(),
            timeout_ms: None,
            retry: None,
        }
    }

    fn mk_http_task(id: &str, method: &str, url: &str) -> TaskSpec {
        TaskSpec {
            id: id.to_string(),
            task_type: "http".to_string(),
            cmd: None,
            method: Some(method.to_string()),
            url: Some(url.to_string()),
            expected_status: Some(vec![200]),
            depends_on: Vec::new(),
            timeout_ms: None,
            retry: None,
        }
    }

    #[tokio::test]
    async fn run_job_fail_fast_returns_err() {
        let job = mk_job(true, vec![mk_task("a", "unknown", &[])]);
        let err = run_job(&job).await.unwrap_err();
        assert!(err.contains("unsupported task type"));
    }

    #[tokio::test]
    async fn run_job_non_fail_fast_collects_errors() {
        let job = mk_job(
            false,
            vec![mk_task("a", "unknown", &[]), mk_task("b", "unknown", &[])],
        );
        let result = run_job(&job).await.expect("run should not hard fail");
        assert!(!result.success);
        assert_eq!(result.total, 2);
        assert_eq!(result.failed, 2);
        assert_eq!(result.tasks.len(), 2);
        assert!(result.tasks.iter().all(|t| t.duration_ms <= 1_000));
        assert!(result.events.iter().any(|e| e.kind == "run_started"));
        assert!(result.events.iter().any(|e| e.kind == "run_finished"));
    }

    #[tokio::test]
    async fn run_job_http_task_success() {
        let job = mk_job(true, vec![mk_http_task("health", "GET", "https://example.com")]);
        let result = run_job(&job).await.expect("http task should succeed");
        assert!(result.success);
        assert_eq!(result.total, 1);
        assert_eq!(result.failed, 0);
        assert_eq!(result.tasks[0].status_code, Some(200));
    }
}
