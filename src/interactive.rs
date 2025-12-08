use anyhow::{Context, Result};
use inquire::Select;

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

            // Create display options with job name and status
            let options: Vec<String> = root_jobs
                .iter()
                .map(|job| format!("{} [{}]", job.name, format_color(job.color.as_deref())))
                .collect();

            let selection = Select::new("Select a job:", options)
                .with_help_message("Use ↑↓ to navigate, type to search, Enter to select")
                .prompt()
                .context("Failed to get user selection")?;

            // Extract job name from selection (remove the status part)
            let job_name = selection.split(" [").next().unwrap().to_string();
            job_name
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

        // Create display options with job name and status
        let options: Vec<String> = sub_jobs
            .iter()
            .map(|job| format!("{} [{}]", job.name, format_color(job.color.as_deref())))
            .collect();

        let prompt_msg = format!("'{}' contains {} sub-job(s). Select a job:", current_job_name, sub_jobs.len());
        let selection = Select::new(&prompt_msg, options)
            .with_help_message("Use ↑↓ to navigate, type to search, Enter to select")
            .prompt()
            .context("Failed to get user selection")?;

        // Extract job name from selection (remove the status part)
        let selected_job_name = selection.split(" [").next().unwrap().to_string();

        // Find the original job info
        let selected_job = sub_jobs
            .iter()
            .find(|job| job.name == selected_job_name)
            .unwrap();

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

            // Create display options with job name and status
            let options: Vec<String> = root_jobs
                .iter()
                .map(|job| format!("{} [{}]", job.name, format_color(job.color.as_deref())))
                .collect();

            let selection = Select::new("Select a job:", options)
                .with_help_message("Use ↑↓ to navigate, type to search, Enter to select")
                .prompt()
                .context("Failed to get user selection")?;

            // Extract job name from selection (remove the status part)
            let job_name = selection.split(" [").next().unwrap().to_string();
            job_name
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

        // Create display options with "Open this job/folder" as first option
        let mut options: Vec<String> = vec!["[Open this job/folder]".to_string()];
        options.extend(
            sub_jobs
                .iter()
                .map(|job| format!("{} [{}]", job.name, format_color(job.color.as_deref())))
        );

        let prompt_msg = format!("'{}' contains {} sub-job(s). Select an option:", current_job_name, sub_jobs.len());
        let selection = Select::new(&prompt_msg, options)
            .with_help_message("Use ↑↓ to navigate, type to search, Enter to select")
            .prompt()
            .context("Failed to get user selection")?;

        // If user selected "Open this job/folder", return current job
        if selection == "[Open this job/folder]" {
            return Ok(current_job_name);
        }

        // Extract job name from selection (remove the status part)
        let selected_job_name = selection.split(" [").next().unwrap().to_string();

        // Find the original job info
        let selected_job = sub_jobs
            .iter()
            .find(|job| job.name == selected_job_name)
            .unwrap();

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

    #[test]
    fn test_format_color_all_anime_variants() {
        assert_eq!(format_color(Some("red_anime")), "Building (red)");
        assert_eq!(format_color(Some("yellow_anime")), "Building (yellow)");
        assert_eq!(format_color(Some("aborted_anime")), "Building (aborted)");
    }

    #[test]
    fn test_format_color_edge_cases() {
        assert_eq!(format_color(Some("")), "");
        assert_eq!(format_color(Some("BLUE")), "BLUE");
        assert_eq!(format_color(Some("Blue")), "Blue");
    }

    #[test]
    fn test_format_color_anime_suffix() {
        let color = "blue_anime";
        assert!(color.ends_with("_anime"));
        let trimmed = color.trim_end_matches("_anime");
        assert_eq!(trimmed, "blue");
    }

    #[test]
    fn test_format_color_all_standard_states() {
        // Test all Jenkins standard job states
        let states = vec![
            ("blue", "Success"),
            ("red", "Failed"),
            ("yellow", "Unstable"),
            ("aborted", "Aborted"),
            ("notbuilt", "Not Built"),
            ("disabled", "Disabled"),
        ];

        for (input, expected) in states {
            assert_eq!(format_color(Some(input)), expected, "Failed for state: {}", input);
        }
    }
}
