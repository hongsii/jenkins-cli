use anyhow::Result;
use crate::config::Config;
use crate::client::JenkinsClient;
use crate::interactive;
use crate::output;
use inquire::{Text, Select, Confirm};

pub fn execute_add(alias: Option<String>, job_name: Option<String>) -> Result<()> {
    let mut config = Config::load()?;

    // Get alias name - either from argument or prompt
    let alias = match alias {
        Some(a) => a,
        None => {
            Text::new("Enter alias name:")
                .with_validator(|input: &str| {
                    if input.trim().is_empty() {
                        Ok(inquire::validator::Validation::Invalid(
                            "Alias name cannot be empty".into()
                        ))
                    } else {
                        Ok(inquire::validator::Validation::Valid)
                    }
                })
                .prompt()?
        }
    };

    // Check if the alias already exists
    if config.job_aliases.contains_key(&alias) {
        let overwrite = Confirm::new(&format!("Job alias '{}' already exists. Do you want to overwrite it?", alias))
            .with_default(false)
            .prompt()?;

        if !overwrite {
            return Ok(());
        }
    }

    // Select Jenkins host for job selection
    use crate::helpers::init::{resolve_jenkins_host, prompt_jenkins_selection};
    
    let selected_jenkins = prompt_jenkins_selection()?;

    // Get job name - either from argument or interactively
    let final_job_name = match job_name {
        Some(name) => name,
        None => {
            let selected_jenkins_host = resolve_jenkins_host(selected_jenkins.clone())?;
            let client = JenkinsClient::new(selected_jenkins_host)?;
            interactive::resolve_job_name(&client, None)?
        }
    };

    config.add_job_alias(alias.clone(), final_job_name.clone(), selected_jenkins.clone());
    config.save()?;

    if let Some(j) = selected_jenkins {
        output::success(&format!("Job alias '{}' → '{}' (Jenkins: {}) added successfully!", alias, final_job_name, j));
    } else {
        output::success(&format!("Job alias '{}' → '{}' added successfully!", alias, final_job_name));
    }

    Ok(())
}

pub fn execute_list() -> Result<()> {
    let config = Config::load()?;

    if config.job_aliases.is_empty() {
        output::info("No job aliases configured.");
        return Ok(());
    }

    output::header("Configured job aliases");

    // Sort aliases for consistent output
    let mut aliases: Vec<_> = config.job_aliases.iter().collect();
    aliases.sort_by_key(|(alias, _)| *alias);

    for (alias, job_alias) in aliases {
        let display = if let Some(ref jenkins) = job_alias.jenkins {
            format!("{} (Jenkins: {})", job_alias.job_name, jenkins)
        } else {
            job_alias.job_name.clone()
        };
        output::list_item(format!("{}:", alias).as_str(), &display);
    }

    Ok(())
}

pub fn execute_remove(alias: Option<String>) -> Result<()> {
    let mut config = Config::load()?;

    if config.job_aliases.is_empty() {
        anyhow::bail!("No job aliases configured.\nUse 'jenkins alias add <alias> <job-name>' to add one.");
    }

    // Prompt for alias if not provided
    let alias = match alias {
        Some(a) => a,
        None => {
            let aliases: Vec<String> = config.job_aliases.keys().cloned().collect();
            Select::new("Select a job alias to remove:", aliases)
                .with_help_message("Use ↑↓ to navigate, type to search, Enter to select")
                .prompt()?
        }
    };

    // Confirm removal
    let job_alias = config.job_aliases.get(&alias)
        .ok_or_else(|| anyhow::anyhow!("Job alias '{}' not found", alias))?;

    let display = if let Some(ref jenkins) = job_alias.jenkins {
        format!("{} (Jenkins: {})", job_alias.job_name, jenkins)
    } else {
        job_alias.job_name.clone()
    };

    let confirm = Confirm::new(&format!("Remove job alias '{}' → '{}'?", alias, display))
        .with_default(false)
        .prompt()?;

    if !confirm {
        output::info("Operation cancelled.");
        return Ok(());
    }

    config.remove_job_alias(&alias)?;
    config.save()?;

    output::success(&format!("Job alias '{}' removed successfully!", alias));

    Ok(())
}
