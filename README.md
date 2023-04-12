# The smart contracts for halotrade
[![CircleCI](https://dl.circleci.com/status-badge/img/gh/halotrade-zone/smart-contracts/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/gh/halotrade-zone/smart-contracts/tree/main)
[![codecov](https://codecov.io/gh/halotrade-zone/smart-contracts/branch/main/graph/badge.svg?token=VWCAZGAVH2)](https://codecov.io/gh/halotrade-zone/smart-contracts)

The automated market-maker on [Aura](https://aura.network/) network.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Cosmos SDK](https://docs.cosmos.network/master/run-node/)
- [CosmWasm](https://docs.cosmwasm.com/0.16/getting-started/installation.html)

## Contracts

|                  Name                    |                        Description                           |
| ---------------------------------------- | ------------------------------------------------------------ |
| [`halo_factory`](contracts/halo_factory) | Handle the information related to pairs                      |
| [`halo_pair`](contracts/halo_pair)       | Containing a pair of assets                                  |
| [`halo_router`](contracts/halo_router)   | Containing the logic to facilitate multi-hop swap operations |

* halo_router

   Mainnet: `aura...`

   Testnet: `aura...`

* halo_factory

   Mainnet: `aura...`

   Testnet: `aura...`

* halo_pair

   Mainnet (CodeID):

   Testnet (CodeID):

## Running these contracts

You will need Rust 1.66.0+ with wasm32-unknown-unknown target installed.

### Build the contract
The contracts can be compiled using [beaker](https://github.com/osmosis-labs/beaker)

```
beaker wasm build
```
with the optimizer is
```toml
optimizer_version = '0.12.11'
```

Build .wasm file stored in `target/wasm32-unknown-unknown/release/<CONTRACT_NAME>.wasm`
`--no-wasm-opt` is suitable for development, explained below

```bash
beaker wasm build --no-wasm-opt
```

The optimized contracts are generated in the `artifacts/` directory.

### Testing the contract
To run the tests for the contract, run the following command:

```bash
    RUST_BACKTRACE=1 cargo unit-test
```

This will build the contract and run a series of tests to ensure that it functions correctly. The tests are defined in the ./tests directory.
