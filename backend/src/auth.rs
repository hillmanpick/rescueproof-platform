use crate::{
    AppState,
    domain::text,
    error::{ApiError, ApiResult},
};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::{
    Json,
    extract::State,
    http::{HeaderMap, header},
};
use rand::RngCore;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
pub struct Principal {
    pub id: uuid::Uuid,
    pub email: String,
    pub name: String,
    pub is_admin: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Register {
    pub email: String,
    pub name: String,
    pub password: String,
    pub organization_name: String,
    pub organization_kind: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Login {
    pub email: String,
    pub password: String,
}
pub fn token_hash(token: &str) -> String {
    hex::encode(Sha256::digest(token.as_bytes()))
}
pub fn random_token() -> String {
    let mut b = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut b);
    hex::encode(b)
}
pub async fn principal(state: &AppState, headers: &HeaderMap) -> ApiResult<Principal> {
    let raw = headers
        .get(header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|s| {
            s.split(';')
                .find_map(|c| c.trim().strip_prefix("rp_session="))
        })
        .ok_or_else(ApiError::unauthorized)?;
    if raw.len() != 64 || !raw.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(ApiError::unauthorized());
    }
    let row=sqlx::query_as::<_,(uuid::Uuid,String,String,bool)>("SELECT u.id,u.email,u.name,u.is_admin FROM sessions s JOIN users u ON u.id=s.user_id WHERE s.token_hash=$1 AND s.expires_at>now() AND u.disabled=false")
        .bind(token_hash(raw)).fetch_optional(&state.db).await?.ok_or_else(ApiError::unauthorized)?;
    Ok(Principal {
        id: row.0,
        email: row.1,
        name: row.2,
        is_admin: row.3,
    })
}
fn set_cookie(token: &str, secure: bool) -> String {
    format!(
        "rp_session={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000{}",
        if secure { "; Secure" } else { "" }
    )
}
fn session_cookie(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|c| c.trim().strip_prefix("rp_session="))
}
pub async fn register(
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Register>,
) -> ApiResult<(HeaderMap, Json<Value>)> {
    limit(&s, &headers, "register", 5).await?;
    text(&body.name, 1, 100)?;
    text(&body.organization_name, 2, 160)?;
    let email = body.email.trim().to_lowercase();
    if email.len() > 254 || !email.contains('@') || email.chars().any(char::is_whitespace) {
        return Err(ApiError::bad("VALIDATION", "Enter a valid email address"));
    }
    if body.password.len() < 12 || body.password.len() > 128 {
        return Err(ApiError::bad(
            "WEAK_PASSWORD",
            "Password must be 12 to 128 characters",
        ));
    }
    if !["donor", "recipient"].contains(&body.organization_kind.as_str()) {
        return Err(ApiError::bad("VALIDATION", "Invalid organisation type"));
    }
    let password = body.password;
    let hash = tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|_| ApiError::internal())
    })
    .await
    .map_err(|_| ApiError::internal())??;
    let mut tx = s.db.begin().await?;
    let uid = uuid::Uuid::new_v4();
    let oid = uuid::Uuid::new_v4();
    let is_admin = false;
    sqlx::query("INSERT INTO users(id,email,name,password_hash,is_admin) VALUES($1,$2,$3,$4,$5)")
        .bind(uid)
        .bind(&email)
        .bind(body.name.trim())
        .bind(hash)
        .bind(is_admin)
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(d) = &e {
                if d.is_unique_violation() {
                    ApiError::conflict("EMAIL_EXISTS", "An account with this email already exists")
                } else {
                    ApiError::from(e)
                }
            } else {
                ApiError::from(e)
            }
        })?;
    sqlx::query("INSERT INTO organizations(id,name,kind) VALUES($1,$2,$3)")
        .bind(oid)
        .bind(body.organization_name.trim())
        .bind(&body.organization_kind)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO organization_members(user_id,organization_id,role) VALUES($1,$2,'owner')",
    )
    .bind(uid)
    .bind(oid)
    .execute(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO audit_events(id,actor_id,action,entity_id) VALUES($1,$2,'account.registered',$3)").bind(uuid::Uuid::new_v4()).bind(uid).bind(oid).execute(&mut *tx).await?;
    tx.commit().await?;
    let token = random_token();
    sqlx::query("INSERT INTO sessions(token_hash,user_id,expires_at) VALUES($1,$2,now()+interval '30 days')").bind(token_hash(&token)).bind(uid).execute(&s.db).await?;
    let mut out = HeaderMap::new();
    out.insert(
        header::SET_COOKIE,
        set_cookie(&token, s.config.secure_cookie)
            .parse()
            .map_err(|_| ApiError::internal())?,
    );
    Ok((
        out,
        Json(
            json!({"user":{"id":uid,"email":email,"name":body.name,"is_admin":is_admin},"organization":{"id":oid,"name":body.organization_name,"kind":body.organization_kind,"status":"pending"}}),
        ),
    ))
}
pub async fn login(
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Login>,
) -> ApiResult<(HeaderMap, Json<Value>)> {
    limit(&s, &headers, "login", 8).await?;
    if body.password.len() > 128 || body.email.len() > 254 {
        return Err(ApiError::unauthorized());
    }
    let email = body.email.trim().to_lowercase();
    let row = sqlx::query_as::<_, (uuid::Uuid, String, String, bool)>(
        "SELECT id,password_hash,name,disabled FROM users WHERE email=$1",
    )
    .bind(&email)
    .fetch_optional(&s.db)
    .await?;
    let Some((uid, hash, name, disabled)) = row else {
        return Err(ApiError::unauthorized());
    };
    let password = body.password;
    let valid = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash).ok().is_some_and(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        })
    })
    .await
    .map_err(|_| ApiError::internal())?;
    if disabled || !valid {
        return Err(ApiError::unauthorized());
    }
    let token = random_token();
    sqlx::query("INSERT INTO sessions(token_hash,user_id,expires_at) VALUES($1,$2,now()+interval '30 days')").bind(token_hash(&token)).bind(uid).execute(&s.db).await?;
    let mut out = HeaderMap::new();
    out.insert(
        header::SET_COOKIE,
        set_cookie(&token, s.config.secure_cookie)
            .parse()
            .map_err(|_| ApiError::internal())?,
    );
    Ok((
        out,
        Json(json!({"user":{"id":uid,"email":email,"name":name}})),
    ))
}
pub async fn logout(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<(HeaderMap, Json<Value>)> {
    if let Some(t) = session_cookie(&headers) {
        sqlx::query("DELETE FROM sessions WHERE token_hash=$1")
            .bind(token_hash(t))
            .execute(&s.db)
            .await?;
    }
    let mut out = HeaderMap::new();
    out.insert(
        header::SET_COOKIE,
        format!(
            "rp_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{}",
            if s.config.secure_cookie {
                "; Secure"
            } else {
                ""
            }
        )
        .parse()
        .map_err(|_| ApiError::internal())?,
    );
    Ok((out, Json(json!({"ok":true}))))
}
pub async fn me(State(s): State<AppState>, headers: HeaderMap) -> ApiResult<Json<Value>> {
    let p = principal(&s, &headers).await?;
    let orgs=sqlx::query_as::<_,(uuid::Uuid,String,String,String)>("SELECT o.id,o.name,o.kind,o.status FROM organizations o JOIN organization_members m ON m.organization_id=o.id WHERE m.user_id=$1 ORDER BY o.created_at").bind(p.id).fetch_all(&s.db).await?;
    Ok(Json(
        json!({"user":{"id":p.id,"email":p.email,"name":p.name,"is_admin":p.is_admin},"organizations":orgs.into_iter().map(|(id,name,kind,status)|json!({"id":id,"name":name,"kind":kind,"status":status})).collect::<Vec<_>>()}),
    ))
}
async fn limit(s: &AppState, headers: &HeaderMap, route: &str, max: i64) -> ApiResult<()> {
    let ip = headers
        .get("x-rp-client-ip")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");
    let key = format!(
        "{}:rate:{route}:{}",
        s.config.redis_namespace,
        hex::encode(Sha256::digest(ip.trim().as_bytes()))
    );
    let mut c = s.redis.clone();
    let n:i64=redis::Script::new("local n=redis.call('INCR',KEYS[1]); if n==1 then redis.call('EXPIRE',KEYS[1],60) end; return n").key(&key).invoke_async(&mut c).await.map_err(|_|ApiError::internal())?;
    if n > max {
        return Err(ApiError(
            StatusCode::TOO_MANY_REQUESTS,
            "RATE_LIMITED",
            "Try again in a minute".into(),
        ));
    }
    Ok(())
}
use axum::http::StatusCode;
