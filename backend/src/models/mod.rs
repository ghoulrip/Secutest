// Models for Secutest backend

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ScanJob {
    pub id: String,
    pub target: String,
    pub status: ScanStatus,
    pub progress: f32,
    pub created_at: String,
    pub updated_at: String,
    pub result: Option<ScanResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ScanResult {
    pub url: String,
    pub ip_address: Option<String>,
    pub open_ports: Vec<u16>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub risk_score: f32,
    pub scanned_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Vulnerability {
    pub id: String,
    pub severity: SeverityLevel,
    pub title: String,
    pub description: String,
    pub cve: Option<String>,
    pub affected_url: String,
    pub recommendation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SeverityLevel {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ScanOptions {
    pub include_subdomains: Option<bool>,
    pub follow_redirects: Option<bool>,
    pub max_depth: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CreateScanRequest {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ScanOptions>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JobUpdate {
    pub job_id: String,
    pub progress: Option<f32>,
    pub status: Option<ScanStatus>,
    pub result: Option<ScanResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ScanStatus {
    Queued,
    Running,
    Completed,
    Failed,
}
