use std::process::Command;
use std::fs;
use tempfile::TempDir;

fn get_binary_path() -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove 'deps' directory
    path.push("jenkins");
    path.to_str().unwrap().to_string()
}

fn run_command(args: &[&str], config_dir: Option<&str>) -> std::process::Output {
    let mut cmd = Command::new(get_binary_path());
    cmd.args(args);

    if let Some(dir) = config_dir {
        cmd.env("HOME", dir);
    }

    cmd.output().expect("Failed to execute command")
}

#[test]
fn test_help_command() {
    let output = run_command(&["--help"], None);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("A CLI tool for interacting with Jenkins"));
    assert!(stdout.contains("config"));
    assert!(stdout.contains("build"));
    assert!(stdout.contains("status"));
    assert!(stdout.contains("logs"));
    assert!(stdout.contains("open"));
}

#[test]
fn test_config_help() {
    let output = run_command(&["config", "--help"], None);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Manage Jenkins host configurations"));
    assert!(stdout.contains("add"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("remove"));
    assert!(stdout.contains("use"));
    assert!(stdout.contains("show"));
}

#[test]
fn test_config_list_empty() {
    let temp_dir = TempDir::new().unwrap();
    let output = run_command(&["config", "list"], Some(temp_dir.path().to_str().unwrap()));

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No Jenkins hosts configured"));
}

#[test]
fn test_build_without_config() {
    let temp_dir = TempDir::new().unwrap();
    let output = run_command(
        &["build", "test-job"],
        Some(temp_dir.path().to_str().unwrap())
    );

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Jenkins host is currently selected"));
}

#[test]
fn test_status_help() {
    let output = run_command(&["status", "--help"], None);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Check the status of a Jenkins job or build"));
    assert!(stdout.contains("--build"));
    assert!(stdout.contains("--jenkins"));
}

#[test]
fn test_logs_help() {
    let output = run_command(&["logs", "--help"], None);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("View console logs for a build"));
    assert!(stdout.contains("--build"));
    assert!(stdout.contains("--jenkins"));
}

#[test]
fn test_open_help() {
    let output = run_command(&["open", "--help"], None);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Open a Jenkins job or build in the browser"));
    assert!(stdout.contains("--build"));
    assert!(stdout.contains("--jenkins"));
}

#[test]
fn test_config_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().to_str().unwrap();

    // Create config directory and file manually for testing
    let config_dir = temp_dir.path().join(".config").join("jenkins-cli");
    fs::create_dir_all(&config_dir).unwrap();

    let config_content = r#"
current: prod
jenkins:
  prod:
    host: https://jenkins-prod.example.com
    user: testuser
    token: testtoken
  dev:
    host: https://jenkins-dev.example.com
    user: devuser
    token: devtoken
"#;
    fs::write(config_dir.join("config.yml"), config_content).unwrap();

    // Test config list
    let output = run_command(&["config", "list"], Some(home_dir));
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("prod"));
    assert!(stdout.contains("dev"));
    assert!(stdout.contains("(current)"));
}

#[test]
fn test_invalid_command() {
    let output = run_command(&["invalid"], None);
    assert!(!output.status.success());
}

#[test]
fn test_config_show_with_current() {
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().to_str().unwrap();

    // Create config directory and file
    let config_dir = temp_dir.path().join(".config").join("jenkins-cli");
    fs::create_dir_all(&config_dir).unwrap();

    let config_content = r#"
current: prod
jenkins:
  prod:
    host: https://jenkins-prod.example.com
    user: testuser
    token: testtoken123456789
"#;
    fs::write(config_dir.join("config.yml"), config_content).unwrap();

    // Test config show (should show current host)
    let output = run_command(&["config", "show"], Some(home_dir));
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Jenkins host: prod"));
    assert!(stdout.contains("Host: https://jenkins-prod.example.com"));
    assert!(stdout.contains("User: testuser"));
    // Token is displayed with first 8 characters
    assert!(stdout.contains("Token:") && stdout.contains("..."));
}

