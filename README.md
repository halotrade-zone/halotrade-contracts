# The smart contracts for halotrade
[![CircleCI](https://dl.circleci.com/status-badge/img/gh/halotrade-zone/halotrade-contracts/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/gh/halotrade-zone/halotrade-contracts/tree/main)
[![codecov](https://codecov.io/gh/halotrade-zone/halotrade-contracts/branch/main/graph/badge.svg?token=VWCAZGAVH2)](https://codecov.io/gh/halotrade-zone/halotrade-contracts)

The automated market-maker on [Aura network](https://aura.network/).

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Cosmos SDK](https://docs.cosmos.network/main)
- [CosmWasm](https://cosmwasm.com/)

## Contracts

|                  Name                    |                        Description                           |
| ---------------------------------------- | ------------------------------------------------------------ |
| [`halo_factory`](https://github.com/halotrade-zone/halotrade-contracts/tree/main/contracts/halo-factory) | Handle the information related to pairs                      |
| [`halo_pair`](https://github.com/halotrade-zone/halotrade-contracts/tree/main/contracts/halo-pair)       | Containing a pair of assets                                  |
| [`halo_router`](https://github.com/halotrade-zone/halotrade-contracts/tree/main/contracts/halo-router)   | Containing the logic to facilitate multi-hop swap operations |

## Running these contracts

You will need Rust 1.66.0+ with wasm32-unknown-unknown target installed.

### Build the contract
The contracts can be compiled using [cargo](https://doc.rust-lang.org/cargo/commands/cargo-build.html)

```
cargo build
```
with the optimizer is
```toml
optimizer_version = '0.12.11'
```

Build .wasm file stored in `target/wasm32-unknown-unknown/release/<CONTRACT_NAME>.wasm`
`--no-wasm-opt` is suitable for development, explained below

### Testing the contract
To run the tests for the contract, run the following command:

```bash
RUST_BACKTRACE=1 cargo unit-test
```

This will build the contract and run a series of tests to ensure that it functions correctly. The tests are defined in the ./tests directory.
