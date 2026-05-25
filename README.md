# solana-ai-agent

> A **Rust** framework for building AI-driven on-chain agents on **Solana**, powered by **Xiaomi MiMo**. Stream account state, let MiMo reason about it, sign and submit transactions — all in a single low-latency Rust process.

[![Rust 1.78+](https://img.shields.io/badge/rust-1.78+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Solana](https://img.shields.io/badge/Solana-mainnet--beta-9945FF.svg)](https://solana.com/)

Python and TypeScript bots dominate Solana tooling, but neither can keep up with the latency Solana actually demands. solana-ai-agent is written in **Rust**: tokio for I/O, `solana-client` for RPC + WebSocket, and `reqwest` for the MiMo Open Platform. The result is an agent loop that reads on-chain state and reaches a model decision in under 2 seconds.

## Why MiMo

- **`mimo-v2.5-pro`** runs long chain-of-thought against transaction graphs and price feeds — perfect for arbitrage and liquidation judgement calls.
- **`mimo-v2.5`** handles fast tactical decisions (rebalance, vote, exit).
- OpenAI-compatible HTTP — works cleanly with `reqwest` + `serde_json`.

## Features

- Async agent loop: subscribe to accounts/logs over WebSocket, batch into prompts, route to MiMo.
- Pluggable strategies (`trait Strategy`) — three batteries-included: arb-scout, liquidation-watch, dao-vote.
- Built-in signer with `solana-sdk` keypair management.
- Dry-run mode (simulate transaction, no submit).
- Structured prompt templates with on-chain data interpolation.
- TOML config + `.env` secrets.

## Quick start

```bash
git clone https://github.com/FlipZ3ro/solana-ai-agent
cd solana-ai-agent
cargo build --release

cp .env.example .env  # add MIMO_API_KEY + KEYPAIR_PATH
./target/release/solana-ai-agent --config configs/arb-scout.toml --dry-run
```

Run against mainnet:

```bash
./target/release/solana-ai-agent --config configs/arb-scout.toml
```

## Architecture

```
   Solana RPC + WS
          │
          ▼
   ┌─────────────────┐
   │  account_stream │  tokio + solana-client
   └────────┬────────┘
            │  AccountUpdate
            ▼
   ┌─────────────────┐
   │  strategy       │  trait — decides if MiMo should be consulted
   └────────┬────────┘
            │  Prompt
            ▼
   ┌─────────────────┐
   │  mimo client    │  reqwest → mimo-v2.5(-pro)
   └────────┬────────┘
            │  Decision { action, reason, confidence }
            ▼
   ┌─────────────────┐
   │  signer + RPC   │  build, sign, send tx (or dry-run)
   └─────────────────┘
```

## Bundled strategies

| Strategy           | Trigger                                  | Model           |
|--------------------|------------------------------------------|-----------------|
| `arb-scout`        | Price-feed divergence > 0.4 % across DEXs| mimo-v2.5-pro   |
| `liquidation-watch`| Lending position health-factor < 1.05    | mimo-v2.5-pro   |
| `dao-vote`         | New governance proposal posted           | mimo-v2.5-pro   |

## Roadmap

- [x] Async account/log subscription with reconnect
- [x] Strategy trait + 3 reference impls
- [x] MiMo `reqwest` client with retry
- [x] Dry-run + submit modes
- [ ] Jito bundle support (priority lanes)
- [ ] gRPC Yellowstone Geyser plugin client
- [ ] Reinforcement-learning fine-tuning loop using MiMo trajectories

## Token economics

| Trigger rate         | Daily MiMo tokens |
|----------------------|-------------------|
| 1 decision / minute  | ~10M              |
| 1 decision / 10 sec  | ~60M              |
| Continuous (logs)    | ~150M             |

A serious 24/7 production bot consumes ~3 B tokens / month.

## License

MIT — see [LICENSE](LICENSE).
