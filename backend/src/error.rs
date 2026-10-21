use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub struct ApiError(pub StatusCode, pub &'static str, pub String);
pub type ApiResult<T> = Result<T, ApiError>;
impl ApiError {
    pub fn bad(code: &'static str, msg: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, code, msg.into())
    }
    pub fn conflict(code: &'static str, msg: impl Into<String>) -> Self {
        Self(StatusCode::CONFLICT, code, msg.into())
    }
    pub fn forbidden() -> Self {
        Self(
            StatusCode::FORBIDDEN,
            "FORBIDDEN",
            "You do not have permission for this action".into(),
        )
    }
    pub fn unauthorized() -> Self {
        Self(
            StatusCode::UNAUTHORIZED,
            "UNAUTHENTICATED",
            "Please sign in".into(),
        )
    }
    pub fn missing() -> Self {
        Self(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "Record not found".into(),
        )
    }
    pub fn internal() -> Self {
        Self(
            StatusCode::INTERNAL_SERVER_ERROR,
            "INTERNAL_ERROR",
            "An internal error occurred".into(),
        )
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.0,
            Json(json!({"error":{"code":self.1,"message":self.2}})),
        )
            .into_response()
    }
}
impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        // Never return SQL, connection strings or personal data to the client.
        match &e {
            sqlx::Error::RowNotFound => Self::missing(),
            sqlx::Error::Database(db) if db.is_unique_violation() => {
                Self::conflict("CONFLICT", "This record already exists")
            }
            _ => {
                tracing::error!(error_kind = ?std::mem::discriminant(&e), "database operation failed");
                Self::internal()
            }
        }
    }
}
