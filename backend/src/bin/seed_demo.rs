//! Explicit, repeatable fixtures for a loopback development database only.
use anyhow::{Context, ensure};
use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use rescueproof::{auth::random_token, config::Config, connect, worker};
use serde_json::{Value, json};
use sqlx::{Postgres, Transaction};
use std::{fs, io::Write, path::PathBuf};
use uuid::Uuid;

fn id(n: u128) -> Uuid {
    Uuid::from_u128(0xde000000_0000_4000_8000_000000000000 + n)
}

async fn event(
    tx: &mut Transaction<'_, Postgres>,
    batch: Uuid,
    claim: Option<Uuid>,
    actor: Uuid,
    descriptor: (u128, &str),
    payload: Value,
    at: chrono::DateTime<Utc>,
) -> anyhow::Result<()> {
    let (n, kind) = descriptor;
    sqlx::query("INSERT INTO handover_events(id,batch_id,claim_id,actor_id,kind,payload,created_at) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(id) DO NOTHING")
        .bind(id(n)).bind(batch).bind(claim).bind(actor).bind(kind).bind(payload).bind(at).execute(&mut **tx).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let existing_user = std::env::args()
        .nth(1)
        .context("Usage: seed_demo <existing-local-user-uuid>")?
        .parse::<Uuid>()?;
    let mut config = Config::load()?;
    let db_url = url::Url::parse(&config.database_url)?;
    let origin = url::Url::parse(&config.origin)?;
    ensure!(
        matches!(db_url.host_str(), Some("localhost" | "127.0.0.1" | "::1")),
        "Demo seeding requires a loopback database"
    );
    ensure!(
        matches!(origin.host_str(), Some("localhost" | "127.0.0.1" | "::1")),
        "Demo seeding requires a local PUBLIC_ORIGIN"
    );
    // Fixture creation never submits public-chain transactions, even if local anchoring is enabled.
    config.relayer_url = None;
    config.relayer_token = None;
    config.chain_id = None;
    config.contract_address = None;
    config.rpc_url = None;
    let state = connect(config).await?;
    ensure!(
        sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id=$1 AND disabled=false)"
        )
        .bind(existing_user)
        .fetch_one(&state.db)
        .await?,
        "Existing local user not found"
    );
    let credential_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../work/demo-accounts.json");
    fs::create_dir_all(credential_path.parent().unwrap())?;
    let credentials: Value = if credential_path.exists() {
        serde_json::from_slice(&fs::read(&credential_path)?)?
    } else {
        let data = json!({"notice":"Local demonstration accounts only. Do not publish this file.","accounts":[
            {"email":"donor@demo.rescueproof.test","role":"donor","password":random_token()},
            {"email":"recipient@demo.rescueproof.test","role":"recipient","password":random_token()},
            {"email":"admin@demo.rescueproof.test","role":"admin","password":random_token()}
        ]});
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        options
            .open(&credential_path)?
            .write_all(serde_json::to_string_pretty(&data)?.as_bytes())?;
        data
    };
    let mut tx = state.db.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(716281948)")
        .execute(&mut *tx)
        .await?;
    for (i, account) in credentials["accounts"]
        .as_array()
        .context("Invalid demo credential file")?
        .iter()
        .enumerate()
    {
        let uid = id(1 + i as u128);
        let hash = Argon2::default()
            .hash_password(
                account["password"]
                    .as_str()
                    .context("Missing password")?
                    .as_bytes(),
                &SaltString::generate(&mut OsRng),
            )
            .map_err(|_| anyhow::anyhow!("Cannot hash demo password"))?
            .to_string();
        sqlx::query("INSERT INTO users(id,email,name,password_hash,is_admin) VALUES($1,$2,$3,$4,$5) ON CONFLICT(id) DO NOTHING")
            .bind(uid).bind(account["email"].as_str().unwrap()).bind(["演示商家","演示接收方","演示管理员"][i]).bind(hash).bind(i == 2).execute(&mut *tx).await?;
    }
    for (i, name, kind, status) in [
        (10, "【演示】青芽面包坊", "donor", "approved"),
        (11, "【演示】田野果蔬站", "donor", "approved"),
        (12, "【演示】社区食堂", "donor", "approved"),
        (13, "【演示】邻里互助中心", "recipient", "approved"),
        (14, "【演示】待审核志愿站", "recipient", "pending"),
    ] {
        sqlx::query("INSERT INTO organizations(id,name,kind,status) VALUES($1,$2,$3,$4) ON CONFLICT(id) DO NOTHING")
            .bind(id(i)).bind(name).bind(kind).bind(status).execute(&mut *tx).await?;
        let member = if kind == "donor" { id(1) } else { id(2) };
        sqlx::query("INSERT INTO organization_members(user_id,organization_id,role) VALUES($1,$2,'owner') ON CONFLICT DO NOTHING").bind(member).bind(id(i)).execute(&mut *tx).await?;
        if kind == "donor" {
            sqlx::query("INSERT INTO organization_members(user_id,organization_id,role) VALUES($1,$2,'member') ON CONFLICT DO NOTHING").bind(existing_user).bind(id(i)).execute(&mut *tx).await?;
        }
        if status == "approved" {
            sqlx::query("INSERT INTO organization_reviews(id,organization_id,reviewer_id,decision,note) VALUES($1,$2,$3,'approved','本地演示审批，不代表真实机构核验') ON CONFLICT(id) DO NOTHING").bind(id(50+i)).bind(id(i)).bind(id(3)).execute(&mut *tx).await?;
        }
    }
    let fixtures = [
        (
            "手作全麦面包",
            "bakery",
            10,
            24,
            "items",
            6.0,
            "上海·静安",
            "open",
        ),
        (
            "当季番茄与青菜",
            "produce",
            11,
            12,
            "boxes",
            18.0,
            "上海·徐汇",
            "open",
        ),
        (
            "密封燕麦与牛奶",
            "packaged",
            12,
            30,
            "items",
            15.0,
            "上海·浦东",
            "open",
        ),
        (
            "新鲜蔬食午餐",
            "prepared",
            12,
            20,
            "portions",
            10.0,
            "上海·静安",
            "open",
        ),
        (
            "苹果与柑橘分享箱",
            "produce",
            11,
            10,
            "boxes",
            16.0,
            "上海·徐汇",
            "open",
        ),
        (
            "黄油可颂分享包",
            "bakery",
            10,
            18,
            "items",
            3.6,
            "上海·浦东",
            "open",
        ),
        (
            "待取货面包组合",
            "bakery",
            10,
            16,
            "items",
            4.0,
            "上海·静安",
            "claimed",
        ),
        (
            "待取货时蔬箱",
            "produce",
            11,
            8,
            "boxes",
            12.0,
            "上海·徐汇",
            "claimed",
        ),
        (
            "待确认收货餐盒",
            "prepared",
            12,
            15,
            "portions",
            7.5,
            "上海·浦东",
            "handed_over",
        ),
        (
            "已完成爱心午餐",
            "prepared",
            12,
            17,
            "portions",
            8.5,
            "上海·静安",
            "completed",
        ),
        (
            "已完成果蔬分享",
            "produce",
            11,
            8,
            "boxes",
            12.0,
            "上海·徐汇",
            "completed",
        ),
        (
            "数量待核实点心包",
            "bakery",
            10,
            20,
            "items",
            5.0,
            "上海·浦东",
            "disputed",
        ),
        (
            "已过期取货示例",
            "prepared",
            12,
            10,
            "portions",
            5.0,
            "上海·静安",
            "expired",
        ),
        (
            "已取消食品示例",
            "packaged",
            12,
            6,
            "boxes",
            6.0,
            "上海·徐汇",
            "cancelled",
        ),
    ];
    let now = Utc::now();
    let mut inserted = 0;
    for (i, (title, category, donor, quantity, unit, weight, area, status)) in
        fixtures.iter().enumerate()
    {
        let batch = id(100 + i as u128);
        let past = matches!(*status, "completed" | "disputed" | "expired" | "cancelled");
        let start = now - Duration::hours(if past { 28 } else { 1 });
        let end = if past {
            now - Duration::hours(24)
        } else {
            now + Duration::hours(24 + i as i64)
        };
        let count = sqlx::query("INSERT INTO food_batches(id,donor_id,title,category,quantity,unit,weight_kg,area,pickup_address,contact,storage_notes,allergens,pickup_start,pickup_end,expires_at,status,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,'演示地址：社区服务中心1号窗口（非真实取货点）','演示联系人（请勿实际联系）',$9,$10,$11,$12,$13,$14,$15) ON CONFLICT(id) DO NOTHING")
            .bind(batch).bind(id(*donor)).bind(format!("【演示】{title}")).bind(category).bind(quantity).bind(unit).bind(weight).bind(area)
            .bind(if *category == "prepared" {"演示说明：需冷藏；此条记录不对应真实食品"} else {"演示说明：密封保存；此条记录不对应真实食品"})
            .bind(if *category == "bakery" {"示例：小麦、奶制品"} else {"示例：请在实际交接前核实过敏原"})
            .bind(start).bind(end).bind(end+Duration::hours(3)).bind(status).bind(start-Duration::hours(2)).execute(&mut *tx).await?.rows_affected();
        if count == 0 {
            continue;
        }
        inserted += 1;
        let claim = if matches!(
            *status,
            "claimed" | "handed_over" | "completed" | "disputed"
        ) {
            Some(id(200 + i as u128))
        } else {
            None
        };
        if let Some(cid) = claim {
            sqlx::query("INSERT INTO claims(id,batch_id,recipient_id,status,actual_weight_kg,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7)")
                .bind(cid).bind(batch).bind(id(13)).bind(status).bind(if *status=="completed" {Some(*weight)} else {None}).bind(start-Duration::hours(1)).bind(start+Duration::minutes(30)).execute(&mut *tx).await?;
        }
        let e = 1000 + i as u128 * 10;
        event(
            &mut tx,
            batch,
            None,
            id(1),
            (e, "batch.created"),
            json!({"batch_id":batch,"donor_id":id(*donor),"status":"open","demo":true}),
            start - Duration::hours(2),
        )
        .await?;
        if let Some(cid) = claim {
            event(
                &mut tx,
                batch,
                Some(cid),
                id(2),
                (e + 1, "claim.created"),
                json!({"batch_id":batch,"claim_id":cid,"status":"claimed","demo":true}),
                start - Duration::hours(1),
            )
            .await?;
            if matches!(*status, "handed_over" | "completed" | "disputed") {
                event(
                    &mut tx,
                    batch,
                    Some(cid),
                    id(1),
                    (e + 2, "handover.donor_confirmed"),
                    json!({"batch_id":batch,"claim_id":cid,"status":"handed_over","demo":true}),
                    start,
                )
                .await?;
            }
            if *status == "completed" {
                event(&mut tx,batch,Some(cid),id(2),(e+3,"handover.completed"),json!({"batch_id":batch,"claim_id":cid,"status":"completed","actual_weight_kg":weight,"demo":true}),start+Duration::minutes(20)).await?;
            }
            if *status == "disputed" {
                let did = id(300 + i as u128);
                sqlx::query("INSERT INTO disputes(id,batch_id,claim_id,opened_by,reason,created_at) VALUES($1,$2,$3,$4,'【演示】实际数量与发布数量不一致，请双方核实；非真实投诉',$5)")
                    .bind(did).bind(batch).bind(cid).bind(id(2)).bind(start+Duration::minutes(20)).execute(&mut *tx).await?;
                event(
                    &mut tx,
                    batch,
                    Some(cid),
                    id(2),
                    (e + 4, "dispute.opened"),
                    json!({"batch_id":batch,"claim_id":cid,"dispute_id":did,"demo":true}),
                    start + Duration::minutes(20),
                )
                .await?;
            }
        }
        if *status == "cancelled" {
            event(
                &mut tx,
                batch,
                None,
                id(1),
                (e + 5, "batch.cancelled"),
                json!({"batch_id":batch,"status":"cancelled","demo":true}),
                start,
            )
            .await?;
        }
    }
    for org in [10, 11, 12, 13] {
        sqlx::query("INSERT INTO notifications(id,event_id,organization_id,message) VALUES($1,$2,$3,'【演示】本地测试数据已就绪：可查看交接时间线、待取货和争议处理。') ON CONFLICT DO NOTHING")
            .bind(id(400+org)).bind(id(500+org)).bind(id(org)).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    // Use the real batching implementation; no fake confirmed transaction hashes.
    worker::tick(&state).await?;
    println!(
        "{}",
        json!({"inserted_batches":inserted,"fixture_total":fixtures.len(),"credentials_file":credential_path,"current_user_demo_donor_memberships":3,"public_chain_transactions_submitted":0,"notice":"Demo fixtures only; completed weight is simulated, not real impact."})
    );
    Ok(())
}
