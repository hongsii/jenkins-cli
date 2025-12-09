use anyhow::Result;
use crate::client::JenkinsClient;
use crate::config::Config;
use crate::interactive;
use crate::output;
use std::thread;
use std::time::Duration;

pub fn execute(job_name: Option<String>, jenkins_name: Option<String>, follow: bool) -> Result<()> {
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

    output::info(&format!("Triggering build for job '{}'...", final_job_name));
    let queue_location = client.trigger_build(&final_job_name)?;

    output::success("Build triggered successfully!");

    if !follow {
        output::tip(&format!("Use 'jenkins status {}' to check build status", final_job_name));
        return Ok(());
    }

    // Follow the build logs
    if let Some(queue_url) = queue_location {
        output::info("Waiting for build to start...");

        // Poll queue until build starts (with timeout)
        let mut attempts = 0;
        let max_attempts = 30; // 30 seconds max wait
        let build_number = loop {
            thread::sleep(Duration::from_secs(1));
            attempts += 1;

            match client.get_build_number_from_queue(&queue_url) {
                Ok(Some(num)) => break Some(num),
                Ok(None) => {
                    if attempts >= max_attempts {
                        output::warning("Timeout waiting for build to start");
                        break None;
                    }
                    continue;
                }
                Err(_) => {
                    // Queue item might be gone - try to get last build number
                    match client.get_job(&final_job_name) {
                        Ok(job) => {
                            if let Some(last_build) = job.last_build {
                                output::info("Build already started, using last build number");
                                break Some(last_build.number);
                            }
                        }
                        Err(_) => {}
                    }

                    if attempts >= max_attempts {
                        output::warning("Could not determine build number");
                        break None;
                    }
                    continue;
                }
            }
        };

        let build_number = match build_number {
            Some(num) => num,
            None => {
                output::tip(&format!("Use 'jenkins logs {}' to view logs later", final_job_name));
                return Ok(());
            }
        };

        output::success(&format!("Build #{} started", build_number));
        output::header("Console Output");
        output::newline();

        // Stream logs
        let mut offset = 0;
        loop {
            match client.get_console_log_progressive(&final_job_name, build_number, offset) {
                Ok((text, new_offset, more_data)) => {
                    if !text.is_empty() {
                        print!("{}", text);
                    }
                    offset = new_offset;

                    if !more_data {
                        output::newline();
                        output::success("Build finished");
                        break;
                    }

                    thread::sleep(Duration::from_millis(500));
                }
                Err(e) => {
                    output::warning(&format!("Failed to fetch logs: {}", e));
                    break;
                }
            }
        }
    } else {
        output::warning("Could not get queue location to follow build");
        output::tip(&format!("Use 'jenkins status {}' to check build status", final_job_name));
    }

    Ok(())
}
