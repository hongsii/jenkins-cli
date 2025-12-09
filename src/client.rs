use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::config::JenkinsHost;
use crate::helpers::url::{build_api_url, build_job_url, normalize_host_url};

pub struct JenkinsClient {
    client: Client,
    host: JenkinsHost,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JobInfo {
    pub name: Option<String>,
    pub url: Option<String>,
    pub color: Option<String>,
    pub buildable: Option<bool>,
    #[serde(rename = "lastBuild")]
    pub last_build: Option<BuildInfo>,
    pub jobs: Option<Vec<SubJobInfo>>,
    pub property: Option<Vec<JobProperty>>,
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct JobProperty {
    #[serde(rename = "parameterDefinitions")]
    pub parameter_definitions: Option<Vec<ParameterDefinition>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ParameterDefinition {
    #[serde(rename = "_class")]
    pub class: String,
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: Option<String>,
    #[serde(rename = "defaultParameterValue")]
    pub default_value: Option<DefaultParameterValue>,
    pub choices: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct DefaultParameterValue {
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct ParameterValue {
    pub name: String,
    pub value: String,
}

impl JenkinsClient {
    pub fn new(host: JenkinsHost) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, host })
    }

    pub fn get_root_jobs(&self) -> Result<Vec<SubJobInfo>> {
        let url = format!(
            "{}?tree=jobs[name,url,color]",
            build_api_url(&self.host.host)
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
            "{}/api/json",
            build_job_url(&self.host.host, job_name)
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
            "{}/api/json",
            crate::helpers::url::build_build_url(&self.host.host, job_name, build_number)
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
            "{}/consoleText",
            crate::helpers::url::build_build_url(&self.host.host, job_name, build_number)
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

    pub fn get_job_parameters(&self, job_name: &str) -> Result<Vec<ParameterDefinition>> {
        let url = format!(
            "{}/api/json?tree=property[parameterDefinitions[*]]",
            build_job_url(&self.host.host, job_name)
        );

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.host.user, Some(&self.host.token))
            .send()
            .context("Failed to send request")?;

        let job_info: JobInfo = response
            .error_for_status()
            .context("Request failed")?
            .json()
            .context("Failed to parse response")?;

        // Extract parameter definitions from properties
        if let Some(properties) = job_info.property {
            for prop in properties {
                if let Some(params) = prop.parameter_definitions {
                    return Ok(params);
                }
            }
        }

        // No parameters found - return empty vec
        Ok(vec![])
    }

    pub fn trigger_build(&self, job_name: &str, parameters: Option<Vec<ParameterValue>>) -> Result<Option<String>> {
        let (url, form_data) = if let Some(params) = parameters {
            // Use buildWithParameters endpoint
            let url = format!(
                "{}/buildWithParameters",
                build_job_url(&self.host.host, job_name)
            );

            // Build form data: param1=value1&param2=value2
            let mut form_pairs: Vec<(String, String)> = Vec::new();
            for param in params {
                form_pairs.push((param.name, param.value));
            }

            (url, Some(form_pairs))
        } else {
            // Use regular build endpoint
            let url = format!(
                "{}/build",
                build_job_url(&self.host.host, job_name)
            );
            (url, None)
        };

        let mut request = self.client.post(&url)
            .basic_auth(&self.host.user, Some(&self.host.token));

        // Add form data if parameters exist
        if let Some(form) = form_data {
            request = request.form(&form);
        }

        let response = request
            .send()
            .context("Failed to send request")?;

        let response = response
            .error_for_status()
            .context("Failed to trigger build")?;

        // Get queue item location from Location header
        let queue_location = response
            .headers()
            .get("Location")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        Ok(queue_location)
    }

    /// Get build number from queue item
    pub fn get_build_number_from_queue(&self, queue_url: &str) -> Result<Option<i32>> {
        let api_url = format!("{}api/json", normalize_host_url(queue_url));

        let response = self
            .client
            .get(&api_url)
            .basic_auth(&self.host.user, Some(&self.host.token))
            .send()
            .context("Failed to query queue item")?;

        #[derive(Deserialize)]
        struct QueueItem {
            executable: Option<QueueExecutable>,
        }

        #[derive(Deserialize)]
        struct QueueExecutable {
            number: i32,
        }

        let queue_item: QueueItem = response
            .error_for_status()
            .context("Failed to get queue item")?
            .json()
            .context("Failed to parse queue response")?;

        Ok(queue_item.executable.map(|e| e.number))
    }

    /// Stream console log progressively (from start_bytes offset)
    pub fn get_console_log_progressive(&self, job_name: &str, build_number: i32, start: usize) -> Result<(String, usize, bool)> {
        let url = format!(
            "{}/logText/progressiveText?start={}",
            crate::helpers::url::build_build_url(&self.host.host, job_name, build_number),
            start
        );

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.host.user, Some(&self.host.token))
            .send()
            .context("Failed to send request")?;

        // Check X-More-Data header to see if build is still running
        let more_data = response
            .headers()
            .get("X-More-Data")
            .and_then(|v| v.to_str().ok())
            .map(|v| v == "true")
            .unwrap_or(false);

        // Get X-Text-Size header for next offset
        let text_size = response
            .headers()
            .get("X-Text-Size")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(start);

        let text = response
            .error_for_status()
            .context("Request failed")?
            .text()
            .context("Failed to read response")?;

        Ok((text, text_size, more_data))
    }

    pub fn get_job_url(&self, job_name: &str) -> String {
        build_job_url(&self.host.host, job_name)
    }

    /// Verify connection to Jenkins by making a simple API call
    pub fn verify_connection(&self) -> Result<()> {
        let url = build_api_url(&self.host.host);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.host.user, Some(&self.host.token))
            .send()
            .context("Failed to connect to Jenkins server")?;

        let status = response.status();

        match status {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                anyhow::bail!("Authentication failed. Please check your username and API token.")
            }
            StatusCode::NOT_FOUND => {
                anyhow::bail!("Jenkins server not found. Please check the URL.")
            }
            _ => {
                anyhow::bail!("Failed to connect to Jenkins: HTTP {}", status)
            }
        }
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
        let client = JenkinsClient::new(host.clone()).unwrap();
        assert_eq!(client.host.host, host.host);
        assert_eq!(client.host.user, host.user);
        assert_eq!(client.host.token, host.token);
    }

