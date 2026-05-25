//! TOML config + .env merge.

use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub mimo: MimoSettings,
    pub solana: SolanaSettings,
    pub agent: AgentSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MimoSettings {
    pub api_key: String,
    pub base_url: String,
    pub fast_model: String,
    pub deep_model: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolanaSettings {
    pub rpc_url: String,
    pub ws_url: String,
    pub keypair_path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentSettings {
    pub strategy: String,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default = "default_poll_ms")]
    pub poll_interval_ms: u64,
}

fn default_poll_ms() -> u64 {
    2_000
}

impl Settings {
    pub fn load(toml_path: Option<&Path>) -> Result<Self> {
        let _ = dotenvy::dotenv();
        let mut builder = config::Config::builder()
            .set_default("mimo.base_url", "https://token-plan-sgp.xiaomimimo.com/v1")?
            .set_default("mimo.fast_model", "mimo-v2.5")?
            .set_default("mimo.deep_model", "mimo-v2.5-pro")?
            .set_default("solana.rpc_url", "https://api.mainnet-beta.solana.com")?
            .set_default("solana.ws_url", "wss://api.mainnet-beta.solana.com")?
            .set_default("agent.dry_run", true)?
            .set_default("agent.strategy", "arb-scout")?
            // env overrides everything, with double-underscore as nested separator
            .add_source(
                config::Environment::with_prefix("")
                    .separator("_")
                    .try_parsing(true),
            );
        if let Some(p) = toml_path {
            builder = builder.add_source(config::File::from(p));
        }
        let env_map: Vec<(&str, &str)> = vec![
            ("MIMO_API_KEY", "mimo.api_key"),
            ("MIMO_BASE_URL", "mimo.base_url"),
            ("MIMO_FAST_MODEL", "mimo.fast_model"),
            ("MIMO_DEEP_MODEL", "mimo.deep_model"),
            ("SOLANA_RPC_URL", "solana.rpc_url"),
            ("SOLANA_WS_URL", "solana.ws_url"),
            ("KEYPAIR_PATH", "solana.keypair_path"),
            ("AGENT_DRY_RUN", "agent.dry_run"),
        ];
        for (env_var, key) in env_map {
            if let Ok(v) = std::env::var(env_var) {
                builder = builder.set_override(key, v)?;
            }
        }
        let cfg = builder.build().context("config build")?;
        let s: Settings = cfg.try_deserialize().context("config deserialize")?;
        Ok(s)
    }
}
