mod cli;
mod output;

use clap::CommandFactory;
use clap::Parser;
use std::path::Path;
use std::process;

fn resolve_file(file: Option<String>) -> Result<String, String> {
    if let Some(file) = file {
        return Ok(file);
    }

    let default_file = "examples/demo.yaml";
    if Path::new(default_file).exists() {
        Ok(default_file.to_string())
    } else {
        Err("missing config file: please pass -f/--file <path>".to_string())
    }
}

fn load_job_or_exit(file: &str) -> runner_core::domain::JobSpec {
    match runner_infra::config_loader::load_yaml(file) {
        Ok(job) => job,
        Err(err) => {
            eprintln!("load config failed [{}]: {err}", err.code());
            process::exit(2);
        }
    }
}

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    let output_format = args.output;

    if args.version {
        println!("neo-runner 0.1.0");
        return;
    }

    if args.command.is_none() && args.file.is_none() {
        let _ = cli::Args::command().print_help();
        println!();
        return;
    }

    match args.command.unwrap_or(cli::Command::Run) {
        cli::Command::Validate => {
            let file = match resolve_file(args.file.clone()) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("{err}");
                    process::exit(2);
                }
            };
            let job = load_job_or_exit(&file);
            output::print_validate_ok(job.tasks.len(), output_format);
        }
        cli::Command::Plan => {
            let file = match resolve_file(args.file.clone()) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("{err}");
                    process::exit(2);
                }
            };
            let job = load_job_or_exit(&file);
            let plan = match runner_app::scheduler::build_plan(&job) {
                Ok(plan) => plan,
                Err(err) => {
                    eprintln!("plan failed [{}]: {err}", err.code());
                    process::exit(1);
                }
            };
            let ids: Vec<String> = plan.iter().map(|t| t.id.clone()).collect();
            output::print_plan(&ids, output_format);
        }
        cli::Command::Run => {
            let file = match resolve_file(args.file.clone()) {
                Ok(file) => file,
                Err(err) => {
                    eprintln!("{err}");
                    process::exit(2);
                }
            };
            let job = load_job_or_exit(&file);
            match runner_app::runner::run_job(&job).await {
                Ok(result) => {
                    output::print_result(&result, output_format);
                    if !result.success {
                        process::exit(1);
                    }
                }
                Err(err) => {
                    eprintln!("run failed [{}]: {err}", err.code());
                    process::exit(1);
                }
            }
        }
        cli::Command::Completion { shell } => {
            let mut cmd = cli::Args::command();
            let bin_name = cmd.get_name().to_string();
            let generator: clap_complete::Shell = shell.into();
            clap_complete::generate(generator, &mut cmd, bin_name, &mut std::io::stdout());
        }
    }
}
