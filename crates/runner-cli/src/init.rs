// @Time    : 2026/9/17 16:10
// @Author  : fzf
// @FileName: init.rs
// @Software: RustRover

use crate::cli::{InitPreset, OutputFormat};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

const GENERIC_TEMPLATE: &str = include_str!("../../../examples/agent-loop.yaml");
const RUST_TEMPLATE: &str = include_str!("../../../examples/loop-rust.yaml");
const GO_TEMPLATE: &str = include_str!("../../../examples/loop-go.yaml");
const PYTHON_TEMPLATE: &str = include_str!("../../../examples/loop-python.yaml");
const NODE_TEMPLATE: &str = include_str!("../../../examples/loop-node.yaml");
const CLAUDE_SKILL: &str = include_str!("../../../skills/neo-runner/SKILL.md");
const CODEX_SKILL: &str = include_str!("../../../.agents/skills/neo-runner/SKILL.md");

const LOOP_PATH: &str = ".agents/loop.yaml";
const CLAUDE_SKILL_PATH: &str = ".claude/skills/neo-runner/SKILL.md";
const CODEX_SKILL_PATH: &str = ".agents/skills/neo-runner/SKILL.md";

pub struct InitOptions {
    pub force: bool,
    pub skill: bool,
    pub preset: Option<InitPreset>,
    pub format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectedPreset {
    preset: InitPreset,
    source: &'static str,
    marker: Option<&'static str>,
    package_manager: Option<&'static str>,
}

#[derive(Debug)]
enum InitError {
    Exists(PathBuf),
    Io { path: PathBuf, source: io::Error },
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::Exists(path) => write!(
                f,
                "{} already exists; pass --force to overwrite",
                path.display()
            ),
            InitError::Io { path, source } => {
                write!(f, "failed to write {}: {source}", path.display())
            }
        }
    }
}

pub fn run(opts: InitOptions) {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match init(&root, opts.force, opts.skill, opts.preset) {
        Ok((written, selected)) => print_ok(&written, &selected, opts.format),
        Err(err) => {
            print_err(&err, opts.format);
            process::exit(1);
        }
    }
}

fn init(
    root: &Path,
    force: bool,
    skill: bool,
    preset: Option<InitPreset>,
) -> Result<(Vec<String>, SelectedPreset), InitError> {
    let selected = resolve_preset(root, preset);
    let contents = render_loop(root, selected.preset);
    let mut written = Vec::new();
    write_file(root, LOOP_PATH, &contents, force)?;
    written.push(LOOP_PATH.to_string());

    if skill {
        write_file(root, CLAUDE_SKILL_PATH, CLAUDE_SKILL, force)?;
        written.push(CLAUDE_SKILL_PATH.to_string());
        write_file(root, CODEX_SKILL_PATH, CODEX_SKILL, force)?;
        written.push(CODEX_SKILL_PATH.to_string());
    }

    Ok((written, selected))
}

fn resolve_preset(root: &Path, preset: Option<InitPreset>) -> SelectedPreset {
    if let Some(preset) = preset {
        let mut selected = SelectedPreset {
            preset,
            source: "flag",
            marker: None,
            package_manager: None,
        };
        if preset == InitPreset::Node {
            selected.package_manager = Some(detect_node_package_manager(root));
        }
        selected
    } else {
        detect_preset(root)
    }
}

fn detect_preset(root: &Path) -> SelectedPreset {
    if root.join("Cargo.toml").is_file() {
        return SelectedPreset {
            preset: InitPreset::Rust,
            source: "detected",
            marker: Some("Cargo.toml"),
            package_manager: None,
        };
    }
    if root.join("go.mod").is_file() {
        return SelectedPreset {
            preset: InitPreset::Go,
            source: "detected",
            marker: Some("go.mod"),
            package_manager: None,
        };
    }
    if root.join("pyproject.toml").is_file() || root.join("uv.lock").is_file() {
        let marker = if root.join("pyproject.toml").is_file() {
            "pyproject.toml"
        } else {
            "uv.lock"
        };
        return SelectedPreset {
            preset: InitPreset::Python,
            source: "detected",
            marker: Some(marker),
            package_manager: None,
        };
    }
    if root.join("package.json").is_file() {
        return SelectedPreset {
            preset: InitPreset::Node,
            source: "detected",
            marker: Some("package.json"),
            package_manager: Some(detect_node_package_manager(root)),
        };
    }
    SelectedPreset {
        preset: InitPreset::Generic,
        source: "detected",
        marker: None,
        package_manager: None,
    }
}

