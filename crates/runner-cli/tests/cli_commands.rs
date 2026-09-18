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

#[test]
fn init_writes_loop_and_refuses_without_force() {
    let dir = tempfile::tempdir().expect("temp dir");
    let first = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .args(["init", "--output", "json"])
        .current_dir(dir.path())
        .output()
        .expect("init");
    assert!(
        first.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&first.stderr)
    );
    let v = parse_json_line(&String::from_utf8_lossy(&first.stdout));
    assert_eq!(v["ok"], true);
    assert_eq!(v["written"][0], ".agents/loop.yaml");
    assert_eq!(v["preset"], "generic");
    assert_eq!(v["source"], "detected");
    let loop_path = dir.path().join(".agents/loop.yaml");
    assert!(loop_path.is_file());
    assert!(std::fs::read_to_string(&loop_path)
        .expect("read loop")
        .contains("echo fmt-ok"));

    let second = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .args(["init", "--output", "json"])
        .current_dir(dir.path())
        .output()
        .expect("init again");
    assert_eq!(second.status.code(), Some(1));
    let v = parse_json_line(&String::from_utf8_lossy(&second.stdout));
    assert_eq!(v["ok"], false);
}

#[test]
fn init_force_and_skill() {
    let dir = tempfile::tempdir().expect("temp dir");
    std::fs::create_dir_all(dir.path().join(".agents")).expect("agents");
    std::fs::write(dir.path().join(".agents/loop.yaml"), "stale\n").expect("stale");

    let output = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .args(["init", "--force", "--skill", "--output", "json"])
        .current_dir(dir.path())
        .output()
        .expect("init --force --skill");
    assert!(output.status.success());
    let v = parse_json_line(&String::from_utf8_lossy(&output.stdout));
    assert_eq!(v["ok"], true);
    assert_eq!(v["written"].as_array().unwrap().len(), 3);
    assert!(dir
        .path()
        .join(".claude/skills/neo-runner/SKILL.md")
        .is_file());
    assert!(dir
        .path()
        .join(".agents/skills/neo-runner/SKILL.md")
        .is_file());
    assert!(
        std::fs::read_to_string(dir.path().join(".agents/loop.yaml"))
            .expect("loop")
            .contains("echo test-ok")
    );
}

#[test]
fn init_preset_flag_writes_rust_loop() {
    let dir = tempfile::tempdir().expect("temp dir");
    let output = Command::new(env!("CARGO_BIN_EXE_neo-runner"))
        .args(["init", "--preset", "rust", "--output", "json"])
        .current_dir(dir.path())
        .output()
        .expect("init --preset rust");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v = parse_json_line(&String::from_utf8_lossy(&output.stdout));
    assert_eq!(v["ok"], true);
    assert_eq!(v["preset"], "rust");
    assert_eq!(v["source"], "flag");
    let body = std::fs::read_to_string(dir.path().join(".agents/loop.yaml")).expect("loop");
    assert!(body.contains("cargo fmt --all -- --check"));
    assert!(body.contains("cargo test --workspace"));
    assert!(!body.contains("echo fmt-ok"));
}

#[test]
fn install_sh_binary_only_does_not_write_cwd_loop() {
    let dir = tempfile::tempdir().expect("temp dir");
    let prefix = dir.path().join("bin");
    let home = dir.path().join("home");
    std::fs::create_dir_all(&home).expect("home");

    let dummy_dir = dir.path().join("dummy");
    std::fs::create_dir_all(&dummy_dir).expect("dummy");
    let dummy_bin = dummy_dir.join("neo-runner-linux-x86_64");
    std::fs::write(&dummy_bin, "#!/bin/sh\necho neo-runner 0.2.0-test\n").expect("dummy bin");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&dummy_bin).expect("meta").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&dummy_bin, perms).expect("chmod");
    }

    let tar_path = dir.path().join("neo-runner-linux-x86_64.tar.gz");
    let tar_status = Command::new("tar")
        .args([
            "-czf",
            tar_path.to_str().unwrap(),
            "-C",
            dummy_dir.to_str().unwrap(),
            "neo-runner-linux-x86_64",
        ])
        .status()
        .expect("tar");
    assert!(tar_status.success());

    let script = format!("{}/../../scripts/install.sh", env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("bash")
        .arg(&script)
        .current_dir(dir.path())
        .env("HOME", &home)
        .env("NEO_RUNNER_PREFIX", &prefix)
        .env("NEO_RUNNER_SKIP_PLUGINS", "1")
        .env("NEO_RUNNER_SKIP_PATH", "1")
        .env("NEO_RUNNER_DOWNLOAD_URL", &tar_path)
        .output()
        .expect("install.sh");

    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(prefix.join("neo-runner").is_file());
    assert!(!dir.path().join(".agents/loop.yaml").exists());
    let version = Command::new(prefix.join("neo-runner"))
        .arg("--version")
        .output()
        .expect("version");
    assert!(String::from_utf8_lossy(&version.stdout).contains("0.2.0-test"));
}

