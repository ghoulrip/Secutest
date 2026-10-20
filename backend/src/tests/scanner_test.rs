#[cfg(test)]
mod tests {
    use crate::services::scanner::{analyze_response_body, ScanResult};
    use crate::models::{SeverityLevel, Vulnerability};

    #[test]
    fn test_analyze_response_body_xss() {
        let body = r#"<html><script>alert('xss')</script></html>"#;
        let vulns = analyze_response_body(body, "http://example.com");
        assert_eq!(vulns.len(), 1);
        assert_eq!(vulns[0].severity, SeverityLevel::Medium);
        assert!(vulns[0].title.contains("XSS"));
    }

    #[test]
    fn test_analyze_response_body_xss_no_script() {
        let body = r#"<html><p>Hello world</p></html>"#;
        let vulns = analyze_response_body(body, "http://example.com");
        assert!(vulns.is_empty());
    }

    #[test]
    fn test_analyze_response_body_sql_injection() {
        let body = r#"SELECT * FROM users WHERE id = 1"#;
        let vulns = analyze_response_body(body, "http://example.com");
        assert_eq!(vulns.len(), 1);
        assert_eq!(vulns[0].severity, SeverityLevel::High);
        assert!(vulns[0].title.contains("SQL Injection"));
    }

    #[test]
    fn test_analyze_response_body_no_patterns() {
        let body = r#"<!DOCTYPE html><html><body>No scripts or SQL here</body></html>"#;
        let vulns = analyze_response_body(body, "http://example.com");
        assert!(vulns.is_empty());
    }

    #[test]
    fn test_analyze_response_body_empty() {
        let body = "";
        let vulns = analyze_response_body(body, "http://example.com");
        assert!(vulns.is_empty());
    }
}