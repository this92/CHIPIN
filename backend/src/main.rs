/// CHIPIN Backend — Main entry point
///
/// Stack:  Tokio (async runtime) + Axum (routing) + rusqlite (SQLite)
/// Serves: http://127.0.0.1:8080
///
/// Routes:
///   POST  /api/login
///   GET   /api/students
///   GET   /api/students/:id
///   PUT   /api/students/:id
///   GET   /api/rooms/:code/mates
///   GET   /api/projects
///   POST  /api/projects
///   GET   /api/stats

use std::sync::{Arc, Mutex};

use axum::{
    http::{HeaderValue, Method},
    routing::{get, post, put},
    Router,
};
use rusqlite::Connection;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod db;
mod error;
mod models;
mod routes;

/// Shared application state — the DB connection wrapped in Arc<Mutex>.
/// Using a Mutex here keeps it simple; for higher concurrency, switch to
/// a connection pool (e.g., r2d2-sqlite), but for a college app this is fine.
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
}

#[tokio::main]
async fn main() {
    // ── Logging setup ─────────────────────────────────────────────────────────
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "chipin=info,tower_http=info".parse().unwrap()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🎓 CHIPIN Backend starting…");

    // ── Database ──────────────────────────────────────────────────────────────
    let db_path = std::env::var("CHIPIN_DB").unwrap_or_else(|_| "chipin.db".into());
    let conn    = db::init_db(&db_path).expect("Failed to initialise database");
    let state   = AppState {
        db: Arc::new(Mutex::new(conn)),
    };

    // ── CORS ──────────────────────────────────────────────────────────────────
    // Allows the frontend (served from file:// or localhost:*) to call the API
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(Any)
        .allow_origin([
            "http://127.0.0.1:5500".parse::<HeaderValue>().unwrap(),
            "http://localhost:5500".parse::<HeaderValue>().unwrap(),
            "http://127.0.0.1:3000".parse::<HeaderValue>().unwrap(),
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
            "null".parse::<HeaderValue>().unwrap(), // file:// origin
        ]);

    // ── Router ────────────────────────────────────────────────────────────────
    let app = Router::new()
        // Auth
        .route("/api/login",                  post(routes::auth::login))
        // Students
        .route("/api/students",               get(routes::students::list_students))
        .route("/api/students/:id",           get(routes::students::get_student))
        .route("/api/students/:id",           put(routes::students::update_student))
        // Rooms
        .route("/api/rooms/:code/mates",      get(routes::rooms::room_mates))
        // Projects
        .route("/api/projects",               get(routes::projects::list_projects))
        .route("/api/projects",               post(routes::projects::create_project))
        // Stats
        .route("/api/stats",                  get(routes::stats::global_stats))
        // Health check
        .route("/api/health",                 get(|| async { "OK" }))
        // Middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // ── Listen ────────────────────────────────────────────────────────────────
    let addr = std::env::var("CHIPIN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".into());

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect(&format!("Failed to bind to {addr}"));

    info!("✅ CHIPIN API listening on http://{addr}");
    info!("   POST  http://{addr}/api/login");
    info!("   GET   http://{addr}/api/students");
    info!("   GET   http://{addr}/api/rooms/107/mates");
    info!("   GET   http://{addr}/api/health");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
