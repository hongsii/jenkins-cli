use anyhow::Result;
use crate::helpers::init::create_client;
use crate::interactive;
use crate::output;
use std::thread;
use std::time::Duration;

pub fn execute(job_name: Option<String>, jenkins_name: Option<String>, follow: bool) -> Result<()> {
    let client = create_client(jenkins_name)?;

    // Resolve the final job name (handle sub-jobs if present)
    let final_job_name = interactive::resolve_job_name(&client, job_name.as_deref())?;

    // Check if job is buildable
    let sp = output::spinner("Checking job status...");
    let job_info = client.get_job(&final_job_name)?;
    sp.finish_and_clear();

    // Verify job is buildable
    if let Some(buildable) = job_info.buildable {
        if !buildable {
            let reason = match job_info.color.as_deref() {
                Some("disabled") => "The job is disabled",
                _ => "The job is not buildable",
            };
            anyhow::bail!("{reason}. Please check the job configuration in Jenkins.");
        }
    }

    // Fetch and collect parameters
    let sp = output::spinner("Checking job parameters...");
    let parameter_definitions = client.get_job_parameters(&final_job_name)?;
    sp.finish_and_clear();

    let parameters = if !parameter_definitions.is_empty() {
        let param_values = interactive::collect_parameters(parameter_definitions)?;
        Some(param_values)
    } else {
        None
    };

    let sp = output::spinner(&format!("Triggering build for job '{}'...", final_job_name));
    let queue_location = client.trigger_build(&final_job_name, parameters)?;

    let job_url = client.get_job_url(&final_job_name);
    output::finish_spinner_success(sp, &format!("Build triggered successfully! => {}", job_url));

    if !follow {
        return Ok(());
    }

    // Follow the build logs
    if let Some(queue_url) = queue_location {
        let sp = output::spinner("Waiting for build to start...");

        // Poll queue until build starts (with timeout)
        let mut attempts = 0;
        let max_attempts = 30; // 30 seconds max wait
        let build_number = loop {
            thread::sleep(Duration::from_secs(1));
            attempts += 1;
            sp.set_message(format!("Waiting for build to start... ({}/30s)", attempts));

            match client.get_build_number_from_queue(&queue_url) {
                Ok(Some(num)) => {
                    output::finish_spinner_success(sp, &format!("Build #{} started", num));
                    break Some(num);
                }
                Ok(None) => {
                    if attempts >= max_attempts {
                        output::finish_spinner_warning(sp, "Timeout waiting for build to start");
                        break None;
                    }
                    continue;
                }
                Err(_) => {
                    // Queue item might be gone - try to get last build number
                    match client.get_job(&final_job_name) {
                        Ok(job) => {
                            if let Some(last_build) = job.last_build {
                                output::finish_spinner_success(sp, &format!("Build #{} already started", last_build.number));
                                break Some(last_build.number);
                            }
                        }
                        Err(_) => {}
                    }

                    if attempts >= max_attempts {
                        output::finish_spinner_warning(sp, "Could not determine build number");
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

        output::header("Console Output");
        output::newline();

        // Stream logs
        let sp = output::spinner("Streaming build logs...");
        let mut offset = 0;
        loop {
            match client.get_console_log_progressive(&final_job_name, build_number, offset) {
                Ok((text, new_offset, more_data)) => {
                    if !text.is_empty() {
                        sp.suspend(|| print!("{}", text));
                    }
                    offset = new_offset;

                    if !more_data {
                        sp.finish_and_clear();
                        output::newline();
                        output::success("Build finished");
                        break;
                    }

                    thread::sleep(Duration::from_millis(500));
                }
                Err(e) => {
                    output::finish_spinner_warning(sp, "Failed to fetch logs");
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
