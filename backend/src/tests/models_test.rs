#[cfg(test)]
mod tests {
    use crate::models::{ScanJob, ScanStatus, ScanResult, Vulnerability, SeverityLevel, CreateScanRequest, ScanOptions, JobUpdate};
    use chrono::Utc;
    use uuid::Uuid;

    fn make_scan_job(status: ScanStatus) -> ScanJob {
        ScanJob {
            id: Uuid::new_v4().to_string(),
            target: "https://example.com".to_string(),
            status,
            progress: 0.5,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
            result: None,
        }
    }

    #[test]
    fn test_scan_status_serialization() {
        let queued = ScanStatus::Queued;
        let running = ScanStatus::Running;
        let completed = ScanStatus::Completed;
        let failed = ScanStatus::Failed;

        let queued_json = serde_json::to_string(&queued).unwrap();
        let running_json = serde_json::to_string(&running).unwrap();
        let completed_json = serde_json::to_string(&completed).unwrap();
        let failed_json = serde_json::to_string(&failed).unwrap();

        assert_eq!(queued_json, "\"queued\"");
        assert_eq!(running_json, "\"running\"");
        assert_eq!(completed_json, "\"completed\"");
        assert_eq!(failed_json, "\"failed\"");
    }

    #[test]
    fn test_scan_status_deserialization() {
        let queued: ScanStatus = serde_json::from_str("\"queued\"").unwrap();
        let running: ScanStatus = serde_json::from_str("\"running\"").unwrap();
        let completed: ScanStatus = serde_json::from_str("\"completed\"").unwrap();
        let failed: ScanStatus = serde_json::from_str("\"failed\"").unwrap();

        assert_eq!(queued, ScanStatus::Queued);
        assert_eq!(running, ScanStatus::Running);
        assert_eq!(completed, ScanStatus::Completed);
        assert_eq!(failed, ScanStatus::Failed);
    }

    #[test]
    fn test_severity_level_serialization() {
        let low = SeverityLevel::Low;
        let medium = SeverityLevel::Medium;
        let high = SeverityLevel::High;
        let critical = SeverityLevel::Critical;

        assert_eq!(serde_json::to_string(&low).unwrap(), "\"low\"");
        assert_eq!(serde_json::to_string(&medium).unwrap(), "\"medium\"");
        assert_eq!(serde_json::to_string(&high).unwrap(), "\"high\"");
        assert_eq!(serde_json::to_string(&critical).unwrap(), "\"critical\"");
    }

    #[test]
    fn test_severity_level_deserialization() {
        let low: SeverityLevel = serde_json::from_str("\"low\"").unwrap();
        let medium: SeverityLevel = serde_json::from_str("\"medium\"").unwrap();
        let high: SeverityLevel = serde_json::from_str("\"high\"").unwrap();
        let critical: SeverityLevel = serde_json::from_str("\"critical\"").unwrap();

        assert_eq!(low, SeverityLevel::Low);
        assert_eq!(medium, SeverityLevel::Medium);
        assert_eq!(high, SeverityLevel::High);
        assert_eq!(critical, SeverityLevel::Critical);
    }

    #[test]
    fn test_create_scan_request_serialization() {
        let req = CreateScanRequest {
            url: "https://example.com".to_string(),
            options: Some(ScanOptions {
                include_subdomains: Some(true),
                follow_redirects: Some(false),
                max_depth: Some(3),
            }),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("https://example.com"));
        assert!(json.contains("include_subdomains"));
        assert!(json.contains("follow_redirects"));
        assert!(json.contains("max_depth"));
    }

    #[test]
    fn test_create_scan_request_without_options() {
        let req = CreateScanRequest {
            url: "https://example.com".to_string(),
            options: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("https://example.com"));
        assert!(!json.contains("options"));
    }

    #[test]
    fn test_job_update_partial_fields() {
        let update = JobUpdate {
            job_id: "test-123".to_string(),
            progress: Some(0.75),
            status: None,
            result: None,
        };

        assert_eq!(update.job_id, "test-123");
        assert_eq!(update.progress, Some(0.75));
        assert_eq!(update.status, None);
    }

    #[test]
    fn test_vulnerability_serialization() {
        let vuln = Vulnerability {
            id: "vuln-1".to_string(),
            severity: SeverityLevel::High,
            title: "SQL Injection".to_string(),
            description: "Unsanitized input".to_string(),
            cve: Some("CVE-2021-1234".to_string()),
            affected_url: "/api/users".to_string(),
            recommendation: "Use parameterized queries".to_string(),
        };

        let json = serde_json::to_string(&vuln).unwrap();
        assert!(json.contains("SQL Injection"));
        assert!(json.contains("high"));
        assert!(json.contains("CVE-2021-1234"));
        assert!(json.contains("parameterized queries"));
    }

    #[test]
    fn test_scan_result_with_vulnerabilities() {
        let result = ScanResult {
            url: "https://example.com".to_string(),
            ip_address: Some("192.168.1.1".to_string()),
            open_ports: vec![80, 443],
            vulnerabilities: vec![Vulnerability {
                id: "v1".to_string(),
                severity: SeverityLevel::Critical,
                title: "RCE".to_string(),
                description: "Remote code execution".to_string(),
                cve: None,
                affected_url: "/admin".to_string(),
                recommendation: "Patch immediately".to_string(),
            }],
            risk_score: 9.8,
            scanned_at: Utc::now().to_rfc3339(),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("192.168.1.1"));
        assert!(json.contains("80"));
        assert!(json.contains("443"));
        assert!(json.contains("RCE"));
        assert!(json.contains("9.8"));
    }

    #[test]
    fn test_scan_job_equality() {
        let job1 = make_scan_job(ScanStatus::Queued);
        let job2 = ScanJob {
            id: job1.id.clone(),
            ..job1.clone()
        };
        assert_eq!(job1.id, job2.id);
        assert_eq!(job1.status, job2.status);
    }

    #[test]
    fn test_scan_status_equality() {
        assert_eq!(ScanStatus::Queued, ScanStatus::Queued);
        assert_eq!(ScanStatus::Running, ScanStatus::Running);
        assert_ne!(ScanStatus::Queued, ScanStatus::Running);
        assert_ne!(ScanStatus::Completed, ScanStatus::Failed);
    }
}