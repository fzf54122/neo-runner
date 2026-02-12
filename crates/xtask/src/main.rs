use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
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
    Release,
    Doctor {
        #[arg(long, help = "Also run cargo check --workspace")]
        with_check: bool,
    },
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
        Task::Release => run_release(args.dry_run),
        Task::Doctor { with_check } => run_doctor(args.dry_run, with_check),
    };

    if let Err(err) = result {
        eprintln!("[xtask] error: {err}");
        std::process::exit(1);
    }
}

fn run_doctor(dry_run: bool, with_check: bool) -> Result<(), String> {
    run_cmd(dry_run, "rustc", &["--version"])?;
    run_cmd(dry_run, "cargo", &["--version"])?;

    for path in [
        "Cargo.toml",
        "README.md",
        "docs/architecture.md",
        "examples/demo.yaml",
        "examples/demo-http.yaml",
        "examples/demo-sql.yaml",
        "examples/demo-all.yaml",
    ] {
        println!("[xtask] check path exists: {path}");
        if !dry_run && !Path::new(path).exists() {
            return Err(format!("required path not found: {path}"));
        }
    }

    if with_check {
        run_cmd(dry_run, "cargo", &["check", "--workspace"])?;
    }

    Ok(())
}

fn run_release(dry_run: bool) -> Result<(), String> {
    run_cmd(
        dry_run,
        "cargo",
        &["build", "-p", "runner-cli", "--release"],
    )?;

    let bin_path = Path::new("target/release/neo-runner");
    println!("[xtask] verify binary exists: {}", bin_path.display());
    if !dry_run && !bin_path.exists() {
        return Err(format!("release binary missing: {}", bin_path.display()));
    }

    run_cmd(dry_run, "target/release/neo-runner", &["--help"])?;

    let dist_dir = Path::new("dist");
    let package_version = env!("CARGO_PKG_VERSION");
    let deb_name = format!("neo-runner_{}_amd64.deb", package_version);
    let deb_path = format!("dist/{deb_name}");
    let checksum_path = format!("dist/{deb_name}.sha256");

    if dry_run {
        println!("[xtask] dpkg-deb --version");
        println!("[xtask] upx --version");
        println!("[xtask] upx -9 target/release/neo-runner");
        println!("[xtask] include completion: bash/zsh/fish");
        println!("[xtask] package: {deb_path}");
        println!("[xtask] checksum: {checksum_path}");
        return Ok(());
    }

    run_cmd(dry_run, "dpkg-deb", &["--version"])?;
    run_cmd(dry_run, "upx", &["--version"])?;
    run_cmd(dry_run, "upx", &["-9", "target/release/neo-runner"])?;

    if dist_dir.exists() {
        fs::remove_dir_all(dist_dir)
            .map_err(|e| format!("failed to clean dist directory: {}", e))?;
    }
    fs::create_dir_all(dist_dir).map_err(|e| format!("failed to create dist directory: {}", e))?;

    let package_root = dist_dir.join(format!("neo-runner_{}_amd64", package_version));
    let debian_dir = package_root.join("DEBIAN");
    let usr_bin_dir = package_root.join("usr/bin");
    let bash_completion_dir = package_root.join("usr/share/bash-completion/completions");
    let zsh_completion_dir = package_root.join("usr/share/zsh/vendor-completions");
    let fish_completion_dir = package_root.join("usr/share/fish/vendor_completions.d");

    fs::create_dir_all(&debian_dir)
        .map_err(|e| format!("failed to create DEBIAN directory: {}", e))?;
    fs::create_dir_all(&usr_bin_dir)
        .map_err(|e| format!("failed to create usr/bin directory: {}", e))?;
    fs::create_dir_all(&bash_completion_dir)
        .map_err(|e| format!("failed to create bash completion directory: {}", e))?;
    fs::create_dir_all(&zsh_completion_dir)
        .map_err(|e| format!("failed to create zsh completion directory: {}", e))?;
    fs::create_dir_all(&fish_completion_dir)
        .map_err(|e| format!("failed to create fish completion directory: {}", e))?;

    let control = format!(
        "Package: neo-runner\nVersion: {}\nSection: utils\nPriority: optional\nArchitecture: amd64\nMaintainer: neo-runner contributors\nDescription: A production-oriented Rust task orchestrator\n",
        package_version
    );
    fs::write(debian_dir.join("control"), control)
        .map_err(|e| format!("failed to write DEBIAN/control: {}", e))?;

    let staged_bin = usr_bin_dir.join("neo-runner");
    fs::copy(bin_path, &staged_bin)
        .map_err(|e| format!("failed to stage release binary: {}", e))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(&staged_bin, perms)
            .map_err(|e| format!("failed to set binary permissions: {}", e))?;
    }

    let bash_completion = generate_completion(bin_path, "bash")?;
    fs::write(bash_completion_dir.join("neo-runner"), bash_completion)
        .map_err(|e| format!("failed to write bash completion: {}", e))?;

    let zsh_completion = generate_completion(bin_path, "zsh")?;
    fs::write(zsh_completion_dir.join("_neo-runner"), zsh_completion)
        .map_err(|e| format!("failed to write zsh completion: {}", e))?;

    let fish_completion = generate_completion(bin_path, "fish")?;
    fs::write(fish_completion_dir.join("neo-runner.fish"), fish_completion)
        .map_err(|e| format!("failed to write fish completion: {}", e))?;

    run_cmd(
        dry_run,
        "dpkg-deb",
        &[
            "--uniform-compression",
            "-Zgzip",
            "--build",
            package_root
                .to_str()
                .ok_or_else(|| "invalid package root path".to_string())?,
            &deb_path,
        ],
    )?;

    run_cmd(
        dry_run,
        "sh",
        &["-c", &format!("sha256sum {deb_path} > {checksum_path}")],
    )?;

    println!("[xtask] release artifacts generated in ./dist");
    Ok(())
}

fn generate_completion(binary: &Path, shell: &str) -> Result<String, String> {
    let output = Command::new(binary)
        .args(["completion", shell])
        .output()
        .map_err(|e| format!("failed to generate {shell} completion: {}", e))?;

    ensure_success(
        output.status,
        &format!("{} completion {shell}", binary.display()),
    )?;

    String::from_utf8(output.stdout)
        .map_err(|e| format!("invalid utf8 in {shell} completion output: {}", e))
}
