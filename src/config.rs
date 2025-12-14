use anyhow::{Context, Result};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JenkinsHost {
    pub host: String,
    pub user: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobAlias {
    pub job_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jenkins: Option<String>,
}

impl<'de> Deserialize<'de> for JobAlias {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum JobAliasHelper {
            Simple(String),
            Full {
                job_name: String,
                #[serde(default)]
                jenkins: Option<String>,
            },
        }

        match JobAliasHelper::deserialize(deserializer)? {
            JobAliasHelper::Simple(job_name) => Ok(JobAlias {
                job_name,
                jenkins: None,
            }),
            JobAliasHelper::Full { job_name, jenkins } => Ok(JobAlias { job_name, jenkins }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub jenkins: HashMap<String, JenkinsHost>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub job_aliases: HashMap<String, JobAlias>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&config_path)
            .context("Failed to read config file")?;

        let config: Config = serde_yaml::from_str(&content)
            .context("Failed to parse config file")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }

        let content = serde_yaml::to_string(self)
            .context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .context("Failed to write config file")?;

        Ok(())
    }

    pub fn add_jenkins(&mut self, name: String, host: JenkinsHost) {
        self.jenkins.insert(name, host);
    }

    pub fn remove_jenkins(&mut self, name: &str) -> Result<()> {
        if self.jenkins.remove(name).is_none() {
            anyhow::bail!("Jenkins '{}' not found", name);
        }
        Ok(())
    }

    pub fn get_jenkins(&self, name: &str) -> Result<&JenkinsHost> {
        self.jenkins.get(name)
            .ok_or_else(|| anyhow::anyhow!("Jenkins '{}' not found", name))
    }

    pub fn add_job_alias(&mut self, alias: String, job_name: String, jenkins: Option<String>) {
        self.job_aliases.insert(alias, JobAlias { job_name, jenkins });
    }

    pub fn remove_job_alias(&mut self, alias: &str) -> Result<()> {
        if self.job_aliases.remove(alias).is_none() {
            anyhow::bail!("Job alias '{}' not found", alias);
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_job_name(&self, alias_or_name: &str) -> String {
        self.job_aliases
            .get(alias_or_name)
            .map(|alias| alias.job_name.clone())
            .unwrap_or_else(|| alias_or_name.to_string())
    }

    pub fn resolve_job_name(&self, alias_or_name: &str) -> (String, bool, Option<String>) {
        if let Some(alias) = self.job_aliases.get(alias_or_name) {
            (alias.job_name.clone(), true, alias.jenkins.clone())
        } else {
            (alias_or_name.to_string(), false, None)
        }
    }

    fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Failed to get home directory")?;
        Ok(home.join(".config").join("jenkins-cli").join("config.yml"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_host(name: &str) -> JenkinsHost {
        JenkinsHost {
            host: format!("https://jenkins-{}.example.com", name),
            user: format!("user-{}", name),
            token: format!("token-{}", name),
        }
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.jenkins.is_empty());
    }

    #[test]
    fn test_add_jenkins() {
        let mut config = Config::default();
        let host = create_test_host("prod");

        config.add_jenkins("prod".to_string(), host.clone());

        assert_eq!(config.jenkins.len(), 1);
        assert!(config.jenkins.contains_key("prod"));
        assert_eq!(config.jenkins.get("prod").unwrap().host, host.host);
    }

    #[test]
    fn test_remove_jenkins() {
        let mut config = Config::default();
        config.add_jenkins("prod".to_string(), create_test_host("prod"));
        config.add_jenkins("dev".to_string(), create_test_host("dev"));

        let result = config.remove_jenkins("prod");
        assert!(result.is_ok());
        assert_eq!(config.jenkins.len(), 1);
        assert!(!config.jenkins.contains_key("prod"));
        assert!(config.jenkins.contains_key("dev"));
    }

    #[test]
    fn test_remove_nonexistent_jenkins() {
        let mut config = Config::default();
        let result = config.remove_jenkins("nonexistent");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Jenkins 'nonexistent' not found");
    }

    #[test]
    fn test_get_jenkins() {
        let mut config = Config::default();
        config.add_jenkins("prod".to_string(), create_test_host("prod"));

        let result = config.get_jenkins("prod");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().host, "https://jenkins-prod.example.com");
    }

    #[test]
    fn test_yaml_serialization() {
        let mut config = Config::default();
        config.add_jenkins("prod".to_string(), create_test_host("prod"));

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("jenkins:"));
        assert!(yaml.contains("prod:"));
        assert!(yaml.contains("host: https://jenkins-prod.example.com"));
    }

    #[test]
    fn test_yaml_deserialization() {
        let yaml = r#"
jenkins:
  prod:
    host: https://jenkins-prod.example.com
    user: user-prod
    token: token-prod
  dev:
    host: https://jenkins-dev.example.com
    user: user-dev
    token: token-dev
"#;

        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.jenkins.len(), 2);
        assert!(config.jenkins.contains_key("prod"));
        assert!(config.jenkins.contains_key("dev"));
    }

    #[test]
    fn test_add_job_alias() {
        let mut config = Config::default();
        config.add_job_alias("my-job".to_string(), "my-very-long-job-name".to_string(), None);

        assert_eq!(config.job_aliases.len(), 1);
        let alias = config.job_aliases.get("my-job").unwrap();
        assert_eq!(alias.job_name, "my-very-long-job-name");
        assert_eq!(alias.jenkins, None);
    }

    #[test]
    fn test_remove_job_alias() {
        let mut config = Config::default();
        config.add_job_alias("my-job".to_string(), "my-very-long-job-name".to_string(), None);

        let result = config.remove_job_alias("my-job");
        assert!(result.is_ok());
        assert!(config.job_aliases.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_job_alias() {
        let mut config = Config::default();
        let result = config.remove_job_alias("nonexistent");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Job alias 'nonexistent' not found");
    }

    #[test]
    fn test_get_job_name_with_alias() {
        let mut config = Config::default();
        config.add_job_alias("my-job".to_string(), "my-very-long-job-name".to_string(), None);

        assert_eq!(config.get_job_name("my-job"), "my-very-long-job-name");
    }

    #[test]
    fn test_get_job_name_without_alias() {
        let config = Config::default();
        assert_eq!(config.get_job_name("actual-job-name"), "actual-job-name");
    }

    #[test]
    fn test_resolve_job_name_with_alias() {
        let mut config = Config::default();
        config.add_job_alias("my-job".to_string(), "my-very-long-job-name".to_string(), None);

        let (job_name, is_alias, jenkins) = config.resolve_job_name("my-job");
        assert_eq!(job_name, "my-very-long-job-name");
        assert!(is_alias);
        assert_eq!(jenkins, None);
    }

    #[test]
    fn test_resolve_job_name_without_alias() {
        let config = Config::default();
        let (job_name, is_alias, jenkins) = config.resolve_job_name("actual-job-name");
        assert_eq!(job_name, "actual-job-name");
        assert!(!is_alias);
        assert_eq!(jenkins, None);
    }

    #[test]
    fn test_add_job_alias_with_jenkins() {
        let mut config = Config::default();
        config.add_job_alias("my-job".to_string(), "my-very-long-job-name".to_string(), Some("dev".to_string()));

        assert_eq!(config.job_aliases.len(), 1);
        let alias = config.job_aliases.get("my-job").unwrap();
        assert_eq!(alias.job_name, "my-very-long-job-name");
        assert_eq!(alias.jenkins, Some("dev".to_string()));
    }

    #[test]
    fn test_resolve_job_name_with_jenkins() {
        let mut config = Config::default();
        config.add_job_alias("my-job".to_string(), "my-very-long-job-name".to_string(), Some("dev".to_string()));

        let (job_name, is_alias, jenkins) = config.resolve_job_name("my-job");
        assert_eq!(job_name, "my-very-long-job-name");
        assert!(is_alias);
        assert_eq!(jenkins, Some("dev".to_string()));
    }

    #[test]
    fn test_yaml_serialization_with_job_aliases() {
        let mut config = Config::default();
        config.add_jenkins("prod".to_string(), create_test_host("prod"));
        config.add_job_alias("my-job".to_string(), "my-very-long-job-name".to_string(), None);

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("job_aliases:"));
        assert!(yaml.contains("my-job:"));
        assert!(yaml.contains("job_name: my-very-long-job-name"));
    }

    #[test]
    fn test_yaml_deserialization_with_job_aliases() {
        // Test backward compatibility - simple string format
        let yaml = r#"
jenkins:
  prod:
    host: https://jenkins-prod.example.com
    user: user-prod
    token: token-prod
job_aliases:
  my-job: my-very-long-job-name
  prod-build: production-build-job
"#;

        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.job_aliases.len(), 2);
        let alias1 = config.job_aliases.get("my-job").unwrap();
        assert_eq!(alias1.job_name, "my-very-long-job-name");
        assert_eq!(alias1.jenkins, None);

        let alias2 = config.job_aliases.get("prod-build").unwrap();
        assert_eq!(alias2.job_name, "production-build-job");
        assert_eq!(alias2.jenkins, None);
    }

    #[test]
    fn test_yaml_deserialization_with_job_aliases_full_format() {
        // Test new format with jenkins field
        let yaml = r#"
jenkins:
  prod:
    host: https://jenkins-prod.example.com
    user: user-prod
    token: token-prod
  dev:
    host: https://jenkins-dev.example.com
    user: user-dev
    token: token-dev
job_aliases:
  my-job:
    job_name: my-very-long-job-name
  dev-job:
    job_name: dev-build-job
    jenkins: dev
"#;

        let config: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.job_aliases.len(), 2);

        let alias1 = config.job_aliases.get("my-job").unwrap();
        assert_eq!(alias1.job_name, "my-very-long-job-name");
        assert_eq!(alias1.jenkins, None);

        let alias2 = config.job_aliases.get("dev-job").unwrap();
        assert_eq!(alias2.job_name, "dev-build-job");
        assert_eq!(alias2.jenkins, Some("dev".to_string()));
    }

    #[test]
    fn test_yaml_serialization_with_jenkins_in_alias() {
        let mut config = Config::default();
        config.add_jenkins("prod".to_string(), create_test_host("prod"));
        config.add_jenkins("dev".to_string(), create_test_host("dev"));
        config.add_job_alias("dev-job".to_string(), "dev-build-job".to_string(), Some("dev".to_string()));

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("job_aliases:"));
        assert!(yaml.contains("dev-job:"));
        assert!(yaml.contains("job_name: dev-build-job"));
        assert!(yaml.contains("jenkins: dev"));
    }
}
