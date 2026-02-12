use tokio::process::Command;
use tokio::time::{timeout, Duration};

pub async fn run_shell(cmd: &str, timeout_ms: Option<u64>) -> Result<i32, String> {
    if cmd.trim().is_empty() {
        return Err("shell command cannot be empty".to_string());
    }

    let mut command = Command::new("sh");
    command.arg("-c").arg(cmd);

    let status = if let Some(ms) = timeout_ms {
        timeout(Duration::from_millis(ms), command.status())
            .await
            .map_err(|_| format!("command timed out after {}ms", ms))?
            .map_err(|e| format!("failed to execute shell command: {}", e))?
    } else {
        command
            .status()
            .await
            .map_err(|e| format!("failed to execute shell command: {}", e))?
    };

    match status.code() {
        Some(code) => Ok(code),
        None => Err("process terminated by signal".to_string()),
    }
}
