use crate::cli::OutputFormat;
use runner_core::domain::RunResult;

pub fn print_result(result: &RunResult, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!(
                "success={} total={} failed={} events={} batches={}",
                result.success,
                result.total,
                result.failed,
                result.events.len(),
                result.batches.len()
            )
        }
        OutputFormat::Json => {
            let payload = serde_json::json!({
                "success": result.success,
                "total": result.total,
                "failed": result.failed,
                "tasks": result.tasks,
                "events": result.events,
                "batches": result.batches,
                "retry_distribution": result.retry_distribution,
                "failure_groups": result.failure_groups,
            });
            println!("{}", payload);
        }
    }
}

pub fn print_plan(task_ids: &[String], format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            for id in task_ids {
                println!("{id}");
            }
        }
        OutputFormat::Json => {
            let payload = serde_json::json!({ "tasks": task_ids });
            println!("{}", payload);
        }
    }
}

pub fn print_validate_ok(task_total: usize, format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("config valid, tasks={task_total}"),
        OutputFormat::Json => {
            let payload = serde_json::json!({
                "valid": true,
                "tasks": task_total,
            });
            println!("{}", payload);
        }
    }
}
