use anyhow::Result;
use crate::helpers::init::create_client_for_job;
use crate::interactive;
use crate::output;
use std::thread;
use std::time::Duration;

pub fn execute(job_name: Option<String>, build_number: Option<i32>, follow: bool) -> Result<()> {
    let client = create_client_for_job(job_name.as_deref(), None)?;

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

    if !follow {
        // Original behavior - fetch full log once
        let sp = output::spinner(&format!("Fetching console log for {}#{}...", final_job_name, build_num));
        let log = client.get_console_log(&final_job_name, build_num)?;
        sp.finish_and_clear();

        output::newline();
        println!("{}", log);
    } else {
        // Follow mode - stream logs in real-time
        output::header(&format!("Console Output for {}#{}", final_job_name, build_num));
        output::newline();

        let sp = output::spinner("Streaming build logs...");
        let mut offset = 0;
        loop {
            match client.get_console_log_progressive(&final_job_name, build_num, offset) {
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
    }

    Ok(())
}
