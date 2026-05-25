//! Strategy trait + three reference implementations.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::mimo::{ChatMessage, MimoClient};

#[derive(Debug, Clone)]
pub struct ObservedState {
    pub kind: String,           // e.g. "price_divergence", "loan_health", "proposal"
    pub summary: String,        // short human-readable snapshot
    pub raw_json: String,       // full structured data the model can re-read
    pub slot: u64,
    pub timestamp_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub action: String,         // strategy-specific action name
    pub reason: String,
    #[serde(default)]
    pub confidence: f32,        // 0-1
    #[serde(default)]
    pub params: serde_json::Value,
}

#[async_trait]
pub trait Strategy: Send + Sync {
    fn name(&self) -> &str;

    /// Inspect new on-chain state. Return `Some(prompt)` if MiMo should be consulted.
    fn relevant(&self, state: &ObservedState) -> Option<String>;

    /// Parse the model's reply into a `Decision`.
    fn parse_decision(&self, reply: &str) -> Result<Decision> {
        let d: Decision = serde_json::from_str(reply.trim())?;
        Ok(d)
    }

    /// System prompt used when MiMo is consulted.
    fn system_prompt(&self) -> &'static str;

    /// Which MiMo model to use for this strategy.
    fn model<'a>(&self, fast: &'a str, deep: &'a str) -> &'a str {
        deep
    }

    /// After a Decision, this returns whether the agent should actually submit a tx.
    fn should_execute(&self, _decision: &Decision, dry_run: bool) -> bool {
        !dry_run
    }
}

// ---- arb-scout -----------------------------------------------------------

pub struct ArbScout;

#[async_trait]
impl Strategy for ArbScout {
    fn name(&self) -> &str {
        "arb-scout"
    }
    fn relevant(&self, s: &ObservedState) -> Option<String> {
        if s.kind == "price_divergence" {
            Some(format!(
                "Detected price divergence across DEXs.\nSnapshot:\n{}\nDecide whether to arb.",
                s.raw_json
            ))
        } else {
            None
        }
    }
    fn system_prompt(&self) -> &'static str {
        r#"You are a Solana arbitrage analyst. Given a divergence snapshot across
DEXs, output strict JSON: {action: "arb"|"skip", reason: str, confidence: 0..1,
params: {legs: [{dex, side, mint, size_lamports}, ...]}}. Skip if the gross
edge after estimated 5 bps swap fees and 0.001 SOL gas is below 0.2 %."#
    }
}

// ---- liquidation-watch ----------------------------------------------------

pub struct LiquidationWatch;

#[async_trait]
impl Strategy for LiquidationWatch {
    fn name(&self) -> &str {
        "liquidation-watch"
    }
    fn relevant(&self, s: &ObservedState) -> Option<String> {
        if s.kind == "loan_health" {
            Some(format!(
                "Lending position update.\n{}\nDecide whether to liquidate.",
                s.raw_json
            ))
        } else {
            None
        }
    }
    fn system_prompt(&self) -> &'static str {
        r#"You are a liquidation bot operator on Solana. Output strict JSON:
{action: "liquidate"|"watch"|"skip", reason: str, confidence: 0..1, params:
{user, market, repay_lamports, expected_profit_lamports}}. Liquidate only when
expected net profit > 0.01 SOL and health factor < 1.0."#
    }
}

// ---- dao-vote -------------------------------------------------------------

pub struct DaoVote;

#[async_trait]
impl Strategy for DaoVote {
    fn name(&self) -> &str {
        "dao-vote"
    }
    fn relevant(&self, s: &ObservedState) -> Option<String> {
        if s.kind == "proposal" {
            Some(format!(
                "New DAO proposal.\n{}\nRecommend a vote.",
                s.raw_json
            ))
        } else {
            None
        }
    }
    fn system_prompt(&self) -> &'static str {
        r#"You are a DAO governance analyst. Output strict JSON: {action:
"yes"|"no"|"abstain", reason: str, confidence: 0..1, params: {proposal_id,
weight}}. Reason should cite specific risks/benefits in <= 2 sentences."#
    }
}

// ---- helper to dispatch --------------------------------------------------

pub async fn consult<S: Strategy + ?Sized>(
    strategy: &S,
    mimo: &MimoClient,
    user_prompt: &str,
    fast_model: &str,
    deep_model: &str,
) -> Result<Decision> {
    let model = strategy.model(fast_model, deep_model);
    let messages = vec![
        ChatMessage {
            role: "system",
            content: strategy.system_prompt(),
        },
        ChatMessage {
            role: "user",
            content: user_prompt,
        },
    ];
    let reply = mimo.chat(model, messages, 1024, 0.2, true).await?;
    strategy.parse_decision(&reply.content)
}
