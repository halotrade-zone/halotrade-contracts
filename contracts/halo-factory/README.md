# The factory contract for Haloswap

## Introduction
The factory contract will handle the information related to pairs. It will also create new pairs when users provide assets to the contract.

## InstantiateMsg
We must provide the source code id of `halo_pair` contract and (`cw-20 base` contract)[https://github.com/halotrade-zone/cw-plus/tree/main/contracts/cw20-base] for `halo-factory` contract.
```javascript
{
    "pair_code_id": 123,
    "token_code_id": 123
}
```

## ExecuteMsg

### UpdateConfig
```javascript
{
    "update_config": {
        "owner": "aura...",
        "token_code_id": 321,
        "pair_code_id": 321
    }
}
```
Where:
- `owner` is the address of the owner of the factory contract.
- `token_code_id` is the new source code id of (`cw-20 base` contract)[https://github.com/halotrade-zone/cw-plus/tree/main/contracts/cw20-base].
- `pair_code_id` is the new source code id of `halo-pair` contract.

### CreatePair
The parameters in `requirements` include the whitelisted users who can provide liquidity for the first time when pair is empty and the minimum amount of assets that users must provide in the first time.
```javascript
{
    "create_pair": {
        "asset_infos": [
            {
                "token": {
                    "contract_addr": "aura..."
                }
            },
            {
                "native_token": {
                    "denom": "uaura"
                }
            }
        ],
        "requirements": {
            "whitelist": [
                "aura...",
                "aura..."
            ],
            "first_asset_minimum": 10000,
            "second_asset_minimum": 20000
        }
        "commission_rate": "0.003",
        "lp_token_info": {
            "lp_token_name": "AURA_HALO_LP",
            "lp_token_symbol": "AURA_HALO_LP",
        }
    },
}
```
Where:
- `asset_infos` is the information of assets in the pair.
- `requirements` is the whitelist wallet address list and requirements for providing liquidity for the first time.
- `commission_rate` is the commission rate of the pair.
- `lp_token_info` is the information of the LP token.


### AddNativeTokenDecimals
Before can be added to any pair, a native token must be specified its decimals.
```javascript
{
    "add_native_token_decimals": {
        "denom": "uaura",
        "decimals": 6,
    }
}
```
Where:
- `denom` is the denom of the native token.
- `decimals` is the decimals of the native token will be added.

## QueryMsg
### Config
```javascript
{
    "config": {}
}
```
#[returns(ConfigResponse)]

### Pair
```javascript
{
    "pair": {
        "asset_infos": [
            {
                "token": {
                    "contract_addr": "aura..."
                }
            },
            {
                "native_token": {
                    "denom": "uaura"
                }
            }
        ]
    }
}
```
#[returns(PairInfo)]

### Pairs
```javascript
{
    "pairs": { }
}
```
#[returns(PairsResponse)]

### NativeTokenDecimals
```javascript
{
    "native_token_decimals" {
        "denom": "uaura",
    },
}
```
#[returns(NativeTokenDecimalsResponse)]
