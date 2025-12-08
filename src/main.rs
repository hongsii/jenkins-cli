mod cli;
mod client;
mod config;
mod commands;
mod interactive;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, ConfigAction};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Config { action } => match action {
            ConfigAction::Add { name } => commands::config::execute_add(name)?,
            ConfigAction::List => commands::config::execute_list()?,
            ConfigAction::Remove { name } => commands::config::execute_remove(name)?,
            ConfigAction::Use { name } => commands::config::execute_use(name)?,
            ConfigAction::Show { name } => commands::config::execute_show(name)?,
        },
        Commands::Build { job_name, jenkins } => {
            commands::build::execute(job_name, jenkins)?;
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
    }

    Ok(())
}
