mod cli;
mod client;
mod config;
mod commands;
mod helpers;
mod interactive;
mod output;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, ConfigAction, AliasAction};
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { action } => match action {
            ConfigAction::Add => commands::config::execute_add()?,
            ConfigAction::List => commands::config::execute_list()?,
            ConfigAction::Remove => commands::config::execute_remove()?,
        },
        Commands::Alias { action } => match action {
            AliasAction::Add { alias, job_name } => {
                commands::alias::execute_add(alias, job_name)?;
            }
            AliasAction::List => commands::alias::execute_list()?,
            AliasAction::Remove { alias } => commands::alias::execute_remove(alias)?,
        },
        Commands::Build { job_name, follow } => {
            commands::build::execute(job_name, follow)?;
        }
        Commands::Status { job_name, build } => {
            commands::status::execute(job_name, build)?;
        }
        Commands::Logs { job_name, build, follow } => {
            commands::logs::execute(job_name, build, follow)?;
        }
        Commands::Open { job_name, build } => {
            commands::open::execute(job_name, build)?;
        }
        Commands::Completion { shell } => {
            commands::completion::execute(shell)?;
        }
    }

    Ok(())
}
