use tokio::process::Command;
use tokio::time::{timeout, Duration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

impl ShellOutput {
    pub fn combined_excerpt(&self) -> String {
        let mut parts = Vec::new();
        if !self.stderr.trim().is_empty() {
            parts.push(self.stderr.trim().to_string());
        }
        if !self.stdout.trim().is_empty() {
            parts.push(self.stdout.trim().to_string());
        }
        parts.join("\n")
    }
}

pub async fn run_shell(cmd: &str, timeout_ms: Option<u64>) -> Result<i32, String> {
    Ok(run_shell_captured(cmd, timeout_ms).await?.exit_code)
}

pub async fn run_shell_captured(cmd: &str, timeout_ms: Option<u64>) -> Result<ShellOutput, String> {
    if cmd.trim().is_empty() {
        return Err("shell command cannot be empty".to_string());
    }

    let mut command = Command::new("sh");
    command.arg("-c").arg(cmd);

    let output = if let Some(ms) = timeout_ms {
        timeout(Duration::from_millis(ms), command.output())
            .await
            .map_err(|_| format!("command timed out after {}ms", ms))?
            .map_err(|e| format!("failed to execute shell command: {}", e))?
    } else {
        command
            .output()
            .await
            .map_err(|e| format!("failed to execute shell command: {}", e))?
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    match output.status.code() {
        Some(exit_code) => Ok(ShellOutput {
            exit_code,
            stdout,
            stderr,
        }),
        None => Err("process terminated by signal".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_shell_captured_keeps_stdout_and_exit_code() {
        let output = run_shell_captured("echo hello-evidence", None)
            .await
            .expect("command should run");
        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("hello-evidence"));
    }

    #[tokio::test]
    async fn run_shell_captured_keeps_stderr_on_failure() {
        let output = run_shell_captured("echo boom-evidence >&2; exit 7", None)
            .await
            .expect("command should run");
        assert_eq!(output.exit_code, 7);
        assert!(output.stderr.contains("boom-evidence"));
        assert!(output.combined_excerpt().contains("boom-evidence"));
    }
}
