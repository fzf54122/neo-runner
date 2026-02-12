use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "neo-runner", about = "A pluggable job runner")]
pub struct Args {
    #[arg(long)]
    pub version: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Run {
        #[arg(short = 'f', long = "file", default_value = "examples/demo.yaml")]
        file: String,
    },
    Plan {
        #[arg(short = 'f', long = "file", default_value = "examples/demo.yaml")]
        file: String,
    },
    Validate {
        #[arg(short = 'f', long = "file", default_value = "examples/demo.yaml")]
        file: String,
    },
}
