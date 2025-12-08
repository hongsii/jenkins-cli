use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::config::JenkinsHost;

pub struct JenkinsClient {
    client: Client,
    host: JenkinsHost,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JobInfo {
    pub name: String,
    pub url: String,
    pub color: Option<String>,
    #[serde(rename = "lastBuild")]
    pub last_build: Option<BuildInfo>,
    pub jobs: Option<Vec<SubJobInfo>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct SubJobInfo {
    pub name: String,
    pub url: String,
    pub color: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct BuildInfo {
    pub number: i32,
    pub url: String,
    pub result: Option<String>,
    pub building: Option<bool>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BuildDetails {
    pub number: i32,
    pub url: String,
    pub result: Option<String>,
    pub building: bool,
    pub timestamp: i64,
    pub duration: i64,
    #[serde(rename = "fullDisplayName")]
    pub full_display_name: String,
}

impl JenkinsClient {
    pub fn new(host: JenkinsHost) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, host }
    }

    pub fn get_root_jobs(&self) -> Result<Vec<SubJobInfo>> {
        let url = format!(
            "{}/api/json?tree=jobs[name,url,color]",
            self.host.host.trim_end_matches('/')
        );

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.host.user, Some(&self.host.token))
            .send()
            .context("Failed to send request")?;

        #[derive(Deserialize)]
        struct RootResponse {
            jobs: Vec<SubJobInfo>,
        }

        let root: RootResponse = response
            .error_for_status()
            .context("Request failed")?
            .json()
            .context("Failed to parse response")?;

        Ok(root.jobs)
    }

    pub fn get_job(&self, job_name: &str) -> Result<JobInfo> {
        let url = format!(
            "{}/job/{}/api/json",
            self.host.host.trim_end_matches('/'),
            job_name
        );

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.host.user, Some(&self.host.token))
            .send()
            .context("Failed to send request")?;

        if response.status() == StatusCode::NOT_FOUND {
            anyhow::bail!("Job '{}' not found", job_name);
        }

        response
            .error_for_status()
            .context("Request failed")?
            .json::<JobInfo>()
            .context("Failed to parse response")
    }

    pub fn get_build(&self, job_name: &str, build_number: i32) -> Result<BuildDetails> {
        let url = format!(
            "{}/job/{}/{}/api/json",
            self.host.host.trim_end_matches('/'),
            job_name,
            build_number
        );

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.host.user, Some(&self.host.token))
            .send()
            .context("Failed to send request")?;

