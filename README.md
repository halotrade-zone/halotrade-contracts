# The smart contracts for halotrade
[![CircleCI](https://dl.circleci.com/status-badge/img/gh/halotrade-zone/smart-contracts/tree/main.svg?style=svg)](https://dl.circleci.com/status-badge/redirect/gh/halotrade-zone/smart-contracts/tree/main)
[![codecov](https://codecov.io/gh/halotrade-zone/smart-contracts/branch/main/graph/badge.svg?token=VWCAZGAVH2)](https://codecov.io/gh/halotrade-zone/smart-contracts)

The automated market-maker on [Aura](https://aura.network/) network.
## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Cosmos SDK](https://docs.cosmos.network/master/run-node/)
- [CosmWasm](https://docs.cosmwasm.com/0.16/getting-started/installation.html)

### Installing:

- Clone the repository from: [Halo-swap repo](https://github.com/aura-nw/halo-swap)

```bash
git clone https://github.com/halotrade-zone/halotrade-contracts.git
```

- Beaker tools:

```bash
cargo install -f beaker # `-f` flag for up-to-date version
```

## Contracts

|                  Name                    |                        Description                         |
| ---------------------------------------- | ---------------------------------------------------------- |
| [`halo_factory`](contracts/halo_factory) |                                                            |
| [`halo_pair`](contracts/halo_pair)       |                                                            |
| [`halo_router`](contracts/halo_router)   |                                                            |
| [`halo_token`](contracts/halo_token)     | CW20 (ERC20 equivalent) token implementation for LP tokens |

* halo_router

   Mainnet: `aura...`

   Testnet: `aura...`

* halo_factory

   Mainnet: `aura...`

   Testnet: `aura...`

* halo_pair

   Mainnet (CodeID):

   Testnet (CodeID):

* halo_token

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
optimizer_version = '0.12.9'
```

Build .wasm file stored in `target/wasm32-unknown-unknown/release/<CONTRACT_NAME>.wasm`
`--no-wasm-opt` is suitable for development, explained below

```bash
beaker wasm build --no-wasm-opt
```

The optimized contracts are generated in the `artifacts/` directory.
### Deployment

1. Update Beaker.toml file

```bash
name = "halo-swap"
gas_price = '0.025uaura'
gas_adjustment = 1.3
account_prefix = 'aura'
derivation_path = '''m/44'/118'/0'/0/0'''

[networks.serenity]
chain_id = 'serenity-testnet-001'
network_variant = 'Shared'
grpc_endpoint = 'https://grpc.serenity.aura.network:9092'
rpc_endpoint = 'https://rpc.serenity.aura.network'

[accounts.signer]
mnemonic = 'around cushion believe vicious member trophy grit disease diagram nice only post nut beef mosquito thumb huge pelican disorder orchard response left phrase degree'

[wasm]
contract_dir = 'contracts'
optimizer_version = '0.12.9'
```

2. Store code on chain

Read .wasm in `target/wasm32-unknown-unknown/release/<CONTRACT_NAME>.wasm` due to `--no-wasm-opt` flag
use `--signer-account test1` which is predefined.
The list of all predefined accounts are here: https://github.com/osmosis-labs/LocalOsmosis#accounts
code-id` is stored in the beaker state, local by default

```bash
beaker wasm store-code halo-token --signer-account signer --no-wasm-opt --network serenity
```

The result should be like this:

    ```bash
      Code stored successfully!! 🎉
    +
    ├── code_id: 1050
    └── instantiate_permission: –
    ```

3. Instantiate, Execute and Query the contract
For each contract, you need to specify the instantiate, execute and query message. Please refer to the contract's README.md for more details.

### Testing the contract
To run the tests for the contract, run the following command:

```bash
    RUST_BACKTRACE=1 cargo unit-test
```

This will build the contract and run a series of tests to ensure that it functions correctly. The tests are defined in the ./tests directory.

To run the code coverage for the contract, run the following command:

```bash
    cargo tarpaulin --out Html
```

You can receive tarpaulin-report.html that gives you the percentage of code coverage.
