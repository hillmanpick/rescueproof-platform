use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use rescueproof::{AppState, config::Config, connect, router};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn call(
    app: &Router,
    method: &str,
    path: &str,
    cookie: &str,
    body: Value,
    key: &str,
) -> (StatusCode, Value, String) {
    let req = Request::builder()
        .method(method)
        .uri(format!("/api/v1{path}"))
        .header("content-type", "application/json")
        .header("x-requested-with", "RescueProof")
        .header("idempotency-key", key)
        .header("cookie", cookie)
        .body(if method == "GET" {
            Body::empty()
        } else {
            Body::from(body.to_string())
        })
        .unwrap();
    let res = app.clone().oneshot(req).await.unwrap();
    let status = res.status();
    let cookie = res
        .headers()
        .get("set-cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .to_owned();
    let raw = res.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&raw)
        .unwrap_or_else(|_| json!({"raw":String::from_utf8_lossy(&raw).to_string()}));
    (status, body, cookie)
}
async fn register(app: &Router, name: &str, kind: &str) -> (Value, String) {
    let email = format!("{}-{}@example.test", name, uuid::Uuid::new_v4().simple());
    let(s,b,c)=call(app,"POST","/auth/register","",json!({"email":email,"name":name,"password":"test-only-password-12345","organization_name":format!("Test {name}"),"organization_kind":kind}),"register-ignored").await;
    assert_eq!(s, StatusCode::OK, "{b}");
    (b, c)
}
async fn approve(s: &AppState, id: &str) {
    sqlx::query("UPDATE organizations SET status='approved' WHERE id=$1")
        .bind(uuid::Uuid::parse_str(id).unwrap())
        .execute(&s.db)
        .await
        .unwrap();
}
async fn new_batch(app: &Router, cookie: &str, org: &str, area: &str, key: &str) -> Value {
    let now = chrono::Utc::now();
    let(s,b,_)=call(app,"POST","/batches",cookie,json!({"organization_id":org,"title":"Fresh bakery surplus","category":"bakery","quantity":12,"unit":"portions","weight_kg":3.5,"area":area,"pickup_address":"Private pickup address","contact":"Private phone","storage_notes":"Keep sealed and cool","allergens":"Wheat and milk","pickup_start":now-chrono::Duration::hours(1),"pickup_end":now+chrono::Duration::hours(2),"expires_at":now+chrono::Duration::hours(3),"safety_acknowledged":true}),key).await;
    assert_eq!(s, StatusCode::OK, "{b}");
    b
}

