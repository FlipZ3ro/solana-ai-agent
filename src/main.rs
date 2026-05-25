//! CLI entrypoint: `solana-ai-agent --config configs/arb-scout.toml --dry-run`.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

use solana_ai_agent::{
    agent::Agent,
    config::Settings,
    mimo::MimoClient,
    signer::AgentSigner,
    strategy::{ArbScout, DaoVote, LiquidationWatch, ObservedState, Strategy},
};

#[derive(Parser, Debug)]
#[command(name = "solana-ai-agent", about = "AI-driven Solana on-chain agent")]
struct Cli {
    /// TOML config file path
    #[arg(long)]
    config: Option<PathBuf>,
    /// Force dry-run (override config)
    #[arg(long)]
    dry_run: bool,
    /// Strategy override: arb-scout | liquidation-watch | dao-vote
    #[arg(long)]
    strategy: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("AGENT_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let args = Cli::parse();
    let mut settings = Settings::load(args.config.as_deref()).context("load settings")?;
    if args.dry_run {
        settings.agent.dry_run = true;
    }
    if let Some(s) = args.strategy {
        settings.agent.strategy = s;
    }

    let mimo = MimoClient::new(&settings.mimo.api_key, &settings.mimo.base_url)?;
    let signer = AgentSigner::new(&settings.solana.keypair_path, &settings.solana.rpc_url)?;
    tracing::info!("loaded keypair: {}", signer.pubkey());

    // Demo source: in a real deploy this would be a WS / Yellowstone subscription
    // pushing real `ObservedState` updates.
    let (tx, rx) = mpsc::channel::<ObservedState>(64);
    tokio::spawn(async move {
        let _ = tx
            .send(ObservedState {
                kind: "price_divergence".into(),
                summary: "demo divergence".into(),
                raw_json: r#"{"dexes":[{"name":"Orca","price":24.10},{"name":"Raydium","price":24.32}],"mint":"So11..."}"#.into(),
                slot: 0,
                timestamp_ms: chrono::Utc::now().timestamp_millis(),
            })
            .await;
    });

    match settings.agent.strategy.as_str() {
        "arb-scout" => {
            let agent = Agent {
                strategy: ArbScout,
                mimo,
                signer,
                fast_model: settings.mimo.fast_model,
                deep_model: settings.mimo.deep_model,
                dry_run: settings.agent.dry_run,
            };
            agent.run(rx).await?;
        }
        "liquidation-watch" => {
            let agent = Agent {
                strategy: LiquidationWatch,
                mimo,
                signer,
                fast_model: settings.mimo.fast_model,
                deep_model: settings.mimo.deep_model,
                dry_run: settings.agent.dry_run,
            };
            agent.run(rx).await?;
        }
        "dao-vote" => {
            let agent = Agent {
                strategy: DaoVote,
                mimo,
                signer,
                fast_model: settings.mimo.fast_model,
                deep_model: settings.mimo.deep_model,
                dry_run: settings.agent.dry_run,
            };
            agent.run(rx).await?;
        }
        other => anyhow::bail!("unknown strategy: {other}"),
    }
    Ok(())
}
