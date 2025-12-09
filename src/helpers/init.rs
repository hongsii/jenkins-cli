use anyhow::Result;
use crate::client::JenkinsClient;
use crate::config::{Config, JenkinsHost};

/// Create a JenkinsClient with the specified or current host
pub fn create_client(jenkins_name: Option<String>) -> Result<JenkinsClient> {
    let host = resolve_jenkins_host(jenkins_name)?;
    JenkinsClient::new(host)
}

/// Load config and get the specified or current Jenkins host
pub fn resolve_jenkins_host(jenkins_name: Option<String>) -> Result<JenkinsHost> {
    let config = Config::load()?;

    let host = if let Some(name) = jenkins_name {
        config.get_jenkins(&name)?.clone()
    } else {
        let (_, host) = config.get_current()?;
        host.clone()
    };

    Ok(host)
}
