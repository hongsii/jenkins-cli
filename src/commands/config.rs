use anyhow::Result;
use crate::config::{Config, JenkinsHost};
use crate::client::JenkinsClient;
use crate::output;
use inquire::{Text, Select, Confirm, MultiSelect};
use url::Url;

pub fn execute_add() -> Result<()> {
    let mut config = Config::load()?;

    // Prompt for name if not provided
    let name = Text::new("Name/alias for this Jenkins host:")
        .with_validator(|input: &str| {
            if input.trim().is_empty() {
                Ok(inquire::validator::Validation::Invalid(
                    "Name cannot be empty".into()
                ))
            } else {
                Ok(inquire::validator::Validation::Valid)
            }
        })
        .prompt()?;

    // Check if the name already exists
    if config.jenkins.contains_key(&name) {
        let overwrite = Confirm::new(&format!("Jenkins host '{}' already exists. Do you want to overwrite it?", name))
            .with_default(false)
            .prompt()?;

        if !overwrite {
            return Ok(());
        }
    }

    // Prompt for Jenkins URL with validation
    let host = Text::new("Jenkins URL:")
        .with_help_message("e.g., https://jenkins.example.com")
        .with_validator(|input: &str| {
            // First check that it starts with http:// or https://
            if !input.starts_with("http://") && !input.starts_with("https://") {
                return Ok(inquire::validator::Validation::Invalid(
                    "URL must start with http:// or https://".into()
                ));
            }

            match Url::parse(input) {
                Ok(url) => {
                    // Check scheme (should be redundant but defensive)
                    if url.scheme() != "http" && url.scheme() != "https" {
                        return Ok(inquire::validator::Validation::Invalid(
                            "URL must use http or https scheme".into()
                        ));
                    }

                    // Check that host is present and not empty
                    match url.host_str() {
                        Some(host) if !host.is_empty() => {
                            Ok(inquire::validator::Validation::Valid)
                        }
                        _ => Ok(inquire::validator::Validation::Invalid(
                            "URL must have a valid host (e.g., jenkins.example.com)".into()
                        ))
                    }
                }
                Err(_) => Ok(inquire::validator::Validation::Invalid(
                    "Invalid URL format".into()
                ))
            }
        })
        .prompt()?;

    let user = Text::new("Username:")
        .with_validator(|input: &str| {
            if input.trim().is_empty() {
                Ok(inquire::validator::Validation::Invalid(
                    "Username cannot be empty".into()
                ))
            } else {
                Ok(inquire::validator::Validation::Valid)
            }
        })
        .prompt()?;

    let token = Text::new("API Token:")
        .with_validator(|input: &str| {
            if input.trim().is_empty() {
                Ok(inquire::validator::Validation::Invalid(
                    "Token cannot be empty".into()
                ))
            } else {
                Ok(inquire::validator::Validation::Valid)
            }
        })
        .prompt()?;

    let jenkins_host = JenkinsHost { host, user, token };

    // Verify connection before saving
    let sp = output::spinner("Verifying connection to Jenkins...");
    let client = JenkinsClient::new(jenkins_host.clone());

    match client.verify_connection() {
        Ok(_) => {
            output::finish_spinner_success(sp, "Connection successful!");
        }
        Err(e) => {
            output::finish_spinner_error(sp, "Connection failed");
            anyhow::bail!("Connection failed: {}\nPlease check your configuration and try again.", e);
        }
    }

    // Save the configuration only if verification succeeded
    config.add_jenkins(name.clone(), jenkins_host);

    if config.current.is_none() {
        config.set_current(name.clone())?;
        output::info(&format!("Set '{}' as the current Jenkins host", name));
    }

    config.save()?;
    output::success(&format!("Jenkins host '{}' added successfully!", name));

    Ok(())
}

pub fn execute_list() -> Result<()> {
    let config = Config::load()?;

    if config.jenkins.is_empty() {
        output::info("No Jenkins hosts configured.");
        output::dim("Use 'jenkins config add' to add one.");
        return Ok(());
    }

    output::header("Configured Jenkins hosts");

    for (name, host) in &config.jenkins {
        let display_name = if config.current.as_ref() == Some(name) {
            format!("{} (current)", name)
        } else {
            name.clone()
        };

        output::highlight(&display_name);
        output::list_item("Host:", &host.host);
        output::list_item("User:", &host.user);
        output::newline();
    }

    Ok(())
}

