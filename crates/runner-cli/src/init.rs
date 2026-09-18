// @Time    : 2026/9/17 16:10
// @Author  : fzf
// @FileName: init.rs
// @Software: RustRover

use crate::cli::OutputFormat;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;

const LOOP_TEMPLATE: &str = include_str!("../../../examples/agent-loop.yaml");
const CLAUDE_SKILL: &str = include_str!("../../../skills/neo-runner/SKILL.md");
const CODEX_SKILL: &str = include_str!("../../../.agents/skills/neo-runner/SKILL.md");

const LOOP_PATH: &str = ".agents/loop.yaml";
const CLAUDE_SKILL_PATH: &str = ".claude/skills/neo-runner/SKILL.md";
const CODEX_SKILL_PATH: &str = ".agents/skills/neo-runner/SKILL.md";

pub struct InitOptions {
    pub force: bool,
    pub skill: bool,
    pub format: OutputFormat,
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
    match init(&root, opts.force, opts.skill) {
        Ok(written) => print_ok(&written, opts.format),
        Err(err) => {
            print_err(&err, opts.format);
            process::exit(1);
        }
    }
}

fn init(root: &Path, force: bool, skill: bool) -> Result<Vec<String>, InitError> {
    let mut written = Vec::new();
    write_file(root, LOOP_PATH, LOOP_TEMPLATE, force)?;
    written.push(LOOP_PATH.to_string());

    if skill {
        write_file(root, CLAUDE_SKILL_PATH, CLAUDE_SKILL, force)?;
        written.push(CLAUDE_SKILL_PATH.to_string());
        write_file(root, CODEX_SKILL_PATH, CODEX_SKILL, force)?;
        written.push(CODEX_SKILL_PATH.to_string());
    }

    Ok(written)
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

fn print_ok(written: &[String], format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            for path in written {
                println!("wrote {path}");
            }
            println!(
                "Replace echo placeholders in .agents/loop.yaml with this project's real gates."
            );
        }
        OutputFormat::Json => {
            let payload = serde_json::json!({
                "ok": true,
                "written": written,
            });
            println!("{payload}");
        }
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

    #[test]
    fn writes_loop_file() {
        let dir = tempfile::tempdir().expect("temp dir");
        let written = init(dir.path(), false, false).expect("init");
        assert_eq!(written, vec![LOOP_PATH]);
        let body = fs::read_to_string(dir.path().join(LOOP_PATH)).expect("read loop");
        assert!(body.contains("agent-loop"));
        assert!(body.contains("echo fmt-ok"));
    }

    #[test]
    fn refuses_existing_without_force() {
        let dir = tempfile::tempdir().expect("temp dir");
        init(dir.path(), false, false).expect("first init");
        match init(dir.path(), false, false) {
            Err(InitError::Exists(path)) => {
                assert_eq!(path, Path::new(LOOP_PATH));
            }
            other => panic!("expected Exists, got {other:?}"),
        }
    }

    #[test]
    fn force_overwrites() {
        let dir = tempfile::tempdir().expect("temp dir");
        init(dir.path(), false, false).expect("first init");
        fs::write(dir.path().join(LOOP_PATH), "stale\n").expect("stale");
        let written = init(dir.path(), true, false).expect("force");
        assert_eq!(written, vec![LOOP_PATH]);
        let body = fs::read_to_string(dir.path().join(LOOP_PATH)).expect("read loop");
        assert!(body.contains("echo test-ok"));
    }

    #[test]
    fn skill_flag_writes_project_skills() {
        let dir = tempfile::tempdir().expect("temp dir");
        let written = init(dir.path(), false, true).expect("init skill");
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
