# halo-router
## Introduction
The Router Contract contains the logic to facilitate multi-hop swap operations. It is the entry point for all swap operations include.
## InstantiateMsg
```javascript
{
    "halo_factory": "aura...", // The address of the factory contract
}
```

## ExecuteMsg

### Receive
```javascript
{
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
}
```

### ExecuteSwapOperations
```javascript
{
    "execute_swap_operations" {
        "operations": [
            "offer_asset_info": {
                "token": {
                    "contract_addr": "aura...",
                }
            },
            "ask_asset_info": {
                "native_token": {
                    "denom": "uaura"
                }
            },
        ],
        "minimum_receive": None,
        "to": "aura...",
    },
}
```
### ExecuteSwapOperations
```javascript
{
    "execute_swap_operations" {
        "operations": [
            "offer_asset_info": {
                "token": {
                    "contract_addr": "aura...",
                }
            },
            "ask_asset_info": {
                "native_token": {
                    "denom": "uaura"
                }
            },
        ],
        "to": "aura...",
    },
}
```
### AssertMinimumReceive
```javascript
{
    "assert_minimum_receive" {
        "asset_info": {
            "token": {
                "contract_addr": "aura...",
            }
        },
        "prev_balance": 10000,
        "minimum_receive": 9980,
        "receiver": "aura...",
    },
}
```

## QueryMsg
### Config
```javascript
{
    "config" {}
}
```
#[returns(ConfigResponse)]

### SimulateSwapOperations
```javascript
{
    "simulate_swap_operations" {
        "offer_amount": 10000,
        "operations": [
            "offer_asset_info": {
                "token": {
                    "contract_addr": "aura...",
                }
            },
            "ask_asset_info": {
                "native_token": {
                    "denom": "uaura"
                }
            },
        ],
    },
}
```
#[returns(SimulateSwapOperationsResponse)]

### ReverseSimulateSwapOperations
```javascript
{
    "reverse_simulate_swap_operations" {
        "ask_amount": 10000,
        "operations": [
            "offer_asset_info": {
                "token": {
                    "contract_addr": "aura...",
                }
            },
            "ask_asset_info": {
                "native_token": {
                    "denom": "uaura"
                }
            },
        ],
    },
}
```
#[returns(SimulateSwapOperationsResponse)]
