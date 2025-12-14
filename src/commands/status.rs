use anyhow::Result;
use crate::helpers::formatting::{format_job_color_styled as format_color, format_build_result as format_result};
use crate::helpers::init::create_client_for_job;
use crate::interactive;
use crate::output;

pub fn execute(job_name: Option<String>, build_number: Option<i32>) -> Result<()> {
    let client = create_client_for_job(job_name.as_deref(), None)?;

    // Resolve the final job name (handle sub-jobs if present)
    let final_job_name = interactive::resolve_job_name(&client, job_name.as_deref())?;

    if let Some(build_num) = build_number {
        let sp = output::spinner("Fetching build details...");
        let build = client.get_build(&final_job_name, build_num)?;
        sp.finish_and_clear();
        print_build_details(&client, &final_job_name, &build);
    } else {
        let sp = output::spinner("Fetching job information...");
        let job = client.get_job(&final_job_name)?;
        sp.finish_and_clear();
        print_job_info(&client, &final_job_name, &job);
    }

    Ok(())
}

fn print_job_info(client: &crate::client::JenkinsClient, job_name: &str, job: &crate::client::JobInfo) {
    output::header(&format!("Job: {}", job.name.as_deref().unwrap_or("Unknown")));
    // Use configured host to build URL instead of API response URL
    output::list_item("URL:", &client.get_job_url(job_name));
    output::list_item("Status:", &format_color(job.color.as_deref()));

    if let Some(last_build) = &job.last_build {
        output::newline();
        output::highlight("Last Build:");
        output::list_item("Number:", &format!("#{}", last_build.number));
        output::list_item("Result:", &format_result(&last_build.result));
        output::list_item("Building:", &last_build.building.unwrap_or(false).to_string());
        // Use configured host to build build URL
        output::list_item("URL:", &format!("{}/{}", client.get_job_url(job_name), last_build.number));
    } else {
        output::info("No builds found");
    }
}

fn print_build_details(client: &crate::client::JenkinsClient, job_name: &str, build: &crate::client::BuildDetails) {
    output::header(&format!("Build: {}", build.full_display_name));
    output::list_item("Number:", &format!("#{}", build.number));
    output::list_item("Result:", &format_result(&build.result));
    output::list_item("Building:", &build.building.to_string());
    output::list_item("Duration:", &format!("{} ms", build.duration));
    // Use configured host to build build URL
    output::list_item("URL:", &format!("{}/{}", client.get_job_url(job_name), build.number));
}
