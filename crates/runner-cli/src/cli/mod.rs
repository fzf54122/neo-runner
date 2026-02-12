use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "neo-runner", about = "A pluggable job runner")]
pub struct Args {
    #[arg(long)]
    pub version: bool,

    #[arg(short = 'f', long = "file", default_value = "examples/demo.yaml")]
    pub file: String,
}
