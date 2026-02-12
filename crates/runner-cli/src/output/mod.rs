use crate::cli::OutputFormat;

pub fn print_result(success: bool, total: usize, format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("success={success} total={total}"),
        OutputFormat::Json => {
            let payload = serde_json::json!({
                "success": success,
                "total": total,
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
