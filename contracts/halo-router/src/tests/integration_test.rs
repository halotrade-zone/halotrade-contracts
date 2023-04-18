#[cfg(test)]
mod tests {
    use crate::tests::env_setup::env::{instantiate_contracts, ADMIN, NATIVE_DENOM_2, USER_1};
    use bignumber::Decimal256;
    use cosmwasm_std::{
        from_binary, to_binary, Addr, BalanceResponse as BankBalanceResponse, BankQuery, Coin,
        QueryRequest, Uint128,
    };
    use cw20::{BalanceResponse, TokenInfoResponse};
    use cw20_base::{msg::ExecuteMsg as Cw20ExecuteMsg, msg::QueryMsg as Cw20QueryMsg};
    use haloswap::asset::{AssetInfo, CreatePairRequirements, PairInfo};
    use haloswap::factory::{
        ExecuteMsg as FactoryExecuteMsg, NativeTokenDecimalsResponse, QueryMsg as FactoryQueryMsg,
    };
    use haloswap::pair::Cw20HookMsg;
    // Mock information for CW20 token contract
    const MOCK_1000_HALO_TOKEN_AMOUNT: u128 = 1_000_000_000;
    // Mock information for native token
    const MOCK_1000_NATIVE_TOKEN_AMOUNT: u128 = 1_000_000_000;
    const MOCK_TRANSACTION_FEE: u128 = 5000;

    // This module to verify Native Token works with cw20-token
    // USER_1 Mint 1000 tokens to HALO Token
    // USER_1 Create Pair: AURA - HALO Token
    // USER_1 Add Liquidity: 1000 AURA - 1000 HALO Token
    // USER_1 Swap: 1000 AURA -> HALO Token
    // USER_1 Withdraw Liquidity: 1000 AURA - 1000 HALO Token
    mod execute_contract_native_with_cw20_token {
        use std::str::FromStr;

        use cosmwasm_std::Querier;
        use cw_multi_test::Executor;
        use haloswap::{
            asset::{Asset, LPTokenInfo, LP_TOKEN_RESERVED_AMOUNT},
            pair::{
                ExecuteMsg, PoolResponse, QueryMsg as RouterQueryMsg, ReverseSimulationResponse,
                SimulationResponse,
            },
            router::{ExecuteMsg as RouterExecuteMsg, SwapOperation},
        };

        use super::*;

        #[test]
        fn proper_operation() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            // Get factory contract
            let factory_contract = contracts[0].contract_addr.clone();
            // Get router contract
            let router_contract = contracts[1].contract_addr.clone();
            // Get cw20 token contract
            let cw20_token_contract = contracts[2].contract_addr.clone();

