use crate::{AppState, merkle};
use serde_json::{Value, json};
use uuid::Uuid;

pub async fn run(s: AppState) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
    loop {
        tokio::select! {
            _=interval.tick()=>{if let Err(e)=tick(&s).await {tracing::error!(error=%e,"worker tick failed");}}
            _=tokio::signal::ctrl_c()=>break,
        }
    }
    Ok(())
}
pub async fn tick(s: &AppState) -> anyhow::Result<()> {
    // The database transaction and SKIP LOCKED allow multiple worker processes.
    // No external network call is made while this transaction holds row locks.
    let mut tx = s.db.begin().await?;
    let events:Vec<(Uuid,String,Value)>=sqlx::query_as("SELECT id,kind,payload FROM outbox_events WHERE kind!='relay_anchor' AND processed_at IS NULL AND dead_at IS NULL AND available_at<=now() ORDER BY created_at,id LIMIT 50 FOR UPDATE SKIP LOCKED").fetch_all(&mut *tx).await?;
    for (id, kind, payload) in events {
        match kind.as_str() {
            "notify_donor" | "notify_recipient" => {
                let field = if kind == "notify_donor" {
                    "donor_id"
                } else {
                    "recipient_id"
                };
                if let Some(org) = payload
                    .get(field)
                    .and_then(Value::as_str)
                    .and_then(|v| Uuid::parse_str(v).ok())
                {
                    let message = if kind == "notify_donor" {
                        "A community organisation has claimed your food batch."
                    } else {
                        "The donor confirmed handover. Please confirm receipt after checking the delivery."
                    };
                    sqlx::query("INSERT INTO notifications(id,event_id,organization_id,message) VALUES($1,$2,$3,$4) ON CONFLICT(event_id,organization_id) DO NOTHING").bind(Uuid::new_v4()).bind(id).bind(org).bind(message).execute(&mut *tx).await?;
                }
            }
            "anchor_event" => {}
            _ => {
                sqlx::query("UPDATE outbox_events SET attempts=attempts+1,dead_at=now(),last_error='Unsupported task kind' WHERE id=$1").bind(id).execute(&mut *tx).await?;
                continue;
            }
        }
        sqlx::query("UPDATE outbox_events SET processed_at=now() WHERE id=$1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    expire(s).await?;
    make_anchor(s).await?;
    relay(s).await?;
    Ok(())
}
async fn expire(s: &AppState) -> anyhow::Result<()> {
    let mut tx = s.db.begin().await?;
    let ids:Vec<Uuid>=sqlx::query_scalar("SELECT id FROM food_batches WHERE status IN ('open','claimed') AND pickup_end<=now() ORDER BY id LIMIT 100 FOR UPDATE SKIP LOCKED").fetch_all(&mut *tx).await?;
    if !ids.is_empty() {
        sqlx::query("UPDATE claims SET status='expired',pickup_token_hash=NULL,token_expires_at=NULL,updated_at=now() WHERE batch_id=ANY($1) AND status='claimed'").bind(&ids).execute(&mut *tx).await?;
        sqlx::query("UPDATE food_batches SET status='expired' WHERE id=ANY($1)")
            .bind(&ids)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query("DELETE FROM sessions WHERE expires_at<now()")
        .execute(&mut *tx)
        .await?;
    // Keep successful idempotency responses for 7 days. Clients must not retry older requests.
    sqlx::query("DELETE FROM idempotency_keys WHERE created_at<now()-interval '7 days'")
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
async fn make_anchor(s: &AppState) -> anyhow::Result<()> {
    let mut tx = s.db.begin().await?;
    let locked: bool = sqlx::query_scalar("SELECT pg_try_advisory_xact_lock(716281947)")
        .fetch_one(&mut *tx)
        .await?;
    if !locked {
        return Ok(());
    }
    // All state changes are anchored, including disputes and corrections. No actor or PII is exposed.
    let events:Vec<(Uuid,Uuid,String,Value,chrono::DateTime<chrono::Utc>)>=sqlx::query_as("SELECT e.id,e.batch_id,e.kind,e.payload,e.created_at FROM handover_events e LEFT JOIN anchor_leaves l ON l.event_id=e.id WHERE l.event_id IS NULL ORDER BY e.created_at,e.id LIMIT 100").fetch_all(&mut *tx).await?;
    if events.is_empty() {
        return Ok(());
    }
    let canonical: Vec<String> = events
        .iter()
        .map(|(id, batch, kind, payload, date)| {
            // Explicit fixed-order tuple is the versioned canonical encoding.
            serde_json::to_string(&json!([
                "rescueproof:v1",
                id,
                batch,
                kind,
                payload,
                date.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
            ]))
            .expect("JSON tuple serializes")
        })
        .collect();
    let leaves: Vec<_> = canonical
        .iter()
        .map(|s| merkle::leaf(s.as_bytes()))
        .collect();
    let (root, proofs) = merkle::build(&leaves).ok_or_else(|| anyhow::anyhow!("empty tree"))?;
    let aid = Uuid::new_v4();
    sqlx::query("INSERT INTO anchor_batches(id,root) VALUES($1,$2)")
        .bind(aid)
        .bind(format!("0x{}", hex::encode(root)))
        .execute(&mut *tx)
        .await?;
    for (i, ((event, _, _, _, _), payload)) in events.iter().zip(canonical.iter()).enumerate() {
        let proof: Vec<String> = proofs[i]
            .iter()
            .map(|p| format!("0x{}", hex::encode(p)))
            .collect();
        sqlx::query("INSERT INTO anchor_leaves(event_id,anchor_id,leaf_index,leaf_hash,canonical_payload,proof) VALUES($1,$2,$3,$4,$5,$6)").bind(event).bind(aid).bind(i as i32).bind(format!("0x{}",hex::encode(leaves[i]))).bind(payload).bind(json!(proof)).execute(&mut *tx).await?;
    }
    sqlx::query("INSERT INTO outbox_events(id,kind,payload) VALUES($1,'relay_anchor',$2)")
        .bind(Uuid::new_v4())
        .bind(json!({"anchor_id":aid,"root":format!("0x{}",hex::encode(root))}))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}
async fn relay(s: &AppState) -> anyhow::Result<()> {
    let (Some(base), Some(secret), Some(chain), Some(contract), Some(rpc)) = (
        &s.config.relayer_url,
        &s.config.relayer_token,
        s.config.chain_id,
        &s.config.contract_address,
        &s.config.rpc_url,
    ) else {
        return Ok(());
    };
    let url = url::Url::parse(base)?;
    anyhow::ensure!(
        url.scheme() == "https"
            || url.scheme() == "http"
                && matches!(
                    url.host_str(),
                    Some("localhost" | "127.0.0.1" | "::1" | "relayer")
                ),
        "Relayer must use HTTPS"
    );
    let mut tx = s.db.begin().await?;
    let job:Option<(Uuid,Value,i32)>=sqlx::query_as("SELECT id,payload,attempts FROM outbox_events WHERE kind='relay_anchor' AND processed_at IS NULL AND dead_at IS NULL AND available_at<=now() ORDER BY created_at LIMIT 1 FOR UPDATE SKIP LOCKED").fetch_optional(&mut *tx).await?;
    let Some((id, payload, attempts)) = job else {
        return Ok(());
    };
    // A durable lease: if this worker crashes, another can retry after 2 minutes.
    sqlx::query("UPDATE outbox_events SET attempts=attempts+1,available_at=now()+interval '2 minutes' WHERE id=$1").bind(id).execute(&mut *tx).await?;
    tx.commit().await?;
    let aid = payload["anchor_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("invalid anchor payload"))?;
    let root = payload["root"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("invalid root"))?;
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()?;
    let result=async {
        let res:Value=client.post(format!("{}/anchors",base.trim_end_matches('/'))).bearer_auth(secret).header("Idempotency-Key",aid).json(&json!({"anchor_id":aid,"root":root,"chain_id":chain,"contract_address":contract})).send().await?.error_for_status()?.json().await?;
        let hash=res["transaction_hash"].as_str().filter(|h|h.len()==66&&h.starts_with("0x")&&h[2..].bytes().all(|b|b.is_ascii_hexdigit())).ok_or_else(||anyhow::anyhow!("invalid transaction hash"))?;
        sqlx::query("UPDATE anchor_batches SET status='submitted',transaction_hash=$2,chain_id=$3,contract_address=$4 WHERE id=$1 AND status!='confirmed'").bind(Uuid::parse_str(aid)?).bind(hash).bind(chain).bind(contract).execute(&s.db).await?;
        // Verify receipt AND the exact root through eth_call to our configured contract.
        let receipt:Value=client.post(rpc).json(&json!({"jsonrpc":"2.0","id":1,"method":"eth_getTransactionReceipt","params":[hash]})).send().await?.error_for_status()?.json().await?;
        let r=&receipt["result"];
        anyhow::ensure!(r["status"]=="0x1"&&r["to"].as_str().is_some_and(|a|a.eq_ignore_ascii_case(contract)),"Transaction not confirmed by expected contract");
        let block=r["blockNumber"].as_str().ok_or_else(||anyhow::anyhow!("missing block number"))?;
        let bn=i64::from_str_radix(block.trim_start_matches("0x"),16)?;
        let mut selector=sha3::Keccak256::new();use sha3::Digest;selector.update(b"roots(bytes32)");
        let data=format!("0x{}{}",hex::encode(&selector.finalize()[..4]),root.trim_start_matches("0x"));
        let check:Value=client.post(rpc).json(&json!({"jsonrpc":"2.0","id":2,"method":"eth_call","params":[{"to":contract,"data":data},block]})).send().await?.error_for_status()?.json().await?;
        anyhow::ensure!(check["result"].as_str().is_some_and(|r|u64::from_str_radix(r.trim_start_matches("0x"),16).unwrap_or(0)>0),"Root is not recorded in contract");
        let chain_res:Value=client.post(rpc).json(&json!({"jsonrpc":"2.0","id":3,"method":"eth_chainId","params":[]})).send().await?.error_for_status()?.json().await?;
        anyhow::ensure!(chain_res["result"].as_str().and_then(|v|i64::from_str_radix(v.trim_start_matches("0x"),16).ok())==Some(chain),"RPC chain mismatch");
        let mut tx=s.db.begin().await?;
        sqlx::query("UPDATE anchor_batches SET status='confirmed',block_number=$2,confirmed_at=now() WHERE id=$1").bind(Uuid::parse_str(aid)?).bind(bn).execute(&mut *tx).await?;
        sqlx::query("UPDATE outbox_events SET processed_at=now(),last_error=NULL WHERE id=$1").bind(id).execute(&mut *tx).await?;
        tx.commit().await?;Ok::<_,anyhow::Error>(())
    }.await;
    if result.is_err() {
        // Never persist upstream bodies or URLs: they can contain credentials.
        let delay = 2_i64.pow((attempts + 1).min(10) as u32).min(3600);
        sqlx::query("UPDATE outbox_events SET available_at=now()+make_interval(secs=>$2::float8),dead_at=CASE WHEN attempts>=10 THEN now() ELSE NULL END,last_error='Relayer or chain verification failed; inspect configured endpoints' WHERE id=$1").bind(id).bind(delay as f64).execute(&s.db).await?;
        tracing::warn!(job_id=%id,"anchor relay deferred");
    }
    Ok(())
}
