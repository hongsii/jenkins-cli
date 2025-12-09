/// Normalize Jenkins host URL by removing trailing slash
pub fn normalize_host_url(host: &str) -> &str {
    host.trim_end_matches('/')
}

/// Build a Jenkins job URL
pub fn build_job_url(host: &str, job_name: &str) -> String {
    format!("{}/job/{}", normalize_host_url(host), job_name)
}

/// Build a Jenkins API URL
pub fn build_api_url(base_url: &str) -> String {
    format!("{}/api/json", normalize_host_url(base_url))
}

/// Build a Jenkins build URL
pub fn build_build_url(host: &str, job_name: &str, build_number: i32) -> String {
    format!(
        "{}/job/{}/{}",
        normalize_host_url(host),
        job_name,
        build_number
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_host_url() {
        assert_eq!(
            normalize_host_url("https://jenkins.example.com"),
            "https://jenkins.example.com"
        );
        assert_eq!(
            normalize_host_url("https://jenkins.example.com/"),
            "https://jenkins.example.com"
        );
        assert_eq!(
            normalize_host_url("https://jenkins.example.com///"),
            "https://jenkins.example.com"
        );
    }

    #[test]
    fn test_build_job_url() {
        assert_eq!(
            build_job_url("https://jenkins.example.com", "my-job"),
            "https://jenkins.example.com/job/my-job"
        );
        assert_eq!(
            build_job_url("https://jenkins.example.com/", "my-job"),
            "https://jenkins.example.com/job/my-job"
        );
    }

    #[test]
    fn test_build_api_url() {
        assert_eq!(
            build_api_url("https://jenkins.example.com"),
            "https://jenkins.example.com/api/json"
        );
        assert_eq!(
            build_api_url("https://jenkins.example.com/"),
            "https://jenkins.example.com/api/json"
        );
    }

    #[test]
    fn test_build_build_url() {
        assert_eq!(
            build_build_url("https://jenkins.example.com", "my-job", 123),
            "https://jenkins.example.com/job/my-job/123"
        );
        assert_eq!(
            build_build_url("https://jenkins.example.com/", "my-job", 123),
            "https://jenkins.example.com/job/my-job/123"
        );
    }
}
