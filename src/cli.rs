use clap::{Parser, Subcommand};

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

        #[arg(short = 'j', long, help = "Jenkins host name to use (uses current if not specified)")]
        jenkins: Option<String>,
    },

    #[command(about = "Check the status of a Jenkins job or build")]
    Status {
        #[arg(help = "Name of the Jenkins job (optional - will prompt to select if not provided)")]
        job_name: Option<String>,

        #[arg(short, long, help = "Specific build number to check")]
        build: Option<i32>,

        #[arg(short = 'j', long, help = "Jenkins host name to use (uses current if not specified)")]
        jenkins: Option<String>,
    },

    #[command(about = "View console logs for a build")]
    Logs {
        #[arg(help = "Name of the Jenkins job (optional - will prompt to select if not provided)")]
        job_name: Option<String>,

        #[arg(short, long, help = "Specific build number (defaults to last build)")]
        build: Option<i32>,

        #[arg(short = 'j', long, help = "Jenkins host name to use (uses current if not specified)")]
        jenkins: Option<String>,
    },

    #[command(about = "Open a Jenkins job or build in the browser")]
    Open {
        #[arg(help = "Name of the Jenkins job (optional - will prompt to select if not provided)")]
        job_name: Option<String>,

        #[arg(short, long, help = "Specific build number to open")]
        build: Option<i32>,

        #[arg(short = 'j', long, help = "Jenkins host name to use (uses current if not specified)")]
        jenkins: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    #[command(about = "Add a new Jenkins host")]
    Add,

    #[command(about = "List all configured Jenkins hosts")]
    List,

    #[command(about = "Remove a Jenkins host")]
    Remove,

    #[command(about = "Set the current Jenkins host to use")]
    Use {
        #[arg(help = "Name of the Jenkins host to use (optional - will prompt to select if not provided)")]
        name: Option<String>,
    },

    #[command(about = "Show Jenkins host configuration")]
    Show {
        #[arg(help = "Name of the Jenkins host (shows current if not specified, or prompts to select)")]
        name: Option<String>,
    },
}
