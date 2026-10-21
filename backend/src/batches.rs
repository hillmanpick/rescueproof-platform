use crate::idempotency::Command;
use crate::{
    AppState,
    auth::{Principal, principal, random_token, token_hash},
    domain::{CreateBatch, text},
    error::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::HeaderMap,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;

#[derive(Deserialize)]
pub struct ListQuery {
    pub area: Option<String>,
    pub category: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
#[derive(Serialize)]
pub struct BatchSummary {
    pub id: uuid::Uuid,
    pub title: String,
    pub category: String,
    pub quantity: i32,
    pub unit: String,
    pub weight_kg: f64,
    pub area: String,
    pub pickup_start: DateTime<Utc>,
    pub pickup_end: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub status: String,
    pub donor_name: String,
    pub created_at: DateTime<Utc>,
}
#[derive(Serialize)]
pub struct CreatedBatch {
    pub id: uuid::Uuid,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
pub async fn list(State(s): State<AppState>, q: Query<ListQuery>) -> ApiResult<Json<Value>> {
    if let Some(a) = &q.area {
        text(a, 1, 100)?;
    }
    if let Some(c) = &q.category
        && !["bakery", "produce", "packaged", "prepared"].contains(&c.as_str())
    {
        return Err(ApiError::bad("VALIDATION", "Invalid category"));
    }
    let limit = q.limit.unwrap_or(30).clamp(1, 100);
    let offset = q.offset.unwrap_or(0).max(0);
    let rows=sqlx::query_as::<_,(uuid::Uuid,String,String,i32,String,f64,String,DateTime<Utc>,DateTime<Utc>,DateTime<Utc>,String,String,DateTime<Utc>)>("SELECT b.id,b.title,b.category,b.quantity,b.unit,b.weight_kg::float8,b.area,b.pickup_start,b.pickup_end,b.expires_at,b.status,o.name,b.created_at FROM food_batches b JOIN organizations o ON o.id=b.donor_id WHERE b.status='open' AND b.pickup_end>now() AND o.status='approved' AND ($1::text IS NULL OR lower(b.area)=lower($1)) AND ($2::text IS NULL OR b.category=$2) ORDER BY b.pickup_end ASC,b.created_at DESC LIMIT $3 OFFSET $4")
        .bind(q.area.as_deref()).bind(q.category.as_deref()).bind(limit).bind(offset).fetch_all(&s.db).await?;
    let items: Vec<BatchSummary> = rows
        .into_iter()
        .map(|r| BatchSummary {
            id: r.0,
            title: r.1,
            category: r.2,
            quantity: r.3,
            unit: r.4,
            weight_kg: r.5,
            area: r.6,
            pickup_start: r.7,
            pickup_end: r.8,
            expires_at: r.9,
            status: r.10,
            donor_name: r.11,
            created_at: r.12,
        })
        .collect();
    Ok(Json(json!({"items":items,"limit":limit,"offset":offset})))
}
pub async fn create(
    State(s): State<AppState>,
    h: HeaderMap,
    Json(b): Json<CreateBatch>,
) -> ApiResult<Json<Value>> {
    let p = principal(&s, &h).await?;
    b.validate()?;
    ensure_org(&s, &p, b.organization_id, "donor").await?;
    let cmd = Command::new(&h, "batch.create", &b)?;
    let id = uuid::Uuid::new_v4();
    let mut tx = s.db.begin().await?;
    if let Some(c) = cmd.begin(&mut tx, p.id).await? {
        return Ok(Json(c));
    }
    ensure_org_tx(&mut tx, &p, b.organization_id, "donor").await?;
    sqlx::query("INSERT INTO food_batches(id,donor_id,title,category,quantity,unit,weight_kg,area,pickup_address,contact,storage_notes,allergens,pickup_start,pickup_end,expires_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)")
        .bind(id).bind(b.organization_id).bind(b.title.trim()).bind(&b.category).bind(b.quantity).bind(&b.unit).bind(b.weight_kg).bind(b.area.trim()).bind(b.pickup_address.trim()).bind(b.contact.trim()).bind(b.storage_notes.trim()).bind(b.allergens.trim()).bind(b.pickup_start).bind(b.pickup_end).bind(b.expires_at).execute(&mut *tx).await?;
    let payload = json!({"batch_id":id,"donor_id":b.organization_id,"status":"open"});
    record(&mut tx, &p, "batch.created", id, None, payload).await?;
    cmd.finish(
        tx,
        p.id,
        json!({"id":id,"status":"open","created_at":Utc::now()}),
    )
    .await
}
pub async fn detail(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> ApiResult<Json<Value>> {
    let p = principal(&s, &h).await?;
    let r=sqlx::query("SELECT b.id,b.donor_id,b.title,b.category,b.quantity,b.unit,b.weight_kg::float8,b.area,b.pickup_address,b.contact,b.storage_notes,b.allergens,b.pickup_start,b.pickup_end,b.expires_at,b.status,o.name FROM food_batches b JOIN organizations o ON o.id=b.donor_id WHERE b.id=$1").bind(id).fetch_optional(&s.db).await?.ok_or_else(ApiError::missing)?;
    let donor: uuid::Uuid = r.try_get("donor_id").map_err(|_| ApiError::internal())?;
    let is_donor = is_member(&s, p.id, donor).await?;
    let access = if is_donor {
        true
    } else {
        sqlx::query_scalar::<_,bool>("SELECT EXISTS(SELECT 1 FROM claims c JOIN organization_members m ON m.organization_id=c.recipient_id WHERE c.batch_id=$1 AND m.user_id=$2 AND c.status IN ('claimed','handed_over','completed','disputed'))").bind(id).bind(p.id).fetch_one(&s.db).await?
    };
    let public = json!({"id":r.try_get::<uuid::Uuid,_>("id").map_err(|_|ApiError::internal())?,"title":r.try_get::<String,_>("title").map_err(|_|ApiError::internal())?,"category":r.try_get::<String,_>("category").map_err(|_|ApiError::internal())?,"quantity":r.try_get::<i32,_>("quantity").map_err(|_|ApiError::internal())?,"unit":r.try_get::<String,_>("unit").map_err(|_|ApiError::internal())?,"weight_kg":r.try_get::<f64,_>("weight_kg").map_err(|_|ApiError::internal())?,"area":r.try_get::<String,_>("area").map_err(|_|ApiError::internal())?,"pickup_start":r.try_get::<DateTime<Utc>,_>("pickup_start").map_err(|_|ApiError::internal())?,"pickup_end":r.try_get::<DateTime<Utc>,_>("pickup_end").map_err(|_|ApiError::internal())?,"expires_at":r.try_get::<DateTime<Utc>,_>("expires_at").map_err(|_|ApiError::internal())?,"status":r.try_get::<String,_>("status").map_err(|_|ApiError::internal())?,"donor_name":r.try_get::<String,_>("name").map_err(|_|ApiError::internal())?});
    if access {
        Ok(Json(
            json!({"batch":public,"private":{"pickup_address":r.try_get::<String,_>("pickup_address").map_err(|_|ApiError::internal())?,"contact":r.try_get::<String,_>("contact").map_err(|_|ApiError::internal())?,"storage_notes":r.try_get::<String,_>("storage_notes").map_err(|_|ApiError::internal())?,"allergens":r.try_get::<String,_>("allergens").map_err(|_|ApiError::internal())?}}),
        ))
    } else {
        Ok(Json(json!({"batch":public,"private":null})))
    }
}
pub async fn cancel(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> ApiResult<Json<Value>> {
    let p = principal(&s, &h).await?;
    let mut tx = s.db.begin().await?;
    let cmd = Command::new(&h, "batch.cancel", &id)?;
    if let Some(c) = cmd.begin(&mut tx, p.id).await? {
        return Ok(Json(c));
    }
    let row = sqlx::query_as::<_, (uuid::Uuid, String)>(
        "SELECT donor_id,status FROM food_batches WHERE id=$1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(ApiError::missing)?;
    ensure_org_tx(&mut tx, &p, row.0, "donor").await?;
    if row.1 != "open" {
        return Err(ApiError::conflict(
            "INVALID_STATE",
            "Only open batches can be cancelled",
        ));
    }
    sqlx::query("UPDATE food_batches SET status='cancelled' WHERE id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    record(
        &mut tx,
        &p,
        "batch.cancelled",
        id,
        None,
        json!({"batch_id":id,"status":"cancelled"}),
    )
    .await?;
    cmd.finish(tx, p.id, json!({"id":id,"status":"cancelled"}))
        .await
}
pub async fn claim(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<uuid::Uuid>,
    body: Option<Json<Value>>,
) -> ApiResult<Json<Value>> {
    let p = principal(&s, &h).await?;
    let org = body
        .as_ref()
        .and_then(|v| v.0.get("organization_id"))
        .and_then(Value::as_str)
        .and_then(|v| uuid::Uuid::parse_str(v).ok())
        .ok_or_else(|| ApiError::bad("VALIDATION", "organization_id is required"))?;
    let mut tx = s.db.begin().await?;
    let cmd = Command::new(&h, "batch.claim", &(id, org))?;
    if let Some(c) = cmd.begin(&mut tx, p.id).await? {
        return Ok(Json(c));
    }
    ensure_org_tx(&mut tx, &p, org, "recipient").await?;
    let batch = sqlx::query_as::<_, (uuid::Uuid, String, DateTime<Utc>)>(
        "SELECT donor_id,status,pickup_end FROM food_batches WHERE id=$1 FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(ApiError::missing)?;
    if batch.1 != "open" || batch.2 <= Utc::now() {
        return Err(ApiError::conflict(
            "BATCH_UNAVAILABLE",
            "This batch is no longer available",
        ));
    }
    if batch.0 == org {
        return Err(ApiError::forbidden());
    }
    let cid = uuid::Uuid::new_v4();
    sqlx::query("INSERT INTO claims(id,batch_id,recipient_id,status) VALUES($1,$2,$3,'claimed')")
        .bind(cid)
        .bind(id)
        .bind(org)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE food_batches SET status='claimed' WHERE id=$1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    record(
        &mut tx,
        &p,
        "claim.created",
        id,
        Some(cid),
        json!({"batch_id":id,"claim_id":cid,"status":"claimed"}),
    )
    .await?;
    sqlx::query("INSERT INTO outbox_events(id,kind,payload) VALUES($1,'notify_donor',$2)")
        .bind(uuid::Uuid::new_v4())
        .bind(json!({"batch_id":id,"claim_id":cid,"donor_id":batch.0}))
        .execute(&mut *tx)
        .await?;
    cmd.finish(
        tx,
        p.id,
        json!({"claim":{"id":cid,"batch_id":id,"status":"claimed"}}),
    )
    .await
}
// Intentionally not idempotency-cached: token rotation returns a capability only once.
pub async fn pickup_token(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(cid): Path<uuid::Uuid>,
) -> ApiResult<Json<Value>> {
    let p = principal(&s, &h).await?;
    let mut tx = s.db.begin().await?;
    let r=sqlx::query_as::<_,(uuid::Uuid,String,DateTime<Utc>)>("SELECT c.recipient_id,c.status,b.pickup_end FROM claims c JOIN food_batches b ON b.id=c.batch_id WHERE c.id=$1 FOR UPDATE OF b,c").bind(cid).fetch_optional(&mut *tx).await?.ok_or_else(ApiError::missing)?;
    ensure_org_tx(&mut tx, &p, r.0, "recipient").await?;
    if r.1 != "claimed" || r.2 <= Utc::now() {
        return Err(ApiError::conflict(
            "INVALID_STATE",
            "This claim is not awaiting pickup",
        ));
    }
    let expires = r.2.min(Utc::now() + chrono::Duration::minutes(30));
    let token = random_token();
    sqlx::query("UPDATE claims SET pickup_token_hash=$2,token_expires_at=$3 WHERE id=$1")
        .bind(cid)
        .bind(token_hash(&token))
        .bind(expires)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(
        json!({"pickup_token":token,"expires_at":expires,"claim_id":cid}),
    ))
}
pub async fn cancel_claim(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(cid): Path<uuid::Uuid>,
) -> ApiResult<Json<Value>> {
    let p = principal(&s, &h).await?;
    let mut tx = s.db.begin().await?;
    let cmd = Command::new(&h, "claim.cancel", &cid)?;
    if let Some(c) = cmd.begin(&mut tx, p.id).await? {
        return Ok(Json(c));
    }
    let r=sqlx::query_as::<_,(uuid::Uuid,uuid::Uuid,String,DateTime<Utc>)>("SELECT c.batch_id,c.recipient_id,c.status,b.pickup_end FROM claims c JOIN food_batches b ON b.id=c.batch_id WHERE c.id=$1 FOR UPDATE OF b,c").bind(cid).fetch_optional(&mut *tx).await?.ok_or_else(ApiError::missing)?;
    ensure_org_tx(&mut tx, &p, r.1, "recipient").await?;
    if r.2 != "claimed" {
        return Err(ApiError::conflict(
            "INVALID_STATE",
            "Only uncollected claims can be cancelled",
        ));
    }
    let status = if r.3 > Utc::now() { "open" } else { "expired" };
    sqlx::query("UPDATE claims SET status='cancelled',pickup_token_hash=NULL,token_expires_at=NULL,updated_at=now() WHERE id=$1").bind(cid).execute(&mut *tx).await?;
    sqlx::query("UPDATE food_batches SET status=$2 WHERE id=$1")
        .bind(r.0)
        .bind(status)
        .execute(&mut *tx)
        .await?;
    record(
        &mut tx,
        &p,
        "claim.cancelled",
        r.0,
        Some(cid),
        json!({"batch_id":r.0,"claim_id":cid,"status":status}),
    )
    .await?;
    cmd.finish(tx, p.id, json!({"claim_id":cid,"status":"cancelled"}))
        .await
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TokenBody {
    pub pickup_token: String,
}
pub async fn handover(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(cid): Path<uuid::Uuid>,
    Json(b): Json<TokenBody>,
) -> ApiResult<Json<Value>> {
    if b.pickup_token.len() != 64 || !b.pickup_token.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(ApiError::bad("INVALID_TOKEN", "Invalid pickup token"));
    }
    let p = principal(&s, &h).await?;
    let mut tx = s.db.begin().await?;
    let cmd = Command::new(&h, "claim.handover", &(cid, &b))?;
    if let Some(c) = cmd.begin(&mut tx, p.id).await? {
        return Ok(Json(c));
    }
    let r=sqlx::query_as::<_,(uuid::Uuid,uuid::Uuid,uuid::Uuid,String,Option<String>,Option<DateTime<Utc>>)>("SELECT c.batch_id,c.recipient_id,b.donor_id,c.status,c.pickup_token_hash,c.token_expires_at FROM claims c JOIN food_batches b ON b.id=c.batch_id WHERE c.id=$1 FOR UPDATE OF c,b").bind(cid).fetch_optional(&mut *tx).await?.ok_or_else(ApiError::missing)?;
    ensure_org_tx(&mut tx, &p, r.2, "donor").await?;
    if r.3 != "claimed"
        || r.4.as_deref() != Some(token_hash(&b.pickup_token).as_str())
        || r.5.is_none_or(|t| t <= Utc::now())
    {
        return Err(ApiError::conflict(
            "INVALID_HANDOVER",
            "Claim token is invalid, expired or already used",
        ));
    }
    sqlx::query("UPDATE claims SET status='handed_over',pickup_token_hash=NULL,token_expires_at=NULL,updated_at=now() WHERE id=$1").bind(cid).execute(&mut *tx).await?;
    sqlx::query("UPDATE food_batches SET status='handed_over' WHERE id=$1")
        .bind(r.0)
        .execute(&mut *tx)
        .await?;
    record(
        &mut tx,
        &p,
        "handover.donor_confirmed",
        r.0,
        Some(cid),
        json!({"batch_id":r.0,"claim_id":cid,"status":"handed_over"}),
    )
    .await?;
    sqlx::query("INSERT INTO outbox_events(id,kind,payload) VALUES($1,'notify_recipient',$2)")
        .bind(uuid::Uuid::new_v4())
        .bind(json!({"batch_id":r.0,"claim_id":cid,"recipient_id":r.1}))
        .execute(&mut *tx)
        .await?;
    cmd.finish(tx, p.id, json!({"claim_id":cid,"status":"handed_over"}))
        .await
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptBody {
    pub actual_weight_kg: Option<f64>,
}
pub async fn receipt(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(cid): Path<uuid::Uuid>,
    Json(b): Json<ReceiptBody>,
) -> ApiResult<Json<Value>> {
    if b.actual_weight_kg
        .is_some_and(|w| !w.is_finite() || !(0.0..=100000.0).contains(&w))
    {
        return Err(ApiError::bad("VALIDATION", "Invalid received weight"));
    }
    let p = principal(&s, &h).await?;
    let mut tx = s.db.begin().await?;
    let cmd = Command::new(&h, "claim.receipt", &(cid, &b))?;
    if let Some(c) = cmd.begin(&mut tx, p.id).await? {
        return Ok(Json(c));
    }
    let r=sqlx::query_as::<_,(uuid::Uuid,uuid::Uuid,String)>("SELECT c.batch_id,c.recipient_id,c.status FROM claims c JOIN food_batches b ON b.id=c.batch_id WHERE c.id=$1 FOR UPDATE OF c,b").bind(cid).fetch_optional(&mut *tx).await?.ok_or_else(ApiError::missing)?;
    ensure_org_tx(&mut tx, &p, r.1, "recipient").await?;
    if r.2 != "handed_over" {
        return Err(ApiError::conflict(
            "INVALID_STATE",
            "The donor must confirm handover first",
        ));
    }
    sqlx::query(
        "UPDATE claims SET status='completed',actual_weight_kg=$2,updated_at=now() WHERE id=$1",
    )
    .bind(cid)
    .bind(b.actual_weight_kg)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE food_batches SET status='completed' WHERE id=$1")
        .bind(r.0)
        .execute(&mut *tx)
        .await?;
    record(&mut tx,&p,"handover.completed",r.0,Some(cid),json!({"batch_id":r.0,"claim_id":cid,"status":"completed","actual_weight_kg":b.actual_weight_kg})).await?;
    sqlx::query("INSERT INTO outbox_events(id,kind,payload) VALUES($1,'anchor_event',$2)")
        .bind(uuid::Uuid::new_v4())
        .bind(json!({"batch_id":r.0,"claim_id":cid}))
        .execute(&mut *tx)
        .await?;
    cmd.finish(
        tx,
        p.id,
        json!({"claim_id":cid,"status":"completed","anchor_status":"pending"}),
    )
    .await
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DisputeBody {
    pub reason: String,
}
pub async fn dispute(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(cid): Path<uuid::Uuid>,
    Json(b): Json<DisputeBody>,
) -> ApiResult<Json<Value>> {
    text(&b.reason, 5, 1000)?;
    let p = principal(&s, &h).await?;
    let mut tx = s.db.begin().await?;
    let cmd = Command::new(&h, "claim.dispute", &(cid, &b))?;
    if let Some(c) = cmd.begin(&mut tx, p.id).await? {
        return Ok(Json(c));
    }
    let r=sqlx::query_as::<_,(uuid::Uuid,uuid::Uuid,uuid::Uuid,String)>("SELECT c.batch_id,c.recipient_id,b.donor_id,c.status FROM claims c JOIN food_batches b ON b.id=c.batch_id WHERE c.id=$1 FOR UPDATE OF c,b").bind(cid).fetch_optional(&mut *tx).await?.ok_or_else(ApiError::missing)?;
    if !is_member_tx(&mut tx, p.id, r.1).await? && !is_member_tx(&mut tx, p.id, r.2).await? {
        return Err(ApiError::forbidden());
    }
    if !["claimed", "handed_over", "completed"].contains(&r.3.as_str()) {
        return Err(ApiError::conflict(
            "INVALID_STATE",
            "This claim cannot be disputed",
        ));
    }
    let did = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO disputes(id,batch_id,claim_id,opened_by,reason) VALUES($1,$2,$3,$4,$5)",
    )
    .bind(did)
    .bind(r.0)
    .bind(cid)
    .bind(p.id)
    .bind(b.reason.trim())
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE claims SET status='disputed',updated_at=now() WHERE id=$1")
        .bind(cid)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE food_batches SET status='disputed' WHERE id=$1")
        .bind(r.0)
        .execute(&mut *tx)
        .await?;
    record(
        &mut tx,
        &p,
        "dispute.opened",
        r.0,
        Some(cid),
        json!({"batch_id":r.0,"claim_id":cid,"dispute_id":did}),
    )
    .await?;
    cmd.finish(tx, p.id, json!({"dispute_id":did,"status":"open"}))
        .await
}

pub async fn impact(State(s): State<AppState>) -> ApiResult<Json<Value>> {
    let r=sqlx::query_as::<_,(i64,i64,f64,i64)>("SELECT count(*),count(*) FILTER(WHERE status='completed'),COALESCE(sum(actual_weight_kg) FILTER(WHERE status='completed'),0)::float8,count(*) FILTER(WHERE status='disputed') FROM claims").fetch_one(&s.db).await?;
    let batches: i64 =
        sqlx::query_scalar("SELECT count(*) FROM food_batches WHERE status='completed'")
            .fetch_one(&s.db)
            .await?;
    Ok(Json(
        json!({"completed_batches":batches,"claims":r.0,"completed_claims":r.1,"completed_weight_kg":r.2,"disputes":r.3,"evidence":"self-reported by participating organisations; not independently audited"}),
    ))
}

pub async fn ensure_org(s: &AppState, p: &Principal, id: uuid::Uuid, kind: &str) -> ApiResult<()> {
    let ok:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM organization_members m JOIN organizations o ON o.id=m.organization_id WHERE m.user_id=$1 AND o.id=$2 AND o.kind=$3 AND o.status='approved')").bind(p.id).bind(id).bind(kind).fetch_one(&s.db).await?;
    if ok {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}
async fn ensure_org_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    p: &Principal,
    id: uuid::Uuid,
    kind: &str,
) -> ApiResult<()> {
    let ok:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM organization_members m JOIN organizations o ON o.id=m.organization_id WHERE m.user_id=$1 AND o.id=$2 AND o.kind=$3 AND o.status='approved')").bind(p.id).bind(id).bind(kind).fetch_one(&mut **tx).await?;
    if ok {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}
async fn is_member(s: &AppState, user: uuid::Uuid, org: uuid::Uuid) -> ApiResult<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM organization_members WHERE user_id=$1 AND organization_id=$2)",
    )
    .bind(user)
    .bind(org)
    .fetch_one(&s.db)
    .await?)
}
async fn is_member_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user: uuid::Uuid,
    org: uuid::Uuid,
) -> ApiResult<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM organization_members WHERE user_id=$1 AND organization_id=$2)",
    )
    .bind(user)
    .bind(org)
    .fetch_one(&mut **tx)
    .await?)
}
async fn record(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    p: &Principal,
    kind: &str,
    batch: uuid::Uuid,
    claim: Option<uuid::Uuid>,
    payload: Value,
) -> ApiResult<()> {
    let id = uuid::Uuid::new_v4();
    sqlx::query("INSERT INTO handover_events(id,batch_id,claim_id,actor_id,kind,payload) VALUES($1,$2,$3,$4,$5,$6)").bind(id).bind(batch).bind(claim).bind(p.id).bind(kind).bind(payload).execute(&mut **tx).await?;
    sqlx::query("INSERT INTO audit_events(id,actor_id,action,entity_id) VALUES($1,$2,$3,$4)")
        .bind(uuid::Uuid::new_v4())
        .bind(p.id)
        .bind(kind)
        .bind(batch)
        .execute(&mut **tx)
        .await?;
    Ok(())
}
