# Shell Completion

Jenkins CLI supports shell completion for the following shells:
- Bash
- Zsh
- Fish
- PowerShell

## Installation

### Bash

Add the following to your `~/.bashrc`:

```bash
eval "$(jenkins completion bash)"
```

Or save to a completion directory:

```bash
jenkins completion bash > /usr/local/etc/bash_completion.d/jenkins
```

### Zsh

Add the following to your `~/.zshrc`:

```zsh
eval "$(jenkins completion zsh)"
```

Or save to a file in your fpath:

```zsh
jenkins completion zsh > /usr/local/share/zsh/site-functions/_jenkins
```

Then reload completions:

```zsh
autoload -U compinit && compinit
```

### Fish

Save the completion script to Fish's completions directory:

```fish
jenkins completion fish > ~/.config/fish/completions/jenkins.fish
```

Completions will be available in new fish sessions.

### PowerShell

Add the following to your PowerShell profile:

```powershell
jenkins completion powershell | Out-String | Invoke-Expression
```

Or save to a file and source it in your profile:

```powershell
jenkins completion powershell > jenkins.ps1
```

## Usage

After installation, you can use Tab to autocomplete:

```bash
jenkins <TAB>          # Shows all available commands
jenkins config <TAB>   # Shows config subcommands (add, list, remove, use, show)
jenkins build <TAB>    # Shows available options and flags
```

## Features

The completion provides:
- Command and subcommand completion
- Option and flag completion
- Value completion for enum options (like shell types)
- Help text for each command and option
