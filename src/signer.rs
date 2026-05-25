//! Keypair loader and transaction submitter.

use std::path::Path;

use anyhow::{Context, Result};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signature, Signer},
    transaction::Transaction,
};

pub struct AgentSigner {
    keypair: Keypair,
    rpc: RpcClient,
}

impl AgentSigner {
    pub fn new(keypair_path: impl AsRef<Path>, rpc_url: &str) -> Result<Self> {
        let keypair = read_keypair_file(keypair_path.as_ref())
            .map_err(|e| anyhow::anyhow!("read keypair: {e}"))?;
        let rpc = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
        Ok(Self { keypair, rpc })
    }

    pub fn pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    pub async fn submit(&self, instructions: Vec<Instruction>) -> Result<Signature> {
        let blockhash = self
            .rpc
            .get_latest_blockhash()
            .await
            .context("fetch latest blockhash")?;
        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&self.keypair.pubkey()),
            &[&self.keypair],
            blockhash,
        );
        let sig = self
            .rpc
            .send_and_confirm_transaction(&tx)
            .await
            .context("send and confirm")?;
        Ok(sig)
    }

    pub fn rpc(&self) -> &RpcClient {
        &self.rpc
    }
}
