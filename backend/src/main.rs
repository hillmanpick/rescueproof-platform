use rescueproof::{config::Config, connect, router, worker};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("rescueproof=info,tower_http=info")),
        )
        .json()
        .init();
    let args: Vec<String> = std::env::args().collect();
    let cfg = Config::load()?;
    let state = connect(cfg).await?;
    match args.get(1).map(String::as_str).unwrap_or("serve") {
        "serve" => {
            let listener = tokio::net::TcpListener::bind(&state.config.bind).await?;
            tracing::info!(address=%state.config.bind,"API listening");
            axum::serve(
                listener,
                router(state).into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .with_graceful_shutdown(async {
                tokio::signal::ctrl_c().await.ok();
            })
            .await?;
        }
        "worker" => worker::run(state).await?,
        "worker-once" => worker::tick(&state).await?,
        "migrate" => tracing::info!("migrations applied"),
        "promote-admin" => {
            let email = args
                .get(2)
                .ok_or_else(|| {
                    anyhow::anyhow!("Usage: rescueproof promote-admin <existing-email>")
                })?
                .trim()
                .to_lowercase();
            let n = sqlx::query("UPDATE users SET is_admin=true WHERE email=$1 AND disabled=false")
                .bind(&email)
                .execute(&state.db)
                .await?
                .rows_affected();
            anyhow::ensure!(n == 1, "Existing active user not found");
            tracing::info!("administrator granted by local operator");
        }
        _ => anyhow::bail!("Commands: serve, worker, worker-once, migrate, promote-admin <email>"),
    }
    Ok(())
}
