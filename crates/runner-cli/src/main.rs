mod cli;
mod output;

use clap::Parser;
use std::process;

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
    let file = args
        .file
        .unwrap_or_else(|| "examples/demo.yaml".to_string());
    let output_format = args.output;

    if args.version {
        println!("neo-runner 0.1.0");
        return;
    }

    match args.command.unwrap_or(cli::Command::Run) {
        cli::Command::Validate => {
            let job = load_job_or_exit(&file);
            output::print_validate_ok(job.tasks.len(), output_format);
        }
        cli::Command::Plan => {
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
            let job = load_job_or_exit(&file);
            match runner_app::runner::run_job(&job).await {
                Ok(result) => {
                    output::print_result(&result, output_format);
                }
                Err(err) => {
                    eprintln!("run failed [{}]: {err}", err.code());
                    process::exit(1);
                }
            }
        }
    }
}