fn detect_node_package_manager(root: &Path) -> &'static str {
    if root.join("pnpm-lock.yaml").is_file() {
        "pnpm"
    } else if root.join("yarn.lock").is_file() {
        "yarn"
    } else if root.join("bun.lock").is_file() || root.join("bun.lockb").is_file() {
        "bun"
    } else {
        "npm"
    }
}

fn render_loop(root: &Path, preset: InitPreset) -> String {
    match preset {
        InitPreset::Generic => GENERIC_TEMPLATE.to_string(),
        InitPreset::Rust => RUST_TEMPLATE.to_string(),
        InitPreset::Go => GO_TEMPLATE.to_string(),
        InitPreset::Python => PYTHON_TEMPLATE.replace("{pytest_cmd}", pytest_cmd(root)),
        InitPreset::Node => NODE_TEMPLATE.replace("{pm}", detect_node_package_manager(root)),
    }
}

fn pytest_cmd(root: &Path) -> &'static str {
    if root.join("uv.lock").is_file() {
        "uv run pytest"
    } else {
        "python -m pytest"
    }
}

fn write_file(root: &Path, rel: &str, contents: &str, force: bool) -> Result<(), InitError> {
    let path = root.join(rel);
    if path.exists() && !force {
        return Err(InitError::Exists(PathBuf::from(rel)));
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| InitError::Io {
            path: PathBuf::from(rel),
            source,
        })?;
    }
    fs::write(&path, contents).map_err(|source| InitError::Io {
        path: PathBuf::from(rel),
        source,
    })?;
    Ok(())
}

fn print_ok(written: &[String], selected: &SelectedPreset, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            for path in written {
                println!("wrote {path}");
            }
            println!("preset: {}", format_preset_line(selected));
            if let Some(pm) = selected.package_manager {
                println!("package manager: {pm}");
            }
            println!(
                "Edit .agents/loop.yaml if these gates are not this project's Definition of Done."
            );
        }
        OutputFormat::Json => {
            let mut payload = serde_json::json!({
                "ok": true,
                "written": written,
                "preset": selected.preset.as_str(),
                "source": selected.source,
            });
            if let Some(pm) = selected.package_manager {
                payload["package_manager"] = serde_json::Value::String(pm.to_string());
            }
            println!("{payload}");
        }
    }
}

fn format_preset_line(selected: &SelectedPreset) -> String {
    match (selected.source, selected.marker) {
        ("flag", _) => format!("{} (flag)", selected.preset.as_str()),
        ("detected", Some(marker)) => {
            format!("{} (detected from {marker})", selected.preset.as_str())
        }
        _ => format!("{} (detected)", selected.preset.as_str()),
    }
}

