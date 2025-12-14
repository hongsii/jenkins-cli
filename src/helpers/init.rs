use anyhow::Result;
use crate::client::JenkinsClient;
use crate::config::{Config, JenkinsHost};
use inquire::Select;

/// Create a JenkinsClient with the specified or current host
pub fn create_client(jenkins_name: Option<String>) -> Result<JenkinsClient> {
    let host = resolve_jenkins_host(jenkins_name)?;
    JenkinsClient::new(host)
}

/// Create a JenkinsClient for a specific job, using alias jenkins if available
/// Priority: explicit jenkins_name > alias jenkins > prompt selection (if multiple) > single jenkins
pub fn create_client_for_job(job_name: Option<&str>, jenkins_name: Option<String>) -> Result<JenkinsClient> {
    let jenkins_to_use = if jenkins_name.is_some() {
        // User explicitly specified jenkins
        jenkins_name
    } else if let Some(job) = job_name {
        // Check if job_name is an alias with jenkins info
        let config = Config::load()?;
        let (_, _, alias_jenkins) = config.resolve_job_name(job);
        if alias_jenkins.is_some() {
            alias_jenkins
        } else {
            // No alias jenkins, prompt for selection if multiple hosts
            prompt_jenkins_selection()?
        }
    } else {
        // No job name, prompt for selection if multiple hosts
        prompt_jenkins_selection()?
    };

    create_client(jenkins_to_use)
}

/// Prompt user to select a Jenkins host if multiple are configured
/// Returns None if only one host exists (will use it automatically)
pub fn prompt_jenkins_selection() -> Result<Option<String>> {
    let config = Config::load()?;

    match config.jenkins.len() {
        0 => anyhow::bail!("No Jenkins configured. Use 'jenkins config add' to add one."),
        1 => {
            // Only one jenkins, use it automatically
            let name = config.jenkins.keys().next().unwrap().clone();
            Ok(Some(name))
        }
        _ => {
            // Multiple jenkins hosts, prompt user to select
            let mut jenkins_names: Vec<String> = config.jenkins.keys().cloned().collect();
            jenkins_names.sort();

            let selection = Select::new("Select Jenkins:", jenkins_names)
                .with_help_message("Use ↑↓ to navigate, type to search, Enter to select, ESC to cancel")
                .prompt()?;

            Ok(Some(selection))
        }
    }
}

/// Load config and get the specified Jenkins host
/// If no host is specified, prompts for selection (if multiple hosts exist)
pub fn resolve_jenkins_host(jenkins_name: Option<String>) -> Result<JenkinsHost> {
    let jenkins_to_use = if jenkins_name.is_some() {
        jenkins_name
    } else {
        // Prompt for selection
        prompt_jenkins_selection()?
    };

    let config = Config::load()?;
    let host = if let Some(name) = jenkins_to_use {
        config.get_jenkins(&name)?.clone()
    } else {
        // This shouldn't happen, but handle it anyway
        anyhow::bail!("No Jenkins host specified")
    };

    Ok(host)
}