fn uninstall_script() -> String {
    format!("{}/../../scripts/uninstall.sh", env!("CARGO_MANIFEST_DIR"))
}

fn write_unix_file(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent");
    }
    std::fs::write(path, contents).expect("write file");
}

struct UninstallFixture {
    dir: tempfile::TempDir,
    home: std::path::PathBuf,
    prefix: std::path::PathBuf,
    cargo_home: std::path::PathBuf,
}

impl UninstallFixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp dir");
        let home = dir.path().join("home");
        let prefix = dir.path().join("bin");
        let cargo_home = home.join(".cargo");
        std::fs::create_dir_all(&home).expect("home");
        std::fs::create_dir_all(&prefix).expect("prefix");
        std::fs::create_dir_all(cargo_home.join("bin")).expect("cargo bin");

        write_unix_file(&prefix.join("neo-runner"), "#!/bin/sh\necho neo-runner\n");
        write_unix_file(
            &cargo_home.join("bin/neo-runner"),
            "#!/bin/sh\necho cargo-neo-runner\n",
        );
        write_unix_file(&home.join(".claude/skills/neo-runner/SKILL.md"), "skill\n");
        write_unix_file(&home.join(".codex/skills/neo-runner/SKILL.md"), "skill\n");
        write_unix_file(&home.join(".agents/skills/neo-runner/SKILL.md"), "skill\n");
        write_unix_file(
            &home.join(".zprofile"),
            "export PATH=\"$HOME/.local/bin:$PATH\" # neo-runner\nkeep-this-line\n",
        );
        write_unix_file(
            &home.join(".claude/plugins/cache/neo-runner/neo-runner/0.2.0/SKILL.md"),
            "plugin\n",
        );
        write_unix_file(
            &home.join(".claude/plugins/data/neo-runner-neo-runner/.keep"),
            "",
        );
        write_unix_file(
            &home.join(".claude/plugins/marketplaces/neo-runner/README.md"),
            "marketplace\n",
        );
        write_unix_file(
            &home.join(".claude/plugins/installed_plugins.json"),
            r#"{
  "version": 2,
  "plugins": {
    "keep-me@official": [{"scope": "user"}],
    "neo-runner@fzf54122": [{"scope": "user"}],
    "neo-runner@neo-runner": [{"scope": "user"}]
  }
}
"#,
        );
        write_unix_file(
            &home.join(".claude/plugins/known_marketplaces.json"),
            r#"{
  "keep-me": {"source": {"repo": "keep/me"}},
  "neo-runner": {"source": {"repo": "fzf54122/neo-runner"}}
}
"#,
        );
        write_unix_file(&dir.path().join(".agents/loop.yaml"), "keep-project-loop\n");

        Self {
            dir,
            home,
            prefix,
            cargo_home,
        }
    }

    fn run(&self, args: &[&str]) -> std::process::Output {
        Command::new("bash")
            .arg(uninstall_script())
            .args(args)
            .current_dir(self.dir.path())
            .env("HOME", &self.home)
            .env("CARGO_HOME", &self.cargo_home)
            .env("NEO_RUNNER_PREFIX", &self.prefix)
            .env("NEO_RUNNER_SKIP_CLAUDE", "1")
            .env("NEO_RUNNER_SKIP_DEB", "1")
            .env("PATH", prepend_path(&self.prefix))
            .output()
            .expect("uninstall.sh")
    }
}

#[test]
fn uninstall_sh_removes_global_files_and_keeps_project_loop() {
    let fx = UninstallFixture::new();
    let output = fx.run(&[]);
    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );

    assert!(!fx.prefix.join("neo-runner").exists());
    assert!(!fx.cargo_home.join("bin/neo-runner").exists());
    assert!(!fx.home.join(".claude/skills/neo-runner").exists());
    assert!(!fx.home.join(".codex/skills/neo-runner").exists());
    assert!(!fx.home.join(".agents/skills/neo-runner").exists());
    assert!(!fx.home.join(".claude/plugins/cache/neo-runner").exists());
    assert!(!fx
        .home
        .join(".claude/plugins/data/neo-runner-neo-runner")
        .exists());
    assert!(!fx
        .home
        .join(".claude/plugins/marketplaces/neo-runner")
        .exists());
    assert!(fx.dir.path().join(".agents/loop.yaml").is_file());

    let zprofile = std::fs::read_to_string(fx.home.join(".zprofile")).expect("zprofile");
    assert!(!zprofile.contains("# neo-runner"));
    assert!(zprofile.contains("keep-this-line"));

    let plugins: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fx.home.join(".claude/plugins/installed_plugins.json"))
            .expect("plugins json"),
    )
    .expect("plugins parse");
    assert!(plugins["plugins"].get("keep-me@official").is_some());
    assert!(plugins["plugins"].get("neo-runner@fzf54122").is_none());
    assert!(plugins["plugins"].get("neo-runner@neo-runner").is_none());

    let markets: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(fx.home.join(".claude/plugins/known_marketplaces.json"))
            .expect("markets json"),
    )
    .expect("markets parse");
    assert!(markets.get("keep-me").is_some());
    assert!(markets.get("neo-runner").is_none());
}

