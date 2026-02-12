use crate::scheduler::build_batches;
use runner_core::domain::{JobSpec, RetrySpec, RunResult, TaskSpec};
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
    timeout_ms: Option<u64>,
    attempts: u32,
}

impl RunnableTask {
    fn from_task(job: &JobSpec, task: &TaskSpec) -> Self {
        Self {
            id: task.id.clone(),
            task_type: task.task_type.clone(),
            cmd: task.cmd.clone(),
            timeout_ms: task.timeout_ms,
            attempts: resolve_max_attempts(job, task),
        }
    }
}

async fn execute_task(task: RunnableTask) -> Result<(), String> {
    match task.task_type.as_str() {
        "shell" => {
            let cmd = task
                .cmd
                .as_deref()
                .ok_or_else(|| format!("task '{}' missing shell cmd", task.id))?;

            let mut last_error: Option<String> = None;
            for attempt in 1..=task.attempts {
                println!(
                    "execute shell task: {} (attempt {}/{})",
                    task.id, attempt, task.attempts
                );
                match runner_infra::process::run_shell(cmd, task.timeout_ms).await {
                    Ok(0) => {
                        last_error = None;
                        break;
                    }
                    Ok(code) => {
                        last_error = Some(format!("task '{}' exited with status {}", task.id, code));
                    }
                    Err(err) => {
                        last_error = Some(format!("task '{}' failed: {}", task.id, err));
                    }
                }
            }

            match last_error {
                Some(err) => Err(err),
                None => Ok(()),
            }
        }
        other => Err(format!("unsupported task type: {}", other)),
    }
}

async fn execute_batch(tasks: Vec<RunnableTask>, max_concurrency: usize) -> Vec<(String, Result<(), String>)> {
    let mut results: Vec<(String, Result<(), String>)> = Vec::new();
    let mut set: JoinSet<(String, Result<(), String>)> = JoinSet::new();
    let mut idx = 0usize;
    let concurrency = max_concurrency.max(1);

    while idx < tasks.len() || !set.is_empty() {
        while set.len() < concurrency && idx < tasks.len() {
            let task = tasks[idx].clone();
            idx += 1;
            set.spawn(async move {
                let id = task.id.clone();
                let outcome = execute_task(task).await;
                (id, outcome)
            });
        }

        if let Some(joined) = set.join_next().await {
            match joined {
                Ok(result) => results.push(result),
                Err(err) => results.push((
                    "<join-error>".to_string(),
                    Err(format!("task join error: {}", err)),
                )),
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
    })
}

pub async fn run_job(job: &JobSpec) -> Result<RunResult, String> {
    let batches = build_batches(job)?;
    let mut failed = 0usize;

    for batch in batches {
        let runnable: Vec<RunnableTask> = batch
            .into_iter()
            .map(|task| RunnableTask::from_task(job, task))
            .collect();

        let outcomes = execute_batch(runnable, job.max_concurrency).await;
        for (_id, outcome) in outcomes {
            if let Err(err) = outcome {
                if job.fail_fast {
                    return Err(err);
                }
                eprintln!("{err}");
                failed += 1;
            }
        }
    }

    Ok(RunResult {
        success: failed == 0,
        total: job.tasks.len(),
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
            depends_on: deps.iter().map(|v| v.to_string()).collect(),
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
    }
}