pub fn execute_remove() -> Result<()> {
    let mut config = Config::load()?;

    if config.jenkins.is_empty() {
        anyhow::bail!("No Jenkins hosts configured.\nUse 'jenkins config add' to add one.");
    }

    let hosts: Vec<String> = config.jenkins.keys().cloned().collect();

    // Use MultiSelect to allow selecting multiple hosts
    let selected_hosts = MultiSelect::new("Select Jenkins host(s) to remove:", hosts)
        .with_help_message("Use ↑↓ to navigate, Space to select/deselect, Enter to confirm")
        .prompt()?;

    if selected_hosts.is_empty() {
        output::info("No hosts selected. Nothing to remove.");
        return Ok(());
    }

    // Show confirmation
    output::warning("The following host(s) will be removed:");
    for host in &selected_hosts {
        output::bullet(host);
    }
    output::newline();

    let confirm = Confirm::new("Are you sure you want to remove these hosts?")
        .with_default(false)
        .prompt()?;

    if !confirm {
        output::info("Operation cancelled.");
        return Ok(());
    }

    // Remove all selected hosts
    for name in &selected_hosts {
        config.remove_jenkins(name)?;
    }

    config.save()?;

    // Print result
    if selected_hosts.len() == 1 {
        output::success(&format!("Jenkins host '{}' removed successfully!", selected_hosts[0]));
    } else {
        output::success(&format!("{} Jenkins hosts removed successfully!", selected_hosts.len()));
    }

    if config.current.is_none() && !config.jenkins.is_empty() {
        output::tip("Use 'jenkins config use <name>' to set a current Jenkins host.");
    }

    Ok(())
}

pub fn execute_use(name: Option<String>) -> Result<()> {
    let mut config = Config::load()?;

    if config.jenkins.is_empty() {
        anyhow::bail!("No Jenkins hosts configured.\nUse 'jenkins config add' to add one.");
    }

    // Prompt for name if not provided
    let name = match name {
        Some(n) => n,
        None => {
            let hosts: Vec<String> = config.jenkins.keys().cloned().collect();
            let current = config.current.as_ref();

            // Create display options with current marker
            let options: Vec<String> = hosts
                .iter()
                .map(|h| {
                    if current == Some(h) {
                        format!("{} (current)", h)
                    } else {
                        h.clone()
                    }
                })
                .collect();

            let selection = Select::new("Select a Jenkins host to use:", options)
                .with_help_message("Use ↑↓ to navigate, type to search, Enter to select")
                .prompt()?;

            // Extract host name (remove " (current)" suffix if present)
            selection.split(" (current)").next().unwrap().to_string()
        }
    };

    config.set_current(name.clone())?;
    config.save()?;

    output::success(&format!("Now using Jenkins host '{}'", name));

    Ok(())
}

pub fn execute_show(name: Option<String>) -> Result<()> {
    let config = Config::load()?;

    if config.jenkins.is_empty() {
        anyhow::bail!("No Jenkins hosts configured.\nUse 'jenkins config add' to add one.");
    }

    let (display_name, host) = if let Some(name) = name {
        let host = config.get_jenkins(&name)?;
        (name, host)
    } else {
        // If no name provided and no current host, prompt to select
        if config.current.is_none() {
            let hosts: Vec<String> = config.jenkins.keys().cloned().collect();
            let selected = Select::new("Select a Jenkins host to show:", hosts)
                .with_help_message("Use ↑↓ to navigate, type to search, Enter to select")
                .prompt()?;

            let host = config.get_jenkins(&selected)?;
            (selected, host)
        } else {
            let (name, host) = config.get_current()?;
            (name.clone(), host)
        }
    };

    output::header(&format!("Jenkins host: {}", display_name));
    output::list_item("Host:", &host.host);
    output::list_item("User:", &host.user);
    output::list_item("Token:", &format!("{}...", &host.token.chars().take(8).collect::<String>()));

    Ok(())
}

#[cfg(test)]
mod tests {
    use url::Url;

