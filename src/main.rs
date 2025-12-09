mod cli;
mod client;
mod config;
mod commands;
mod interactive;
mod output;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, ConfigAction};
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
            ConfigAction::Use { name } => commands::config::execute_use(name)?,
            ConfigAction::Show { name } => commands::config::execute_show(name)?,
        },
        Commands::Build { job_name, jenkins, follow } => {
            commands::build::execute(job_name, jenkins, follow)?;
        }
        Commands::Status { job_name, build, jenkins } => {
            commands::status::execute(job_name, build, jenkins)?;
        }
        Commands::Logs { job_name, build, jenkins } => {
            commands::logs::execute(job_name, build, jenkins)?;
        }
        Commands::Open { job_name, build, jenkins } => {
            commands::open::execute(job_name, build, jenkins)?;
        }
        Commands::Completion { shell } => {
            commands::completion::execute(shell)?;
        }
    }

    Ok(())
}
