mod cli;
mod output;

use clap::Parser;
use std::process;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    if args.version {
        println!("neo-runner 0.1.0");
        return;
    }

    let job = match runner_infra::config_loader::load_yaml(&args.file) {
        Ok(job) => job,
        Err(err) => {
            eprintln!("load config failed: {err}");
            process::exit(2);
        }
    };

    match runner_app::runner::run_job(&job).await {
        Ok(result) => {
            output::print_result(result.success, result.total);
        }
        Err(err) => {
            eprintln!("run failed: {err}");
            process::exit(1);
        }
    }
}
