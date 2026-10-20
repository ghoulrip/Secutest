/// Data models for the Secutest backend

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScanJob {
    pub id: String,
    pub target: String,
    pub status: ScanStatus,
    pub progress: f32,
    pub created_at: String,
    pub updated_at: String,
    pub result: Option<ScanResult>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ScanStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScanResult {
    pub url: String,
    pub ip_address: Option<String>,
    pub open_ports: Vec<u16>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub risk_score: f32,
    pub scanned_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Vulnerability {
    pub id: String,
    pub severity: SeverityLevel,
    pub title: String,
    pub description: String,
    pub cve: Option<String>,
    pub affected_url: String,
    pub recommendation: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateScanRequest {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ScanOptions>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScanOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_subdomains: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub follow_redirects: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JobUpdate {
    pub job_id: String,
    pub progress: Option<f32>,
    pub status: Option<ScanStatus>,
    pub result: Option<ScanResult>,
}