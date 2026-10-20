/// API handlers for the Secutest backend

pub mod scan_handlers {
    use actix_web::{get, post, put, web, HttpResponse, Responder};
    use crate::models::*;
    use crate::services::scanner::run_http_scan;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;
    use chrono::Utc;
    use tracing::info;

    /// Convert internal ScanJob to frontend Scan format
    fn job_to_scan_response(job: &ScanJob) -> serde_json::Value {
        let mut findings = Vec::new();
        let (summary, status, error, completed_at) = match (&job.status, &job.result) {
            (ScanStatus::Failed, _) => {
                let err = job.result.as_ref()
                    .map(|r| format!("Scan failed: {} vulnerabilities found", r.vulnerabilities.len()))
                    .unwrap_or_else(|| "Scan failed".to_string());
                (None, "failed", Some(err), Some(job.updated_at.clone()))
            }
            (ScanStatus::Completed, Some(result)) => {
                // Convert vulnerabilities to findings
                for v in &result.vulnerabilities {
                    findings.push(serde_json::json!({
                        "id": v.id,
                        "title": v.title,
                        "severity": format!("{:?}", v.severity).to_lowercase(),
                        "category": "security",
                        "description": v.description,
                        "location": v.affected_url,
                        "impact": match v.severity {
                            crate::models::SeverityLevel::Critical => "Critical security risk",
                            crate::models::SeverityLevel::High => "High security risk",
                            crate::models::SeverityLevel::Medium => "Medium security risk",
                            crate::models::SeverityLevel::Low => "Low security risk",
                        },
                        "evidence": [],
                        "solution": {
                            "summary": v.recommendation,
                            "steps": [v.recommendation.clone()],
                            "references": []
                        }
                    }));
                }
                let total = findings.len();
                let critical = findings.iter().filter(|f| f["severity"] == "critical").count();
                let high = findings.iter().filter(|f| f["severity"] == "high").count();
                let medium = findings.iter().filter(|f| f["severity"] == "medium").count();
                let low = findings.iter().filter(|f| f["severity"] == "low").count();
                let info = 0;

                let summary = serde_json::json!({
                    "total": total,
                    "critical": critical,
                    "high": high,
                    "medium": medium,
                    "low": low,
                    "info": info,
                    "riskScore": result.risk_score,
                });
                (Some(summary), "completed", None, Some(job.updated_at.clone()))
            }
            (ScanStatus::Completed, None) => (None, "running", None, None),
            (ScanStatus::Running, _) => (None, "running", None, None),
            (ScanStatus::Queued, _) => (None, "queued", None, None),
        };

        serde_json::json!({
            "scanId": job.id,
            "url": job.target,
            "status": status,
            "progress": (job.progress * 100.0) as u32,
            "startedAt": job.created_at,
            "completedAt": completed_at,
            "error": error,
            "summary": summary,
            "findings": findings,
        })
    }

