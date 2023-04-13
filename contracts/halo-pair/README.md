# The pair contract for Haloswap
## Introduction
Each contract contains a pair of assets. When users provide these assets to the contract, they will receive the Liquidity Provider (LP) Token.

## InstantiateMsg
```javascript
{
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
    "token_code_id": 123,
    "asset_decimals": [ 6, 6 ],
    "requirements": {
        "whitelist": [
            "aura...",
            "aura..."
        ],
        "first_asset_minimum": 10000,
        "second_asset_minimum": 20000
    },
    "commission_rate": "0.003",
    "lp_token_info": {
        "lp_token_name": "AURA_HALO_LP",
        "lp_token_symbol": "AURA_HALO_LP",
    },
}
```
Where:
- `asset_infos` is the list of assets in the pair (The pair can be token<->native, token-token or native-native).
- `token_code_id` is the source code id of `halo-token` contract.
- `asset_decimals` is the list of decimals of assets in the pair.
- `requirements` is the whitelist wallet address list and requirements for providing liquidity for the first time.
- `commission_rate` is the commission rate of the pair.
- `lp_token_info` is the information of the LP token.

## ExecuteMsg
### ProvideLiquidity
```javascript
    "provide_liquidity" {
        "assets": [
            {
                "info": {
                    "token": {
                        "contract_addr": "aura...",
                    }
                },
                "amount": 10000000000,
            },
            {
                "info": {
                    "native_token": {
                        "denom": "uaura"
                    }
                },
                "amount": 500000000,
            }
        ],
        "slippage_tolerance": 5,
        "receiver": "aura...",
    },
```
Where:
- `assets` is the list of assets that the sender wants to provide to the contract.
- `slippage_tolerance` is the slippage tolerance of the swap. The value is in percentage.
- `receiver` is the address of the receiver who will receive the LP Token.

### Swap
```javascript
    "swap" {
        "offer_asset": {
            "info": {
                "token": {
                    "contract_addr": "aura...",
                }
            }
            "amount": 10000000000,
        },
        "belief_price": None,
        "max_spread": None,
        "to": "aura...",
    },
```
Where:
- `offer_asset` is the asset that the sender wants to swap.
- `belief_price` is the belief price of the swap.
- `max_spread` is the maximum spread of the swap.

## QueryMsg
### Pair
```javascript
{
    "pair": {}
}
```
#[returns(PairInfo)]

### Pool
```javascript
{
    "pool": {}
}
```
#[returns(PoolResponse)]

### Simulation
```javascript
{
    "simulation": {
        "offer_asset": {
            "info": {
                "token": {
                    "contract_addr": "aura...",
                }
            }
            "amount": 10000000000,
        }
    }
}
```
#[returns(SimulationResponse)]

### ReverseSimulation
```javascript
{
    "reverseSimulation": {
        "ask_asset": {
            "info": {
                "native_token": {
                    "denom": "uaura"
                }
            },
            "amount": 500000000,
        }
    }
}
```
#[returns(ReverseSimulationResponse)]
