use anyhow::{Context, Result};
use std::io::{self, Write};

use crate::client::JenkinsClient;

/// Resolves the final job name by interactively selecting from sub-jobs if present
pub fn resolve_job_name(client: &JenkinsClient, initial_job_name: Option<&str>) -> Result<String> {
    let mut current_job_name = match initial_job_name {
        Some(name) => name.to_string(),
        None => {
            // No job name provided, start from root
            let root_jobs = client.get_root_jobs()?;

            if root_jobs.is_empty() {
                anyhow::bail!("No jobs found on this Jenkins instance");
            }

            println!("\nAvailable jobs:");
            println!();

            for (idx, job) in root_jobs.iter().enumerate() {
                println!("  {}. {} [{}]", idx + 1, job.name, format_color(job.color.as_deref()));
            }

            println!();
            print!("Select a job (1-{}): ", root_jobs.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .context("Failed to read input")?;

            let selection: usize = input
                .trim()
                .parse()
                .context("Invalid number")?;

            if selection < 1 || selection > root_jobs.len() {
                anyhow::bail!("Invalid selection. Please enter a number between 1 and {}.", root_jobs.len());
            }

            root_jobs[selection - 1].name.clone()
        }
    };

    loop {
        let job_info = client.get_job(&current_job_name)?;

        // If no sub-jobs, return the current job name
        if job_info.jobs.is_none() || job_info.jobs.as_ref().unwrap().is_empty() {
            return Ok(current_job_name);
        }

        // Display sub-jobs and let user select
        let sub_jobs = job_info.jobs.unwrap();
        println!("\n'{}' contains {} sub-job(s):", current_job_name, sub_jobs.len());
        println!();

        for (idx, job) in sub_jobs.iter().enumerate() {
            println!("  {}. {} [{}]", idx + 1, job.name, format_color(job.color.as_deref()));
        }

        println!();
        print!("Select a job (1-{}): ", sub_jobs.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("Failed to read input")?;

        let selection: usize = input
            .trim()
            .parse()
            .context("Invalid number")?;

        if selection < 1 || selection > sub_jobs.len() {
            println!("Invalid selection. Please enter a number between 1 and {}.", sub_jobs.len());
            continue;
        }

        let selected_job = &sub_jobs[selection - 1];

        // Build the full job path
        // Jenkins uses the format: parent/job/child
        current_job_name = format!("{}/job/{}", current_job_name, selected_job.name);
    }
}

/// Resolves the job name for the open command, allowing to stop at any level
pub fn resolve_job_name_for_open(client: &JenkinsClient, initial_job_name: Option<&str>) -> Result<String> {
    let mut current_job_name = match initial_job_name {
        Some(name) => name.to_string(),
        None => {
            // No job name provided, start from root
            let root_jobs = client.get_root_jobs()?;

            if root_jobs.is_empty() {
                anyhow::bail!("No jobs found on this Jenkins instance");
            }

            println!("\nAvailable jobs:");
            println!();

            for (idx, job) in root_jobs.iter().enumerate() {
                println!("  {}. {} [{}]", idx + 1, job.name, format_color(job.color.as_deref()));
            }

            println!();
            print!("Select a job (1-{}): ", root_jobs.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .context("Failed to read input")?;

            let selection: usize = input
                .trim()
                .parse()
                .context("Invalid number")?;

            if selection < 1 || selection > root_jobs.len() {
                anyhow::bail!("Invalid selection. Please enter a number between 1 and {}.", root_jobs.len());
            }

            root_jobs[selection - 1].name.clone()
        }
    };

    loop {
        let job_info = client.get_job(&current_job_name)?;

        // If no sub-jobs, return the current job name
        if job_info.jobs.is_none() || job_info.jobs.as_ref().unwrap().is_empty() {
            return Ok(current_job_name);
        }

        // Display options: open current or select sub-job
        let sub_jobs = job_info.jobs.unwrap();
        println!("\n'{}' contains {} sub-job(s):", current_job_name, sub_jobs.len());
        println!();
        println!("  0. Open this job/folder");

        for (idx, job) in sub_jobs.iter().enumerate() {
            println!("  {}. {} [{}]", idx + 1, job.name, format_color(job.color.as_deref()));
        }

        println!();
        print!("Select an option (0-{}): ", sub_jobs.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("Failed to read input")?;

        let selection: usize = input
            .trim()
            .parse()
            .context("Invalid number")?;

        if selection > sub_jobs.len() {
            println!("Invalid selection. Please enter a number between 0 and {}.", sub_jobs.len());
            continue;
        }

        // If 0, open current job
        if selection == 0 {
            return Ok(current_job_name);
        }

        let selected_job = &sub_jobs[selection - 1];

        // Build the full job path
        // Jenkins uses the format: parent/job/child
        current_job_name = format!("{}/job/{}", current_job_name, selected_job.name);
    }
}

fn format_color(color: Option<&str>) -> String {
    match color {
        Some("blue") => "Success".to_string(),
        Some("red") => "Failed".to_string(),
        Some("yellow") => "Unstable".to_string(),
        Some("aborted") => "Aborted".to_string(),
        Some("notbuilt") => "Not Built".to_string(),
        Some("disabled") => "Disabled".to_string(),
        Some(c) if c.ends_with("_anime") => {
            format!("Building ({})", c.trim_end_matches("_anime"))
        }
        Some(c) => c.to_string(),
        None => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_color() {
        assert_eq!(format_color(Some("blue")), "Success");
        assert_eq!(format_color(Some("red")), "Failed");
        assert_eq!(format_color(Some("yellow")), "Unstable");
        assert_eq!(format_color(Some("aborted")), "Aborted");
        assert_eq!(format_color(Some("notbuilt")), "Not Built");
        assert_eq!(format_color(Some("disabled")), "Disabled");
        assert_eq!(format_color(Some("blue_anime")), "Building (blue)");
        assert_eq!(format_color(Some("unknown")), "unknown");
        assert_eq!(format_color(None), "Unknown");
    }
}
