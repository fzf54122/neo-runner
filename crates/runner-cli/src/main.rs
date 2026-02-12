mod cli;
mod output;

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = cli::Args::parse();
    if args.version {
        println!("neo-runner 0.1.0");
        return;
    }

    let result = runner_app::runner::run().await;
    output::print_result(result.success, result.total);
}
