use std::process::Command;

fn demo_config_path() -> String {
    format!("{}/../../examples/demo.yaml", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn validate_command_works() {
    let output = Command::new(env!("CARGO_BIN_EXE_runner-cli"))
        .args(["validate", "-f", &demo_config_path()])
        .output()
        .expect("failed to execute runner-cli validate");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("config valid"));
}

#[test]
fn plan_command_works() {
    let output = Command::new(env!("CARGO_BIN_EXE_runner-cli"))
        .args(["plan", "-f", &demo_config_path()])
        .output()
        .expect("failed to execute runner-cli plan");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello"));
}