#[test]
fn uninstall_sh_dry_run_does_not_delete() {
    let fx = UninstallFixture::new();
    let output = fx.run(&["--dry-run"]);
    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("would remove"));
    assert!(fx.prefix.join("neo-runner").is_file());
    assert!(fx.home.join(".claude/skills/neo-runner/SKILL.md").is_file());
    assert!(fx.dir.path().join(".agents/loop.yaml").is_file());
}

#[test]
fn uninstall_sh_is_idempotent() {
    let fx = UninstallFixture::new();
    let first = fx.run(&[]);
    assert!(first.status.success());
    let second = fx.run(&[]);
    assert!(
        second.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&second.stderr),
        String::from_utf8_lossy(&second.stdout)
    );
}

#[test]
fn uninstall_sh_keep_plugins_leaves_skills() {
    let fx = UninstallFixture::new();
    let output = fx.run(&["--keep-plugins"]);
    assert!(output.status.success());
    assert!(!fx.prefix.join("neo-runner").exists());
    assert!(fx.home.join(".claude/skills/neo-runner/SKILL.md").is_file());
    assert!(fx.home.join(".codex/skills/neo-runner/SKILL.md").is_file());
}

#[test]
fn claude_plugin_install_id_matches_marketplace_name() {
    let root = format!("{}/../..", env!("CARGO_MANIFEST_DIR"));
    let marketplace = std::fs::read_to_string(format!("{root}/.claude-plugin/marketplace.json"))
        .expect("marketplace.json");
    let plugin =
        std::fs::read_to_string(format!("{root}/.claude-plugin/plugin.json")).expect("plugin.json");
    let install =
        std::fs::read_to_string(format!("{root}/scripts/install.sh")).expect("install.sh");

    assert!(
        marketplace.contains("\"name\": \"neo-runner\""),
        "marketplace.json name must stay neo-runner; @ right-hand side is this field, not the GitHub user"
    );
    assert!(
        plugin.contains("\"name\": \"neo-runner\""),
        "plugin.json name must stay neo-runner"
    );
    assert!(
        !plugin.contains("\"hooks\":"),
        "plugin.json must not redeclare hooks/hooks.json; Claude loads it automatically"
    );
    assert!(
        install.contains("plugin_id=\"neo-runner@neo-runner\""),
        "install.sh must install neo-runner@neo-runner"
    );
    assert!(
        !install.contains("legacy_plugin_id") && !install.contains("neo-runner@fzf54122"),
        "install.sh must not probe the old fzf54122 marketplace name"
    );
    assert!(
        install.contains("claude plugin marketplace update"),
        "install.sh must update an already-added marketplace; add is a no-op"
    );
    assert!(
        install.contains("claude plugin uninstall"),
        "install.sh must uninstall before reinstall so a same-version cache still refreshes"
    );
}

fn install_script() -> String {
    format!("{}/../../scripts/install.sh", env!("CARGO_MANIFEST_DIR"))
}

fn detect_install_asset(os: &str, arch: &str) -> std::process::Output {
    Command::new("bash")
        .arg(install_script())
        .env("NEO_RUNNER_DETECT_ONLY", "1")
        .env("NEO_RUNNER_OS", os)
        .env("NEO_RUNNER_ARCH", arch)
        .output()
        .expect("install.sh detect")
}

#[test]
fn install_sh_selects_linux_x86_64_release_asset() {
    let output = detect_install_asset("Linux", "x86_64");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("asset=neo-runner-linux-x86_64.tar.gz"));
    assert!(stdout.contains("installed=neo-runner"));
}

#[test]
fn install_sh_selects_windows_exe_release_asset() {
    let output = detect_install_asset("MINGW64_NT-10.0", "x86_64");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("asset=neo-runner.exe"));
    assert!(stdout.contains("installed=neo-runner.exe"));
}

#[test]
fn install_sh_rejects_unreleased_platform() {
    let output = detect_install_asset("Darwin", "arm64");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unsupported platform"));
}