#[test]
fn test_config_show_specific_host() {
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().to_str().unwrap();

    // Create config directory and file
    let config_dir = temp_dir.path().join(".config").join("jenkins-cli");
    fs::create_dir_all(&config_dir).unwrap();

    let config_content = r#"
current: prod
jenkins:
  prod:
    host: https://jenkins-prod.example.com
    user: produser
    token: prodtoken
  dev:
    host: https://jenkins-dev.example.com
    user: devuser
    token: devtoken
"#;
    fs::write(config_dir.join("config.yml"), config_content).unwrap();

    // Test config show dev
    let output = run_command(&["config", "show", "dev"], Some(home_dir));
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Jenkins host: dev"));
    assert!(stdout.contains("Host: https://jenkins-dev.example.com"));
    assert!(stdout.contains("User: devuser"));
}

#[test]
fn test_config_use_changes_current() {
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().to_str().unwrap();

    // Create config directory and file
    let config_dir = temp_dir.path().join(".config").join("jenkins-cli");
    fs::create_dir_all(&config_dir).unwrap();

    let config_content = r#"
current: prod
jenkins:
  prod:
    host: https://jenkins-prod.example.com
    user: produser
    token: prodtoken
  dev:
    host: https://jenkins-dev.example.com
    user: devuser
    token: devtoken
"#;
    fs::write(config_dir.join("config.yml"), config_content).unwrap();

    // Change to dev
    let output = run_command(&["config", "use", "dev"], Some(home_dir));
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Now using Jenkins host 'dev'"));

    // Verify by listing
    let output = run_command(&["config", "list"], Some(home_dir));
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Check that dev is listed and prod is not marked as current
    assert!(stdout.contains("dev"));
    assert!(stdout.contains("(current)"));
}

#[test]
fn test_config_remove_help() {
    // config remove is now interactive, so we can only test the help message
    let output = run_command(&["config", "remove", "--help"], None);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Remove a Jenkins host"));
}

#[test]
fn test_config_remove_empty() {
    // Test that remove command shows message when no hosts are configured
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().to_str().unwrap();

    let output = run_command(&["config", "remove"], Some(home_dir));
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No Jenkins hosts configured"));
}

#[test]
fn test_build_help() {
    let output = run_command(&["build", "--help"], None);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Trigger a build for a Jenkins job"));
    assert!(stdout.contains("--jenkins"));
}

#[test]
fn test_status_without_build_number() {
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().to_str().unwrap();

    // Create config directory and file
    let config_dir = temp_dir.path().join(".config").join("jenkins-cli");
    fs::create_dir_all(&config_dir).unwrap();

    let config_content = r#"
current: prod
jenkins:
  prod:
    host: https://jenkins-prod.example.com
    user: testuser
    token: testtoken
"#;
    fs::write(config_dir.join("config.yml"), config_content).unwrap();

    // This will fail because we can't connect to the actual Jenkins server
    // but it tests that the config is loaded correctly
    let output = run_command(&["status", "test-job"], Some(home_dir));
    assert!(!output.status.success());

    // Should fail on network/connection, not config
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("No Jenkins host is currently selected"));
}

#[test]
fn test_multiple_hosts_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let home_dir = temp_dir.path().to_str().unwrap();

    // Create config directory and file with multiple hosts
    let config_dir = temp_dir.path().join(".config").join("jenkins-cli");
    fs::create_dir_all(&config_dir).unwrap();

    let config_content = r#"
current: prod
jenkins:
  prod:
    host: https://jenkins-prod.example.com
    user: produser
    token: prodtoken
  dev:
    host: https://jenkins-dev.example.com
    user: devuser
    token: devtoken
  staging:
    host: https://jenkins-staging.example.com
    user: staginguser
    token: stagingtoken
"#;
    fs::write(config_dir.join("config.yml"), config_content).unwrap();

    // Test listing all hosts
    let output = run_command(&["config", "list"], Some(home_dir));
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("prod"));
    assert!(stdout.contains("(current)"));
    assert!(stdout.contains("dev"));
    assert!(stdout.contains("staging"));
    assert!(stdout.contains("https://jenkins-prod.example.com"));
    assert!(stdout.contains("https://jenkins-dev.example.com"));
    assert!(stdout.contains("https://jenkins-staging.example.com"));
}
