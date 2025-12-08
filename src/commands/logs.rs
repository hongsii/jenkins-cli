use anyhow::Result;
use crate::client::JenkinsClient;
use crate::config::Config;
use crate::interactive;
use crate::output;

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

    let build_num = if let Some(num) = build_number {
        num
    } else {
        let job = client.get_job(&final_job_name)?;
        job.last_build
            .map(|b| b.number)
            .ok_or_else(|| anyhow::anyhow!("No builds found for job '{}'", final_job_name))?
    };

    output::info(&format!("Fetching console log for {}#{}...", final_job_name, build_num));
    println!();

    let log = client.get_console_log(&final_job_name, build_num)?;
    println!("{}", log);

    Ok(())
}
