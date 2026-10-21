#[cfg(test)]
mod tests {
    use crate::handlers::scan_handlers::{
        create_scan, get_scan_results, get_scan_status, health_check, list_scans,
        update_scan_progress, AppState,
    };
    use crate::models::{
        CreateScanRequest, JobUpdate, ScanOptions, ScanResult, ScanStatus, SeverityLevel,
        Vulnerability,
    };
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use chrono::Utc;
    use serde_json::json;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    async fn setup_app() -> (
        AppState,
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
    ) {
        let state = AppState {
            jobs: Arc::new(Mutex::new(Vec::new())),
        };
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state.clone()))
                .service(health_check)
                .service(create_scan)
                .service(list_scans)
                .service(get_scan_status)
                .service(get_scan_results)
                .service(update_scan_progress),
        )
        .await;
        (state, app)
    }

    #[actix_web::test]
    async fn test_health_check() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "healthy");
        assert!(body["timestamp"].is_string());
    }

    #[actix_web::test]
    async fn test_create_scan_returns_scan_id() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://example.com".to_string(),
                options: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["scanId"].is_string());
        assert!(!body["scanId"].as_str().unwrap().is_empty());
    }

    #[actix_web::test]
    async fn test_create_scan_with_options() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://example.com".to_string(),
                options: Some(ScanOptions {
                    include_subdomains: Some(true),
                    follow_redirects: Some(false),
                    max_depth: Some(5),
                }),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["scanId"].is_string());
    }

    #[actix_web::test]
    async fn test_list_scans_empty() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::get().uri("/api/scans").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert!(body.is_empty());
    }

    #[actix_web::test]
    async fn test_list_scans_after_create() {
        let (state, app) = setup_app().await;

        // Create a scan first
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://example.com".to_string(),
                options: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let scan_id = body["scanId"].as_str().unwrap().to_string();

        // Now list scans
        let req = test::TestRequest::get().uri("/api/scans").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 1);
        assert_eq!(body[0]["scanId"], scan_id);
        assert_eq!(body[0]["url"], "https://example.com");
        assert_eq!(body[0]["status"], "queued");
    }

    #[actix_web::test]
    async fn test_get_scan_status_existing() {
        let (state, app) = setup_app().await;

        // Create a scan
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://example.com".to_string(),
                options: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let scan_id = body["scanId"].as_str().unwrap().to_string();

        // Get scan status
        let req = test::TestRequest::get()
            .uri(&format!("/api/scans/{}", scan_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["scanId"], scan_id);
        assert_eq!(body["url"], "https://example.com");
        assert_eq!(body["status"], "queued");
        assert_eq!(body["progress"], 0);
    }

    #[actix_web::test]
    async fn test_get_scan_status_not_found() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::get()
            .uri("/api/scans/nonexistent-id")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["error"], "Scan job not found");
        assert_eq!(body["job_id"], "nonexistent-id");
    }

    #[actix_web::test]
    async fn test_get_scan_results_not_ready() {
        let (_, app) = setup_app().await;

        // Create a scan
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://example.com".to_string(),
                options: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let scan_id = body["scanId"].as_str().unwrap().to_string();

        // Try to get results before scan completes
        let req = test::TestRequest::get()
            .uri(&format!("/api/scans/{}/results", scan_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["error"].as_str().unwrap().contains("not available"));
    }

    #[actix_web::test]
    async fn test_get_scan_results_not_found() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::get()
            .uri("/api/scans/nonexistent-id/results")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["error"], "Scan job not found");
    }

    #[actix_web::test]
    async fn test_update_scan_progress() {
        let (state, app) = setup_app().await;

        // Create a scan
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://example.com".to_string(),
                options: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let scan_id = body["scanId"].as_str().unwrap().to_string();

        // Update progress
        let req = test::TestRequest::put()
            .uri(&format!("/api/scans/{}/progress", scan_id))
            .set_json(&JobUpdate {
                job_id: scan_id.clone(),
                progress: Some(0.5),
                status: Some(ScanStatus::Running),
                result: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["scanId"], scan_id);
        assert_eq!(body["progress"], 50);
        assert_eq!(body["status"], "running");
    }

    #[actix_web::test]
    async fn test_update_scan_progress_not_found() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::put()
            .uri("/api/scans/nonexistent-id/progress")
            .set_json(&JobUpdate {
                job_id: "nonexistent-id".to_string(),
                progress: Some(0.5),
                status: None,
                result: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["error"], "Scan job not found");
    }

    #[actix_web::test]
    async fn test_update_scan_completed_with_results() {
        let (state, app) = setup_app().await;

        // Create a scan
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://example.com".to_string(),
                options: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let scan_id = body["scanId"].as_str().unwrap().to_string();

        // Update with completed status and results
        let vuln = Vulnerability {
            id: Uuid::new_v4().to_string(),
            severity: SeverityLevel::High,
            title: "Test Vulnerability".to_string(),
            description: "Test description".to_string(),
            cve: None,
            affected_url: "/test".to_string(),
            recommendation: "Test fix".to_string(),
        };

        let result = ScanResult {
            url: "https://example.com".to_string(),
            ip_address: None,
            open_ports: vec![],
            vulnerabilities: vec![vuln],
            risk_score: 7.5,
            scanned_at: Utc::now().to_rfc3339(),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/api/scans/{}/progress", scan_id))
            .set_json(&JobUpdate {
                job_id: scan_id.clone(),
                progress: Some(1.0),
                status: Some(ScanStatus::Completed),
                result: Some(result),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["scanId"], scan_id);
        assert_eq!(body["progress"], 100);
        assert_eq!(body["status"], "completed");
        assert!(body["summary"].is_object());
        assert_eq!(body["summary"]["total"], 1);
        assert_eq!(body["summary"]["high"], 1);
        assert!(body["findings"].is_array());
        assert_eq!(body["findings"].as_array().unwrap().len(), 1);
    }

    #[actix_web::test]
    async fn test_update_scan_failed() {
        let (state, app) = setup_app().await;

        // Create a scan
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://example.com".to_string(),
                options: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let scan_id = body["scanId"].as_str().unwrap().to_string();

        // Update with failed status
        let req = test::TestRequest::put()
            .uri(&format!("/api/scans/{}/progress", scan_id))
            .set_json(&JobUpdate {
                job_id: scan_id.clone(),
                progress: None,
                status: Some(ScanStatus::Failed),
                result: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["scanId"], scan_id);
        assert_eq!(body["status"], "failed");
        assert!(body["error"].is_string());
        assert!(body["error"].as_str().unwrap().contains("Scan failed"));
    }

    #[actix_web::test]
    async fn test_multiple_scans_isolation() {
        let (state, app) = setup_app().await;

        // Create first scan
        let req1 = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://first.com".to_string(),
                options: None,
            })
            .to_request();
        let resp1 = test::call_service(&app, req1).await;
        let body1: serde_json::Value = test::read_body_json(resp1).await;
        let scan_id1 = body1["scanId"].as_str().unwrap().to_string();

        // Create second scan
        let req2 = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://second.com".to_string(),
                options: None,
            })
            .to_request();
        let resp2 = test::call_service(&app, req2).await;
        let body2: serde_json::Value = test::read_body_json(resp2).await;
        let scan_id2 = body2["scanId"].as_str().unwrap().to_string();

        // Update first scan
        let req = test::TestRequest::put()
            .uri(&format!("/api/scans/{}/progress", scan_id1))
            .set_json(&JobUpdate {
                job_id: scan_id1.clone(),
                progress: Some(0.75),
                status: Some(ScanStatus::Running),
                result: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["scanId"], scan_id1);
        assert_eq!(body["progress"], 75);

        // Verify second scan is still queued
        let req = test::TestRequest::get()
            .uri(&format!("/api/scans/{}", scan_id2))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["scanId"], scan_id2);
        assert_eq!(body["status"], "queued");
        assert_eq!(body["progress"], 0);
    }
}
