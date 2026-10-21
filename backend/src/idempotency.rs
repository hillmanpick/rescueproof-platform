use crate::error::{ApiError, ApiResult};
use axum::http::HeaderMap;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

pub struct Command {
    pub key: String,
    pub fingerprint: String,
}
impl Command {
    pub fn new(h: &HeaderMap, operation: &str, body: &impl Serialize) -> ApiResult<Self> {
        let key = h
            .get("idempotency-key")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                ApiError::bad("IDEMPOTENCY_REQUIRED", "Provide an Idempotency-Key header")
            })?;
        if key.len() < 8
            || key.len() > 100
            || !key
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        {
            return Err(ApiError::bad(
                "INVALID_IDEMPOTENCY_KEY",
                "Use an 8–100 character unique key",
            ));
        }
        let mut hash = Sha256::new();
        hash.update(operation);
        hash.update([0]);
        hash.update(serde_json::to_vec(body).map_err(|_| ApiError::internal())?);
        Ok(Self {
            key: key.into(),
            fingerprint: hex::encode(hash.finalize()),
        })
    }
    pub async fn begin(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        user: uuid::Uuid,
    ) -> ApiResult<Option<Value>> {
        sqlx::query("INSERT INTO idempotency_keys(user_id,key,fingerprint) VALUES($1,$2,$3) ON CONFLICT DO NOTHING").bind(user).bind(&self.key).bind(&self.fingerprint).execute(&mut **tx).await?;
        let (fp,cached):(String,Option<Value>)=sqlx::query_as("SELECT fingerprint,response FROM idempotency_keys WHERE user_id=$1 AND key=$2 FOR UPDATE").bind(user).bind(&self.key).fetch_one(&mut **tx).await?;
        if fp != self.fingerprint {
            return Err(ApiError::conflict(
                "IDEMPOTENCY_MISMATCH",
                "This key was used for a different request",
            ));
        }
        Ok(cached)
    }
    pub async fn finish(
        &self,
        mut tx: sqlx::Transaction<'_, sqlx::Postgres>,
        user: uuid::Uuid,
        response: Value,
    ) -> ApiResult<axum::Json<Value>> {
        sqlx::query("UPDATE idempotency_keys SET response=$3 WHERE user_id=$1 AND key=$2")
            .bind(user)
            .bind(&self.key)
            .bind(&response)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(axum::Json(response))
    }
}
