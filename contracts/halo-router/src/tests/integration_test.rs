#[cfg(test)]
mod tests {
    use crate::tests::env_setup::env::{
        instantiate_contracts, ADMIN, NATIVE_DENOM, NATIVE_DENOM_2, USER_1,
    };
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
    use haloswap::router::QueryMsg as RouterQueryMsg;
    // Mock information for CW20 token contract
    const MOCK_1000_HALO_TOKEN_AMOUNT: u128 = 1_000_000_000;
    // Mock information for native token
    const MOCK_1000_NATIVE_TOKEN_AMOUNT: u128 = 1_000_000_000;
    const MOCK_TRANSACTION_FEE: u128 = 5000;
    // Decimal macros
    const DECIMAL_FRACTIONAL_6: u128 = 1_000_000u128;
    const DECIMAL_FRACTIONAL_18: u128 = 1_000_000_000_000_000_000u128;

    mod execute_contract_native_with_cw20_token {
        use std::str::FromStr;

        use cosmwasm_std::Querier;
        use cw_multi_test::Executor;
        use haloswap::{
            asset::{Asset, LPTokenInfo, LP_TOKEN_RESERVED_AMOUNT},
            pair::{
                ExecuteMsg, PoolResponse, QueryMsg, ReverseSimulationResponse, SimulationResponse,
            },
            router::{ExecuteMsg as RouterExecuteMsg, SwapOperation},
        };

