use anyhow::Result;
use crate::client::JenkinsClient;
use crate::config::Config;
use crate::interactive;
use crate::output;
use console::style;

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
        let sp = output::spinner("Fetching build details...");
        let build = client.get_build(&final_job_name, build_num)?;
        sp.finish_and_clear();
        print_build_details(&build);
    } else {
        let sp = output::spinner("Fetching job information...");
        let job = client.get_job(&final_job_name)?;
        sp.finish_and_clear();
        print_job_info(&job);
    }

    Ok(())
}

fn print_job_info(job: &crate::client::JobInfo) {
    output::header(&format!("Job: {}", job.name.as_deref().unwrap_or("Unknown")));
    output::list_item("URL:", job.url.as_deref().unwrap_or("N/A"));
    output::list_item("Status:", &format_color(job.color.as_deref()));

    if let Some(last_build) = &job.last_build {
        output::newline();
        output::highlight("Last Build:");
        output::list_item("Number:", &format!("#{}", last_build.number));
        output::list_item("Result:", &format_result(&last_build.result));
        output::list_item("Building:", &last_build.building.unwrap_or(false).to_string());
        output::list_item("URL:", &last_build.url);
    } else {
        output::info("No builds found");
    }
}

fn print_build_details(build: &crate::client::BuildDetails) {
    output::header(&format!("Build: {}", build.full_display_name));
    output::list_item("Number:", &format!("#{}", build.number));
    output::list_item("Result:", &format_result(&build.result));
    output::list_item("Building:", &build.building.to_string());
    output::list_item("Duration:", &format!("{} ms", build.duration));
    output::list_item("URL:", &build.url);
}

fn format_color(color: Option<&str>) -> String {
    match color {
        Some("blue") => style("Success").green().to_string(),
        Some("red") => style("Failed").red().to_string(),
        Some("yellow") => style("Unstable").yellow().to_string(),
        Some("aborted") => style("Aborted").dim().to_string(),
        Some("notbuilt") => style("Not Built").dim().to_string(),
        Some(c) if c.ends_with("_anime") => style(format!("Building ({})", c.trim_end_matches("_anime"))).cyan().to_string(),
        Some(c) => c.to_string(),
        None => style("Unknown").dim().to_string(),
    }
}

fn format_result(result: &Option<String>) -> String {
    match result.as_deref() {
        Some("SUCCESS") => style("SUCCESS").green().to_string(),
        Some("FAILURE") => style("FAILURE").red().to_string(),
        Some("UNSTABLE") => style("UNSTABLE").yellow().to_string(),
        Some("ABORTED") => style("ABORTED").dim().to_string(),
        Some(r) => r.to_string(),
        None => style("IN_PROGRESS").cyan().to_string(),
    }
}
