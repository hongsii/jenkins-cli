use anyhow::Result;
use clap::CommandFactory;
use clap_complete::{generate, Shell as CompletionShell};
use crate::cli::{Cli, Shell};
use crate::output;
use std::io;

pub fn execute(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let bin_name = "jenkins";

    let shell_type = match shell {
        Shell::Bash => CompletionShell::Bash,
        Shell::Zsh => CompletionShell::Zsh,
        Shell::Fish => CompletionShell::Fish,
        Shell::PowerShell => CompletionShell::PowerShell,
    };

    output::info(&format!("Generating {} completion script...", format!("{:?}", shell)));
    output::newline();

    generate(shell_type, &mut cmd, bin_name, &mut io::stdout());

    output::newline();
    print_installation_instructions(shell);

    Ok(())
}

fn print_installation_instructions(shell: Shell) {
    output::header("Installation Instructions");

    match shell {
        Shell::Bash => {
            output::plain("Add the following to your ~/.bashrc:");
            output::newline();
            output::dim("  eval \"$(jenkins completion bash)\"");
            output::newline();
            output::plain("Or save to a file:");
            output::newline();
            output::dim("  jenkins completion bash > /usr/local/etc/bash_completion.d/jenkins");
        }
        Shell::Zsh => {
            output::plain("Add the following to your ~/.zshrc:");
            output::newline();
            output::dim("  eval \"$(jenkins completion zsh)\"");
            output::newline();
            output::plain("Or save to a file in your fpath:");
            output::newline();
            output::dim("  jenkins completion zsh > /usr/local/share/zsh/site-functions/_jenkins");
            output::newline();
            output::plain("Then restart your shell or run:");
            output::newline();
            output::dim("  autoload -U compinit && compinit");
        }
        Shell::Fish => {
            output::plain("Save the completion script:");
            output::newline();
            output::dim("  jenkins completion fish > ~/.config/fish/completions/jenkins.fish");
            output::newline();
            output::plain("Completions will be available in new fish sessions.");
        }
        Shell::PowerShell => {
            output::plain("Add the following to your PowerShell profile:");
            output::newline();
            output::dim("  jenkins completion powershell | Out-String | Invoke-Expression");
            output::newline();
            output::plain("Or save to a file and source it in your profile:");
            output::newline();
            output::dim("  jenkins completion powershell > jenkins.ps1");
        }
    }

    output::newline();
    output::tip("After installation, restart your shell or source the config file");
}