fn print_err(err: &InitError, format: OutputFormat) {
    match format {
        OutputFormat::Text => eprintln!("{err}"),
        OutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": false,
                "error": err.to_string(),
            });
            println!("{payload}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn init_default(root: &Path) -> Result<(Vec<String>, SelectedPreset), InitError> {
        init(root, false, false, None)
    }

    #[test]
    fn writes_generic_loop_when_no_markers() {
        let dir = tempfile::tempdir().expect("temp dir");
        let (written, selected) = init_default(dir.path()).expect("init");
        assert_eq!(written, vec![LOOP_PATH]);
        assert_eq!(selected.preset, InitPreset::Generic);
        assert_eq!(selected.source, "detected");
        let body = fs::read_to_string(dir.path().join(LOOP_PATH)).expect("read loop");
        assert!(body.contains("agent-loop"));
        assert!(body.contains("echo fmt-ok"));
    }

    #[test]
    fn detects_go_from_go_mod() {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::write(dir.path().join("go.mod"), "module demo\n").expect("go.mod");
        let (_, selected) = init_default(dir.path()).expect("init");
        assert_eq!(selected.preset, InitPreset::Go);
        assert_eq!(selected.marker, Some("go.mod"));
        let body = fs::read_to_string(dir.path().join(LOOP_PATH)).expect("read loop");
        assert!(body.contains("go test ./..."));
        assert!(!body.contains("echo fmt-ok"));
    }

    #[test]
    fn detects_rust_from_cargo_toml() {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .expect("cargo");
        let (_, selected) = init_default(dir.path()).expect("init");
        assert_eq!(selected.preset, InitPreset::Rust);
        assert_eq!(selected.marker, Some("Cargo.toml"));
        let body = fs::read_to_string(dir.path().join(LOOP_PATH)).expect("read loop");
        assert!(body.contains("cargo fmt --all -- --check"));
        assert!(body.contains("cargo test --workspace"));
        assert!(!body.contains("echo fmt-ok"));
    }

    #[test]
    fn preset_flag_overrides_detected_rust() {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .expect("cargo");
        let (_, selected) = init(dir.path(), false, false, Some(InitPreset::Python)).expect("init");
        assert_eq!(selected.preset, InitPreset::Python);
        assert_eq!(selected.source, "flag");
        let body = fs::read_to_string(dir.path().join(LOOP_PATH)).expect("read loop");
        assert!(body.contains("python -m pytest"));
        assert!(!body.contains("cargo test"));
    }

    #[test]
    fn detects_node_pnpm_from_lockfile() {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::write(dir.path().join("package.json"), "{}\n").expect("pkg");
        fs::write(
            dir.path().join("pnpm-lock.yaml"),
            "lockfileVersion: '9.0'\n",
        )
        .expect("lock");
        let (_, selected) = init_default(dir.path()).expect("init");
        assert_eq!(selected.preset, InitPreset::Node);
        assert_eq!(selected.package_manager, Some("pnpm"));
        let body = fs::read_to_string(dir.path().join(LOOP_PATH)).expect("read loop");
        assert!(body.contains("pnpm test"));
        assert!(!body.contains("{pm}"));
    }

    #[test]
    fn detects_node_npm_without_lockfile() {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::write(dir.path().join("package.json"), "{}\n").expect("pkg");
        let (_, selected) = init_default(dir.path()).expect("init");
        assert_eq!(selected.preset, InitPreset::Node);
        assert_eq!(selected.package_manager, Some("npm"));
        let body = fs::read_to_string(dir.path().join(LOOP_PATH)).expect("read loop");
        assert!(body.contains("npm test"));
        assert!(!body.contains("{pm}"));
    }

    #[test]
    fn detects_python_uv_pytest() {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::write(
            dir.path().join("pyproject.toml"),
            "[project]\nname = \"demo\"\n",
        )
        .expect("pyproject");
        fs::write(dir.path().join("uv.lock"), "version = 1\n").expect("uv.lock");
        let (_, selected) = init_default(dir.path()).expect("init");
        assert_eq!(selected.preset, InitPreset::Python);
        let body = fs::read_to_string(dir.path().join(LOOP_PATH)).expect("read loop");
        assert!(body.contains("uv run pytest"));
        assert!(!body.contains("{pytest_cmd}"));
    }

    #[test]
    fn cargo_toml_wins_over_package_json() {
        let dir = tempfile::tempdir().expect("temp dir");
        fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .expect("cargo");
        fs::write(dir.path().join("package.json"), "{}\n").expect("pkg");
        let (_, selected) = init_default(dir.path()).expect("init");
        assert_eq!(selected.preset, InitPreset::Rust);
        let body = fs::read_to_string(dir.path().join(LOOP_PATH)).expect("read loop");
        assert!(body.contains("cargo test --workspace"));
    }

    #[test]
    fn refuses_existing_without_force() {
        let dir = tempfile::tempdir().expect("temp dir");
        init_default(dir.path()).expect("first init");
        match init_default(dir.path()) {
            Err(InitError::Exists(path)) => {
                assert_eq!(path, Path::new(LOOP_PATH));
            }
            other => panic!("expected Exists, got {other:?}"),
        }
    }

    #[test]
    fn force_overwrites() {
        let dir = tempfile::tempdir().expect("temp dir");
        init_default(dir.path()).expect("first init");
        fs::write(dir.path().join(LOOP_PATH), "stale\n").expect("stale");
        let (written, _) = init(dir.path(), true, false, None).expect("force");
        assert_eq!(written, vec![LOOP_PATH]);
        let body = fs::read_to_string(dir.path().join(LOOP_PATH)).expect("read loop");
        assert!(body.contains("echo test-ok"));
    }

    #[test]
    fn skill_flag_writes_project_skills() {
        let dir = tempfile::tempdir().expect("temp dir");
        let (written, _) = init(dir.path(), false, true, None).expect("init skill");
        assert_eq!(
            written,
            vec![LOOP_PATH, CLAUDE_SKILL_PATH, CODEX_SKILL_PATH]
        );
        assert!(fs::read_to_string(dir.path().join(CLAUDE_SKILL_PATH))
            .expect("claude skill")
            .contains("neo-runner"));
        assert!(fs::read_to_string(dir.path().join(CODEX_SKILL_PATH))
            .expect("codex skill")
            .contains("SKILL.md"));
    }
}
