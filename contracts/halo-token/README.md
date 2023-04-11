# halo-token
## Introduction

This repository contains the source code for a CW-20 token contract written in CosmWasm. The contract implements the CW-20 token standard on the AURA blockchain and provides basic functionality for transferring and managing tokens. The CW20 token contract provides a simple and easy way to create custom tokens without needing to write any code.

### Functionality

This CW-20 token contract implements all the required functionality for a [CW-20 token specification](https://github.com/CosmWasm/cw-plus/blob/main/packages/cw20/README.md).

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

## ExecuteMsg

### Transfer
```javascript
{
    "transfer": {
        "recipient": "aura...",
        "amount": "100000000000"
    }
}
```

### Burn
```javascript
{
    "burn": {
        "amount": "100000000000"
    }
}
```
### Send
```javascript
{
    "send": {
        "contract": "aura...", // Cw20 Token contract address
        "amount": "100000000000",
        "msg": {
            "transfer": {
                "recipient": "aura...",
                "amount": "100000000000"
            }
        }
    }
}
```

### Mint
```javascript
{
    "mint": {
        "recipient": "aura...",
        "amount": "100000000000"
    }
}
```

### IncreaseAllowance
```javascript
{
    "increase_allowance": {
        "spender": "aura...",
        "amount": "100000000000",
        "expires": None
    }
}
```
### DecreaseAllowance
```javascript
{
    "decrease_allowance": {
        "spender": "aura...",
        "amount": "100000000000",
        "expires": None
    }
}
```

### TransferFrom
```javascript
{
    "transfer_from": {
        "owner": "aura...",
        "recipient": "aura...",
        "amount": "100000000000"
    }
}
```

### BurnFrom
```javascript
{
    "burn_from": {
        "owner": "aura...",
        "amount": "100000000000"
    }
}
```

### SendFrom
```javascript
{
    "send_from": {
        "owner": "aura...",
        "contract": "aura...", // Cw20 Token contract address
        "amount": "100000000000",
        "msg": {
            "transfer": {
                "recipient": "aura...",
                "amount": "100000000000"
            }
        }
    }
}
```

### UpdateMarketing
```javascript
{
    "update_marketing": {
        "project": "Halo Trade",
        "description": "DEX for AURA",
        "marketing": "Marketing Information",
    }
}
```

### UploadLogo
```javascript
{
    "upload_logo": {
        "logo": "data:image/png"
    }
}
```

### UpdateMinter
```javascript
{
    "update_minter": {
        "new_minter": "aura..."
    }
}
```

## QueryMsg

## Balance
```javascript
{
    "balance": {
        "address": "aura...", // Address to query balance of
    }
}
```
#[returns(cw20::BalanceResponse)]

### TokenInfo
```javascript
{
    "token_info": {}
}
```
#[returns(cw20::TokenInfoResponse)]

### Minter
```javascript
{
    "minter": {}
}
```
#[returns(cw20::MinterResponse)]

## Allowance
```javascript
{
    "allowance": {
        "owner": "aura...", // Owner of the allowance
        "spender": "aura...", // Spender of the allowance
    }
}
```
#[returns(cw20::AllowanceResponse)]

### AllAllowances
```javascript
{
    "all_allowances": {
        "owner": "aura...", // Owner of the allowances
        "start_after": None, // Optional start of the pagination (exclusive)
        "limit": None, // Optional page limit
    }
}
```
#[returns(cw20::AllAllowancesResponse)]

### AllSpenderAllowances
```javascript
{
    "all_spender_allowances": {
        "spender": "aura...", // Spender of the allowances
        "start_after": None, // Optional start of the pagination (exclusive)
        "limit": None, // Optional page limit
    }
}
```
#[returns(cw20::AllSpenderAllowancesResponse)]

### AllAccounts
```javascript
{
    "all_accounts": {
        "start_after": None, // Optional start of the pagination (exclusive)
        "limit": None, // Optional page limit
    }
}
```
#[returns(cw20::AllAccountsResponse)]

### MarketingInfo
```javascript
{
    "marketing_info": {}
}
```
#[returns(cw20::MarketingInfoResponse)]

### DownloadLogo
```javascript
{
    "download_logo": {}
}
```
#[returns(cw20::DownloadLogoResponse)]
