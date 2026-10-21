/// Scanner service for running HTTP scans against targets
pub use crate::models::{ScanResult, SeverityLevel, Vulnerability};
use reqwest::Client;
use uuid::Uuid;

const SCAN_TIMEOUT_SECS: u64 = 30;
const CONNECT_TIMEOUT_SECS: u64 = 10;

/// Extract body analysis logic to make it unit testable
pub(crate) fn analyze_response_body(body: &str, target: &str) -> Vec<Vulnerability> {
    let mut vulnerabilities: Vec<Vulnerability> = Vec::new();
    let body_lower = body.to_lowercase();

    if body_lower.contains("<script>") || body_lower.contains("alert(") {
        vulnerabilities.push(Vulnerability {
            id: Uuid::new_v4().to_string(),
            severity: SeverityLevel::Medium,
            title: "Potential Cross-site Scripting (XSS)".to_string(),
            description: "Response contains script tags or inline event handlers, which may indicate an XSS vulnerability".to_string(),
            cve: None,
            affected_url: target.to_string(),
            recommendation: "Implement proper output encoding and a Content Security Policy (CSP)".to_string(),
        });
    }

    if body_lower.contains("sql")
        && (body_lower.contains("select") || body_lower.contains("insert"))
    {
        vulnerabilities.push(Vulnerability {
            id: Uuid::new_v4().to_string(),
            severity: SeverityLevel::High,
            title: "Potential SQL Injection".to_string(),
            description:
                "Response contains SQL-like patterns that may indicate injection vulnerabilities"
                    .to_string(),
            cve: None,
            affected_url: target.to_string(),
            recommendation:
                "Use parameterized queries and prepared statements for all database access"
                    .to_string(),
        });
    }

    vulnerabilities
}

/// Run an HTTP scan against a target URL
pub async fn run_http_scan(target: &str) -> ScanResult {
    let client = build_client();

    let mut vulnerabilities: Vec<Vulnerability> = Vec::new();

    // Attempt to fetch the target to detect basic responses
    match client.get(target).send().await {
        Ok(resp) => {
            let body_text = resp.text().await.unwrap_or_default();
            vulnerabilities = analyze_response_body(&body_text, target);
        }
        Err(e) => {
            tracing::warn!("HTTP scan error for {}: {}", target, e);
            // Return result with no vulnerabilities even on error
        }
    }

    // Build the scan result
    let result = ScanResult {
        url: target.to_string(),
        ip_address: None,
        open_ports: Vec::new(),
        vulnerabilities: vulnerabilities.clone(),
        risk_score: if vulnerabilities.is_empty() { 1.0 } else { 7.5 },
        scanned_at: chrono::Utc::now().to_rfc3339(),
    };

    result
}

fn build_client() -> Client {
    Client::builder()
        .user_agent("Secutest/0.1.0 (security scan)")
        .connect_timeout(std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .timeout(std::time::Duration::from_secs(SCAN_TIMEOUT_SECS))
        .build()
        .unwrap()
}
