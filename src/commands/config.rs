use anyhow::Result;
use crate::config::{Config, JenkinsHost};
use std::io::{self, Write};

pub fn execute_add(name: String) -> Result<()> {
    let mut config = Config::load()?;

    if config.jenkins.contains_key(&name) {
        println!("Jenkins '{}' already exists. Overwriting...", name);
    }

    println!("Adding Jenkins host '{}'...\n", name);

    let host = prompt("Jenkins URL (e.g., https://jenkins.example.com)")?;
    let user = prompt("Username")?;
    let token = prompt("API Token")?;

    let jenkins_host = JenkinsHost { host, user, token };
    config.add_jenkins(name.clone(), jenkins_host);

    if config.current.is_none() {
        config.set_current(name.clone())?;
        println!("\nSet '{}' as the current Jenkins host.", name);
    }

    config.save()?;
    println!("Jenkins host '{}' added successfully!", name);

    Ok(())
}

pub fn execute_list() -> Result<()> {
    let config = Config::load()?;

    if config.jenkins.is_empty() {
        println!("No Jenkins hosts configured.");
        println!("Use 'jenkins config add <name>' to add one.");
        return Ok(());
    }

    println!("Configured Jenkins hosts:\n");

    for (name, host) in &config.jenkins {
        let current_marker = if config.current.as_ref() == Some(name) {
            " (current)"
        } else {
            ""
        };
        println!("  {} {}", name, current_marker);
        println!("    Host: {}", host.host);
        println!("    User: {}", host.user);
        println!();
    }

    Ok(())
}

pub fn execute_remove(name: String) -> Result<()> {
    let mut config = Config::load()?;

    config.remove_jenkins(&name)?;
    config.save()?;

    println!("Jenkins host '{}' removed successfully!", name);

    if config.current.is_none() && !config.jenkins.is_empty() {
        println!("\nTip: Use 'jenkins config use <name>' to set a current Jenkins host.");
    }

    Ok(())
}

pub fn execute_use(name: String) -> Result<()> {
    let mut config = Config::load()?;

    config.set_current(name.clone())?;
    config.save()?;

    println!("Now using Jenkins host '{}'", name);

    Ok(())
}

pub fn execute_show(name: Option<String>) -> Result<()> {
    let config = Config::load()?;

    let (display_name, host) = if let Some(name) = name {
        let host = config.get_jenkins(&name)?;
        (name, host)
    } else {
        let (name, host) = config.get_current()?;
        (name.clone(), host)
    };

    println!("Jenkins host: {}", display_name);
    println!("  Host: {}", host.host);
    println!("  User: {}", host.user);
    println!("  Token: {}...", &host.token.chars().take(8).collect::<String>());

    Ok(())
}

fn prompt(message: &str) -> Result<String> {
    print!("{}: ", message);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}
