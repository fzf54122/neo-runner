use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "neo-runner", about = "A pluggable job runner")]
pub struct Args {
    #[arg(long)]
    pub version: bool,

    #[arg(short = 'f', long = "file", global = true)]
    pub file: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Run,
    Plan,
    Validate,
}
