use crate::models::SubdomainResult;
/// Scanner service for running HTTP scans against targets
pub use crate::models::{ScanResult, SeverityLevel, Vulnerability};
use reqwest::Client;
use std::net::ToSocketAddrs;
use uuid::Uuid;

const SCAN_TIMEOUT_SECS: u64 = 30;
const CONNECT_TIMEOUT_SECS: u64 = 10;
#[allow(dead_code)]
const SUBDOMAIN_TIMEOUT_SECS: u64 = 10;

// Common subdomain prefixes to check
#[allow(dead_code)]
pub(crate) const COMMON_SUBDOMAINS: &[&str] = &[
    "www",
    "mail",
    "ftp",
    "admin",
    "api",
    "dev",
    "test",
    "staging",
    "prod",
    "app",
    "blog",
    "shop",
    "store",
    "portal",
    "dashboard",
    "secure",
    "vpn",
    "remote",
    "webmail",
    "cpanel",
    "whm",
    "phpmyadmin",
    "mysql",
    "db",
    "git",
    "svn",
    "ci",
    "jenkins",
    "jira",
    "confluence",
    "wiki",
    "docs",
];

/// Extract body analysis logic to make it unit testable
pub(crate) fn analyze_response_body(body: &str, target: &str) -> Vec<Vulnerability> {
    let mut vulnerabilities: Vec<Vulnerability> = Vec::new();

    // Handle null bytes gracefully by removing them
    let clean_body = body.replace('\0', "");
    let body_lower = clean_body.to_lowercase();

    // XSS detection - case insensitive, includes script tags, alert(), and inline event handlers
    let has_xss = body_lower.contains("<script>")
        || body_lower.contains("alert(")
        || body_lower.contains("onerror=")
        || body_lower.contains("onload=")
        || body_lower.contains("onfocus=")
        || body_lower.contains("onmouseover=")
        || body_lower.contains("onclick=");

    if has_xss {
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

    // SQL Injection detection - case insensitive, check for SQL keywords as whole words
    let sql_keywords = ["select", "insert", "update", "delete", "drop", "alter"];
    let has_sqli = sql_keywords.iter().any(|keyword| {
        // Simple word boundary check - keyword followed by space, paren, or semicolon
        let pattern = format!("{} ", keyword);
        body_lower.contains(&pattern)
            || body_lower.contains(&format!("{}\\(", keyword))
            || body_lower.contains(&format!("{};", keyword))
            || body_lower.contains(&format!("{}\n", keyword))
            || body_lower.contains(&format!("{}\r", keyword))
            || body_lower.contains(&format!("{}\t", keyword))
    });

    if has_sqli {
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
    ScanResult {
        url: target.to_string(),
        ip_address: None,
        open_ports: Vec::new(),
        vulnerabilities: vulnerabilities.clone(),
        risk_score: if vulnerabilities.is_empty() { 1.0 } else { 7.5 },
        scanned_at: chrono::Utc::now().to_rfc3339(),
    }
}

fn build_client() -> Client {
    Client::builder()
        .user_agent("Secutest/0.1.0 (security scan)")
        .connect_timeout(std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .timeout(std::time::Duration::from_secs(SCAN_TIMEOUT_SECS))
        .build()
        .unwrap()
}

/// Build a client with shorter timeout for subdomain checks
#[allow(dead_code)]
fn build_subdomain_client() -> Client {
    Client::builder()
        .user_agent("Secutest/0.1.0 (subdomain check)")
        .connect_timeout(std::time::Duration::from_secs(5))
        .timeout(std::time::Duration::from_secs(SUBDOMAIN_TIMEOUT_SECS))
        .build()
        .unwrap()
}

/// Extract host from URL
#[allow(dead_code)]
pub(crate) fn extract_host(url: &str) -> Option<String> {
    let url = if !url.contains("://") {
        format!("http://{}", url)
    } else {
        url.to_string()
    };
    url::Url::parse(&url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
}

/// Check if a subdomain is reachable via DNS resolution
#[allow(dead_code)]
async fn check_subdomain_reachable(subdomain: &str) -> Option<(bool, Option<String>)> {
    // Try to resolve the hostname
    tokio::task::spawn_blocking({
        let subdomain = subdomain.to_string();
        move || {
            format!("{}:80", subdomain)
                .to_socket_addrs()
                .ok()
                .and_then(|mut addrs| addrs.next())
                .map(|addr| (true, Some(addr.ip().to_string())))
                .unwrap_or((false, None))
        }
    })
    .await
    .ok()
}

/// Scan a single subdomain with HTTP request
#[allow(dead_code)]
async fn scan_subdomain(target: &str, subdomain: &str) -> SubdomainResult {
    let subdomain_url = if target.starts_with("https://") {
        target.replacen("https://", &format!("https://{}.", subdomain), 1)
    } else if target.starts_with("http://") {
        target.replacen("http://", &format!("http://{}.", subdomain), 1)
    } else {
        format!("https://{}.{}", subdomain, target)
    };

    let (reachable, ip_address) = check_subdomain_reachable(&subdomain_url)
        .await
        .unwrap_or((false, None));

    let scan_result = if reachable {
        Some(run_http_scan(&subdomain_url).await)
    } else {
        None
    };

    SubdomainResult {
        subdomain: subdomain.to_string(),
        ip_address,
        reachable,
        scan_result,
    }
}

/// Enumerate and scan subdomains for a target
#[allow(dead_code)]
pub async fn scan_subdomains(target: &str, max_subdomains: usize) -> Vec<SubdomainResult> {
    let host = extract_host(target).unwrap_or_else(|| target.to_string());
    let client = build_subdomain_client();

    let mut results = Vec::new();

    for subdomain in COMMON_SUBDOMAINS.iter().take(max_subdomains) {
        let full_domain = format!("{}.{}", subdomain, host);

        // Quick HTTP HEAD request to check if subdomain responds
        let subdomain_url = if target.starts_with("https://") {
            format!("https://{}", full_domain)
        } else {
            format!("http://{}", full_domain)
        };

        let reachable = match client.head(&subdomain_url).send().await {
            Ok(resp) => resp.status().is_success() || resp.status().is_redirection(),
            Err(_) => false,
        };

        let ip_address = if reachable {
            // Try to resolve IP
            tokio::task::spawn_blocking({
                let domain = full_domain.clone();
                move || {
                    format!("{}:80", domain)
                        .to_socket_addrs()
                        .ok()
                        .and_then(|mut addrs| addrs.next())
                        .map(|addr| addr.ip().to_string())
                }
            })
            .await
            .ok()
            .flatten()
        } else {
            None
        };

        let scan_result = if reachable {
            Some(run_http_scan(&subdomain_url).await)
        } else {
            None
        };

        results.push(SubdomainResult {
            subdomain: full_domain,
            ip_address,
            reachable,
            scan_result,
        });
    }

    // Deduplicate results by subdomain name (in case overlapping prefixes)
    let mut seen = std::collections::HashSet::new();
    let mut deduped = Vec::new();
    for result in results {
        if seen.insert(result.subdomain.clone()) {
            deduped.push(result);
        }
    }

    deduped
}

/// Run an HTTP scan against a target URL with optional subdomain scanning
#[allow(dead_code)]
pub async fn run_scan_with_options(
    target: &str,
    include_subdomains: bool,
    max_subdomains: usize,
) -> (ScanResult, Vec<SubdomainResult>) {
    let main_result = run_http_scan(target).await;

    let subdomain_results = if include_subdomains {
        scan_subdomains(target, max_subdomains).await
    } else {
        Vec::new()
    };

    (main_result, subdomain_results)
}
