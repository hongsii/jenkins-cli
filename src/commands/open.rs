use anyhow::Result;
use crate::helpers::init::create_client;
use crate::interactive;
use crate::output;
use std::process::Command;

pub fn execute(job_name: Option<String>, build_number: Option<i32>, jenkins_name: Option<String>) -> Result<()> {
    let client = create_client(jenkins_name)?;

    // Resolve the job name (allow stopping at any level for open command)
    let final_job_name = interactive::resolve_job_name_for_open(&client, job_name.as_deref())?;

    let url = if let Some(build_num) = build_number {
        format!("{}/{}", client.get_job_url(&final_job_name), build_num)
    } else {
        client.get_job_url(&final_job_name)
    };

    output::info(&format!("Opening {}...", url));

    #[cfg(target_os = "macos")]
    Command::new("open").arg(&url).spawn()?;

    #[cfg(target_os = "linux")]
    Command::new("xdg-open").arg(&url).spawn()?;

    #[cfg(target_os = "windows")]
    Command::new("cmd").args(&["/C", "start", &url]).spawn()?;

    output::success("Browser opened successfully!");

    Ok(())
}
