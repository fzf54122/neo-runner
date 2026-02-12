use crate::eventbus::{EventBus, InMemoryEventCollector};
use crate::executor::{ExecutionResult, ExecutorRegistry};
use crate::scheduler::build_batches;
use runner_core::domain::{
    BatchSummary, FailureGroup, JobSpec, RetryDistributionItem, RetrySpec, RunEvent, RunResult,
    TaskRunResult, TaskSpec,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::task::JoinSet;

fn resolve_max_attempts(job: &JobSpec, task: &TaskSpec) -> u32 {
    task.retry
        .as_ref()
        .map(|r| r.max_attempts)
        .unwrap_or(job.default_retry.max_attempts)
}

#[derive(Debug, Clone)]
struct RunnableTask {
    spec: TaskSpec,
    attempts: u32,
}

impl RunnableTask {
    fn from_task(job: &JobSpec, task: &TaskSpec) -> Self {
        Self {
            spec: task.clone(),
            attempts: resolve_max_attempts(job, task),
        }
    }
}

async fn execute_task_with_retry(registry: Arc<ExecutorRegistry>, task: RunnableTask) -> TaskRunResult {
    let mut last: Option<(ExecutionResult, u128)> = None;

    for attempt in 1..=task.attempts {
        let started = Instant::now();
        let result = registry.execute(&task.spec).await;
        let elapsed = started.elapsed().as_millis();

        if result.success {
            return TaskRunResult {
                id: task.spec.id,
                success: true,
                attempts: attempt,
                error: None,
                duration_ms: elapsed,
                exit_code: result.exit_code,
                status_code: result.status_code,
            };
        }

        last = Some((result, elapsed));
    }

    let (fallback, elapsed) = last.unwrap_or((
        ExecutionResult::err("task failed without result", None, None),
        0,
    ));

    TaskRunResult {
        id: task.spec.id,
        success: false,
        attempts: task.attempts,
        error: fallback.error,
        duration_ms: elapsed,
        exit_code: fallback.exit_code,
        status_code: fallback.status_code,
    }
}

async fn execute_batch(
    tasks: Vec<RunnableTask>,
    max_concurrency: usize,
    registry: Arc<ExecutorRegistry>,
) -> Vec<TaskRunResult> {
    let mut results: Vec<TaskRunResult> = Vec::new();
    let mut set: JoinSet<TaskRunResult> = JoinSet::new();
    let mut idx = 0usize;
    let concurrency = max_concurrency.max(1);

    while idx < tasks.len() || !set.is_empty() {
        while set.len() < concurrency && idx < tasks.len() {
            let task = tasks[idx].clone();
            let reg = registry.clone();
            idx += 1;
            set.spawn(async move { execute_task_with_retry(reg, task).await });
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

fn build_retry_distribution(tasks: &[TaskRunResult]) -> Vec<RetryDistributionItem> {
    let mut map: BTreeMap<u32, usize> = BTreeMap::new();
    for t in tasks {
        *map.entry(t.attempts).or_insert(0) += 1;
    }
    map.into_iter()
        .map(|(attempts, count)| RetryDistributionItem { attempts, count })
        .collect()
}

fn build_failure_groups(tasks: &[TaskRunResult]) -> Vec<FailureGroup> {
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for t in tasks {
        if !t.success {
            let key = t
                .error
                .clone()
                .unwrap_or_else(|| "unknown failure".to_string());
            map.entry(key).or_default().push(t.id.clone());
        }
    }

    map.into_iter()
        .map(|(reason, task_ids)| FailureGroup {
            count: task_ids.len(),
            reason,
            task_ids,
        })
        .collect()
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
        batches: Vec::new(),
        retry_distribution: Vec::new(),
        failure_groups: Vec::new(),
    })
}

pub async fn run_job(job: &JobSpec) -> Result<RunResult, String> {
    let registry = Arc::new(ExecutorRegistry::with_builtin());
    run_job_with_registry(job, registry).await
}

pub async fn run_job_with_registry(
    job: &JobSpec,
    registry: Arc<ExecutorRegistry>,
) -> Result<RunResult, String> {
    let batches = build_batches(job)?;
    let mut failed = 0usize;
    let mut task_results: Vec<TaskRunResult> = Vec::new();
    let mut batch_summaries: Vec<BatchSummary> = Vec::new();

    let mut bus = EventBus::new();
    let collector = InMemoryEventCollector::new();
    let probe = collector.clone();
    bus.subscribe(collector);

    bus.publish(&RunEvent {
        kind: "run_started".to_string(),
        task_id: None,
    });

    for (batch_index, batch) in batches.into_iter().enumerate() {
        let started = Instant::now();

        for task in &batch {
            bus.publish(&RunEvent {
                kind: "task_started".to_string(),
                task_id: Some(task.id.clone()),
            });
        }

        let runnable: Vec<RunnableTask> = batch
            .into_iter()
            .map(|task| RunnableTask::from_task(job, task))
            .collect();
        let outcomes = execute_batch(runnable, job.max_concurrency, registry.clone()).await;
        let batch_total = outcomes.len();

        let mut batch_failed = 0usize;
        for outcome in outcomes {
            bus.publish(&RunEvent {
                kind: "task_finished".to_string(),
                task_id: Some(outcome.id.clone()),
            });

            if !outcome.success {
                batch_failed += 1;
                if job.fail_fast {
                    bus.publish(&RunEvent {
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

        batch_summaries.push(BatchSummary {
            batch_index,
            total: batch_total,
            failed: batch_failed,
            duration_ms: started.elapsed().as_millis(),
        });
    }

    bus.publish(&RunEvent {
        kind: "run_finished".to_string(),
        task_id: None,
    });

    let retry_distribution = build_retry_distribution(&task_results);
    let failure_groups = build_failure_groups(&task_results);
    let events = probe.snapshot();

    Ok(RunResult {
        success: failed == 0,
        total: job.tasks.len(),
        failed,
        tasks: task_results,
        events,
        batches: batch_summaries,
        retry_distribution,
        failure_groups,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_job(fail_fast: bool, tasks: Vec<TaskSpec>) -> JobSpec {
        mk_job_with_concurrency(fail_fast, 2, tasks)
    }

    fn mk_job_with_concurrency(fail_fast: bool, max_concurrency: usize, tasks: Vec<TaskSpec>) -> JobSpec {
        JobSpec {
            name: "demo".to_string(),
            fail_fast,
            max_concurrency,
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
            dsn: None,
            query: None,
            sql_file: None,
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
            dsn: None,
            query: None,
            sql_file: None,
            depends_on: Vec::new(),
            timeout_ms: None,
            retry: None,
        }
    }

    fn mk_sql_task(id: &str, dsn: &str, query: &str) -> TaskSpec {
        TaskSpec {
            id: id.to_string(),
            task_type: "sql".to_string(),
            cmd: None,
            method: None,
            url: None,
            expected_status: None,
            dsn: Some(dsn.to_string()),
            query: Some(query.to_string()),
            sql_file: None,
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
        assert!(!result.failure_groups.is_empty());
        assert!(!result.retry_distribution.is_empty());
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

    #[tokio::test]
    async fn run_job_sql_task_success() {
        let job = mk_job(
            true,
            vec![mk_sql_task(
                "import",
                "sqlite::memory:",
                "CREATE TABLE IF NOT EXISTS users(id INTEGER); INSERT INTO users(id) VALUES (1);",
            )],
        );
        let result = run_job(&job).await.expect("sql task should succeed");
        assert!(result.success);
        assert_eq!(result.total, 1);
        assert_eq!(result.failed, 0);
    }

    #[tokio::test]
    async fn run_job_http_tasks_execute_concurrently() {
        let tasks = vec![
            mk_http_task("h1", "GET", "mock://delay/120/status/200"),
            mk_http_task("h2", "GET", "mock://delay/120/status/200"),
        ];

        let seq_job = mk_job_with_concurrency(true, 1, tasks.clone());
        let conc_job = mk_job_with_concurrency(true, 2, tasks);

        let seq_started = Instant::now();
        let seq = run_job(&seq_job).await.expect("seq should succeed");
        let seq_elapsed = seq_started.elapsed();

        let conc_started = Instant::now();
        let conc = run_job(&conc_job).await.expect("concurrent should succeed");
        let conc_elapsed = conc_started.elapsed();

        assert!(seq.success && conc.success);
        assert!(conc_elapsed < seq_elapsed);
    }
}
