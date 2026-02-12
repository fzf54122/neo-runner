use crate::scheduler::build_plan;
use runner_core::domain::{JobSpec, RetrySpec, RunResult};

fn resolve_max_attempts(job: &JobSpec, task: &runner_core::domain::TaskSpec) -> u32 {
    task.retry
        .as_ref()
        .map(|r| r.max_attempts)
        .unwrap_or(job.default_retry.max_attempts)
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
    let plan = build_plan(job)?;
    let mut failed = 0usize;

    for task in plan {
        match task.task_type.as_str() {
            "shell" => {
                let cmd = task
                    .cmd
                    .as_deref()
                    .ok_or_else(|| format!("task '{}' missing shell cmd", task.id))?;

                let attempts = resolve_max_attempts(job, task);
                let mut last_error: Option<String> = None;

                for attempt in 1..=attempts {
                    println!("execute shell task: {} (attempt {}/{})", task.id, attempt, attempts);
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

                if let Some(err) = last_error {
                    if job.fail_fast {
                        return Err(err);
                    }
                    eprintln!("{err}");
                    failed += 1;
                }
            }
            other => {
                let err = format!("unsupported task type: {}", other);
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