        response
            .error_for_status()
            .context("Request failed")?
            .json::<BuildDetails>()
            .context("Failed to parse response")
    }

    pub fn get_console_log(&self, job_name: &str, build_number: i32) -> Result<String> {
        let url = format!(
            "{}/job/{}/{}/consoleText",
            self.host.host.trim_end_matches('/'),
            job_name,
            build_number
        );

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.host.user, Some(&self.host.token))
            .send()
            .context("Failed to send request")?;

        response
            .error_for_status()
            .context("Request failed")?
            .text()
            .context("Failed to read response")
    }

    pub fn trigger_build(&self, job_name: &str) -> Result<()> {
        let url = format!(
            "{}/job/{}/build",
            self.host.host.trim_end_matches('/'),
            job_name
        );

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.host.user, Some(&self.host.token))
            .send()
            .context("Failed to send request")?;

        response
            .error_for_status()
            .context("Failed to trigger build")?;

        Ok(())
    }

    pub fn get_job_url(&self, job_name: &str) -> String {
        format!(
            "{}/job/{}",
            self.host.host.trim_end_matches('/'),
            job_name
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_host() -> JenkinsHost {
        JenkinsHost {
            host: "https://jenkins.example.com".to_string(),
            user: "testuser".to_string(),
            token: "testtoken".to_string(),
        }
    }

    #[test]
    fn test_client_creation() {
        let host = create_test_host();
        let client = JenkinsClient::new(host.clone());
        assert_eq!(client.host.host, host.host);
        assert_eq!(client.host.user, host.user);
        assert_eq!(client.host.token, host.token);
    }

    #[test]
    fn test_get_job_url() {
        let host = create_test_host();
        let client = JenkinsClient::new(host);

        let url = client.get_job_url("my-job");
        assert_eq!(url, "https://jenkins.example.com/job/my-job");
    }

    #[test]
    fn test_get_job_url_with_trailing_slash() {
        let mut host = create_test_host();
        host.host = "https://jenkins.example.com/".to_string();
        let client = JenkinsClient::new(host);

        let url = client.get_job_url("my-job");
        assert_eq!(url, "https://jenkins.example.com/job/my-job");
    }

    #[test]
    fn test_get_job_url_build_number() {
        let host = create_test_host();
        let client = JenkinsClient::new(host);

        let base_url = client.get_job_url("my-job");
        let build_url = format!("{}/{}", base_url, 123);
        assert_eq!(build_url, "https://jenkins.example.com/job/my-job/123");
    }

    #[test]
    fn test_job_info_deserialization() {
        let json = r#"{
            "name": "test-job",
            "url": "https://jenkins.example.com/job/test-job/",
            "color": "blue",
            "lastBuild": {
                "number": 42,
                "url": "https://jenkins.example.com/job/test-job/42/",
                "result": "SUCCESS",
                "building": false,
                "timestamp": 1234567890000
            }
        }"#;

        let job_info: JobInfo = serde_json::from_str(json).unwrap();
        assert_eq!(job_info.name, "test-job");
        assert_eq!(job_info.color, Some("blue".to_string()));
        assert!(job_info.last_build.is_some());

        let last_build = job_info.last_build.unwrap();
        assert_eq!(last_build.number, 42);
        assert_eq!(last_build.result, Some("SUCCESS".to_string()));
        assert_eq!(last_build.building, Some(false));
    }

    #[test]
    fn test_job_info_with_subjobs_deserialization() {
        let json = r#"{
            "name": "folder",
            "url": "https://jenkins.example.com/job/folder/",
            "color": "notbuilt",
            "jobs": [
                {
                    "name": "sub-job-1",
                    "url": "https://jenkins.example.com/job/folder/job/sub-job-1/",
                    "color": "blue"
                },
                {
                    "name": "sub-job-2",
                    "url": "https://jenkins.example.com/job/folder/job/sub-job-2/",
                    "color": "red"
                }
            ]
        }"#;

        let job_info: JobInfo = serde_json::from_str(json).unwrap();
        assert_eq!(job_info.name, "folder");
        assert!(job_info.jobs.is_some());

        let jobs = job_info.jobs.unwrap();
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].name, "sub-job-1");
        assert_eq!(jobs[0].color, Some("blue".to_string()));
        assert_eq!(jobs[1].name, "sub-job-2");
        assert_eq!(jobs[1].color, Some("red".to_string()));
    }

    #[test]
    fn test_job_info_without_color() {
        let json = r#"{
            "name": "test-job",
            "url": "https://jenkins.example.com/job/test-job/"
        }"#;

        let job_info: JobInfo = serde_json::from_str(json).unwrap();
        assert_eq!(job_info.name, "test-job");
        assert_eq!(job_info.color, None);
        assert_eq!(job_info.last_build, None);
        assert_eq!(job_info.jobs, None);
    }

    #[test]
    fn test_job_info_deserialization_minimal_last_build() {
        let json = r#"{
            "name": "test-job",
            "url": "https://jenkins.example.com/job/test-job/",
            "color": "blue",
            "lastBuild": {
                "number": 42,
                "url": "https://jenkins.example.com/job/test-job/42/"
            }
        }"#;

        let job_info: JobInfo = serde_json::from_str(json).unwrap();
        assert_eq!(job_info.name, "test-job");
        assert!(job_info.last_build.is_some());

        let last_build = job_info.last_build.unwrap();
        assert_eq!(last_build.number, 42);
        assert_eq!(last_build.url, "https://jenkins.example.com/job/test-job/42/");
        assert_eq!(last_build.result, None);
        assert_eq!(last_build.building, None);
        assert_eq!(last_build.timestamp, None);
    }

    #[test]
    fn test_build_details_deserialization() {
        let json = r#"{
            "number": 42,
            "url": "https://jenkins.example.com/job/test-job/42/",
            "result": "SUCCESS",
            "building": false,
            "timestamp": 1234567890000,
            "duration": 5000,
            "fullDisplayName": "test-job #42"
        }"#;

        let build_details: BuildDetails = serde_json::from_str(json).unwrap();
        assert_eq!(build_details.number, 42);
        assert_eq!(build_details.result, Some("SUCCESS".to_string()));
        assert_eq!(build_details.duration, 5000);
        assert_eq!(build_details.full_display_name, "test-job #42");
    }
}
