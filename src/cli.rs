use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "jenkins")]
#[command(about = "A CLI tool for interacting with Jenkins", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Manage Jenkins host configurations")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    #[command(about = "Trigger a build for a Jenkins job")]
    Build {
        #[arg(help = "Name of the Jenkins job (optional - will prompt to select if not provided)")]
        job_name: Option<String>,

        #[arg(short = 'f', long, help = "Follow the build logs in real-time after triggering")]
        follow: bool,
    },

    #[command(about = "Check the status of a Jenkins job or build")]
    Status {
        #[arg(help = "Name of the Jenkins job (optional - will prompt to select if not provided)")]
        job_name: Option<String>,

        #[arg(short, long, help = "Specific build number to check")]
        build: Option<i32>,
    },

    #[command(about = "View console logs for a build")]
    Logs {
        #[arg(help = "Name of the Jenkins job (optional - will prompt to select if not provided)")]
        job_name: Option<String>,

        #[arg(short, long, help = "Specific build number (defaults to last build)")]
        build: Option<i32>,

        #[arg(short = 'f', long, help = "Follow the build logs in real-time")]
        follow: bool,
    },

    #[command(about = "Open a Jenkins job or build in the browser")]
    Open {
        #[arg(help = "Name of the Jenkins job (optional - will prompt to select if not provided)")]
        job_name: Option<String>,

        #[arg(short, long, help = "Specific build number to open")]
        build: Option<i32>,
    },

    #[command(about = "Generate shell completion scripts")]
    Completion {
        #[arg(value_enum, help = "Shell type to generate completion for")]
        shell: Shell,
    },

    #[command(about = "Manage job aliases")]
    Alias {
        #[command(subcommand)]
        action: AliasAction,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    #[command(about = "Add a new Jenkins host")]
    Add,

    #[command(about = "List all configured Jenkins hosts")]
    List,

    #[command(about = "Remove a Jenkins host")]
    Remove,
}

#[derive(Subcommand)]
pub enum AliasAction {
    #[command(about = "Add a job alias")]
    Add {
        #[arg(help = "Alias name (optional - will prompt to enter if not provided)")]
        alias: Option<String>,

        #[arg(help = "Actual job name (optional - will prompt to select if not provided)")]
        job_name: Option<String>,
    },

    #[command(about = "List all job aliases")]
    List,

    #[command(about = "Remove a job alias")]
    Remove {
        #[arg(help = "Alias to remove (optional - will prompt to select if not provided)")]
        alias: Option<String>,
    },
}
