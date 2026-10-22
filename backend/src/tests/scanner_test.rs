#[cfg(test)]
mod tests {
    use crate::models::{SeverityLevel, Vulnerability};
    use crate::services::scanner::{analyze_response_body, extract_host, COMMON_SUBDOMAINS};

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

    // ===== EDGE CASE TESTS (Phase 3.1) =====

    #[test]
    fn test_analyze_response_body_malformed_html() {
        // Malformed/unclosed tags should still detect patterns
        let body = r#"<html><script>alert('xss')"#; // unclosed script tag
        let vulns = analyze_response_body(body, "http://example.com");
        assert_eq!(vulns.len(), 1);
        assert!(vulns[0].title.contains("XSS"));
    }

    #[test]
    fn test_analyze_response_body_mixed_xss_and_sqli() {
        // Both patterns in same response should produce two findings
        let body = r#"<html><script>alert('xss')</script> SELECT * FROM users</html>"#;
        let vulns = analyze_response_body(body, "http://example.com");
        assert_eq!(vulns.len(), 2);
        let titles: Vec<&str> = vulns.iter().map(|v| v.title.as_str()).collect();
        assert!(titles.iter().any(|t| t.contains("XSS")));
        assert!(titles.iter().any(|t| t.contains("SQL Injection")));
    }

    #[test]
    fn test_analyze_response_body_unicode_payloads() {
        // Unicode/XSS bypass attempts
        let body = r#"<html><script>alert('😀🎉')</script></html>"#;
        let vulns = analyze_response_body(body, "http://example.com");
        assert_eq!(vulns.len(), 1);
        assert!(vulns[0].title.contains("XSS"));

        // RTL override characters
        let body_rtl = "<html>&#x202E;malicious</html>";
        let vulns = analyze_response_body(body_rtl, "http://example.com");
        // Should not crash, may or may not detect depending on patterns
        assert!(vulns.is_empty() || vulns.len() > 0);
    }

    #[test]
    fn test_analyze_response_body_very_long_body() {
        // DoS resilience - very large response body
        let long_body = "A".repeat(100_000);
        let vulns = analyze_response_body(&long_body, "http://example.com");
        assert!(vulns.is_empty());

        // Long body with pattern at the end
        let body_with_pattern = format!("{}<script>alert(1)</script>", "A".repeat(100_000));
        let vulns = analyze_response_body(&body_with_pattern, "http://example.com");
        assert_eq!(vulns.len(), 1);
    }

    #[test]
    fn test_analyze_response_body_whitespace_only() {
        let body = "   \n\t\r\n  ";
        let vulns = analyze_response_body(body, "http://example.com");
        assert!(vulns.is_empty());
    }

    #[test]
    fn test_analyze_response_body_null_bytes() {
        // Null bytes in response
        let body = "test\x00<script>alert(1)</script>";
        let vulns = analyze_response_body(body, "http://example.com");
        // Should handle null bytes gracefully
        assert!(vulns.is_empty() || vulns.len() > 0);
    }

    #[test]
    fn test_analyze_response_body_case_insensitive_xss() {
        // Case variations
        let cases = vec![
            "<SCRIPT>alert(1)</SCRIPT>",
            "<Script>alert(1)</Script>",
            "<sCrIpT>alert(1)</sCrIpT>",
        ];
        for body in cases {
            let vulns = analyze_response_body(body, "http://example.com");
            assert_eq!(vulns.len(), 1, "Failed for body: {}", body);
            assert!(vulns[0].title.contains("XSS"));
        }
    }

    #[test]
    fn test_analyze_response_body_case_insensitive_sqli() {
        let cases = vec![
            "SELECT * FROM users",
            "select * from users",
            "SeLeCt * FrOm UsErS",
            "INSERT INTO table VALUES (1)",
            "UPDATE users SET name='x'",
            "DELETE FROM users",
            "DROP TABLE users",
            "ALTER TABLE users ADD COLUMN x INT",
        ];
        for body in cases {
            let vulns = analyze_response_body(body, "http://example.com");
            assert_eq!(vulns.len(), 1, "Failed for body: {}", body);
            assert!(vulns[0].title.contains("SQL Injection"));
        }
    }

