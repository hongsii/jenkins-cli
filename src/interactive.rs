use anyhow::{Context, Result};
use inquire::{Confirm, InquireError, Select, Text};

use crate::client::{JenkinsClient, ParameterDefinition, ParameterValue};
use crate::config::Config;
use crate::helpers::formatting::format_job_color as format_color;
use crate::output;

/// Handle inquire errors and convert to user-friendly messages
fn handle_inquire_error<T>(result: Result<T, InquireError>) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(InquireError::OperationCanceled) => {
            output::cancelled("Operation cancelled by user");
            std::process::exit(0);
        }
        Err(InquireError::OperationInterrupted) => {
            output::cancelled("Operation interrupted by user");
            std::process::exit(0);
        }
        Err(e) => Err(e).context("Failed to get user input"),
    }
}

/// Resolves the final job name by interactively selecting from sub-jobs if present
pub fn resolve_job_name(client: &JenkinsClient, initial_job_name: Option<&str>) -> Result<String> {
    let mut current_job_name = match initial_job_name {
        Some(name) => {
            // Resolve alias if present
            let config = Config::load()?;
            let (job_name, is_alias, jenkins) = config.resolve_job_name(name);
            if is_alias {
                if let Some(j) = jenkins {
                    output::dim(&format!("Using alias '{}' → '{}' (Jenkins: {})", name, job_name, j));
                } else {
                    output::dim(&format!("Using alias '{}' → '{}'", name, job_name));
                }
            }
            job_name
        },
        None => {
            // No job name provided, start from root
            let sp = output::spinner("Loading jobs...");
            let root_jobs = client.get_root_jobs()?;
            sp.finish_and_clear();

            if root_jobs.is_empty() {
                anyhow::bail!("No jobs found on this Jenkins instance");
            }

            // Create display options with job name and status
            let options: Vec<String> = root_jobs
                .iter()
                .map(|job| format!("{} [{}]", job.name, format_color(job.color.as_deref())))
                .collect();

            let selection = handle_inquire_error(
                Select::new("Select a job:", options)
                    .with_help_message("Use ↑↓ to navigate, type to search, Enter to select, ESC to cancel")
                    .prompt()
            )?;

            // Extract job name from selection (remove the status part)
            let job_name = selection.split(" [").next().unwrap().to_string();
            job_name
        }
    };

    loop {
        let sp = output::spinner("Loading job details...");
        let job_info = client.get_job(&current_job_name)?;
        sp.finish_and_clear();

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

        output::dim(&format!("'{}' contains {} sub-job(s).", current_job_name, sub_jobs.len()));
        let selection = handle_inquire_error(
            Select::new("Select a job:", options)
                .with_help_message("Use ↑↓ to navigate, type to search, Enter to select, ESC to cancel")
                .prompt()
        )?;

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
        Some(name) => {
            // Resolve alias if present
            let config = Config::load()?;
            let (job_name, is_alias, jenkins) = config.resolve_job_name(name);
            if is_alias {
                if let Some(j) = jenkins {
                    output::dim(&format!("Using alias '{}' → '{}' (Jenkins: {})", name, job_name, j));
                } else {
                    output::dim(&format!("Using alias '{}' → '{}'", name, job_name));
                }
            }
            job_name
        },
        None => {
            // No job name provided, start from root
            let sp = output::spinner("Loading jobs...");
            let root_jobs = client.get_root_jobs()?;
            sp.finish_and_clear();

            if root_jobs.is_empty() {
                anyhow::bail!("No jobs found on this Jenkins instance");
            }

            // Create display options with job name and status
            let options: Vec<String> = root_jobs
                .iter()
                .map(|job| format!("{} [{}]", job.name, format_color(job.color.as_deref())))
                .collect();

            let selection = handle_inquire_error(
                Select::new("Select a job:", options)
                    .with_help_message("Use ↑↓ to navigate, type to search, Enter to select, ESC to cancel")
                    .prompt()
            )?;

            // Extract job name from selection (remove the status part)
            let job_name = selection.split(" [").next().unwrap().to_string();
            job_name
        }
    };

    loop {
        let sp = output::spinner("Loading job details...");
        let job_info = client.get_job(&current_job_name)?;
        sp.finish_and_clear();

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

        output::dim(&format!("'{}' contains {} sub-job(s).", current_job_name, sub_jobs.len()));
        let selection = handle_inquire_error(
            Select::new("Select a job:", options)
                .with_help_message("Use ↑↓ to navigate, type to search, Enter to select, ESC to cancel")
                .prompt()
        )?;

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

/// Prompt user to input values for job parameters
pub fn collect_parameters(
    parameter_definitions: Vec<ParameterDefinition>
) -> Result<Vec<ParameterValue>> {
    let mut parameter_values = Vec::new();

    if parameter_definitions.is_empty() {
        return Ok(parameter_values);
    }

    output::header("Job Parameters");
    output::info(&format!("This job requires {} parameter(s):", parameter_definitions.len()));
    output::newline();

    for param_def in parameter_definitions {
        let param_value = prompt_for_parameter(&param_def)?;
        parameter_values.push(param_value);
    }

    Ok(parameter_values)
}

/// Prompt for a single parameter based on its type
fn prompt_for_parameter(param_def: &ParameterDefinition) -> Result<ParameterValue> {
    let description = param_def.description.as_deref().unwrap_or("");
    let help_message = if description.is_empty() {
        format!("Type: {}", param_def.param_type)
    } else {
        format!("{} (Type: {})", description, param_def.param_type)
    };

    // Determine parameter type from class name
    let value = if param_def.class.contains("BooleanParameterDefinition") {
        prompt_boolean_parameter(param_def, &help_message)?
    } else if param_def.class.contains("ChoiceParameterDefinition") {
        prompt_choice_parameter(param_def, &help_message)?
    } else {
        // Default to string parameter (covers StringParameterDefinition and others)
        prompt_string_parameter(param_def, &help_message)?
    };

    Ok(ParameterValue {
        name: param_def.name.clone(),
        value,
    })
}

fn prompt_string_parameter(param_def: &ParameterDefinition, help: &str) -> Result<String> {
    let default_value = extract_default_string(param_def);
    let prompt_message = format!("{}:", param_def.name);

    let mut text_prompt = Text::new(&prompt_message)
        .with_help_message(help);

    if let Some(ref default) = default_value {
        text_prompt = text_prompt.with_default(default);
    }

    let value = handle_inquire_error(text_prompt.prompt())?;

    Ok(value)
}

fn prompt_boolean_parameter(param_def: &ParameterDefinition, help: &str) -> Result<String> {
    let default_value = extract_default_bool(param_def);
    let prompt_message = format!("{}?", param_def.name);

    let mut confirm_prompt = Confirm::new(&prompt_message)
        .with_help_message(help);

    if let Some(default) = default_value {
        confirm_prompt = confirm_prompt.with_default(default);
    } else {
        confirm_prompt = confirm_prompt.with_default(false);
    }

    let value = handle_inquire_error(confirm_prompt.prompt())?;

    // Jenkins expects "true" or "false" as strings
    Ok(value.to_string())
}

fn prompt_choice_parameter(param_def: &ParameterDefinition, help: &str) -> Result<String> {
    let choices = param_def.choices.as_ref()
        .context("ChoiceParameterDefinition missing choices")?;

    if choices.is_empty() {
        anyhow::bail!("ChoiceParameterDefinition has no choices");
    }

    let selection = handle_inquire_error(
        Select::new(&format!("{}:", param_def.name), choices.clone())
            .with_help_message(help)
            .prompt()
    )?;

    Ok(selection)
}

// Helper functions to extract default values
fn extract_default_string(param_def: &ParameterDefinition) -> Option<String> {
    param_def.default_value.as_ref()
        .and_then(|dv| dv.value.as_ref())
        .and_then(|value| {
            match value {
                serde_json::Value::String(s) => Some(s.clone()),
                serde_json::Value::Number(n) => Some(n.to_string()),
                _ => None,
            }
        })
}

fn extract_default_bool(param_def: &ParameterDefinition) -> Option<bool> {
    param_def.default_value.as_ref()
        .and_then(|dv| dv.value.as_ref())
        .and_then(|value| value.as_bool())
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

    #[test]
    fn test_extract_default_string_from_string_value() {
        use crate::client::{DefaultParameterValue, ParameterDefinition};

        let param_def = ParameterDefinition {
            class: "hudson.model.StringParameterDefinition".to_string(),
            name: "BRANCH".to_string(),
            param_type: "StringParameterDefinition".to_string(),
            description: None,
            default_value: Some(DefaultParameterValue {
                value: Some(serde_json::Value::String("main".to_string())),
            }),
            choices: None,
        };

        let result = extract_default_string(&param_def);
        assert_eq!(result, Some("main".to_string()));
    }

    #[test]
    fn test_extract_default_string_from_number_value() {
        use crate::client::{DefaultParameterValue, ParameterDefinition};

        let param_def = ParameterDefinition {
            class: "hudson.model.StringParameterDefinition".to_string(),
            name: "VERSION".to_string(),
            param_type: "StringParameterDefinition".to_string(),
            description: None,
            default_value: Some(DefaultParameterValue {
                value: Some(serde_json::Value::Number(42.into())),
            }),
            choices: None,
        };

        let result = extract_default_string(&param_def);
        assert_eq!(result, Some("42".to_string()));
    }

    #[test]
    fn test_extract_default_string_from_boolean_value() {
        use crate::client::{DefaultParameterValue, ParameterDefinition};

        let param_def = ParameterDefinition {
            class: "hudson.model.StringParameterDefinition".to_string(),
            name: "FLAG".to_string(),
            param_type: "StringParameterDefinition".to_string(),
            description: None,
            default_value: Some(DefaultParameterValue {
                value: Some(serde_json::Value::Bool(true)),
            }),
            choices: None,
        };

        let result = extract_default_string(&param_def);
        assert_eq!(result, None); // Boolean values should return None for string extraction
    }

    #[test]
    fn test_extract_default_string_no_default() {
        use crate::client::ParameterDefinition;

        let param_def = ParameterDefinition {
            class: "hudson.model.StringParameterDefinition".to_string(),
            name: "BRANCH".to_string(),
            param_type: "StringParameterDefinition".to_string(),
            description: None,
            default_value: None,
            choices: None,
        };

        let result = extract_default_string(&param_def);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_default_bool_true() {
        use crate::client::{DefaultParameterValue, ParameterDefinition};

        let param_def = ParameterDefinition {
            class: "hudson.model.BooleanParameterDefinition".to_string(),
            name: "DEPLOY".to_string(),
            param_type: "BooleanParameterDefinition".to_string(),
            description: None,
            default_value: Some(DefaultParameterValue {
                value: Some(serde_json::Value::Bool(true)),
            }),
            choices: None,
        };

        let result = extract_default_bool(&param_def);
        assert_eq!(result, Some(true));
    }

    #[test]
    fn test_extract_default_bool_false() {
        use crate::client::{DefaultParameterValue, ParameterDefinition};

        let param_def = ParameterDefinition {
            class: "hudson.model.BooleanParameterDefinition".to_string(),
            name: "SKIP_TESTS".to_string(),
            param_type: "BooleanParameterDefinition".to_string(),
            description: None,
            default_value: Some(DefaultParameterValue {
                value: Some(serde_json::Value::Bool(false)),
            }),
            choices: None,
        };

        let result = extract_default_bool(&param_def);
        assert_eq!(result, Some(false));
    }

    #[test]
    fn test_extract_default_bool_from_string_value() {
        use crate::client::{DefaultParameterValue, ParameterDefinition};

        let param_def = ParameterDefinition {
            class: "hudson.model.BooleanParameterDefinition".to_string(),
            name: "FLAG".to_string(),
            param_type: "BooleanParameterDefinition".to_string(),
            description: None,
            default_value: Some(DefaultParameterValue {
                value: Some(serde_json::Value::String("true".to_string())),
            }),
            choices: None,
        };

        let result = extract_default_bool(&param_def);
        assert_eq!(result, None); // String values should return None for bool extraction
    }

    #[test]
    fn test_extract_default_bool_no_default() {
        use crate::client::ParameterDefinition;

        let param_def = ParameterDefinition {
            class: "hudson.model.BooleanParameterDefinition".to_string(),
            name: "DEPLOY".to_string(),
            param_type: "BooleanParameterDefinition".to_string(),
            description: None,
            default_value: None,
            choices: None,
        };

        let result = extract_default_bool(&param_def);
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_default_string_with_float_number() {
        use crate::client::{DefaultParameterValue, ParameterDefinition};

        let param_def = ParameterDefinition {
            class: "hudson.model.StringParameterDefinition".to_string(),
            name: "THRESHOLD".to_string(),
            param_type: "StringParameterDefinition".to_string(),
            description: None,
            default_value: Some(DefaultParameterValue {
                value: Some(serde_json::json!(3.14)),
            }),
            choices: None,
        };

        let result = extract_default_string(&param_def);
        assert_eq!(result, Some("3.14".to_string()));
    }
}
