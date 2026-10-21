use crate::{
    AppState,
    auth::principal,
    domain::text,
    error::{ApiError, ApiResult},
};
use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

pub async fn dashboard(State(s): State<AppState>, h: HeaderMap) -> ApiResult<Json<Value>> {
    let p = principal(&s, &h).await?;
    let batches:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',b.id,'title',b.title,'status',b.status,'quantity',b.quantity,'unit',b.unit,'weight_kg',b.weight_kg,'area',b.area,'pickup_start',b.pickup_start,'pickup_end',b.pickup_end,'expires_at',b.expires_at,'category',b.category,'donor_name',o.name,'claim_id',c.id,'recipient_id',c.recipient_id,'recipient_name',r.name,'claim_status',c.status,'is_donor',EXISTS(SELECT 1 FROM organization_members dm WHERE dm.organization_id=b.donor_id AND dm.user_id=$1),'anchor_status',a.status) FROM food_batches b JOIN organizations o ON o.id=b.donor_id LEFT JOIN claims c ON c.batch_id=b.id AND c.status NOT IN ('cancelled','expired') LEFT JOIN organizations r ON r.id=c.recipient_id LEFT JOIN LATERAL (SELECT ab.status FROM anchor_leaves al JOIN anchor_batches ab ON ab.id=al.anchor_id JOIN handover_events he ON he.id=al.event_id WHERE he.batch_id=b.id ORDER BY ab.created_at DESC LIMIT 1) a ON true WHERE EXISTS(SELECT 1 FROM organization_members m WHERE m.user_id=$1 AND (m.organization_id=b.donor_id OR m.organization_id=c.recipient_id)) ORDER BY b.created_at DESC LIMIT 100").bind(p.id).fetch_all(&s.db).await?;
    let notifications:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',n.id,'message',n.message,'created_at',n.created_at) FROM notifications n JOIN organization_members m ON m.organization_id=n.organization_id WHERE m.user_id=$1 ORDER BY n.created_at DESC LIMIT 20").bind(p.id).fetch_all(&s.db).await?;
    Ok(Json(
        json!({"batches":batches,"notifications":notifications}),
    ))
}
pub async fn proof(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> ApiResult<Json<Value>> {
    let p = principal(&s, &h).await?;
    let allowed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM food_batches b LEFT JOIN claims c ON c.batch_id=b.id JOIN organization_members m ON m.user_id=$2 AND (m.organization_id=b.donor_id OR m.organization_id=c.recipient_id) WHERE b.id=$1)").bind(id).bind(p.id).fetch_one(&s.db).await?;
    if !allowed {
        return Err(ApiError::forbidden());
    }
    let evidence:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('event_id',l.event_id,'anchor_id',a.id,'root',a.root,'status',a.status,'transaction_hash',a.transaction_hash,'chain_id',a.chain_id,'contract_address',a.contract_address,'block_number',a.block_number,'leaf_hash',l.leaf_hash,'canonical_payload',l.canonical_payload,'proof',l.proof,'algorithm','keccak256-double-leaf-sorted-pairs-v1') FROM anchor_leaves l JOIN anchor_batches a ON a.id=l.anchor_id JOIN handover_events e ON e.id=l.event_id WHERE e.batch_id=$1 ORDER BY a.created_at").bind(id).fetch_all(&s.db).await?;
    Ok(Json(
        json!({"items":evidence,"status":if evidence.is_empty(){"pending"}else{"batched"},"notice":"Proofs establish record integrity, not food safety or physical delivery."}),
    ))
}
pub async fn events(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<uuid::Uuid>,
) -> ApiResult<Json<Value>> {
    let p = principal(&s, &h).await?;
    let allowed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM food_batches b LEFT JOIN claims c ON c.batch_id=b.id JOIN organization_members m ON m.user_id=$2 AND (m.organization_id=b.donor_id OR m.organization_id=c.recipient_id) WHERE b.id=$1)").bind(id).bind(p.id).fetch_one(&s.db).await?;
    if !allowed {
        return Err(ApiError::forbidden());
    }
    let rows:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',id,'kind',kind,'created_at',created_at,'payload',payload) FROM handover_events WHERE batch_id=$1 ORDER BY created_at,id").bind(id).fetch_all(&s.db).await?;
    Ok(Json(json!({"items":rows})))
}
pub async fn admin(State(s): State<AppState>, h: HeaderMap) -> ApiResult<Json<Value>> {
    if !principal(&s, &h).await?.is_admin {
        return Err(ApiError::forbidden());
    }
    let orgs:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',id,'name',name,'kind',kind,'status',status,'created_at',created_at) FROM organizations ORDER BY created_at DESC LIMIT 200").fetch_all(&s.db).await?;
    let disputes:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',d.id,'claim_id',d.claim_id,'batch_id',d.batch_id,'reason',d.reason,'resolution',d.resolution,'resolved_at',d.resolved_at,'title',b.title) FROM disputes d JOIN food_batches b ON b.id=d.batch_id ORDER BY d.created_at DESC LIMIT 100").fetch_all(&s.db).await?;
    let outbox:Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('id',id,'kind',kind,'attempts',attempts,'dead_at',dead_at,'last_error',last_error) FROM outbox_events WHERE dead_at IS NOT NULL ORDER BY created_at DESC LIMIT 30").fetch_all(&s.db).await?;
    Ok(Json(
        json!({"organizations":orgs,"disputes":disputes,"failed_tasks":outbox}),
    ))
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub decision: String,
    pub note: String,
}
pub async fn review(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<uuid::Uuid>,
    Json(b): Json<Review>,
) -> ApiResult<Json<Value>> {
    let p = principal(&s, &h).await?;
    if !p.is_admin {
        return Err(ApiError::forbidden());
    }
    if !["approved", "suspended"].contains(&b.decision.as_str()) {
        return Err(ApiError::bad("VALIDATION", "Invalid decision"));
    }
    text(&b.note, 5, 1000)?;
    let mut tx = s.db.begin().await?;
    let cmd = crate::idempotency::Command::new(&h, "organization.review", &(id, &b))?;
    if let Some(c) = cmd.begin(&mut tx, p.id).await? {
        return Ok(Json(c));
    }
    let count = sqlx::query("UPDATE organizations SET status=$2 WHERE id=$1")
        .bind(id)
        .bind(&b.decision)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if count == 0 {
        return Err(ApiError::missing());
    }
    sqlx::query("INSERT INTO organization_reviews(id,organization_id,reviewer_id,decision,note) VALUES($1,$2,$3,$4,$5)").bind(uuid::Uuid::new_v4()).bind(id).bind(p.id).bind(&b.decision).bind(b.note).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO audit_events(id,actor_id,action,entity_id) VALUES($1,$2,'organization.reviewed',$3)").bind(uuid::Uuid::new_v4()).bind(p.id).bind(id).execute(&mut *tx).await?;
    cmd.finish(tx, p.id, json!({"id":id,"status":b.decision}))
        .await
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Resolve {
    pub outcome: String,
    pub note: String,
}
pub async fn resolve(
    State(s): State<AppState>,
    h: HeaderMap,
    Path(id): Path<uuid::Uuid>,
    Json(b): Json<Resolve>,
) -> ApiResult<Json<Value>> {
    let p = principal(&s, &h).await?;
    if !p.is_admin {
        return Err(ApiError::forbidden());
    }
    text(&b.note, 5, 1000)?;
    if !["cancelled", "completed"].contains(&b.outcome.as_str()) {
        return Err(ApiError::bad(
            "VALIDATION",
            "Outcome must be cancelled or completed",
        ));
    }
    let mut tx = s.db.begin().await?;
    let cmd = crate::idempotency::Command::new(&h, "dispute.resolve", &(id, &b))?;
    if let Some(c) = cmd.begin(&mut tx, p.id).await? {
        return Ok(Json(c));
    }
    let r = sqlx::query_as::<
        _,
        (
            uuid::Uuid,
            uuid::Uuid,
            Option<chrono::DateTime<chrono::Utc>>,
        ),
    >("SELECT batch_id,claim_id,resolved_at FROM disputes WHERE id=$1 FOR UPDATE")
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(ApiError::missing)?;
    if r.2.is_some() {
        return Err(ApiError::conflict(
            "ALREADY_RESOLVED",
            "This dispute is already resolved",
        ));
    }
    // A reviewer cannot manufacture a completed handover that never received both confirmations.
    if b.outcome == "completed" {
        let confirmed:bool=sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM handover_events WHERE claim_id=$1 AND kind='handover.completed')").bind(r.1).fetch_one(&mut *tx).await?;
        if !confirmed {
            return Err(ApiError::conflict(
                "NO_RECEIPT",
                "Completion requires an existing recipient confirmation; cancel instead",
            ));
        }
    }
    sqlx::query("UPDATE disputes SET resolution=$2,resolved_at=now(),resolved_by=$3 WHERE id=$1")
        .bind(id)
        .bind(&b.note)
        .bind(p.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE food_batches SET status=$2 WHERE id=$1")
        .bind(r.0)
        .bind(&b.outcome)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE claims SET status=$2,updated_at=now() WHERE id=$1")
        .bind(r.1)
        .bind(&b.outcome)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO handover_events(id,batch_id,claim_id,actor_id,kind,payload) VALUES($1,$2,$3,$4,'dispute.resolved',$5)").bind(uuid::Uuid::new_v4()).bind(r.0).bind(r.1).bind(p.id).bind(json!({"batch_id":r.0,"claim_id":r.1,"dispute_id":id,"outcome":b.outcome})).execute(&mut *tx).await?;
    cmd.finish(tx, p.id, json!({"id":id,"outcome":b.outcome}))
        .await
}
