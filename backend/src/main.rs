mod handlers;
mod models;
mod services;

#[cfg(test)]
mod tests;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use handlers::scan_handlers::{
    create_scan, get_scan_results, get_scan_status, health_check, list_scans, scan_worker,
    update_scan_progress, AppState,
};
use std::sync::{Arc, Mutex};
use tracing::info;
use tracing_subscriber::EnvFilter;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging with environment filter
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("secutest_backend=info")),
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