    #[get("/health")]
    pub async fn health_check() -> impl Responder {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "healthy",
            "timestamp": Utc::now().to_rfc3339(),
        }))
    }

    #[post("/api/scans")]
    pub async fn create_scan(
        req: web::Json<CreateScanRequest>,
        data: web::Data<AppState>,
    ) -> impl Responder {
        info!("Creating scan job for target: {}", req.url);

        let mut jobs = data.jobs.lock().unwrap();
        let scan_job = ScanJob {
            id: Uuid::new_v4().to_string(),
            target: req.url.clone(),
            status: ScanStatus::Queued,
            progress: 0.0,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
            result: None,
        };

        jobs.push(scan_job.clone());

        HttpResponse::Ok().json(serde_json::json!({
            "scanId": scan_job.id,
        }))
    }

    #[get("/api/scans")]
    pub async fn list_scans(data: web::Data<AppState>) -> impl Responder {
        let jobs = data.jobs.lock().unwrap();
        let scans: Vec<serde_json::Value> = jobs.iter().map(job_to_scan_response).collect();
        HttpResponse::Ok().json(scans)
    }

    #[get("/api/scans/{job_id}")]
    pub async fn get_scan_status(
        job_id: web::Path<String>,
        data: web::Data<AppState>,
    ) -> impl Responder {
        let jobs = data.jobs.lock().unwrap();

        match jobs.iter().find(|j| j.id == *job_id) {
            Some(job) => HttpResponse::Ok().json(job_to_scan_response(job)),
            None => HttpResponse::NotFound().json(serde_json::json!({
                "error": "Scan job not found",
                "job_id": *job_id,
            })),
        }
    }

    #[get("/api/scans/{job_id}/results")]
    pub async fn get_scan_results(
        job_id: web::Path<String>,
        data: web::Data<AppState>,
    ) -> impl Responder {
        let jobs = data.jobs.lock().unwrap();

        match jobs.iter().find(|j| j.id == *job_id) {
            Some(job) => match &job.result {
                Some(result) => HttpResponse::Ok().json(result),
                None => HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Scan results not available yet — scan still in progress",
                    "job_id": *job_id,
                })),
            },
            None => HttpResponse::NotFound().json(serde_json::json!({
                "error": "Scan job not found",
                "job_id": *job_id,
            })),
        }
    }

    #[put("/api/scans/{job_id}/progress")]
    pub async fn update_scan_progress(
        job_id: web::Path<String>,
        update: web::Json<JobUpdate>,
        data: web::Data<AppState>,
    ) -> impl Responder {
        let mut jobs = data.jobs.lock().unwrap();

        if let Some(job) = jobs.iter_mut().find(|j| j.id == *job_id) {
            if let Some(progress) = update.progress {
                job.progress = progress;
            }
            if let Some(ref status) = update.status {
                job.status = status.clone();
            }
            if let Some(ref result) = update.result {
                job.result = Some(result.clone());
            }
            job.updated_at = Utc::now().to_rfc3339();

            HttpResponse::Ok().json(job_to_scan_response(job))
        } else {
            HttpResponse::NotFound().json(serde_json::json!({
                "error": "Scan job not found",
                "job_id": *job_id,
            }))
        }
    }

    /// Background scan worker that polls pending jobs and runs them.
    pub async fn scan_worker(jobs: Arc<Mutex<Vec<ScanJob>>>) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;

            let pending: Vec<ScanJob> = {
                let guard = jobs.lock().unwrap();
                guard
                    .iter()
                    .filter(|j| j.status == ScanStatus::Queued)
                    .cloned()
                    .collect()
            };

            for job in pending {
                let job_id = job.id.clone();
                let target = job.target.clone();
                let state_clone = Arc::clone(&jobs);

                actix_web::rt::spawn(async move {
                    info!("Starting scan job {} for {}", job_id, target);

                    // Update status to running
                    {
                        let mut jobs_guard = state_clone.lock().unwrap();
                        if let Some(j) = jobs_guard.iter_mut().find(|j| j.id == job_id) {
                            j.status = ScanStatus::Running;
                            j.progress = 0.1;
                            j.updated_at = Utc::now().to_rfc3339();
                        }
                    }

                    let result = run_http_scan(&target).await;

                    // Update the job status in shared state
                    let mut jobs_guard = state_clone.lock().unwrap();
                    if let Some(j) = jobs_guard.iter_mut().find(|j| j.id == job_id) {
                        j.status = ScanStatus::Completed;
                        j.progress = 1.0;
                        j.result = Some(result.clone());
                        j.updated_at = Utc::now().to_rfc3339();
                        info!(
                            "Scan job {} completed with {} vulnerabilities",
                            job_id,
                            j.result.as_ref().unwrap().vulnerabilities.len()
                        );
                    }
                });
            }
        }
    }

    #[derive(Clone)]
    pub struct AppState {
        pub jobs: Arc<Mutex<Vec<ScanJob>>>,
    }
}