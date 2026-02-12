use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[arg(default_value = "help")]
    task: String,
}

fn main() {
    let args = Args::parse();
    println!("xtask: {}", args.task);
}
