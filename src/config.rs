use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JenkinsHost {
    pub host: String,
    pub user: String,
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<String>,
    #[serde(default)]
    pub jenkins: HashMap<String, JenkinsHost>,
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

        if self.current.as_deref() == Some(name) {
            self.current = None;
        }

        Ok(())
    }

    pub fn get_current(&self) -> Result<(&String, &JenkinsHost)> {
        let name = self.current.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No Jenkins host is currently selected. Use 'jenkins config use <name>' to select one."))?;

        let host = self.jenkins.get(name)
            .ok_or_else(|| anyhow::anyhow!("Current Jenkins '{}' not found in config", name))?;

        Ok((name, host))
    }

    pub fn get_jenkins(&self, name: &str) -> Result<&JenkinsHost> {
        self.jenkins.get(name)
            .ok_or_else(|| anyhow::anyhow!("Jenkins '{}' not found", name))
    }

    pub fn set_current(&mut self, name: String) -> Result<()> {
        if !self.jenkins.contains_key(&name) {
            anyhow::bail!("Jenkins '{}' not found", name);
        }
        self.current = Some(name);
        Ok(())
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
        assert!(config.current.is_none());
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
    fn test_set_current() {
        let mut config = Config::default();
        config.add_jenkins("prod".to_string(), create_test_host("prod"));

        let result = config.set_current("prod".to_string());
        assert!(result.is_ok());
        assert_eq!(config.current, Some("prod".to_string()));
    }

    #[test]
    fn test_set_current_nonexistent() {
        let mut config = Config::default();
        let result = config.set_current("nonexistent".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_get_current() {
        let mut config = Config::default();
        config.add_jenkins("prod".to_string(), create_test_host("prod"));
        config.set_current("prod".to_string()).unwrap();

        let result = config.get_current();
        assert!(result.is_ok());
        let (name, host) = result.unwrap();
        assert_eq!(name, "prod");
        assert_eq!(host.host, "https://jenkins-prod.example.com");
    }

    #[test]
    fn test_get_current_when_none() {
        let config = Config::default();
        let result = config.get_current();
        assert!(result.is_err());
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
    fn test_remove_current_jenkins() {
        let mut config = Config::default();
        config.add_jenkins("prod".to_string(), create_test_host("prod"));
        config.set_current("prod".to_string()).unwrap();

        config.remove_jenkins("prod").unwrap();
        assert!(config.current.is_none());
    }

    #[test]
    fn test_yaml_serialization() {
        let mut config = Config::default();
        config.add_jenkins("prod".to_string(), create_test_host("prod"));
        config.set_current("prod".to_string()).unwrap();

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("current: prod"));
        assert!(yaml.contains("jenkins:"));
        assert!(yaml.contains("prod:"));
        assert!(yaml.contains("host: https://jenkins-prod.example.com"));
    }

    #[test]
    fn test_yaml_deserialization() {
        let yaml = r#"
current: prod
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
        assert_eq!(config.current, Some("prod".to_string()));
        assert_eq!(config.jenkins.len(), 2);
        assert!(config.jenkins.contains_key("prod"));
        assert!(config.jenkins.contains_key("dev"));
    }
}