#[tokio::test]
#[ignore = "requires a fresh TEST_DATABASE_URL and TEST_REDIS_URL; run with --ignored"]
async fn real_database_workflow() {
    let db = std::env::var("TEST_DATABASE_URL").expect("dedicated test database required");
    assert!(
        url::Url::parse(&db).unwrap().path().contains("test"),
        "Refusing a database without test in its name"
    );
    let s = connect(Config {
        database_url: db,
        redis_url: std::env::var("TEST_REDIS_URL").unwrap_or("redis://127.0.0.1:6379".into()),
        bind: "127.0.0.1:0".into(),
        origin: "http://localhost:5173".into(),
        secure_cookie: false,
        relayer_url: None,
        relayer_token: None,
        chain_id: None,
        contract_address: None,
        rpc_url: None,
        trust_proxy: false,
        redis_namespace: format!("rescueproof-test-{}", uuid::Uuid::new_v4()),
    })
    .await
    .unwrap();
    let app = router(s.clone());
    let (_, baseline, _) = call(&app, "GET", "/impact", "", json!(null), "unused000").await;
    let baseline_batches = baseline["completed_batches"].as_i64().unwrap();
    let baseline_claims = baseline["completed_claims"].as_i64().unwrap();
    let baseline_weight = baseline["completed_weight_kg"].as_f64().unwrap();
    let (donor, dc) = register(&app, "donor", "donor").await;
    let (recipient, rc) = register(&app, "recipient", "recipient").await;
    let (rival, xc) = register(&app, "rival", "recipient").await;
    let (pending, pc) = register(&app, "pending", "recipient").await;
    assert!(!donor["user"]["is_admin"].as_bool().unwrap());
    let dorg = donor["organization"]["id"].as_str().unwrap();
    let rorg = recipient["organization"]["id"].as_str().unwrap();
    let xorg = rival["organization"]["id"].as_str().unwrap();
    for id in [dorg, rorg, xorg] {
        approve(&s, id).await;
    }
    let scope = format!("pilot-{}", uuid::Uuid::new_v4().simple());
    let batch = new_batch(&app, &dc, dorg, &scope, "batch-create-0001").await;
    let bid = batch["id"].as_str().unwrap();
    let baseline_completions: i64 =
        sqlx::query_scalar("SELECT count(*) FROM handover_events WHERE kind='handover.completed'")
            .fetch_one(&s.db)
            .await
            .unwrap();
    let (status, public, _) = call(
        &app,
        "GET",
        &format!("/batches?area={scope}"),
        "",
        json!(null),
        "unused000",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(public["items"].as_array().unwrap().len(), 1);
    assert!(!public.to_string().contains("Private"));
    let (_, detail, _) = call(
        &app,
        "GET",
        &format!("/batches/{bid}"),
        &rc,
        json!(null),
        "unused000",
    )
    .await;
    assert!(detail["private"].is_null());
    let (status, _, _) = call(
        &app,
        "POST",
        &format!("/batches/{bid}/claims"),
        &pc,
        json!({"organization_id":pending["organization"]["id"]}),
        "pending-claim-0001",
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let path = format!("/batches/{bid}/claims");
    let (a, b) = tokio::join!(
        call(
            &app,
            "POST",
            &path,
            &rc,
            json!({"organization_id":rorg}),
            "race-claim-recipient"
        ),
        call(
            &app,
            "POST",
            &path,
            &xc,
            json!({"organization_id":xorg}),
            "race-claim-rival"
        )
    );
    assert_eq!(
        [a.0, b.0].iter().filter(|&&v| v == StatusCode::OK).count(),
        1,
        "Exactly one concurrent claimant wins"
    );
    assert!([a.0, b.0].contains(&StatusCode::CONFLICT));
    let (winner, wc, wo, key, loser) = if a.0 == StatusCode::OK {
        (a.1, &rc, rorg, "race-claim-recipient", &xc)
    } else {
        (b.1, &xc, xorg, "race-claim-rival", &rc)
    };
    let cid = winner["claim"]["id"].as_str().unwrap();
    let (status, replay, _) =
        call(&app, "POST", &path, wc, json!({"organization_id":wo}), key).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(winner, replay);
    let (status, _, _) = call(
        &app,
        "POST",
        &path,
        wc,
        json!({"organization_id":dorg}),
        key,
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (_, detail, _) = call(
        &app,
        "GET",
        &format!("/batches/{bid}"),
        wc,
        json!(null),
        "unused000",
    )
    .await;
    assert_eq!(
        detail["private"]["pickup_address"],
        "Private pickup address"
    );
    let (status, _, _) = call(
        &app,
        "GET",
        &format!("/batches/{bid}/proof"),
        loser,
        json!(null),
        "unused000",
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let receipt_path = format!("/claims/{cid}/receipt");
    let (status, _, _) = call(
        &app,
        "POST",
        &receipt_path,
        wc,
        json!({"actual_weight_kg":3.2}),
        "receipt-too-early",
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (_, old, _) = call(
        &app,
        "POST",
        &format!("/claims/{cid}/pickup-token"),
        wc,
        json!({}),
        "token-unused000",
    )
    .await;
    let (_, token, _) = call(
        &app,
        "POST",
        &format!("/claims/{cid}/pickup-token"),
        wc,
        json!({}),
        "token-unused001",
    )
    .await;
    assert_ne!(old["pickup_token"], token["pickup_token"]);
    let (status, _, _) = call(
        &app,
        "POST",
        &format!("/claims/{cid}/handover"),
        &dc,
        json!({"pickup_token":old["pickup_token"]}),
        "old-token-handover",
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    let (status, _, _) = call(
        &app,
        "POST",
        &format!("/claims/{cid}/handover"),
        loser,
        json!({"pickup_token":token["pickup_token"]}),
        "unauthorized-handover",
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _, _) = call(
        &app,
        "POST",
        &format!("/claims/{cid}/handover"),
        &dc,
        json!({"pickup_token":token["pickup_token"]}),
        "valid-handover-0001",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, receipt, _) = call(
        &app,
        "POST",
        &receipt_path,
        wc,
        json!({"actual_weight_kg":3.2}),
        "valid-receipt-0001",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, replayed, _) = call(
        &app,
        "POST",
        &receipt_path,
        wc,
        json!({"actual_weight_kg":3.2}),
        "valid-receipt-0001",
    )
    .await;
    assert_eq!(receipt, replayed);
    let (_, impact, _) = call(&app, "GET", "/impact", "", json!(null), "unused000").await;
    assert_eq!(
        impact["completed_batches"].as_i64().unwrap(),
        baseline_batches + 1
    );
    assert_eq!(
        impact["completed_claims"].as_i64().unwrap(),
        baseline_claims + 1
    );
    assert!(
        (impact["completed_weight_kg"].as_f64().unwrap() - baseline_weight - 3.2).abs() < 0.001
    );
    rescueproof::worker::tick(&s).await.unwrap();
    rescueproof::worker::tick(&s).await.unwrap();
    let (_, proof, _) = call(
        &app,
        "GET",
        &format!("/batches/{bid}/proof"),
        wc,
        json!(null),
        "unused000",
    )
    .await;
    let proofs = proof["items"].as_array().unwrap();
    assert!(!proofs.is_empty());
    for p in proofs {
        assert_eq!(p["status"], "pending");
        let leaf = rescueproof::merkle::leaf(p["canonical_payload"].as_str().unwrap().as_bytes());
        let mut computed = leaf;
        for sibling in p["proof"].as_array().unwrap() {
            let a: [u8; 32] = hex::decode(sibling.as_str().unwrap().trim_start_matches("0x"))
                .unwrap()
                .try_into()
                .unwrap();
            computed = rescueproof::merkle::pair(computed, a);
        }
        assert_eq!(
            format!("0x{}", hex::encode(computed)),
            p["root"].as_str().unwrap()
        );
        assert!(!p.to_string().contains("Private"));
    }
    let (status, _, _) = call(
        &app,
        "POST",
        &format!("/claims/{cid}/disputes"),
        wc,
        json!({"reason":"Quantity differs from the original description"}),
        "open-dispute-0001",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, impact, _) = call(&app, "GET", "/impact", "", json!(null), "unused000").await;
    assert_eq!(
        impact["completed_batches"].as_i64().unwrap(),
        baseline_batches
    );
    assert_eq!(
        impact["completed_claims"].as_i64().unwrap(),
        baseline_claims
    );
    assert!((impact["completed_weight_kg"].as_f64().unwrap() - baseline_weight).abs() < 0.001);
    let (status, _, _) = call(&app, "GET", "/admin", &dc, json!(null), "unused000").await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let csrf = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/logout")
        .header("cookie", &dc)
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(csrf).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    let cross = Request::builder()
        .method("POST")
        .uri("/api/v1/auth/logout")
        .header("cookie", &dc)
        .header("x-requested-with", "RescueProof")
        .header("origin", "https://evil.example")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(cross).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    call(
        &app,
        "POST",
        "/auth/logout",
        &dc,
        json!({}),
        "logout-ignored000",
    )
    .await;
    let (status, _, _) = call(&app, "GET", "/auth/me", &dc, json!(null), "unused000").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    // No forged on-chain confirmation or duplicate completion, even after worker retries.
    let completed: i64 =
        sqlx::query_scalar("SELECT count(*) FROM handover_events WHERE kind='handover.completed'")
            .fetch_one(&s.db)
            .await
            .unwrap();
    assert_eq!(completed, baseline_completions + 1);
}
