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
    assert_eq!(v["success"], true);
    assert_eq!(v["total"], 1);
    assert_eq!(v["failed"], 0);
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
