use anyhow::Result;
use crate::helpers::init::create_client_for_job;
use crate::interactive;
use crate::output;

pub fn execute(job_name: Option<String>, build_number: Option<i32>) -> Result<()> {
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

    let sp = output::spinner(&format!("Fetching console log for {}#{}...", final_job_name, build_num));
    let log = client.get_console_log(&final_job_name, build_num)?;
    sp.finish_and_clear();

    output::newline();
    println!("{}", log);

    Ok(())
}
