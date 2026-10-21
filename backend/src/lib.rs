pub mod auth;
pub mod batches;
pub mod config;
pub mod dashboard;
pub mod domain;
pub mod error;
pub mod idempotency;
pub mod merkle;
pub mod worker;

use axum::{
    Json, Router,
    extract::{Request, State},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
};
use serde_json::json;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub redis: redis::aio::ConnectionManager,
    pub config: config::Config,
}
pub async fn connect(config: config::Config) -> anyhow::Result<AppState> {
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(15)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&db).await?;
    let redis = redis::Client::open(config.redis_url.as_str())?
        .get_connection_manager()
        .await?;
    Ok(AppState { db, redis, config })
}
async fn csrf(
    State(s): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, error::ApiError> {
    let peer = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|v| v.0.ip().to_string())
        .unwrap_or("unknown".into());
    let ip = if s.config.trust_proxy {
        req.headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .and_then(|v| v.trim().parse::<std::net::IpAddr>().ok())
            .map(|v| v.to_string())
            .unwrap_or(peer)
    } else {
        peer
    };
    req.headers_mut().insert(
        "x-rp-client-ip",
        ip.parse().map_err(|_| error::ApiError::internal())?,
    );
    if !matches!(
        *req.method(),
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    ) {
        let h = req.headers();
        if h.get("x-requested-with").and_then(|v| v.to_str().ok()) != Some("RescueProof") {
            return Err(error::ApiError::forbidden());
        }
        if let Some(origin) = h.get("origin")
            && origin.to_str().ok() != Some(s.config.origin.as_str())
        {
            return Err(error::ApiError::forbidden());
        }
        if h.get("sec-fetch-site").and_then(|v| v.to_str().ok()) == Some("cross-site") {
            return Err(error::ApiError::forbidden());
        }
    }
    Ok(next.run(req).await)
}
pub fn router(state: AppState) -> Router {
    Router::new()
        .route(
            "/health/live",
            get(|| async { Json(json!({"status":"ok"})) }),
        )
        .route("/health/ready", get(ready))
        .route("/api/v1/auth/register", post(auth::register))
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/logout", post(auth::logout))
        .route("/api/v1/auth/me", get(auth::me))
        .route("/api/v1/batches", get(batches::list).post(batches::create))
        .route("/api/v1/batches/{id}", get(batches::detail))
        .route("/api/v1/batches/{id}/cancel", post(batches::cancel))
        .route("/api/v1/batches/{id}/claims", post(batches::claim))
        .route("/api/v1/claims/{id}/handover", post(batches::handover))
        .route(
            "/api/v1/claims/{id}/pickup-token",
            post(batches::pickup_token),
        )
        .route("/api/v1/claims/{id}/cancel", post(batches::cancel_claim))
        .route("/api/v1/claims/{id}/receipt", post(batches::receipt))
        .route("/api/v1/claims/{id}/disputes", post(batches::dispute))
        .route("/api/v1/dashboard", get(dashboard::dashboard))
        .route("/api/v1/impact", get(batches::impact))
        .route("/api/v1/batches/{id}/proof", get(dashboard::proof))
        .route("/api/v1/batches/{id}/events", get(dashboard::events))
        .route("/api/v1/admin", get(dashboard::admin))
        .route(
            "/api/v1/admin/organizations/{id}/review",
            post(dashboard::review),
        )
        .route(
            "/api/v1/admin/disputes/{id}/resolve",
            post(dashboard::resolve),
        )
        .layer(axum::extract::DefaultBodyLimit::max(32 * 1024))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(middleware::from_fn(|req: Request, next: Next| async move {
            let mut response = next.run(req).await;
            response.headers_mut().insert(
                axum::http::header::CACHE_CONTROL,
                axum::http::HeaderValue::from_static("no-store"),
            );
            response.headers_mut().insert(
                "x-content-type-options",
                axum::http::HeaderValue::from_static("nosniff"),
            );
            response
        }))
        .layer(middleware::from_fn_with_state(state.clone(), csrf))
        .with_state(state)
}
async fn ready(State(s): State<AppState>) -> Result<Json<serde_json::Value>, error::ApiError> {
    sqlx::query("SELECT 1").execute(&s.db).await?;
    let mut c = s.redis.clone();
    let result: Result<String, _> = redis::cmd("PING").query_async(&mut c).await;
    if result.is_err() {
        return Err(error::ApiError(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "NOT_READY",
            "Redis unavailable".into(),
        ));
    }
    Ok(Json(json!({"status":"ready"})))
}
