//! solana-ai-agent: AI-driven on-chain agent framework for Solana.
//!
//! - [`mimo`]      — HTTP client for the MiMo Open Platform.
//! - [`strategy`]  — `Strategy` trait + reference implementations.
//! - [`signer`]    — keypair loader and transaction signer.
//! - [`agent`]     — main event loop wiring it all together.

pub mod agent;
pub mod config;
pub mod mimo;
pub mod signer;
pub mod strategy;
