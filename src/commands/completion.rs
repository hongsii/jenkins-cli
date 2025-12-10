use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell as CompletionShell};
use crate::cli::{Cli, Shell};
use std::io;
use console::style;

pub fn execute(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let bin_name = "jenkins";

    let shell_type = match shell {
        Shell::Bash => CompletionShell::Bash,
        Shell::Zsh => CompletionShell::Zsh,
        Shell::Fish => CompletionShell::Fish,
        Shell::PowerShell => CompletionShell::PowerShell,
    };

    eprintln!("{} Generating {} completion script...", style("ℹ").blue().bold(), format!("{:?}", shell));
    eprintln!();

    generate(shell_type, &mut cmd, bin_name, &mut io::stdout());

    eprintln!();
    print_installation_instructions(shell);

    Ok(())
}

fn print_installation_instructions(shell: Shell) {
    eprintln!("\n{}", style("Installation Instructions").bold().underlined());

    match shell {
        Shell::Bash => {
            eprintln!("Add the following to your ~/.bashrc:");
            eprintln!();
            eprintln!("{}", style("  eval \"$(jenkins completion bash)\"").dim());
            eprintln!();
            eprintln!("Or save to a file:");
            eprintln!();
            eprintln!("{}", style("  jenkins completion bash > /usr/local/etc/bash_completion.d/jenkins").dim());
        }
        Shell::Zsh => {
            eprintln!("Add the following to your ~/.zshrc:");
            eprintln!();
            eprintln!("{}", style("  eval \"$(jenkins completion zsh)\"").dim());
            eprintln!();
            eprintln!("Or save to a file in your fpath:");
            eprintln!();
            eprintln!("{}", style("  jenkins completion zsh > /usr/local/share/zsh/site-functions/_jenkins").dim());
            eprintln!();
            eprintln!("Then restart your shell or run:");
            eprintln!();
            eprintln!("{}", style("  autoload -U compinit && compinit").dim());
        }
        Shell::Fish => {
            eprintln!("Save the completion script:");
            eprintln!();
            eprintln!("{}", style("  jenkins completion fish > ~/.config/fish/completions/jenkins.fish").dim());
            eprintln!();
            eprintln!("Completions will be available in new fish sessions.");
        }
        Shell::PowerShell => {
            eprintln!("Add the following to your PowerShell profile:");
            eprintln!();
            eprintln!("{}", style("  jenkins completion powershell | Out-String | Invoke-Expression").dim());
            eprintln!();
            eprintln!("Or save to a file and source it in your profile:");
            eprintln!();
            eprintln!("{}", style("  jenkins completion powershell > jenkins.ps1").dim());
        }
    }

    eprintln!("\n{} {}", style("💡").bold(), style("After installation, restart your shell or source the config file").italic());
}
