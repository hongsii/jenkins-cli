use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell as CompletionShell};
use crate::cli::{Cli, Shell};
use std::io;

pub fn execute(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    let shell_type = match shell {
        Shell::Bash => CompletionShell::Bash,
        Shell::Zsh => CompletionShell::Zsh,
        Shell::Fish => CompletionShell::Fish,
        Shell::PowerShell => CompletionShell::PowerShell,
    };

    generate(shell_type, &mut cmd, &bin_name, &mut io::stdout());

    Ok(())
}
