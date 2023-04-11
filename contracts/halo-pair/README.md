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

## ExecuteMsg

### Receive
```javascript
    "receive" {
        "sender": "aura...",
        "amount": 10000000000,
        "msg": {
            "provide_liquidity": {
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
        }
    },
```

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
