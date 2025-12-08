use anyhow::Result;
use crate::client::JenkinsClient;
use crate::config::Config;
use crate::interactive;

pub fn execute(job_name: Option<String>, jenkins_name: Option<String>) -> Result<()> {
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

    println!("Triggering build for job '{}'...", final_job_name);
    client.trigger_build(&final_job_name)?;

    println!("Build triggered successfully!");
    println!("Use 'jenkins status {}' to check build status", final_job_name);

    Ok(())
}
