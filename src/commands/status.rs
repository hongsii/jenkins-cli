use anyhow::Result;
use crate::client::JenkinsClient;
use crate::config::Config;
use crate::interactive;

pub fn execute(job_name: Option<String>, build_number: Option<i32>, jenkins_name: Option<String>) -> Result<()> {
    let config = Config::load()?;

    let host = if let Some(name) = jenkins_name {
        config.get_jenkins(&name)?.clone()
    } else {
        let (_, host) = config.get_current()?;
        host.clone()
    };

    let client = JenkinsClient::new(host);

    // Resolve the final job name (handle sub-jobs if present)
    let final_job_name = interactive::resolve_job_name(&client, job_name.as_deref())?;

    if let Some(build_num) = build_number {
        let build = client.get_build(&final_job_name, build_num)?;
        print_build_details(&build);
    } else {
        let job = client.get_job(&final_job_name)?;
        print_job_info(&job);
    }

    Ok(())
}

fn print_job_info(job: &crate::client::JobInfo) {
    println!("Job: {}", job.name);
    println!("URL: {}", job.url);
    println!("Status: {}", format_color(job.color.as_deref()));

    if let Some(last_build) = &job.last_build {
        println!("\nLast Build:");
        println!("  Number: #{}", last_build.number);
        println!("  Result: {}", format_result(&last_build.result));
        println!("  Building: {}", last_build.building.unwrap_or(false));
        println!("  URL: {}", last_build.url);
    } else {
        println!("\nNo builds found");
    }
}

fn print_build_details(build: &crate::client::BuildDetails) {
    println!("Build: {}", build.full_display_name);
    println!("Number: #{}", build.number);
    println!("Result: {}", format_result(&build.result));
    println!("Building: {}", build.building);
    println!("Duration: {} ms", build.duration);
    println!("URL: {}", build.url);
}

fn format_color(color: Option<&str>) -> String {
    match color {
        Some("blue") => "Success".to_string(),
        Some("red") => "Failed".to_string(),
        Some("yellow") => "Unstable".to_string(),
        Some("aborted") => "Aborted".to_string(),
        Some("notbuilt") => "Not Built".to_string(),
        Some(c) if c.ends_with("_anime") => format!("Building ({})", c.trim_end_matches("_anime")),
        Some(c) => c.to_string(),
        None => "Unknown".to_string(),
    }
}

fn format_result(result: &Option<String>) -> String {
    match result {
        Some(r) => r.clone(),
        None => "IN_PROGRESS".to_string(),
    }
}
