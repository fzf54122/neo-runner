use std::process::Command;

fn demo_config_path() -> String {
    format!("{}/../../examples/demo.yaml", env!("CARGO_MANIFEST_DIR"))
}

fn demo_sql_config_path() -> String {
    format!(
        "{}/../../examples/demo-sql.yaml",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
fn validate_command_works() {
    let output = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .args(["validate", "-f", &demo_config_path()])
        .output()
        .expect("failed to execute runner-cli validate");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("config valid"));
}

#[test]
fn plan_command_works() {
    let output = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .args(["plan", "-f", &demo_config_path()])
        .output()
        .expect("failed to execute runner-cli plan");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello"));
}

#[test]
fn validate_command_json_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .args(["validate", "-f", &demo_config_path(), "--output", "json"])
        .output()
        .expect("failed to execute runner-cli validate json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(v["valid"], true);
    assert_eq!(v["tasks"], 1);
}

#[test]
fn plan_command_json_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .args(["plan", "-f", &demo_config_path(), "--output", "json"])
        .output()
        .expect("failed to execute runner-cli plan json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("stdout should be json");
    assert_eq!(v["tasks"][0], "hello");
}

#[test]
fn run_command_json_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .args(["run", "-f", &demo_config_path(), "--output", "json"])
        .output()
        .expect("failed to execute runner-cli run json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_line = stdout
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .expect("stdout should contain json line");
    let v: serde_json::Value = serde_json::from_str(json_line).expect("json line should parse");
    assert_eq!(v["ok"], true);
    assert_eq!(v["success"], true);
    assert_eq!(v["total"], 1);
    assert_eq!(v["failed"], 0);
    assert!(v["duration_ms"].as_u64().is_some());
    assert!(v["failed_tasks"].as_array().unwrap().is_empty());
    assert!(v["evidence"].as_array().unwrap().is_empty());
    assert_eq!(v["tasks"][0]["id"], "hello");
    assert_eq!(v["tasks"][0]["success"], true);
    assert_eq!(v["tasks"][0]["exit_code"], 0);
    assert_eq!(v["tasks"][0]["status_code"], serde_json::Value::Null);
    assert!(v["tasks"][0]["duration_ms"].as_u64().is_some());
    assert!(v["events"].is_array());
    assert!(v["batches"].is_array());
    assert!(v["retry_distribution"].is_array());
    assert!(v["failure_groups"].is_array());
    let kinds: Vec<&str> = v["events"]
        .as_array()
        .expect("events should be array")
        .iter()
        .filter_map(|e| e["kind"].as_str())
        .collect();
    assert!(kinds.contains(&"run_started"));
    assert!(kinds.contains(&"run_finished"));
}

#[test]
fn run_sql_command_json_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .args(["run", "-f", &demo_sql_config_path(), "--output", "json"])
        .output()
        .expect("failed to execute runner-cli run sql json");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_line = stdout
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .expect("stdout should contain json line");
    let v: serde_json::Value = serde_json::from_str(json_line).expect("json line should parse");
    assert_eq!(v["success"], true);
    assert_eq!(v["tasks"][0]["id"], "import-users");
}

#[test]
fn completion_command_zsh_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .args(["completion", "zsh"])
        .output()
        .expect("failed to execute runner-cli completion zsh");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("#compdef neo-runner"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("validate"));
}

#[test]
fn no_args_prints_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .output()
        .expect("failed to execute runner-cli without args");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("neo-runner"));
}

fn write_temp_yaml(contents: &str) -> tempfile::NamedTempFile {
    let file = tempfile::Builder::new()
        .suffix(".yaml")
        .tempfile()
        .expect("temp yaml");
    std::fs::write(file.path(), contents).expect("write yaml");
    file
}

fn parse_json_line(stdout: &str) -> serde_json::Value {
    let json_line = stdout
        .lines()
        .rev()
        .find(|line| line.trim_start().starts_with('{'))
        .expect("stdout should contain json line");
    serde_json::from_str(json_line).expect("json line should parse")
}