    #[test]
    fn test_get_job_url() {
        let host = create_test_host();
        let client = JenkinsClient::new(host).unwrap();

        let url = client.get_job_url("my-job");
        assert_eq!(url, "https://jenkins.example.com/job/my-job");
    }

    #[test]
    fn test_get_job_url_with_trailing_slash() {
        let mut host = create_test_host();
        host.host = "https://jenkins.example.com/".to_string();
        let client = JenkinsClient::new(host).unwrap();

        let url = client.get_job_url("my-job");
        assert_eq!(url, "https://jenkins.example.com/job/my-job");
    }

    #[test]
    fn test_get_job_url_build_number() {
        let host = create_test_host();
        let client = JenkinsClient::new(host).unwrap();

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
        assert_eq!(job_info.name, Some("test-job".to_string()));
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
        assert_eq!(job_info.name, Some("folder".to_string()));
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
        assert_eq!(job_info.name, Some("test-job".to_string()));
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
        assert_eq!(job_info.name, Some("test-job".to_string()));
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

    #[test]
    fn test_verify_connection_url_format() {
        // Test that verify_connection uses the correct URL format
        let host = create_test_host();
        let client = JenkinsClient::new(host).unwrap();

        // Verify the URL format is correct
        let expected_url = "https://jenkins.example.com/api/json";
        let url = format!("{}/api/json", client.host.host.trim_end_matches('/'));
        assert_eq!(url, expected_url);
    }

    #[test]
    fn test_verify_connection_url_with_trailing_slash() {
        let mut host = create_test_host();
        host.host = "https://jenkins.example.com/".to_string();
        let client = JenkinsClient::new(host).unwrap();

        // Verify trailing slash is handled correctly
        let expected_url = "https://jenkins.example.com/api/json";
        let url = format!("{}/api/json", client.host.host.trim_end_matches('/'));
        assert_eq!(url, expected_url);
    }

    #[test]
    fn test_parameter_definition_string_deserialization() {
        let json = r#"{
            "_class": "hudson.model.StringParameterDefinition",
            "name": "BRANCH",
            "type": "StringParameterDefinition",
            "description": "Git branch to build",
            "defaultParameterValue": {
                "value": "main"
            }
        }"#;

        let param: ParameterDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(param.name, "BRANCH");
        assert_eq!(param.class, "hudson.model.StringParameterDefinition");
        assert_eq!(param.param_type, "StringParameterDefinition");
        assert_eq!(param.description, Some("Git branch to build".to_string()));
        assert!(param.default_value.is_some());
        assert_eq!(param.choices, None);

        let default = param.default_value.unwrap();
        assert_eq!(default.value, Some(serde_json::Value::String("main".to_string())));
    }

    #[test]
    fn test_parameter_definition_boolean_deserialization() {
        let json = r#"{
            "_class": "hudson.model.BooleanParameterDefinition",
            "name": "DEPLOY",
            "type": "BooleanParameterDefinition",
            "description": "Deploy after build",
            "defaultParameterValue": {
                "value": true
            }
        }"#;

