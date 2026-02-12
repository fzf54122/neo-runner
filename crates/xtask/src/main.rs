use clap::{Parser, Subcommand};
use std::process::{Command, ExitStatus};

#[derive(Debug, Parser)]
#[command(name = "xtask", about = "Repository engineering tasks")]
struct Args {
    #[arg(long, help = "Print commands without executing")]
    dry_run: bool,
    #[command(subcommand)]
    command: Task,
}

#[derive(Debug, Subcommand)]
enum Task {
    Fmt,
    Clippy,
    Test,
    Check,
    Ci,
    BuildRelease,
}

fn run_cmd(dry_run: bool, program: &str, args: &[&str]) -> Result<(), String> {
    let display = format!("{} {}", program, args.join(" ")).trim().to_string();
    println!("[xtask] {display}");

    if dry_run {
        return Ok(());
    }

    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| format!("failed to spawn '{}': {}", display, e))?;

    ensure_success(status, &display)
}

fn ensure_success(status: ExitStatus, command: &str) -> Result<(), String> {
    if status.success() {
        Ok(())
    } else {
        Err(format!("command failed: {command}"))
    }
}

fn main() {
    let args = Args::parse();
    let result = match args.command {
        Task::Fmt => run_cmd(args.dry_run, "cargo", &["fmt", "--all"]),
        Task::Clippy => run_cmd(
            args.dry_run,
            "cargo",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        ),
        Task::Test => run_cmd(args.dry_run, "cargo", &["test", "--workspace"]),
        Task::Check => run_cmd(args.dry_run, "cargo", &["check", "--workspace"]),
        Task::Ci => run_cmd(args.dry_run, "cargo", &["fmt", "--all", "--check"])
            .and_then(|_| {
                run_cmd(
                    args.dry_run,
                    "cargo",
                    &[
                        "clippy",
                        "--workspace",
                        "--all-targets",
                        "--",
                        "-D",
                        "warnings",
                    ],
                )
            })
            .and_then(|_| run_cmd(args.dry_run, "cargo", &["test", "--workspace"])),
        Task::BuildRelease => run_cmd(
            args.dry_run,
            "cargo",
            &["build", "-p", "runner-cli", "--release"],
        ),
    };

    if let Err(err) = result {
        eprintln!("[xtask] error: {err}");
        std::process::exit(1);
    }
}
