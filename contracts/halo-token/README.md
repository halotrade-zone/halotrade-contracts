# halo-token
## Introduction

This repository contains the source code for a CW-20 token contract written in CosmWasm. The contract implements the CW-20 token standard on the AURA blockchain and provides basic functionality for transferring and managing tokens. The CW20 token contract provides a simple and easy way to create custom tokens without needing to write any code.

## InstantiateMsg
```javascript
{
    "name": "Halo Token",
    "symbol": "HALO",
    "decimals": 6,
    "initial_balances": [
        {
            "address": "aura...",
            "amount": "100000000000"
        },
        {
            "address": "aura...",
            "amount": "100000000000"
        }
    ],
    "mint": {
        "minter": "aura...",
        "cap": "100000000000000000000"
    }
}
```
Where:
- `name` is the name of the token.
- `symbol` is the symbol of the token.
- `decimals` is the number of decimal places for the token. The default value is 6.
- `initial_balances` is the list of initial balances for the token.
- `mint` is the minter information for the token. It contains the minter address and the minting cap.


## ExecuteMsg and QueryMsg
This CW-20 token contract implements all the required functionality for a [CW-20 token specification](https://github.com/CosmWasm/cw-plus/blob/main/packages/cw20/README.md).