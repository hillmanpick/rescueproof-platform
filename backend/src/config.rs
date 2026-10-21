use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub bind: String,
    pub origin: String,
    pub secure_cookie: bool,
    pub relayer_url: Option<String>,
    pub relayer_token: Option<String>,
    pub chain_id: Option<i64>,
    pub contract_address: Option<String>,
    pub rpc_url: Option<String>,
    pub trust_proxy: bool,
    pub redis_namespace: String,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let origin = env::var("PUBLIC_ORIGIN").unwrap_or("http://localhost:5173".into());
        let parsed = url::Url::parse(&origin)?;
        anyhow::ensure!(
            matches!(parsed.scheme(), "http" | "https")
                && parsed.host_str().is_some()
                && parsed.path() == "/"
                && parsed.query().is_none(),
            "PUBLIC_ORIGIN must be an HTTP(S) origin"
        );
        let secure_cookie = env::var("COOKIE_SECURE").unwrap_or("true".into()) == "true";
        anyhow::ensure!(
            parsed.scheme() != "https" || secure_cookie,
            "HTTPS requires COOKIE_SECURE=true"
        );
        let relayer_url = optional("ANCHOR_RELAYER_URL");
        let relayer_token = optional("ANCHOR_RELAYER_TOKEN");
        let chain_id = optional("ANCHOR_CHAIN_ID").map(|s| s.parse()).transpose()?;
        let contract_address = optional("ANCHOR_CONTRACT_ADDRESS").map(|s| s.to_lowercase());
        let rpc_url = optional("ANCHOR_RPC_URL");
        let configured = [
            relayer_url.is_some(),
            relayer_token.is_some(),
            chain_id.is_some(),
            contract_address.is_some(),
            rpc_url.is_some(),
        ];
        anyhow::ensure!(
            configured.iter().all(|v| *v) || configured.iter().all(|v| !*v),
            "All five ANCHOR_* settings must be configured together"
        );
        if let Some(id) = chain_id {
            anyhow::ensure!(id > 0, "ANCHOR_CHAIN_ID must be positive");
        }
        if let Some(address) = &contract_address {
            anyhow::ensure!(
                address.len() == 42
                    && address.starts_with("0x")
                    && address[2..].bytes().all(|b| b.is_ascii_hexdigit()),
                "ANCHOR_CONTRACT_ADDRESS must be a 20-byte hex address"
            );
        }
        for (name, value) in [
            ("ANCHOR_RELAYER_URL", relayer_url.as_ref()),
            ("ANCHOR_RPC_URL", rpc_url.as_ref()),
        ] {
            if let Some(value) = value {
                let u = url::Url::parse(value)?;
                let loopback = matches!(u.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
                let private_relayer =
                    name == "ANCHOR_RELAYER_URL" && u.host_str() == Some("relayer");
                anyhow::ensure!(
                    (u.scheme() == "https"
                        || u.scheme() == "http" && (loopback || private_relayer))
                        && u.username().is_empty()
                        && u.password().is_none(),
                    "{name} must use HTTPS and must not embed credentials"
                );
            }
        }
        Ok(Self {
            database_url: env::var("DATABASE_URL")?,
            redis_url: env::var("REDIS_URL")?,
            bind: env::var("BIND_ADDRESS").unwrap_or("127.0.0.1:3000".into()),
            origin: origin.trim_end_matches('/').to_owned(),
            secure_cookie,
            relayer_url,
            relayer_token,
            chain_id,
            contract_address,
            rpc_url,
            trust_proxy: env::var("TRUST_PROXY").unwrap_or("false".into()) == "true",
            redis_namespace: env::var("REDIS_NAMESPACE").unwrap_or("rescueproof".into()),
        })
    }
}
fn optional(name: &str) -> Option<String> {
    env::var(name).ok().filter(|v| !v.trim().is_empty())
}
