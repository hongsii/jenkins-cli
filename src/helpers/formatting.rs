use console::style;

/// Format Jenkins job color/status for plain text display
pub fn format_job_color(color: Option<&str>) -> String {
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

/// Format Jenkins job color/status with console styling
pub fn format_job_color_styled(color: Option<&str>) -> String {
    match color {
        Some("blue") => style("Success").green().to_string(),
        Some("red") => style("Failed").red().to_string(),
        Some("yellow") => style("Unstable").yellow().to_string(),
        Some("aborted") => style("Aborted").dim().to_string(),
        Some("notbuilt") => style("Not Built").dim().to_string(),
        Some("disabled") => style("Disabled").dim().to_string(),
        Some(c) if c.ends_with("_anime") => {
            style(format!("Building ({})", c.trim_end_matches("_anime")))
                .cyan()
                .to_string()
        }
        Some(c) => c.to_string(),
        None => style("Unknown").dim().to_string(),
    }
}

/// Format Jenkins build result with console styling
pub fn format_build_result(result: &Option<String>) -> String {
    match result.as_deref() {
        Some("SUCCESS") => style("SUCCESS").green().to_string(),
        Some("FAILURE") => style("FAILURE").red().to_string(),
        Some("UNSTABLE") => style("UNSTABLE").yellow().to_string(),
        Some("ABORTED") => style("ABORTED").dim().to_string(),
        Some(r) => r.to_string(),
        None => style("IN_PROGRESS").cyan().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_job_color() {
        assert_eq!(format_job_color(Some("blue")), "Success");
        assert_eq!(format_job_color(Some("red")), "Failed");
        assert_eq!(format_job_color(Some("yellow")), "Unstable");
        assert_eq!(format_job_color(Some("aborted")), "Aborted");
        assert_eq!(format_job_color(Some("notbuilt")), "Not Built");
        assert_eq!(format_job_color(Some("disabled")), "Disabled");
        assert_eq!(format_job_color(Some("blue_anime")), "Building (blue)");
        assert_eq!(format_job_color(Some("red_anime")), "Building (red)");
        assert_eq!(format_job_color(None), "Unknown");
    }

    #[test]
    fn test_format_build_result() {
        // Note: We can't easily test the styled output, but we can test that it doesn't panic
        format_build_result(&Some("SUCCESS".to_string()));
        format_build_result(&Some("FAILURE".to_string()));
        format_build_result(&Some("UNSTABLE".to_string()));
        format_build_result(&Some("ABORTED".to_string()));
        format_build_result(&None);
    }
}
