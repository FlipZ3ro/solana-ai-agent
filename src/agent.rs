//! Agent loop: stream observed state -> filter via Strategy -> consult MiMo -> sign + submit.

use anyhow::Result;
use chrono::Utc;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::{
    mimo::MimoClient,
    signer::AgentSigner,
    strategy::{consult, Decision, ObservedState, Strategy},
};

pub struct Agent<S: Strategy> {
    pub strategy: S,
    pub mimo: MimoClient,
    pub signer: AgentSigner,
    pub fast_model: String,
    pub deep_model: String,
    pub dry_run: bool,
}

impl<S: Strategy> Agent<S> {
    pub async fn run(self, mut state_rx: mpsc::Receiver<ObservedState>) -> Result<()> {
        info!("starting agent — strategy={}, dry_run={}", self.strategy.name(), self.dry_run);
        while let Some(state) = state_rx.recv().await {
            let Some(prompt) = self.strategy.relevant(&state) else {
                continue;
            };
            let consult_result =
                consult(&self.strategy, &self.mimo, &prompt, &self.fast_model, &self.deep_model)
                    .await;
            match consult_result {
                Ok(decision) => self.act_on(state, decision).await?,
                Err(e) => warn!("consult error: {e}"),
            }
        }
        Ok(())
    }

    async fn act_on(&self, state: ObservedState, decision: Decision) -> Result<()> {
        info!(
            "[{}] kind={} confidence={:.2} action={} reason={}",
            Utc::now().to_rfc3339(),
            state.kind,
            decision.confidence,
            decision.action,
            decision.reason
        );
        if !self.strategy.should_execute(&decision, self.dry_run) {
            info!("(dry-run / skip) no tx submitted");
            return Ok(());
        }
        // In a real strategy this is where you build `Instruction`s and call
        // `self.signer.submit(...)`. We leave that pluggable for the user.
        info!("would submit tx via signer for action={}", decision.action);
        Ok(())
    }
}