        let param: ParameterDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(param.name, "DEPLOY");
        assert_eq!(param.class, "hudson.model.BooleanParameterDefinition");
        assert_eq!(param.param_type, "BooleanParameterDefinition");
        assert_eq!(param.description, Some("Deploy after build".to_string()));
        assert!(param.default_value.is_some());

        let default = param.default_value.unwrap();
        assert_eq!(default.value, Some(serde_json::Value::Bool(true)));
    }

    #[test]
    fn test_parameter_definition_choice_deserialization() {
        let json = r#"{
            "_class": "hudson.model.ChoiceParameterDefinition",
            "name": "ENVIRONMENT",
            "type": "ChoiceParameterDefinition",
            "description": "Target environment",
            "choices": ["dev", "staging", "production"],
            "defaultParameterValue": {
                "value": "dev"
            }
        }"#;

        let param: ParameterDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(param.name, "ENVIRONMENT");
        assert_eq!(param.class, "hudson.model.ChoiceParameterDefinition");
        assert_eq!(param.param_type, "ChoiceParameterDefinition");
        assert!(param.choices.is_some());

        let choices = param.choices.unwrap();
        assert_eq!(choices.len(), 3);
        assert_eq!(choices[0], "dev");
        assert_eq!(choices[1], "staging");
        assert_eq!(choices[2], "production");
    }

    #[test]
    fn test_parameter_definition_without_description() {
        let json = r#"{
            "_class": "hudson.model.StringParameterDefinition",
            "name": "VERSION",
            "type": "StringParameterDefinition"
        }"#;

        let param: ParameterDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(param.name, "VERSION");
        assert_eq!(param.description, None);
        assert_eq!(param.default_value, None);
        assert_eq!(param.choices, None);
    }

    #[test]
    fn test_parameter_definition_with_number_default() {
        let json = r#"{
            "_class": "hudson.model.StringParameterDefinition",
            "name": "BUILD_NUMBER",
            "type": "StringParameterDefinition",
            "defaultParameterValue": {
                "value": 42
            }
        }"#;

        let param: ParameterDefinition = serde_json::from_str(json).unwrap();
        assert!(param.default_value.is_some());

        let default = param.default_value.unwrap();
        assert_eq!(default.value, Some(serde_json::Value::Number(42.into())));
    }

    #[test]
    fn test_job_property_deserialization() {
        let json = r#"{
            "parameterDefinitions": [
                {
                    "_class": "hudson.model.StringParameterDefinition",
                    "name": "BRANCH",
                    "type": "StringParameterDefinition"
                },
                {
                    "_class": "hudson.model.BooleanParameterDefinition",
                    "name": "DEPLOY",
                    "type": "BooleanParameterDefinition"
                }
            ]
        }"#;

        let prop: JobProperty = serde_json::from_str(json).unwrap();
        assert!(prop.parameter_definitions.is_some());

        let params = prop.parameter_definitions.unwrap();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "BRANCH");
        assert_eq!(params[1].name, "DEPLOY");
    }

    #[test]
    fn test_job_property_without_parameters() {
        let json = r#"{}"#;

        let prop: JobProperty = serde_json::from_str(json).unwrap();
        assert_eq!(prop.parameter_definitions, None);
    }

    #[test]
    fn test_job_info_with_property_deserialization() {
        let json = r#"{
            "name": "parameterized-job",
            "url": "https://jenkins.example.com/job/parameterized-job/",
            "color": "blue",
            "property": [
                {
                    "parameterDefinitions": [
                        {
                            "_class": "hudson.model.StringParameterDefinition",
                            "name": "BRANCH",
                            "type": "StringParameterDefinition",
                            "description": "Git branch"
                        }
                    ]
                }
            ]
        }"#;

        let job_info: JobInfo = serde_json::from_str(json).unwrap();
        assert_eq!(job_info.name, Some("parameterized-job".to_string()));
        assert!(job_info.property.is_some());

        let properties = job_info.property.unwrap();
        assert_eq!(properties.len(), 1);

        let param_defs = properties[0].parameter_definitions.as_ref().unwrap();
        assert_eq!(param_defs.len(), 1);
        assert_eq!(param_defs[0].name, "BRANCH");
    }

    #[test]
    fn test_job_info_without_property() {
        let json = r#"{
            "name": "simple-job",
            "url": "https://jenkins.example.com/job/simple-job/",
            "color": "blue"
        }"#;

        let job_info: JobInfo = serde_json::from_str(json).unwrap();
        assert_eq!(job_info.name, Some("simple-job".to_string()));
        assert_eq!(job_info.property, None);
    }

    #[test]
    fn test_parameter_value_creation() {
        let param_value = ParameterValue {
            name: "BRANCH".to_string(),
            value: "develop".to_string(),
        };

        assert_eq!(param_value.name, "BRANCH");
        assert_eq!(param_value.value, "develop");
    }
}