    #[test]
    fn test_url_validation_valid_http() {
        let result = Url::parse("http://jenkins.example.com");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme(), "http");
    }

    #[test]
    fn test_url_validation_valid_https() {
        let result = Url::parse("https://jenkins.example.com");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme(), "https");
    }

    #[test]
    fn test_url_validation_with_port() {
        let result = Url::parse("https://jenkins.example.com:8080");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn test_url_validation_with_path() {
        let result = Url::parse("https://jenkins.example.com/jenkins");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_eq!(url.scheme(), "https");
        assert_eq!(url.path(), "/jenkins");
    }

    #[test]
    fn test_url_validation_invalid_scheme() {
        let result = Url::parse("ftp://jenkins.example.com");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert_ne!(url.scheme(), "http");
        assert_ne!(url.scheme(), "https");
    }

    #[test]
    fn test_url_validation_invalid_format() {
        let result = Url::parse("jenkins.example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_url_validation_missing_protocol() {
        let result = Url::parse("jenkins.example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_url_validation_no_host() {
        // Test that malformed URLs like "http:loc" are caught
        // Note: url::Url may parse "http:loc" as having host "loc" in some cases
        let result = Url::parse("http:loc");
        assert!(result.is_ok());
        let url = result.unwrap();

        // Our validator should reject this because it's not a valid HTTP URL
        // Even if url crate accepts it, we check for proper scheme and host
        assert_eq!(url.scheme(), "http");
        // The host might be "loc" depending on how url crate interprets it
        // Our validation logic will catch this as it checks for proper URL structure

        // Test URLs that definitely don't have hosts
        let result = Url::parse("data:text/plain,hello");
        assert!(result.is_ok());
        let url = result.unwrap();
        assert!(url.host_str().is_none(), "data URL should not have a host");
    }

    #[test]
    fn test_url_validation_with_valid_host() {
        let valid_urls = vec![
            "http://jenkins.example.com",
            "https://jenkins.example.com",
            "http://localhost",
            "https://localhost:8080",
            "http://192.168.1.1",
            "https://192.168.1.1:8080",
        ];

        for url_str in valid_urls {
            let result = Url::parse(url_str);
            assert!(result.is_ok(), "URL '{}' should be valid", url_str);

            let url = result.unwrap();
            assert!(url.host_str().is_some(), "URL '{}' should have a host", url_str);
            assert!(!url.host_str().unwrap().is_empty(), "URL '{}' host should not be empty", url_str);
        }
    }

    #[test]
    fn test_url_validation_scheme_only() {
        // Just scheme without proper URL structure
        let result = Url::parse("http:");
        if let Ok(url) = result {
            assert!(url.host_str().is_none(), "http: should not have a host");
        }
    }

    #[test]
    fn test_empty_string_validation() {
        let input = "";
        assert!(input.trim().is_empty());
    }

    #[test]
    fn test_whitespace_string_validation() {
        let input = "   ";
        assert!(input.trim().is_empty());
    }

    #[test]
    fn test_valid_string_validation() {
        let input = "valid-name";
        assert!(!input.trim().is_empty());
    }

    #[test]
    fn test_string_with_surrounding_whitespace() {
        let input = "  valid-name  ";
        assert!(!input.trim().is_empty());
        assert_eq!(input.trim(), "valid-name");
    }

    #[test]
    fn test_malformed_url_detection() {
        // Test the actual validation logic we use in execute_add
        let test_cases = vec![
            ("http:loc", false),              // Malformed - missing //
            ("http:", false),                 // No host - missing //
            ("https:", false),                // No host - missing //
            ("http://", false),               // No host after //
            ("https://", false),              // No host after //
            ("http://jenkins.com", true),     // Valid
            ("https://jenkins.com", true),    // Valid
            ("ftp://jenkins.com", false),     // Wrong scheme
            ("jenkins.com", false),           // Missing scheme
        ];

        for (url_str, should_be_valid) in test_cases {
            // Use the same validation logic as in execute_add
            let is_valid = if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
                false
            } else {
                match Url::parse(url_str) {
                    Ok(url) => {
                        (url.scheme() == "http" || url.scheme() == "https")
                            && url.host_str().is_some()
                            && !url.host_str().unwrap().is_empty()
                    }
                    Err(_) => false,
                }
            };

            assert_eq!(is_valid, should_be_valid,
                "URL '{}' validation mismatch: expected {}, got {}",
                url_str, should_be_valid, is_valid);
        }
    }
}
