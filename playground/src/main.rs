mod config;
mod error;
mod handlers;
mod renderer;
mod session;

use axum::routing::{get, post};
use axum::Router;
use handlers::AppState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("marmite_playground=info".parse().unwrap()),
        )
        .init();

    let config = config::AppConfig::from_env();
    let sessions: session::SessionStore = Arc::new(RwLock::new(HashMap::new()));

    tokio::fs::create_dir_all(&config.sessions_dir)
        .await
        .expect("failed to create sessions directory");

    let cleanup_config = config.clone();
    let cleanup_sessions = sessions.clone();
    tokio::spawn(async move {
        session::cleanup_task(cleanup_config, cleanup_sessions).await;
    });

    let bind_addr = config.bind_addr.clone();
    let state = Arc::new(AppState { config, sessions });

    let api = Router::new()
        .route(
            "/api/sessions",
            get(handlers::list_sessions_handler).post(handlers::create_session_handler),
        )
        .route("/api/sessions/{id}", get(handlers::get_session_handler))
        .route(
            "/api/sessions/{id}/files",
            get(handlers::list_files_handler),
        )
        .route(
            "/api/sessions/{id}/files/{*path}",
            get(handlers::read_file_handler)
                .put(handlers::write_file_handler)
                .delete(handlers::delete_file_handler),
        )
        .route(
            "/api/sessions/{id}/clone",
            post(handlers::clone_session_handler),
        )
        .route("/api/sessions/{id}/render", post(handlers::render_handler))
        .route(
            "/api/sessions/{id}/download/source",
            get(handlers::download_source_handler),
        )
        .route(
            "/api/sessions/{id}/download/site",
            get(handlers::download_site_handler),
        )
        .route("/api/sessions/{id}/upload", post(handlers::upload_handler))
        .route("/preview/{id}/", get(handlers::preview_root_handler))
        .route("/preview/{id}/{*path}", get(handlers::preview_handler));

    let static_dir = std::env::current_dir()
        .expect("failed to get current dir")
        .join("static");

    let app = api
        .fallback_service(
            ServiceBuilder::new()
                .layer(SetResponseHeaderLayer::overriding(
                    axum::http::header::CACHE_CONTROL,
                    axum::http::HeaderValue::from_static("no-cache"),
                ))
                .service(ServeDir::new(static_dir)),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind to {bind_addr}: {e}"));

    tracing::info!("playground server listening on {bind_addr}");
    axum::serve(listener, app).await.expect("server error");
}
