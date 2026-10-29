// Integration tests for full scan flow: create scan → worker runs → status progresses → results retrievable

#[cfg(test)]
mod tests {
    use crate::handlers::scan_handlers::{create_scan, get_scan_results, get_scan_status, health_check, list_scans, update_scan_progress, AppState};
    use crate::models::{CreateScanRequest, JobUpdate, ScanOptions, ScanResult, ScanStatus, SeverityLevel, Vulnerability};
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use chrono::Utc;
    use mockito::Server;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    async fn setup_app() -> (
        AppState,
        impl actix_web::dev::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
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
    async fn test_full_scan_flow_create_to_completed() {
        let (_, app) = setup_app().await;

        // Step 1: Create a scan
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
        let scan_id = body["scanId"].as_str().unwrap().to_string();

        // Step 2: Verify scan is queued
        let req = test::TestRequest::get()
            .uri(&format!("/api/scans/{}", scan_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["scanId"], scan_id);
        assert_eq!(body["status"], "queued");
        assert_eq!(body["progress"], 0);

        // Step 3: Update to running (simulating worker picking up job)
        let req = test::TestRequest::put()
            .uri(&format!("/api/scans/{}/progress", scan_id))
            .set_json(&JobUpdate {
                job_id: scan_id.clone(),
                progress: Some(0.1),
                status: Some(ScanStatus::Running),
                result: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "running");
        assert_eq!(body["progress"], 10);

        // Step 4: Update to completed with results (simulating worker finishing)
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
        assert_eq!(body["status"], "completed");
        assert_eq!(body["progress"], 100);
        assert!(body["summary"].is_object());
        assert_eq!(body["summary"]["total"], 1);
        assert_eq!(body["summary"]["high"], 1);

        // Step 5: Retrieve results via results endpoint
        let req = test::TestRequest::get()
            .uri(&format!("/api/scans/{}/results", scan_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["url"], "https://example.com");
        assert_eq!(body["vulnerabilities"].as_array().unwrap().len(), 1);
    }

    #[actix_web::test]
    async fn test_full_scan_flow_with_mockito() {
        // Test with mock HTTP server for the actual scanner
        let mut mock_server = Server::new_async().await;

        // Mock the target URL to return a response with XSS pattern
        let mock = mock_server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body("<html><script>alert('xss')</script></html>")
            .create_async()
            .await;

        let (_, app) = setup_app().await;

        // Create a scan pointing to our mock server
        let target_url = format!("{}/", mock_server.url());
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: target_url.clone(),
                options: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        let scan_id = body["scanId"].as_str().unwrap().to_string();

        // Simulate the worker running the scan
        // We directly call the internal scanner function
        let result = crate::services::scanner::run_http_scan(&target_url).await;

        // Verify the mock was called
        mock.assert_async().await;

        // Verify the scanner found the XSS
        assert_eq!(result.vulnerabilities.len(), 1);
        assert!(result.vulnerabilities[0].title.contains("XSS"));

        // Update job with completed status and results
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
        assert_eq!(body["status"], "completed");
        assert_eq!(body["summary"]["total"], 1);
        assert_eq!(body["summary"]["medium"], 1); // XSS is medium severity
    }

    #[actix_web::test]
    async fn test_scan_flow_failed_status() {
        let (_, app) = setup_app().await;

        // Create a scan
        let req = test::TestRequest::post()
            .uri("/api/scans")
            .set_json(&CreateScanRequest {
                url: "https://unreachable.example.com".to_string(),
                options: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        let body: serde_json::Value = test::read_body_json(resp).await;
        let scan_id = body["scanId"].as_str().unwrap().to_string();

        // Update to failed status
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
        assert_eq!(body["status"], "failed");
        assert!(body["error"].is_string());
        assert!(body["error"].as_str().unwrap().contains("Scan failed"));

        // Results endpoint should return not found for failed scan
        let req = test::TestRequest::get()
            .uri(&format!("/api/scans/{}/results", scan_id))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn test_multiple_scans_progress_independently() {
        let (_, app) = setup_app().await;

        // Create two scans
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

        // Progress first scan to running
        let req = test::TestRequest::put()
            .uri(&format!("/api/scans/{}/progress", scan_id1))
            .set_json(&JobUpdate {
                job_id: scan_id1.clone(),
                progress: Some(0.5),
                status: Some(ScanStatus::Running),
                result: None,
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["progress"], 50);
        assert_eq!(body["status"], "running");

        // Second scan should still be queued
        let req = test::TestRequest::get()
            .uri(&format!("/api/scans/{}", scan_id2))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "queued");
        assert_eq!(body["progress"], 0);

        // Complete first scan
        let vuln = Vulnerability {
            id: Uuid::new_v4().to_string(),
            severity: SeverityLevel::Critical,
            title: "Critical Issue".to_string(),
            description: "Critical desc".to_string(),
            cve: None,
            affected_url: "/".to_string(),
            recommendation: "Fix it".to_string(),
        };

        let result = ScanResult {
            url: "https://first.com".to_string(),
            ip_address: None,
            open_ports: vec![],
            vulnerabilities: vec![vuln],
            risk_score: 9.0,
            scanned_at: Utc::now().to_rfc3339(),
        };

        let req = test::TestRequest::put()
            .uri(&format!("/api/scans/{}/progress", scan_id1))
            .set_json(&JobUpdate {
                job_id: scan_id1.clone(),
                progress: Some(1.0),
                status: Some(ScanStatus::Completed),
                result: Some(result),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "completed");
        assert_eq!(body["summary"]["critical"], 1);

        // Second scan should still be queued
        let req = test::TestRequest::get()
            .uri(&format!("/api/scans/{}", scan_id2))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["status"], "queued");
    }

    #[actix_web::test]
    async fn test_scan_worker_simulation() {
        // This test simulates the background worker behavior
        let (_, app) = setup_app().await;

        // Create multiple scans
        let mut scan_ids: Vec<String> = Vec::new();
        for i in 0..3 {
            let req = test::TestRequest::post()
                .uri("/api/scans")
                .set_json(&CreateScanRequest {
                    url: format!("https://site{}.com", i),
                    options: None,
                })
                .to_request();
            let resp = test::call_service(&app, req).await;
            let body: serde_json::Value = test::read_body_json(resp).await;
            scan_ids.push(body["scanId"].as_str().unwrap().to_string());
        }

        // Verify all are queued
        for scan_id in &scan_ids {
            let req = test::TestRequest::get()
                .uri(&format!("/api/scans/{}", scan_id))
                .to_request();
            let resp = test::call_service(&app, req).await;
            let body: serde_json::Value = test::read_body_json(resp).await;
            assert_eq!(body["status"], "queued");
        }

        // Simulate worker processing each scan sequentially
        for (i, scan_id) in scan_ids.iter().enumerate() {
            // Update to running
            let req = test::TestRequest::put()
                .uri(&format!("/api/scans/{}/progress", scan_id))
                .set_json(&JobUpdate {
                    job_id: scan_id.clone(),
                    progress: Some(0.1),
                    status: Some(ScanStatus::Running),
                    result: None,
                })
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::OK);

            // Complete with different results
            let vuln = Vulnerability {
                id: Uuid::new_v4().to_string(),
                severity: match i {
                    0 => SeverityLevel::Critical,
                    1 => SeverityLevel::High,
                    _ => SeverityLevel::Medium,
                },
                title: format!("Vulnerability {}", i),
                description: "Description".to_string(),
                cve: None,
                affected_url: "/".to_string(),
                recommendation: "Fix".to_string(),
            };

            let result = ScanResult {
                url: format!("https://site{}.com", i),
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
            assert_eq!(body["status"], "completed");
        }

        // Verify final state of all scans
        let req = test::TestRequest::get().uri("/api/scans").to_request();
        let resp = test::call_service(&app, req).await;
        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 3);

        for scan in body {
            assert_eq!(scan["status"], "completed");
            assert_eq!(scan["progress"], 100);
            assert!(scan["summary"].is_object());
        }
    }
}