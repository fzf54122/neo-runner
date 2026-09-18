use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(
    name = "neo-runner",
    about = "Agent completion gate: YAML loops, JSON evidence, exit 0 only when green"
)]
pub struct Args {
    #[arg(long)]
    pub version: bool,

    #[arg(short = 'f', long = "file", global = true)]
    pub file: Option<String>,

    #[arg(long, value_enum, default_value_t = OutputFormat::Text, global = true)]
    pub output: OutputFormat,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, clap::Args)]
pub struct InitArgs {
    /// Overwrite existing loop/skill files.
    #[arg(long)]
    pub force: bool,

    /// Also write project-level Claude and Codex SKILL.md files.
    #[arg(long)]
    pub skill: bool,
}

#[derive(Debug, Copy, Clone, ValueEnum, Eq, PartialEq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Run,
    Plan,
    Validate,
    /// Write .agents/loop.yaml in the current project. Does not install the binary.
    Init(InitArgs),
    Completion {
        #[arg(value_enum)]
        shell: CompletionShell,
    },
}

#[derive(Debug, Copy, Clone, ValueEnum, Eq, PartialEq)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

impl From<CompletionShell> for clap_complete::Shell {
    fn from(value: CompletionShell) -> Self {
        match value {
            CompletionShell::Bash => clap_complete::Shell::Bash,
            CompletionShell::Zsh => clap_complete::Shell::Zsh,
            CompletionShell::Fish => clap_complete::Shell::Fish,
            CompletionShell::PowerShell => clap_complete::Shell::PowerShell,
            CompletionShell::Elvish => clap_complete::Shell::Elvish,
        }
    }
}
