//! Smoke test: send "pong" to MiMo and print the reply.
//!
//! Run:
//!     cargo run --example ping_mimo

use anyhow::Result;
use dotenvy::dotenv;
use solana_ai_agent::mimo::{ChatMessage, MimoClient};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = dotenv();
    let api_key = std::env::var("MIMO_API_KEY")?;
    let base_url = std::env::var("MIMO_BASE_URL")
        .unwrap_or_else(|_| "https://token-plan-sgp.xiaomimimo.com/v1".into());
    let client = MimoClient::new(api_key, base_url)?;

    let messages = vec![ChatMessage {
        role: "user",
        content: "Reply with exactly one word: pong",
    }];
    let reply = client.chat("mimo-v2.5", messages, 32, 0.0, false).await?;
    println!("content   : {}", reply.content);
    println!("tokens    : {:?}", reply.usage);
    Ok(())
}