        use super::*;
        // This module to verify Native Token works with cw20-token
        // USER_1 Mint 1000 tokens to HALO Token
        // USER_1 Create Pair: AURA - HALO Token
        // USER_1 Add Liquidity: 1000 AURA - 1000 HALO Token
        // USER_1 Swap: 1000 AURA -> HALO Token
        // USER_1 Withdraw Liquidity: 1000 AURA - 1000 HALO Token
        #[test]
        fn proper_operation() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            // Get factory contract
            let factory_contract = contracts[0].contract_addr.clone();
            // Get router contract
            let router_contract = contracts[1].contract_addr.clone();
            // Get halo token contract
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
                    liquidity_token: "contract6".to_string(),
                    asset_infos: [
                        AssetInfo::NativeToken {
                            denom: NATIVE_DENOM_2.to_string(),
                        },
                        AssetInfo::Token {
                            contract_addr: cw20_token_contract.clone(),
                        },
                    ],
                    contract_addr: "contract5".to_string(), // Pair Contract
                    asset_decimals: [6u8, 6u8],
                    requirements: CreatePairRequirements {
                        whitelist: vec![Addr::unchecked(USER_1.to_string())],
                        first_asset_minimum: Uint128::zero(),
                        second_asset_minimum: Uint128::zero(),
                    },
                    // Verify the default commission rate is 3%
                    commission_rate: Decimal256::from_str("0.03").unwrap(),
                }
            );

            // Query LP Token Info
            let response: TokenInfoResponse = app
                .wrap()
                .query_wasm_smart("contract6".to_string(), &cw20::Cw20QueryMsg::TokenInfo {})
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
                spender: "contract5".to_string(), // Pair Contract
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
                Addr::unchecked("contract5".to_string()),
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
                .query_wasm_smart("contract5".to_string(), &QueryMsg::Pool {})
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
                    "contract6".to_string(),
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
                    "contract6".to_string(),
                    &Cw20QueryMsg::Balance {
                        address: "contract6".to_string(),
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
                minimum_receive: Some(Uint128::from(480u128)),
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
                .query_wasm_smart("contract5".to_string(), &QueryMsg::Pool {})
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
                            amount: Uint128::from(2000990u128),
                        },
                        Asset {
                            info: AssetInfo::Token {
                                contract_addr: "contract2".to_string(),
                            },
                            // Verify the cw20 token amount is decreased
                            amount: Uint128::from(999520u128),
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
                    balance: Uint128::from(999000480u128),
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
                contract: "contract5".to_string(),
                amount: Uint128::from(1414212u128),
                msg: to_binary(&Cw20HookMsg::WithdrawLiquidity {}).unwrap(),
            };

            // Execute Send LP token to Pair Contract
            let response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked("contract6".to_string()),
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
                .query_wasm_smart("contract5".to_string(), &QueryMsg::Pool {})
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
                    "contract6".to_string(),
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
                Uint128::from(
                    MOCK_1000_NATIVE_TOKEN_AMOUNT - 2u128 - MOCK_TRANSACTION_FEE * 2 - 10u128
                )
            );
        }

        // Mint 1000 native tokens NATIVE_DENOM to USER_1
        // Mint 1000 native tokens NATIVE_DENOM_2 to USER_1
        // Mint 1000 HALO tokens to USER_1
        // Add Native Token Decimals is 6 for NATIVE_DENOM and NATIVE_DENOM_2
        // Create Pair: NATIVE_DENOM - HALO Token
        // Create Pair: NATIVE_DENOM_2 - HALO Token
        // Create Pair: NATIVE_DENOM - NATIVE_DENOM_2
        // Update Native Token Decimals is 8 for NATIVE_DENOM and 9 NATIVE_DENOM_2
        // Query Native Token Decimals for NATIVE_DENOM and NATIVE_DENOM_2 on all pairs
        // for both halo-factory and halo-pair contracts
        // and it should be [8,9] for [NATIVE_DENOM, NATIVE_DENOM_2]
        #[test]
        fn update_native_token_decimals_for_pairs() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            // Get factory contract
            let factory_contract = contracts[0].contract_addr.clone();
            // Get cw20 token contract
            let cw20_token_contract = contracts[2].contract_addr.clone();

            // Mint 1000 native tokens NATIVE_DENOM to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: Uint128::from(MOCK_1000_NATIVE_TOKEN_AMOUNT),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

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

            // Mint 1000 HALO tokens to USER_1
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

            // Add Native Token Decimals for NATIVE_DENOM
            let msg = FactoryExecuteMsg::AddNativeTokenDecimals {
                denom: NATIVE_DENOM.to_string(),
                decimals: 6u8,
            };

            // Execute add native token decimals
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(factory_contract.clone()),
                &msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );
            assert!(response.is_ok());

            // Add Native Token Decimals for NATIVE_DENOM_2
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

            // Assert decimals of native token of NATIVE_DENOM
            let response: NativeTokenDecimalsResponse = app
                .wrap()
                .query_wasm_smart(
                    factory_contract.clone(),
                    &FactoryQueryMsg::NativeTokenDecimals {
                        denom: NATIVE_DENOM.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(response.decimals, 6u8);

            // Assert decimals of native token of NATIVE_DENOM_2
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

            // Create Pair: NATIVE_DENOM - HALO Token
            let asset_infos = [
                AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
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
                    lp_token_name: "UAURA-HALO".to_string(),
                    lp_token_symbol: "UAURA-HALO".to_string(),
                    lp_token_decimals: None,
                },
            };

            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(factory_contract.clone()),
                &create_pair_msg,
                &[Coin {
                    amount: Uint128::from(500000u128),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Query Pair
            let response: PairInfo = app
                .wrap()
                .query_wasm_smart(
                    factory_contract.clone(),
                    &FactoryQueryMsg::Pair {
                        asset_infos: [
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM.to_string(),
                            },
                            AssetInfo::Token {
                                contract_addr: cw20_token_contract.clone(),
                            },
                        ],
                    },
                )
                .unwrap();

            // Assert token decimals of NATIVE_DENOM in Pair
            assert_eq!(response.asset_decimals, [6u8, 6u8]);

            // Create Pair: NATIVE_DENOM_2 - HALO Token
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
                    lp_token_name: "UTAURA-HALO".to_string(),
                    lp_token_symbol: "UTAURA-HALO".to_string(),
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
                    factory_contract.clone(),
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

            // Assert token decimals of NATIVE_DENOM_2 in Pair
            assert_eq!(response.asset_decimals, [6u8, 6u8]);

            // Create Pair: NATIVE_DENOM - NATIVE_DENOM_2
            let asset_infos = [
                AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                AssetInfo::NativeToken {
                    denom: NATIVE_DENOM_2.to_string(),
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
                    lp_token_name: "UAURA-UTAURA".to_string(),
                    lp_token_symbol: "UAURA-UTAURA".to_string(),
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
                    factory_contract.clone(),
                    &FactoryQueryMsg::Pair {
                        asset_infos: [
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM.to_string(),
                            },
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM_2.to_string(),
                            },
                        ],
                    },
                )
                .unwrap();

            // Assert token decimals of NATIVE_DENOM and NATIVE_DENOM_2 in Pair
            assert_eq!(response.asset_decimals, [6u8, 6u8]);

            // Add Native Token Decimals for NATIVE_DENOM
            let msg = FactoryExecuteMsg::AddNativeTokenDecimals {
                denom: NATIVE_DENOM.to_string(),
                decimals: 8u8,
            };

            // Execute add native token decimals
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(factory_contract.clone()),
                &msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Add Native Token Decimals for NATIVE_DENOM_2
            let msg = FactoryExecuteMsg::AddNativeTokenDecimals {
                denom: NATIVE_DENOM_2.to_string(),
                decimals: 9u8,
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

            // Query NATIVE_DENOM - HALO Pair
            let response: PairInfo = app
                .wrap()
                .query_wasm_smart(
                    factory_contract.clone(),
                    &FactoryQueryMsg::Pair {
                        asset_infos: [
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM.to_string(),
                            },
                            AssetInfo::Token {
                                contract_addr: cw20_token_contract.clone(),
                            },
                        ],
                    },
                )
                .unwrap();

            // Assert token decimals of NATIVE_DENOM in Pair
            assert_eq!(response.asset_decimals, [8u8, 6u8]);

            // Query NATIVE_DENOM_2 - HALO Pair
            let response: PairInfo = app
                .wrap()
                .query_wasm_smart(
                    factory_contract.clone(),
                    &FactoryQueryMsg::Pair {
                        asset_infos: [
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM_2.to_string(),
                            },
                            AssetInfo::Token {
                                contract_addr: cw20_token_contract,
                            },
                        ],
                    },
                )
                .unwrap();

            // Assert token decimals of NATIVE_DENOM_2 in Pair
            assert_eq!(response.asset_decimals, [9u8, 6u8]);

            // Query NATIVE_DENOM - NATIVE_DENOM_2 Pair
            let response: PairInfo = app
                .wrap()
                .query_wasm_smart(
                    factory_contract,
                    &FactoryQueryMsg::Pair {
                        asset_infos: [
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM.to_string(),
                            },
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM_2.to_string(),
                            },
                        ],
                    },
                )
                .unwrap();

            // Assert token decimals of NATIVE_DENOM and NATIVE_DENOM_2 in Pair
            assert_eq!(response.asset_decimals, [8u8, 9u8]);

            // Query Pair of NATIVE_DENOM - HALO Pair
            let response: PairInfo = app
                .wrap()
                .query_wasm_smart("contract5".to_string(), &QueryMsg::Pair {})
                .unwrap();

            // Assert token decimals of NATIVE_DENOM in Pair
            assert_eq!(response.asset_decimals, [8u8, 6u8]);

            // Query Pair of NATIVE_DENOM_2 - HALO Pair
            let response: PairInfo = app
                .wrap()
                .query_wasm_smart("contract7".to_string(), &QueryMsg::Pair {})
                .unwrap();

            // Assert token decimals of NATIVE_DENOM_2 in Pair
            assert_eq!(response.asset_decimals, [9u8, 6u8]);

            // Query Pair of NATIVE_DENOM - NATIVE_DENOM_2 Pair
            let response: PairInfo = app
                .wrap()
                .query_wasm_smart("contract9".to_string(), &QueryMsg::Pair {})
                .unwrap();

            // Assert token decimals of NATIVE_DENOM and NATIVE_DENOM_2 in Pair
            assert_eq!(response.asset_decimals, [8u8, 9u8]);
        }

        // Mint 500_000 MSTR tokens to USER_1
        // Mint 50_000 NATIVE_DENOM tokens to USER_1
        // Add Native Token Decimals is 6 for NATIVE_DENOM
        // Create Pair: MSTR - NATIVE_DENOM Token
        // USER_1 Add Liquidity: 49_867_841_058 AURA - 494_676_638_256_289_699_505_510 MSTR Token
        // USER_1 Swap: 0.49 MSTR -> AURA Token
        // Update commission rate to 5%
        // Update pool fee rate to 2%
        #[test]
        fn test_swap_cw20_decimal_18_with_native_decimal_6() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            // Get factory contract
            let factory_contract = contracts[0].contract_addr.clone();
            // Get router contract
            let router_contract = contracts[1].contract_addr.clone();
            // Get MSTR token contract
            let mstr_token_contract = contracts[3].contract_addr.clone();

            // Mint 500_000 MSTR tokens to USER_1
            let mint_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Mint {
                recipient: USER_1.to_string(),
                amount: Uint128::from(500_000u128 * DECIMAL_FRACTIONAL_18),
            };

            // Execute minting
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(mstr_token_contract.clone()),
                &mint_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Mint 50_000 NATIVE_DENOM tokens to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: Uint128::from(50_000u128 * DECIMAL_FRACTIONAL_6),
                        denom: NATIVE_DENOM.to_string(),
                    }],
                },
            ))
            .unwrap();

            // Add Native Token Decimals for NATIVE_DENOM
            let msg = FactoryExecuteMsg::AddNativeTokenDecimals {
                denom: NATIVE_DENOM.to_string(),
                decimals: 6u8,
            };

            // Execute add native token decimals
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(factory_contract.clone()),
                &msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Create Pair: MSTR - NATIVE_DENOM Token
            let asset_infos = [
                AssetInfo::Token {
                    contract_addr: mstr_token_contract.clone(),
                },
                AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
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
                    lp_token_name: "MSTR-AURA".to_string(),
                    lp_token_symbol: "MSTR-AURA".to_string(),
                    lp_token_decimals: None,
                },
            };

            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(factory_contract.clone()),
                &create_pair_msg,
                &[Coin {
                    amount: Uint128::from(500000u128),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Query Pair
            let response: PairInfo = app
                .wrap()
                .query_wasm_smart(
                    factory_contract.clone(),
                    &FactoryQueryMsg::Pair {
                        asset_infos: [
                            AssetInfo::Token {
                                contract_addr: mstr_token_contract.clone(),
                            },
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM.to_string(),
                            },
                        ],
                    },
                )
                .unwrap();

            // Assert Pair
            assert_eq!(
                response,
                PairInfo {
                    liquidity_token: "contract6".to_string(),
                    asset_infos: [
                        AssetInfo::Token {
                            contract_addr: mstr_token_contract.clone(),
                        },
                        AssetInfo::NativeToken {
                            denom: NATIVE_DENOM.to_string(),
                        },
                    ],
                    contract_addr: "contract5".to_string(), // Pair Contract
                    asset_decimals: [18u8, 6u8],
                    requirements: CreatePairRequirements {
                        whitelist: vec![Addr::unchecked(USER_1.to_string())],
                        first_asset_minimum: Uint128::zero(),
                        second_asset_minimum: Uint128::zero(),
                    },
                    // Verify the default commission rate is 3%
                    commission_rate: Decimal256::from_str("0.03").unwrap(),
                }
            );

            // provide liquidity
            // create provide liquidity message
            // Approve cw20 token to pair contract
            let approve_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: "contract5".to_string(), // Pair Contract
                amount: Uint128::from(900_000u128 * DECIMAL_FRACTIONAL_18),
                expires: None,
            };

            // Execute approve
            let response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked(mstr_token_contract.clone()),
                &approve_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(response.is_ok());

            // USER_1 Provide Liquidity
            let provide_liquidity_msg = ExecuteMsg::ProvideLiquidity {
                assets: [
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: mstr_token_contract.clone(),
                        },
                        amount: Uint128::from(494676638256289699505510u128),
                    },
                    Asset {
                        info: AssetInfo::NativeToken {
                            denom: NATIVE_DENOM.to_string(),
                        },
                        amount: Uint128::from(49867841058u128),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            let response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked("contract5".to_string()),
                &provide_liquidity_msg,
                &[Coin {
                    amount: Uint128::from(49867841058u128),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Query Pool
            let response: PoolResponse = app
                .wrap()
                .query_wasm_smart("contract5".to_string(), &QueryMsg::Pool {})
                .unwrap();

            // Assert Pool
            assert_eq!(
                response,
                PoolResponse {
                    assets: [
                        Asset {
                            info: AssetInfo::Token {
                                contract_addr: mstr_token_contract.clone(),
                            },
                            amount: Uint128::from(494676638256289699505510u128),
                        },
                        Asset {
                            info: AssetInfo::NativeToken {
                                denom: NATIVE_DENOM.to_string(),
                            },
                            amount: Uint128::from(49867841058u128),
                        },
                    ],
                    total_share: Uint128::from(157061949471125618u128),
                }
            );

            // Query Simulation(offer_asset: 0.5 MSTR)
            let response: SimulationResponse = app
                .wrap()
                .query_wasm_smart(
                    "contract5".to_string(),
                    &QueryMsg::Simulation {
                        offer_asset: Asset {
                            info: AssetInfo::Token {
                                contract_addr: mstr_token_contract.clone(),
                            },
                            amount: Uint128::from(500000000000000000u128),
                        },
                    },
                )
                .unwrap();

            // Assert Simulation
            assert_eq!(
                response,
                SimulationResponse {
                    return_amount: Uint128::from(48892u128),
                    spread_amount: Uint128::from(0u128),
                    commission_amount: Uint128::from(1512u128),
                }
            );

            // Query Reverse Simulation(ask_asset: 0.5 MSTR)
            let response: ReverseSimulationResponse = app
                .wrap()
                .query_wasm_smart(
                    "contract5".to_string(),
                    &QueryMsg::ReverseSimulation {
                        ask_asset: Asset {
                            info: AssetInfo::NativeToken {
                                denom: NATIVE_DENOM.to_string(),
                            },
                            amount: Uint128::from(5_000_000u128),
                        },
                    },
                )
                .unwrap();

            // Assert Reverse Simulation
            assert_eq!(
                response,
                ReverseSimulationResponse {
                    offer_amount: Uint128::from(51_138_028_755_970_523_361u128), // Decimal: 18
                    spread_amount: Uint128::from(483u128),
                    commission_amount: Uint128::from(154_639u128),
                }
            );

            // Swap MSTR to AURA
            let msg = RouterExecuteMsg::ExecuteSwapOperations {
                operations: vec![SwapOperation::HaloSwap {
                    offer_asset_info: AssetInfo::Token {
                        contract_addr: mstr_token_contract.clone(),
                    },
                    ask_asset_info: AssetInfo::NativeToken {
                        denom: NATIVE_DENOM.to_string(),
                    },
                }],
                minimum_receive: Some(Uint128::from(46467u128)),
                to: None,
            };

            // Send 0.49 MSTR to Router Contract
            let send_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Send {
                contract: router_contract.clone(),
                amount: Uint128::from(480000000000000000u128),
                msg: to_binary(&msg).unwrap(),
            };

            // Execute send
            let response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked(mstr_token_contract.clone()),
                &send_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Update commission rate to 0.05
            let msg = FactoryExecuteMsg::UpdateCommissionRate {
                contract: "contract5".to_string(),
                commission_rate: Decimal256::from_str("0.05").unwrap(),
            };

            // Execute update commission rate
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(factory_contract.clone()),
                &msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM.to_string(),
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
                            AssetInfo::Token {
                                contract_addr: mstr_token_contract.clone(),
                            },
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM.to_string(),
                            },
                        ],
                    },
                )
                .unwrap();

            // Assert Pair
            assert_eq!(
                response,
                PairInfo {
                    liquidity_token: "contract6".to_string(),
                    asset_infos: [
                        AssetInfo::Token {
                            contract_addr: mstr_token_contract,
                        },
                        AssetInfo::NativeToken {
                            denom: NATIVE_DENOM.to_string(),
                        },
                    ],
                    contract_addr: "contract5".to_string(), // Pair Contract
                    asset_decimals: [18u8, 6u8],
                    requirements: CreatePairRequirements {
                        whitelist: vec![Addr::unchecked(USER_1.to_string())],
                        first_asset_minimum: Uint128::zero(),
                        second_asset_minimum: Uint128::zero(),
                    },
                    // Verify the default rate is 5%
                    commission_rate: Decimal256::from_str("0.05").unwrap(),
                }
            );

            // Update Platform Fee in router contract
            let msg = RouterExecuteMsg::UpdatePlatformFee {
                fee: Decimal256::from_str("0.02").unwrap(),
                manager: ADMIN.to_string(),
            };

            // Execute update platform fee
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(router_contract.clone()),
                &msg,
                &[Coin {
                    amount: Uint128::from(1u128),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Query Router
            let response: Decimal256 = app
                .wrap()
                .query_wasm_smart(router_contract, &RouterQueryMsg::PlatformFee {})
                .unwrap();

            // Assert Router
            assert_eq!(response, Decimal256::from_str("0.02").unwrap());
        }

        // Mint 340_282_366_921 + 2 MSTR tokens to USER_1
        // Mint 340_282_366_921 + 2 USDC tokens to USER_1
        // Create Pair: MSTR - USDC Token
        // USER_1 Successfully Add Liquidity: 2 MSTR - 2 USDC Token for initial liquidity
        // USER_1 Successfully Add Liquidity: 340_282_366_918 MSTR - 340_282_366_918 USDC Token
        // USER_1 Withdraw Liquidity: 340_282_366_918 MSTR - 340_282_366_918 USDC Token
        // USER_1 Fail to Add Liquidity: 340_282_366_921 MSTR - 340_282_366_921 USDC Token with panic:
        // "arithmetic operation overflow"
        #[test]
        #[should_panic(expected = "arithmetic operation overflow")]
        fn test_provide_liquidity_exceed_max_value() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            // Get factory contract
            let factory_contract = contracts[0].contract_addr.clone();
            // Get MSTR token contract
            let mstr_token_contract = contracts[3].contract_addr.clone();
            // Get USDC token contract
            let usdc_token_contract = contracts[4].contract_addr.clone();

            // Mint 50_000 NATIVE_DENOM tokens to USER_1
            app.sudo(cw_multi_test::SudoMsg::Bank(
                cw_multi_test::BankSudo::Mint {
                    to_address: USER_1.to_string(),
                    amount: vec![Coin {
                        amount: Uint128::from(50_000u128 * DECIMAL_FRACTIONAL_6),
                        denom: NATIVE_DENOM_2.to_string(),
                    }],
                },
            ))
            .unwrap();

            // Mint 340_282_366_921 MSTR tokens to USER_1
            let mint_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Mint {
                recipient: USER_1.to_string(),
                amount: Uint128::from((340_282_366_921u128 + 2u128) * DECIMAL_FRACTIONAL_18),
            };

            // Execute minting
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(mstr_token_contract.clone()),
                &mint_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Mint 340_282_366_921 USDC tokens to USER_1
            let mint_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Mint {
                recipient: USER_1.to_string(),
                amount: Uint128::from((340_282_366_921u128 + 2u128) * DECIMAL_FRACTIONAL_18),
            };

            // Execute minting
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(usdc_token_contract.clone()),
                &mint_msg,
                &[Coin {
                    amount: Uint128::from(1u128),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Create Pair: MSTR - USDC Token
            let asset_infos = [
                AssetInfo::Token {
                    contract_addr: mstr_token_contract.clone(),
                },
                AssetInfo::Token {
                    contract_addr: usdc_token_contract.clone(),
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
                    lp_token_name: "MSTR-USDC".to_string(),
                    lp_token_symbol: "MSTR-USDC".to_string(),
                    lp_token_decimals: None,
                },
            };

            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(factory_contract),
                &create_pair_msg,
                &[Coin {
                    amount: Uint128::from(1u128),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(response.is_ok());

            // provide liquidity
            // create provide liquidity message
            // Approve USDC token to pair contract
            let approve_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: "contract5".to_string(), // Pair Contract
                amount: Uint128::from(340_282_366_921u128 * DECIMAL_FRACTIONAL_18 * 10),
                expires: None,
            };

            // Execute approve
            let response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked(usdc_token_contract.clone()),
                &approve_msg,
                &[Coin {
                    amount: Uint128::from(1u128),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Approve MSTR token to pair contract
            let approve_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: "contract5".to_string(), // Pair Contract
                amount: Uint128::from(340_282_366_921u128 * DECIMAL_FRACTIONAL_18 * 10),
                expires: None,
            };

            // Execute approve
            let response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked(mstr_token_contract.clone()),
                &approve_msg,
                &[Coin {
                    amount: Uint128::from(1u128),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // USER_1 Provide Liquidity
            let provide_liquidity_msg = ExecuteMsg::ProvideLiquidity {
                assets: [
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: mstr_token_contract.clone(),
                        },
                        amount: Uint128::from(2u128 * DECIMAL_FRACTIONAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(2u128 * DECIMAL_FRACTIONAL_18),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            let response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked("contract5".to_string()),
                &provide_liquidity_msg,
                &[Coin {
                    amount: Uint128::from(1u128),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // USER_1 Provide Liquidity
            let provide_liquidity_msg = ExecuteMsg::ProvideLiquidity {
                assets: [
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: mstr_token_contract.clone(),
                        },
                        amount: Uint128::from(340_282_366_918u128 * DECIMAL_FRACTIONAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(340_282_366_918u128 * DECIMAL_FRACTIONAL_18),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            let response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked("contract5".to_string()),
                &provide_liquidity_msg,
                &[Coin {
                    amount: Uint128::from(1u128),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Query Balance of USER_1 in LP Token by calling cw20 balance_of
            let user_1_lp_response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    "contract6".to_string(),
                    &Cw20QueryMsg::Balance {
                        address: USER_1.to_string(),
                    },
                )
                .unwrap();

            // Assert Balance of USER_1 in LP Token
            assert_eq!(
                user_1_lp_response.balance,
                Uint128::from(340282366919999999999999999999u128),
            );

            // Withdraw Liquidity msg
            let msg = Cw20HookMsg::WithdrawLiquidity {};

            // Send 340282366919999999999999999999 LP Token to Pair Contract
            let send_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Send {
                contract: "contract5".to_string(),
                amount: user_1_lp_response.balance,
                msg: to_binary(&msg).unwrap(),
            };

            // Execute send
            let response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked("contract6".to_string()),
                &send_msg,
                &[Coin {
                    amount: Uint128::from(1u128),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // USER_1 Provide Overflow Liquidity
            let provide_liquidity_msg = ExecuteMsg::ProvideLiquidity {
                assets: [
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: mstr_token_contract,
                        },
                        amount: Uint128::from(340_282_366_921u128 * DECIMAL_FRACTIONAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract,
                        },
                        amount: Uint128::from(340_282_366_921u128 * DECIMAL_FRACTIONAL_18),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            let _response = app.execute_contract(
                Addr::unchecked(USER_1.to_string()),
                Addr::unchecked("contract5".to_string()),
                &provide_liquidity_msg,
                &[Coin {
                    amount: Uint128::from(1u128),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );
        }
    }
}
