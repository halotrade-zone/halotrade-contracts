# halo-router
## Introduction
The Router Contract contains the logic to facilitate multi-hop swap operations. It is the entry point for all swap operations.
## InstantiateMsg
```javascript
{
    "halo_factory": "aura...", // The address of the factory contract
}
```

## ExecuteMsg

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
Where:
- `operations` is the list of swap operations. Each operation contains the offer asset and the ask asset. The offer asset is the asset that the user wants to swap. The ask asset is the asset that the user wants to receive.
- `minimum_receive` is the minimum amount of the ask asset that the user wants to receive. If the amount of the ask asset is less than the minimum amount, the swap operation will fail.
- `to` is the address that the user wants to receive the ask asset.

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
Where:
- `asset_info` is the asset that the user wants to receive. It can be a token or a native token.
- `prev_balance` is the balance of the asset before the swap operation.
- `minimum_receive` is the minimum amount of the asset that the user wants to receive.
- `receiver` is the address to receive the asset.

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
