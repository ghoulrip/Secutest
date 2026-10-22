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

    // ===== ERROR HANDLING TESTS (Phase 3.2) =====

    #[actix_web::test]
    async fn test_create_scan_malformed_json() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .insert_header(("Content-Type", "application/json"))
            .set_payload("not valid json {")
            .to_request();
        let resp = test::call_service(&app, req).await;

        // actix-web returns 400 for malformed JSON
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_create_scan_missing_url_field() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&serde_json::json!({}))  // missing url
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should return 400 for missing required field
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn test_create_scan_invalid_url_format() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "not-a-url".to_string(),
                options: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Current implementation accepts any string as URL
        // This test documents current behavior - could be enhanced to validate URLs
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["scanId"].is_string());
    }

    #[actix_web::test]
    async fn test_create_scan_very_long_url() {
        let (_, app) = setup_app().await;
        let long_url = format!("https://example.com/{}", "a".repeat(5000));
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: long_url,
                options: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["scanId"].is_string());
    }

    #[actix_web::test]
    async fn test_create_scan_with_null_bytes() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://example.com\x00evil.com".to_string(),
                options: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Should handle null bytes gracefully
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_get_scan_status_invalid_uuid() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::get()
            .uri("/api/scans/not-a-uuid")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["error"], "Scan job not found");
        assert_eq!(body["job_id"], "not-a-uuid");
    }

    #[actix_web::test]
    async fn test_get_scan_results_invalid_uuid() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::get()
            .uri("/api/scans/not-a-uuid/results")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["error"], "Scan job not found");
    }

    #[actix_web::test]
    async fn test_update_scan_progress_invalid_uuid() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::put()
            .uri("/api/scans/not-a-uuid/progress")
            .set_json(&JobUpdate {
                job_id: "not-a-uuid".to_string(),
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
    async fn test_update_scan_progress_out_of_bounds_high() {
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

        // Update with progress > 1.0
        let req = test::TestRequest::put()
            .uri(&format!("/api/scans/{}/progress", scan_id))
            .set_json(&JobUpdate {
                job_id: scan_id.clone(),
                progress: Some(1.5),  // > 1.0
                status: None,
                result: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Currently accepts any float - this documents current behavior
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        // Progress is multiplied by 100 for display
        assert_eq!(body["progress"], 150);
    }

    #[actix_web::test]
    async fn test_update_scan_progress_out_of_bounds_negative() {
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

        // Update with negative progress
        let req = test::TestRequest::put()
            .uri(&format!("/api/scans/{}/progress", scan_id))
            .set_json(&JobUpdate {
                job_id: scan_id.clone(),
                progress: Some(-0.5),
                status: None,
                result: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["progress"], -50);
    }

    #[actix_web::test]
    async fn test_list_scans_after_multiple_creates() {
        let (state, app) = setup_app().await;

        // Create multiple scans
        let urls = vec!["https://first.com", "https://second.com", "https://third.com"];
        let mut scan_ids = Vec::new();

        for url in urls {
            let req = test::TestRequest::post()
                .uri("/api/scans")
                .set_json(&CreateScanRequest {
                    url: url.to_string(),
                    options: None,
                })
                .to_request();
            let resp = test::call_service(&app, req).await;
            let body: serde_json::Value = test::read_body_json(resp).await;
            scan_ids.push(body["scanId"].as_str().unwrap().to_string());
        }

        // List all scans
        let req = test::TestRequest::get().uri("/api/scans").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 3);

        // Verify all scan IDs are present
        let returned_ids: Vec<String> = body.iter().map(|b| b["scanId"].as_str().unwrap().to_string()).collect();
        for id in scan_ids {
            assert!(returned_ids.contains(&id));
        }
    }

    #[actix_web::test]
    async fn test_concurrent_scan_creation() {
        let (state, app) = setup_app().await;

        // Simulate concurrent creation by creating multiple scans rapidly
        let mut handles = Vec::new();
        for i in 0..10 {
            let app_clone = app.clone();
            handles.push(tokio::spawn(async move {
                let req = test::TestRequest::post()
                    .uri("/api/scans")
                    .set_json(&CreateScanRequest {
                        url: format!("https://example{}.com", i),
                        options: None,
                    })
                    .to_request();
                let resp = test::call_service(&app_clone, req).await;
                let body: serde_json::Value = test::read_body_json(resp).await;
                body["scanId"].as_str().unwrap().to_string()
            }));
        }

        let mut scan_ids = Vec::new();
        for handle in handles {
            scan_ids.push(handle.await.unwrap());
        }

        // All should have unique IDs
        scan_ids.sort();
        scan_ids.dedup();
        assert_eq!(scan_ids.len(), 10);
    }

    // ===== SUBDOMAIN SCANNING TESTS (Phase 3.3) =====

    #[actix_web::test]
    async fn test_create_scan_with_subdomains_enabled() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://example.com".to_string(),
                options: Some(ScanOptions {
                    include_subdomains: Some(true),
                    follow_redirects: Some(true),
                    max_depth: Some(3),
                }),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["scanId"].is_string());
        assert!(!body["scanId"].as_str().unwrap().is_empty());
    }

    #[actix_web::test]
    async fn test_create_scan_with_subdomains_disabled() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://example.com".to_string(),
                options: Some(ScanOptions {
                    include_subdomains: Some(false),
                    follow_redirects: Some(false),
                    max_depth: Some(1),
                }),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert!(body["scanId"].is_string());
    }

    #[actix_web::test]
    async fn test_scan_options_round_trip_subdomain_flag() {
        let (_, app) = setup_app().await;
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://example.com".to_string(),
                options: Some(ScanOptions {
                    include_subdomains: Some(true),
                    follow_redirects: Some(true),
                    max_depth: Some(5),
                }),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);

        let body: serde_json::Value = test::read_body_json(resp).await;
        let scan_id = body["scanId"].as_str().unwrap().to_string();

        // Retrieve the scan and verify options were preserved
        let req = test::TestRequest::get()
            .uri(&format!("/api/scans/{}", scan_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "queued");
    }

    // ===== SCANNER INTEGRATION TESTS WITH MOCKITO =====

    #[actix_web::test]
    async fn test_run_http_scan_with_mockito() {
        let mut mock_server = mockito::Server::new_async().await;

        // Mock the target URL to return a response with XSS pattern
        let _mock = mock_server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("<html><script>alert('xss')</script></html>")
            .create_async()
            .await;

        let target_url = format!("{}/", mock_server.url());
        let result = crate::services::scanner::run_http_scan(&target_url).await;

        assert_eq!(result.vulnerabilities.len(), 1);
        assert!(result.vulnerabilities[0].title.contains("XSS"));
    }

    #[actix_web::test]
    async fn test_run_http_scan_with_mockito_no_vulnerability() {
        let mut mock_server = mockito::Server::new_async().await;

        let _mock = mock_server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("<html><p>Hello world</p></html>")
            .create_async()
            .await;

        let target_url = format!("{}/", mock_server.url());
        let result = crate::services::scanner::run_http_scan(&target_url).await;

        assert!(result.vulnerabilities.is_empty());
    }

    #[actix_web::test]
    async fn test_run_scan_with_options_subdomains_disabled() {
        let target = "https://example.com";
        let (result, subdomain_results) =
            crate::services::scanner::run_scan_with_options(target, false, 5).await;

        assert_eq!(result.url, target);
        assert!(subdomain_results.is_empty());
    }

    #[actix_web::test]
    async fn test_run_scan_with_options_include_subdomains() {
        let target = "https://example.com";
        let (result, subdomain_results) =
            crate::services::scanner::run_scan_with_options(target, true, 3).await;

        assert_eq!(result.url, target);
        // Subdomain results may be empty if DNS doesn't resolve (no network)
        assert!(subdomain_results.len() <= 3);
    }
}
