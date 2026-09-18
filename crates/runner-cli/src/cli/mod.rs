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

    /// Loop preset. Auto-detected from the project root when omitted.
    #[arg(long, value_enum)]
    pub preset: Option<InitPreset>,
}

#[derive(Debug, Copy, Clone, ValueEnum, Eq, PartialEq)]
#[value(rename_all = "kebab-case")]
pub enum InitPreset {
    Generic,
    Rust,
    Go,
    Python,
    Node,
}

impl InitPreset {
    pub fn as_str(self) -> &'static str {
        match self {
            InitPreset::Generic => "generic",
            InitPreset::Rust => "rust",
            InitPreset::Go => "go",
            InitPreset::Python => "python",
            InitPreset::Node => "node",
        }
    }
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
