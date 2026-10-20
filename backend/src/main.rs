mod models;
mod services;
mod handlers;

#[cfg(test)]
mod tests;

use handlers::scan_handlers::{AppState, health_check, create_scan, list_scans, get_scan_status, get_scan_results, update_scan_progress, scan_worker};
use actix_web::{App, HttpServer, web};
use actix_cors::Cors;
use std::sync::{Arc, Mutex};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging with environment filter
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("secutest_backend=info"))
        )
        .init();

    info!("Starting Secutest Backend on http://localhost:8080");

    // Initialize shared state
    let state = web::Data::new(AppState {
        jobs: Arc::new(Mutex::new(Vec::new())),
    });

    // Spawn the background scan worker
    let worker_jobs = Arc::clone(&state.jobs);
    actix_web::rt::spawn(scan_worker(worker_jobs));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                jobs: Arc::clone(&state.jobs),
            }))
            .service(health_check)
            .service(create_scan)
            .service(list_scans)
            .service(get_scan_status)
            .service(get_scan_results)
            .service(update_scan_progress)
            .wrap(Cors::permissive())
    })
    .bind("localhost:8080")?
    .run()
    .await
}