            // Mint 1000 native tokens NATIVE_DENOM_2 to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: Uint128::from(MOCK_1000_NATIVE_TOKEN_AMOUNT),
                        denom: NATIVE_DENOM_2.to_string(),
                    }],
                },
            ))
            .unwrap();

            // query balance of USER_1 in native token
            let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance {
                address: USER_1.to_string(),
                denom: NATIVE_DENOM_2.to_string(),
            });
            let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
            let balance: BankBalanceResponse = from_binary(&res).unwrap();

            // It should be 1_000_000_000 NATIVE_DENOM_2 as minting happened
            assert_eq!(
                balance.amount.amount,
                Uint128::from(MOCK_1000_NATIVE_TOKEN_AMOUNT)
            );

            // query balance of USER_1 in Halo token
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_token_contract.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: USER_1.to_string(),
                    },
                )
                .unwrap();
            // It should be zero as no minting happened
            assert_eq!(balance.balance, Uint128::zero());

            // Mint 1000 tokens to USER_1
            let mint_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Mint {
                recipient: USER_1.to_string(),
                amount: Uint128::from(MOCK_1000_HALO_TOKEN_AMOUNT),
            };

            // Execute minting
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(cw20_token_contract.clone()),
                &mint_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // query balance of USER_1 in cw20 base token contract
            let balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_token_contract.clone(),
                    &cw20::Cw20QueryMsg::Balance {
                        address: USER_1.to_string(),
                    },
                )
                .unwrap();
            // It should be 1000 token A as minting happened
            assert_eq!(balance.balance, Uint128::from(MOCK_1000_HALO_TOKEN_AMOUNT));

            // Create Pair: AURA - HALO Token

            // Add Native Token Decimals
            let msg = FactoryExecuteMsg::AddNativeTokenDecimals {
                denom: NATIVE_DENOM_2.to_string(),
                decimals: 6u8,
            };

            // Execute add native token decimals
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(factory_contract.clone()),
                &msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Assert decimals of native token
            let response: NativeTokenDecimalsResponse = app
                .wrap()
                .query_wasm_smart(
                    factory_contract.clone(),
                    &FactoryQueryMsg::NativeTokenDecimals {
                        denom: NATIVE_DENOM_2.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(response.decimals, 6u8);

            let asset_infos = [
                AssetInfo::NativeToken {
                    denom: NATIVE_DENOM_2.to_string(),
                },
                AssetInfo::Token {
                    contract_addr: cw20_token_contract.clone(),
                },
            ];

            let create_pair_msg = FactoryExecuteMsg::CreatePair {
                asset_infos,
                requirements: CreatePairRequirements {
                    whitelist: vec![Addr::unchecked(USER_1.to_string())],
                    first_asset_minimum: Uint128::zero(),
                    second_asset_minimum: Uint128::zero(),
                },
                commission_rate: Some(Decimal256::from_str("0.03").unwrap()),
                lp_token_info: LPTokenInfo {
                    lp_token_name: "aura-HALO".to_string(),
                    lp_token_symbol: "aura-HALO".to_string(),
                    lp_token_decimals: None,
                },
            };

            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(factory_contract.clone()),
                &create_pair_msg,
                &[Coin {
                    amount: Uint128::from(500000u128),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Query Pair
            let response: PairInfo = app
                .wrap()
                .query_wasm_smart(
                    factory_contract,
                    &FactoryQueryMsg::Pair {
                        asset_infos: [
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM_2.to_string(),
                            },
                            AssetInfo::Token {
                                contract_addr: cw20_token_contract.clone(),
                            },
                        ],
                    },
                )
                .unwrap();

            // Assert Pair
            assert_eq!(
                response,
                PairInfo {
                    liquidity_token: "contract4".to_string(),
                    asset_infos: [
                        AssetInfo::NativeToken {
                            denom: NATIVE_DENOM_2.to_string(),
                        },
                        AssetInfo::Token {
                            contract_addr: cw20_token_contract.clone(),
                        },
                    ],
                    contract_addr: "contract3".to_string(), // Pair Contract
                    asset_decimals: [6u8, 6u8],
                    requirements: CreatePairRequirements {
                        whitelist: vec![Addr::unchecked(USER_1.to_string())],
                        first_asset_minimum: Uint128::zero(),
                        second_asset_minimum: Uint128::zero(),
                    },
                    // Verify the default commission rate is 0.3%
                    commission_rate: Decimal256::from_str("0.03").unwrap(),
                }
            );

            // Query LP Token Info
            let response: TokenInfoResponse = app
                .wrap()
                .query_wasm_smart("contract4".to_string(), &cw20::Cw20QueryMsg::TokenInfo {})
                .unwrap();

            // Assert LP Token Info
            assert_eq!(
                response,
                TokenInfoResponse {
                    name: "aura-HALO".to_string(),
                    symbol: "aura-HALO".to_string(),
                    decimals: 6u8,
                    total_supply: Uint128::zero(),
                }
            );

            // provide liquidity
            // create provide liquidity message
            // Approve cw20 token to pair contract
            let approve_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: "contract3".to_string(), // Pair Contract
                amount: Uint128::from(2_000_000u128),
                expires: None,
            };

            // Execute approve
            let response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked(cw20_token_contract.clone()),
                &approve_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // USER_1 Provide Liquidity
            let provide_liquidity_msg = ExecuteMsg::ProvideLiquidity {
                assets: [
                    Asset {
                        info: AssetInfo::NativeToken {
                            denom: NATIVE_DENOM_2.to_string(),
                        },
                        amount: Uint128::from(2_000_000u128),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: cw20_token_contract.clone(),
                        },
                        amount: Uint128::from(1_000_000u128),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            let response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked("contract3".to_string()),
                &provide_liquidity_msg,
                &[Coin {
                    amount: Uint128::from(2_000_000u128),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Query Pool
            let response: PoolResponse = app
                .wrap()
                .query_wasm_smart("contract3".to_string(), &RouterQueryMsg::Pool {})
                .unwrap();

            // Assert Pool: total_share amount
            assert_eq!(
                response,
                PoolResponse {
                    assets: [
                        Asset {
                            info: AssetInfo::NativeToken {
                                denom: NATIVE_DENOM_2.to_string(),
                            },
                            amount: Uint128::from(2000000u128),
                        },
                        Asset {
                            info: AssetInfo::Token {
                                contract_addr: cw20_token_contract.clone(),
                            },
                            amount: Uint128::from(1000000u128),
                        },
                    ],
                    // Verify the total share amount is reserved 1 uLP
                    total_share: 1414213u128.into(),
                }
            );

            // Query LP Balance of USER_1
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    "contract4".to_string(),
                    &Cw20QueryMsg::Balance {
                        address: USER_1.to_string(),
                    },
                )
                .unwrap();

            // Assert LP Balance of USER_1
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(1414212u128),
                }
            );

            // Query LP Balance in LP token contract
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    "contract4".to_string(),
                    &Cw20QueryMsg::Balance {
                        address: "contract4".to_string(),
                    },
                )
                .unwrap();

            // Assert LP Balance in LP token contract
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(LP_TOKEN_RESERVED_AMOUNT),
                }
            );

            // Query SimulateSwapOperations for native token token to cw20
            let msg = RouterQueryMsg::Simulation {
                offer_asset: Asset {
                    info: AssetInfo::NativeToken {
                        denom: NATIVE_DENOM_2.to_string(),
                    },
                    amount: Uint128::from(1000u128),
                },
            };

            let response: SimulationResponse = app
                .wrap()
                .query_wasm_smart("contract3".to_string(), &msg)
                .unwrap();

            // Assert SimulateSwapOperations for native token token to cw20
            assert_eq!(
                response,
                SimulationResponse {
                    return_amount: Uint128::from(485u128),
                    spread_amount: Uint128::from(1u128),
                    commission_amount: Uint128::from(14u128),
                }
            );

            // Query ReverseSimulation for cw20 token to native token
            let msg = RouterQueryMsg::ReverseSimulation {
                ask_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: cw20_token_contract.clone(),
                    },
                    amount: Uint128::from(485u128),
                },
            };

            let response: ReverseSimulationResponse = app
                .wrap()
                .query_wasm_smart("contract3".to_string(), &msg)
                .unwrap();

            // Assert ReverseSimulation for cw20 token to native token
            assert_eq!(
                response,
                ReverseSimulationResponse {
                    offer_amount: Uint128::from(998u128),
                    spread_amount: Uint128::from(0u128),
                    commission_amount: Uint128::from(14u128),
                }
            );

            // Swap native token to cw20 token
            let msg = RouterExecuteMsg::ExecuteSwapOperations {
                operations: vec![SwapOperation::HaloSwap {
                    offer_asset_info: AssetInfo::NativeToken {
                        denom: NATIVE_DENOM_2.to_string(),
                    },
                    ask_asset_info: AssetInfo::Token {
                        contract_addr: cw20_token_contract.clone(),
                    },
                }],
                minimum_receive: Some(Uint128::from(485u128)),
                to: None,
            };

            let response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked(router_contract),
                &msg,
                &[Coin {
                    amount: Uint128::from(1000u128),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Query Pool
            let response: PoolResponse = app
                .wrap()
                .query_wasm_smart("contract3".to_string(), &RouterQueryMsg::Pool {})
                .unwrap();

            // Assert Pool: total_share amount
            assert_eq!(
                response,
                PoolResponse {
                    assets: [
                        Asset {
                            info: AssetInfo::NativeToken {
                                denom: NATIVE_DENOM_2.to_string(),
                            },
                            // Verify the native token amount is increased
                            amount: Uint128::from(2001000u128),
                        },
                        Asset {
                            info: AssetInfo::Token {
                                contract_addr: "contract2".to_string(),
                            },
                            // Verify the cw20 token amount is decreased
                            amount: Uint128::from(999515u128),
                        },
                    ],
                    // Verify the total share amount is reserved 1 uLP
                    total_share: 1414213u128.into(),
                }
            );

            // Query cw20 token balance of USER_1
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: USER_1.to_string(),
                    },
                )
                .unwrap();

            // Assert cw20 token balance of USER_1
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(999000485u128),
                }
            );

            // Query Native token balance of USER_1
            let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance {
                address: USER_1.to_string(),
                denom: NATIVE_DENOM_2.to_string(),
            });
            let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
            let balance: BankBalanceResponse = from_binary(&res).unwrap();

            // Verify the native token amount of USER_1 is decreased
            assert_eq!(
                balance.amount.amount,
                Uint128::from(MOCK_1000_NATIVE_TOKEN_AMOUNT - (2001000u128 + MOCK_TRANSACTION_FEE))
            );

            // USER 1 Withdraw Liquidity
            // Send LP token to Pair Contract
            let send_lp_token_msg = Cw20ExecuteMsg::Send {
                contract: "contract3".to_string(),
                amount: Uint128::from(1414212u128),
                msg: to_binary(&Cw20HookMsg::WithdrawLiquidity {}).unwrap(),
            };

            // Execute Send LP token to Pair Contract
            let response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked("contract4".to_string()),
                &send_lp_token_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Query Pool
            let response: PoolResponse = app
                .wrap()
                .query_wasm_smart("contract3".to_string(), &RouterQueryMsg::Pool {})
                .unwrap();

            // Assert Pool: total_share amount
            assert_eq!(
                response,
                PoolResponse {
                    assets: [
                        Asset {
                            info: AssetInfo::NativeToken {
                                denom: NATIVE_DENOM_2.to_string(),
                            },
                            // Verify the native token amount is increased
                            amount: Uint128::from(2u128),
                        },
                        Asset {
                            info: AssetInfo::Token {
                                contract_addr: "contract2".to_string(),
                            },
                            // Verify the cw20 token amount is decreased
                            amount: Uint128::from(1u128),
                        },
                    ],
                    // Verify the total share amount is reserved 1 uLP
                    total_share: LP_TOKEN_RESERVED_AMOUNT.into(),
                }
            );

            // Query LP Balance of USER_1
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    "contract4".to_string(),
                    &Cw20QueryMsg::Balance {
                        address: USER_1.to_string(),
                    },
                )
                .unwrap();

            // Assert LP Balance of USER_1
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(0u128),
                }
            );

            // Query cw20 token balance of USER_1
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    cw20_token_contract,
                    &Cw20QueryMsg::Balance {
                        address: USER_1.to_string(),
                    },
                )
                .unwrap();

            // Assert cw20 token Balance of USER_1
            assert_eq!(
                response,
                BalanceResponse {
                    // USER_1 should lose 1 HALO token that already reserved for the Pool
                    balance: Uint128::from(MOCK_1000_HALO_TOKEN_AMOUNT - 1u128),
                }
            );

            // Query native token balance of USER_1
            let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance {
                address: USER_1.to_string(),
                denom: NATIVE_DENOM_2.to_string(),
            });
            let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
            let balance: BankBalanceResponse = from_binary(&res).unwrap();

            assert_eq!(
                balance.amount.amount,
                // USER_1 should lose 2 native token that already reserved for the Pool
                // and 10000 utaura native token for transaction fee
                Uint128::from(MOCK_1000_NATIVE_TOKEN_AMOUNT - 2u128 - MOCK_TRANSACTION_FEE * 2)
            );
        }
    }
}