    #[test]
    fn test_analyze_response_body_multiple_xss_instances() {
        // Multiple script tags - should still report one XSS vuln (deduplication by design)
        let body = "<script>alert(1)</script><script>alert(2)</script>";
        let vulns = analyze_response_body(body, "http://example.com");
        // Current implementation produces one vuln per pattern type
        assert_eq!(vulns.len(), 1);
    }

    #[test]
    fn test_analyze_response_body_event_handlers() {
        // Inline event handlers
        let cases = vec![
            r#"<img src="x" onerror="alert(1)"/>"#,
            r#"<body onload="alert(1)">"#,
            r#"<input onfocus="alert(1)"/>"#,
        ];
        for body in cases {
            let vulns = analyze_response_body(body, "http://example.com");
            assert_eq!(vulns.len(), 1, "Failed for body: {}", body);
            assert!(vulns[0].title.contains("XSS"));
        }
    }

    // ===== SUBDOMAIN SCANNING TESTS (Phase 3.3) =====

    #[test]
    fn test_extract_host_https() {
        let host = extract_host("https://example.com/path");
        assert_eq!(host, Some("example.com".to_string()));
    }

    #[test]
    fn test_extract_host_http() {
        let host = extract_host("http://example.com");
        assert_eq!(host, Some("example.com".to_string()));
    }

    #[test]
    fn test_extract_host_no_scheme() {
        let host = extract_host("example.com");
        assert_eq!(host, Some("example.com".to_string()));
    }

    #[test]
    fn test_extract_host_invalid() {
        let host = extract_host("not a valid url");
        assert_eq!(host, None);
    }

    #[test]
    fn test_extract_host_with_port() {
        let host = extract_host("https://example.com:8080/path");
        assert_eq!(host, Some("example.com".to_string()));
    }

    #[test]
    fn test_extract_host_localhost() {
        let host = extract_host("http://localhost:3000");
        assert_eq!(host, Some("localhost".to_string()));
    }

    #[test]
    fn test_extract_host_ip_address() {
        let host = extract_host("https://192.168.1.1");
        assert_eq!(host, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_extract_host_subdomain() {
        let host = extract_host("https://api.example.com");
        assert_eq!(host, Some("api.example.com".to_string()));
    }

    // Test that subdomain result structure is correct
    #[test]
    fn test_subdomain_result_structure() {
        let result = SubdomainResult {
            subdomain: "www.example.com".to_string(),
            ip_address: Some("192.168.1.1".to_string()),
            reachable: true,
            scan_result: None,
        };

        assert_eq!(result.subdomain, "www.example.com");
        assert_eq!(result.ip_address, Some("192.168.1.1".to_string()));
        assert!(result.reachable);
        assert!(result.scan_result.is_none());
    }

    #[test]
    fn test_subdomain_result_unreachable() {
        let result = SubdomainResult {
            subdomain: "nonexistent.example.com".to_string(),
            ip_address: None,
            reachable: false,
            scan_result: None,
        };

        assert!(!result.reachable);
        assert!(result.ip_address.is_none());
        assert!(result.scan_result.is_none());
    }

    // Test that common subdomains list is populated
    #[test]
    fn test_common_subdomains_not_empty() {
        assert!(!COMMON_SUBDOMAINS.is_empty());
        assert!(COMMON_SUBDOMAINS.len() > 10);
    }

    #[test]
    fn test_common_subdomains_contains_expected() {
        assert!(COMMON_SUBDOMAINS.contains(&"www"));
        assert!(COMMON_SUBDOMAINS.contains(&"mail"));
        assert!(COMMON_SUBDOMAINS.contains(&"api"));
        assert!(COMMON_SUBDOMAINS.contains(&"admin"));
    }

    #[test]
    fn test_common_subdomains_unique() {
        let mut seen = std::collections::HashSet::new();
        for sub in *COMMON_SUBDOMAINS {
            assert!(seen.insert(*sub), "Duplicate subdomain: {}", sub);
        }
    }
}