#[test]
fn run_command_json_failure_exits_nonzero_with_evidence() {
    let yaml = write_temp_yaml(
        r#"
version: 1
job:
  name: agent-fail
  fail_fast: true
  tasks:
    - id: boom
      type: shell
      cmd: "echo boom-evidence >&2; exit 9"
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .args([
            "run",
            "-f",
            yaml.path().to_str().unwrap(),
            "--output",
            "json",
        ])
        .output()
        .expect("failed to execute runner-cli fail json");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v = parse_json_line(&stdout);
    assert_eq!(v["ok"], false);
    assert_eq!(v["failed_tasks"][0], "boom");
    assert_eq!(v["evidence"][0]["task"], "boom");
    assert_eq!(v["evidence"][0]["exit_code"], 9);
    assert!(v["evidence"][0]["excerpt"]
        .as_str()
        .unwrap()
        .contains("boom-evidence"));
}

#[test]
fn run_loop_example_json_is_green() {
    let loop_path = format!(
        "{}/../../examples/agent-loop.yaml",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .args(["run", "-f", &loop_path, "--output", "json"])
        .output()
        .expect("failed to execute agent loop example");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v = parse_json_line(&stdout);
    assert_eq!(v["ok"], true);
    assert!(v["failed_tasks"].as_array().unwrap().is_empty());
}

fn stop_gate_script() -> String {
    format!("{}/../../hooks/stop-gate.sh", env!("CARGO_MANIFEST_DIR"))
}

fn prepend_path(dir: &std::path::Path) -> std::ffi::OsString {
    let mut path = dir.as_os_str().to_os_string();
    if let Some(orig) = std::env::var_os("PATH") {
        path.push(":");
        path.push(orig);
    }
    path
}

#[test]
fn stop_gate_skips_when_loop_file_missing() {
    let dir = tempfile::tempdir().expect("temp dir");
    let output = Command::new("/bin/sh")
        .arg(stop_gate_script())
        .current_dir(dir.path())
        .output()
        .expect("failed to execute stop-gate.sh");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("skip gate"));
}

#[test]
fn stop_gate_fails_when_loop_exists_but_binary_missing() {
    let dir = tempfile::tempdir().expect("temp dir");
    let agents = dir.path().join(".agents");
    std::fs::create_dir_all(&agents).expect("create .agents");
    std::fs::write(agents.join("loop.yaml"), "version: 1\n").expect("write loop.yaml");

    let empty_path = dir.path().join("empty-bin");
    std::fs::create_dir_all(&empty_path).expect("create empty PATH dir");

    let output = Command::new("/bin/sh")
        .arg(stop_gate_script())
        .current_dir(dir.path())
        .env("PATH", &empty_path)
        .output()
        .expect("failed to execute stop-gate.sh");

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("binary not found"));
}

#[test]
fn stop_gate_runs_loop_when_binary_is_on_path() {
    let dir = tempfile::tempdir().expect("temp dir");
    let agents = dir.path().join(".agents");
    std::fs::create_dir_all(&agents).expect("create .agents");
    std::fs::write(
        agents.join("loop.yaml"),
        r#"
version: 1
job:
  name: gate-ok
  fail_fast: true
  tasks:
    - id: ping
      type: shell
      cmd: "echo ping-ok"
"#,
    )
    .expect("write loop.yaml");

    let bin = std::path::Path::new(env!("CARGO_BIN_EXE_neo-runner"));
    let bin_dir = bin.parent().expect("bin dir");

    let output = Command::new("/bin/sh")
        .arg(stop_gate_script())
        .current_dir(dir.path())
        .env("PATH", prepend_path(bin_dir))
        .output()
        .expect("failed to execute stop-gate.sh");

    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let v = parse_json_line(&stdout);
    assert_eq!(v["ok"], true);
}
