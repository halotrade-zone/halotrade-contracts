#[cfg(test)]
mod tests {
    use crate::tests::stable_env_setup::env::{instantiate_contracts, ADMIN, NATIVE_DENOM_2};
    use bignumber::Decimal256;
    use cosmwasm_std::{
        from_binary, to_binary, Addr, BalanceResponse as BankBalanceResponse, BankQuery, Coin,
        QueryRequest, Uint128,
    };
    use cw20::BalanceResponse;
    use cw20_base::{msg::ExecuteMsg as Cw20ExecuteMsg, msg::QueryMsg as Cw20QueryMsg};
    use halo_stable_factory::msg::{
        ExecuteMsg as StableFactoryExecuteMsg, QueryMsg as StableFactoryQueryMsg,
    };
    use halo_stable_pair::msg::{
        ExecuteMsg as StablePairExecuteMsg, QueryMsg as StablePairQueryMsg,
    };
    use haloswap::asset::{AssetInfo, CreatePairRequirements, PairInfo};
    use haloswap::factory::ExecuteMsg as FactoryExecuteMsg;
    use haloswap::factory::{ExecuteMsg as HaloFactoryExecuteMsg, QueryMsg as HaloFactoryQueryMsg};

    mod execute_interacting_with_stable_swap {
        use cosmwasm_std::{Querier, WasmQuery};
        use cw_multi_test::Executor;
        use halo_stable_pair::{
            math::AmpFactor,
            state::{CreateStablePairRequirements, StablePairInfo, StablePoolResponse},
        };
        use haloswap::{
            asset::{Asset, LPTokenInfo},
            factory::NativeTokenDecimalsResponse,
            pair::ExecuteMsg,
            router::{
                ExecuteMsg as RouterExecuteMsg, QueryMsg as RouterQueryMsg,
                SimulateSwapOperationsResponse, SwapOperation,
            },
        };
        use std::str::FromStr;

        use crate::tests::stable_env_setup::env::NATIVE_DENOM;

        use super::*;

        // Mock 1_000_000_000 USDC token amount
        const MOCK_1_000_000_000_USDC: u128 = 1_000_000_000_000_000_000_000_000_000u128;
        // Mock 1_000_000_000 USDT token amount
        const MOCK_1_000_000_000_USDT: u128 = 1_000_000_000_000_000_000_000_000_000u128;
        // Mock 1_000_000_000 BUSD token amount
        const MOCK_1_000_000_000_BUSD: u128 = 1_000_000_000_000_000_000_000_000_000u128;
        // Decimal 18 macro
        const ONE_UNIT_OF_DECIMAL_18: u128 = 1_000_000_000_000_000_000u128;
        // Decimal 6 macro
        const ONE_UNIT_OF_DECIMAL_6: u128 = 1_000_000u128;

        const MOCK_TRANSACTION_FEE: u128 = 5000;

        // Create a stable pool with 3 tokens USDC, USDT, BUSD
        // Provide liquidity to the stable pool (10000 USDC, 20000 USDT, 30000 BUSD)
        // Create a pool NATIVE, USDC
        // Provide liquidity to the pool (10000 NATIVE, 5000 USDC)
        // ADMIN swap 100 NATIVE to USDT
        // -> ADMIN should get approximately 50 USDT
        #[test]
        fn test_swap_cw20_stable_pair() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            // get the halo factory contract
            let halo_factory_contract = &contracts[0].contract_addr.clone();
            // get the stable factory contract
            let stable_factory_contract = &contracts[1].contract_addr.clone();
            // ger router contract
            let router_contract = &contracts[2].contract_addr.clone();
            // get the USDC contract
            let usdc_token_contract = &contracts[4].contract_addr.clone();
            // get the USDT contract
            let usdt_token_contract = &contracts[5].contract_addr.clone();
            // get the BUSD contract
            let busd_token_contract = &contracts[6].contract_addr.clone();
            // get current block time
            let current_block_time = app.block_info().time.seconds();
            // mint 1_000_000_000 USDC token to ADMIN
            let mint_msg = Cw20ExecuteMsg::Mint {
                recipient: ADMIN.to_string(),
                amount: MOCK_1_000_000_000_USDC.into(),
            };

            // Execute minting
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(usdc_token_contract.clone()),
                &mint_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // mint 1_000_000_000 USDT token to ADMIN
            let mint_msg = Cw20ExecuteMsg::Mint {
                recipient: ADMIN.to_string(),
                amount: MOCK_1_000_000_000_USDT.into(),
            };

            // Execute minting
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(usdt_token_contract.clone()),
                &mint_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // mint 1_000_000_000 BUSD token to ADMIN
            let mint_msg = Cw20ExecuteMsg::Mint {
                recipient: ADMIN.to_string(),
                amount: MOCK_1_000_000_000_BUSD.into(),
            };

            // Execute minting
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(busd_token_contract.clone()),
                &mint_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // create stable pool USDC, USDT, BUSD
            let asset_infos = vec![
                AssetInfo::Token {
                    contract_addr: usdc_token_contract.clone(),
                },
                AssetInfo::Token {
                    contract_addr: usdt_token_contract.clone(),
                },
                AssetInfo::Token {
                    contract_addr: busd_token_contract.clone(),
                },
            ];

            // create stable pool msg
            let create_stable_pair_msg = StableFactoryExecuteMsg::CreateStablePair {
                asset_infos,
                requirements: CreateStablePairRequirements {
                    whitelist: vec![Addr::unchecked(ADMIN.to_string())],
                    asset_minimum: vec![
                        Uint128::from(1u128),
                        Uint128::from(1u128),
                        Uint128::from(1u128),
                    ],
                },
                commission_rate: None,
                lp_token_info: LPTokenInfo {
                    lp_token_name: "Stable-LP-Token".to_string(),
                    lp_token_symbol: "HALO-SLP".to_string(),
                    lp_token_decimals: None,
                },
                amp_factor_info: AmpFactor {
                    initial_amp_factor: Uint128::from(2000u128),
                    target_amp_factor: Uint128::from(2000u128),
                    current_ts: current_block_time,
                    start_ramp_ts: current_block_time,
                    stop_ramp_ts: current_block_time + 10,
                },
            };

            // Execute create stable pool
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(stable_factory_contract.clone()),
                &create_stable_pair_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // query stable pool info
            let create_stable_pair_response: StablePairInfo = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(stable_factory_contract.clone()),
                    &StableFactoryQueryMsg::StablePair {
                        asset_infos: vec![
                            AssetInfo::Token {
                                contract_addr: usdc_token_contract.clone(),
                            },
                            AssetInfo::Token {
                                contract_addr: usdt_token_contract.clone(),
                            },
                            AssetInfo::Token {
                                contract_addr: busd_token_contract.clone(),
                            },
                        ],
                    },
                )
                .unwrap();

            // Assert stable pool info
            assert_eq!(
                create_stable_pair_response,
                StablePairInfo {
                    contract_addr: create_stable_pair_response.contract_addr.clone(),
                    liquidity_token: create_stable_pair_response.liquidity_token.clone(),
                    asset_infos: vec![
                        AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                    ],
                    asset_decimals: vec![18, 18, 18],
                    requirements: CreateStablePairRequirements {
                        whitelist: vec![Addr::unchecked(ADMIN.to_string())],
                        asset_minimum: vec![
                            Uint128::from(1u128),
                            Uint128::from(1u128),
                            Uint128::from(1u128)
                        ],
                    },
                    commission_rate: Decimal256::from_str("0.003").unwrap(),
                }
            );

            // increase allowance for stable pool contract
            let increase_allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: create_stable_pair_response.contract_addr.clone(),
                amount: Uint128::from(1_000_000_000u128 * ONE_UNIT_OF_DECIMAL_18),
                expires: None,
            };

            // Execute increase allowance for USDC
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(usdc_token_contract.clone()),
                &increase_allowance_msg,
                &[],
            );

            assert!(response.is_ok());

            // Execute increase allowance for USDT
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(usdt_token_contract.clone()),
                &increase_allowance_msg,
                &[],
            );

            assert!(response.is_ok());

            // Execute increase allowance for BUSD
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(busd_token_contract.clone()),
                &increase_allowance_msg,
                &[],
            );

            assert!(response.is_ok());

            // provide liquidity to the pool
            let provide_liquidity_msg = StablePairExecuteMsg::ProvideLiquidity {
                assets: vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(ONE_UNIT_OF_DECIMAL_18),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            // Execute provide liquidity
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(create_stable_pair_response.contract_addr.to_string()),
                &provide_liquidity_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Query LP Balance of ADMIN
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    create_stable_pair_response.liquidity_token.to_string(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // Assert LP Balance of ADMIN
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(2_999_999u128),
                }
            );

            // provide liquidity to the pool one more time
            let provide_liquidity_msg = StablePairExecuteMsg::ProvideLiquidity {
                assets: vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(10_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(20_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(30_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            // Execute provide liquidity
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(create_stable_pair_response.contract_addr),
                &provide_liquidity_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Add native token decimals
            let add_native_token_decimals_msg = FactoryExecuteMsg::AddNativeTokenDecimals {
                denom: NATIVE_DENOM_2.to_string(),
                decimals: 6,
            };

            // Execute add native token decimals
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(halo_factory_contract.to_string()),
                &add_native_token_decimals_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // create pool NATIVE, USDC

            let asset_infos = [
                AssetInfo::NativeToken {
                    denom: NATIVE_DENOM_2.to_string(),
                },
                AssetInfo::Token {
                    contract_addr: usdc_token_contract.clone(),
                },
            ];

            // create pool msg
            let create_pool_msg = HaloFactoryExecuteMsg::CreatePair {
                asset_infos,
                requirements: CreatePairRequirements {
                    whitelist: vec![Addr::unchecked(ADMIN.to_string())],
                    first_asset_minimum: Uint128::zero(),
                    second_asset_minimum: Uint128::zero(),
                },
                commission_rate: Some(Decimal256::from_str("0.03").unwrap()),
                lp_token_info: LPTokenInfo {
                    lp_token_name: "aura-USDC".to_string(),
                    lp_token_symbol: "aura-USDC".to_string(),
                    lp_token_decimals: None,
                },
            };

            // Execute create pool
            let create_classic_pool_response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(halo_factory_contract.to_string()),
                &create_pool_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(create_classic_pool_response.is_ok());

            // query pool info
            let create_classic_pool_response: PairInfo = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(halo_factory_contract.to_string()),
                    &HaloFactoryQueryMsg::Pair {
                        asset_infos: [
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM_2.to_string(),
                            },
                            AssetInfo::Token {
                                contract_addr: usdc_token_contract.clone(),
                            },
                        ],
                    },
                )
                .unwrap();

            // Assert pool info
            assert_eq!(
                create_classic_pool_response,
                PairInfo {
                    contract_addr: create_classic_pool_response.contract_addr.clone(),
                    liquidity_token: create_classic_pool_response.liquidity_token.clone(),
                    asset_infos: [
                        AssetInfo::NativeToken {
                            denom: NATIVE_DENOM_2.to_string(),
                        },
                        AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                    ],
                    asset_decimals: [6, 18],
                    commission_rate: Decimal256::from_str("0.03").unwrap(),
                    requirements: CreatePairRequirements {
                        whitelist: vec![Addr::unchecked(ADMIN.to_string())],
                        first_asset_minimum: Uint128::zero(),
                        second_asset_minimum: Uint128::zero(),
                    },
                }
            );

            // increase allowance for pool contract for classic pool
            let increase_allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: create_classic_pool_response.contract_addr.clone(),
                amount: Uint128::from(10_000u128 * ONE_UNIT_OF_DECIMAL_18),
                expires: None,
            };

            // Execute increase allowance for USDC for classic pool
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(usdc_token_contract.clone()),
                &increase_allowance_msg,
                &[],
            );

            assert!(response.is_ok());

            // Provide liquidity to the pool
            let provide_liquidity_msg = ExecuteMsg::ProvideLiquidity {
                assets: [
                    Asset {
                        info: AssetInfo::NativeToken {
                            denom: NATIVE_DENOM_2.to_string(),
                        },
                        amount: Uint128::from(10_000u128 * ONE_UNIT_OF_DECIMAL_6),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(5_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            // Execute provide liquidity
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(create_classic_pool_response.contract_addr),
                &provide_liquidity_msg,
                &[Coin {
                    amount: Uint128::from(10_000u128 * ONE_UNIT_OF_DECIMAL_6),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // query balance of ADMIN in USDT before swap
            let req: QueryRequest<Cw20QueryMsg> = QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: usdt_token_contract.clone(),
                msg: to_binary(&Cw20QueryMsg::Balance {
                    address: ADMIN.to_string(),
                })
                .unwrap(),
            });

            let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
            let usdt_balance_before_swap: BalanceResponse = from_binary(&res).unwrap();

            // Assert balance of ADMIN in USDT before swap
            assert_eq!(
                usdt_balance_before_swap.balance,
                Uint128::from(999_979_999_000_000_000_000_000_000u128),
            );

            // query balance of ADMIN in NATIVE_DENOM_2 before swap
            let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance {
                address: ADMIN.to_string(),
                denom: NATIVE_DENOM_2.to_string(),
            });
            let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
            let denom_2_balance_before_swap: BankBalanceResponse = from_binary(&res).unwrap();

            // Assert balance of ADMIN in NATIVE_DENOM_2 before swap
            assert_eq!(
                denom_2_balance_before_swap.amount.amount,
                Uint128::from(489999960000u128), //489999960000
            );

            // Swap 100 NATIVE to USDT via router with operation HaloSwap(AURA -> USDC) and HaloSwap(USDC -> USDT)
            let swap_msg = RouterExecuteMsg::ExecuteSwapOperations {
                operations: vec![
                    SwapOperation::HaloSwap {
                        offer_asset_info: AssetInfo::NativeToken {
                            denom: NATIVE_DENOM_2.to_string(),
                        },
                        ask_asset_info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                    },
                    SwapOperation::StableSwap {
                        offer_asset_info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        ask_asset_info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        asset_infos: vec![
                            AssetInfo::Token {
                                contract_addr: usdc_token_contract.clone(),
                            },
                            AssetInfo::Token {
                                contract_addr: usdt_token_contract.clone(),
                            },
                            AssetInfo::Token {
                                contract_addr: busd_token_contract.clone(),
                            },
                        ],
                    },
                ],
                minimum_receive: None,
                to: None,
            };

            // Execute swap
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(router_contract.to_string()),
                &swap_msg,
                &[Coin {
                    amount: Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_6),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // query balance of ADMIN in NATIVE_DENOM_2 after swap
            let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance {
                address: ADMIN.to_string(),
                denom: NATIVE_DENOM_2.to_string(),
            });
            let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
            let balance: BankBalanceResponse = from_binary(&res).unwrap();

            // Assert balance of ADMIN in NATIVE_DENOM_2 after swap
            assert_eq!(
                balance.amount.amount,
                denom_2_balance_before_swap.amount.amount
                    - Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_6)
                    + Uint128::from(ONE_UNIT_OF_DECIMAL_6), // platform fee back to ADMIN
            );

            // query balance of ADMIN in USDT after swap
            let req: QueryRequest<Cw20QueryMsg> = QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: usdt_token_contract.clone(),
                msg: to_binary(&Cw20QueryMsg::Balance {
                    address: ADMIN.to_string(),
                })
                .unwrap(),
            });

            let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
            let usdt_balance_after_swap: BalanceResponse = from_binary(&res).unwrap();

            // Assert balance of ADMIN in USDT after swap
            assert_eq!(
                usdt_balance_after_swap.balance,
                usdt_balance_before_swap.balance + Uint128::from(47_547_806_000_000_000_000u128), // approximately 50 USDT
            );
        }
        // Create a stable pool with 1 native token and 2 cw20 tokens NATIVE_DENOM_2, USDT, BUSD
        // Provide liquidity to the stable pool (10000 NATIVE_DENOM_2, 20000 USDT, 30000 BUSD)
        // Create a pool NATIVE_DENOM, USDT
        // Provide liquidity to the classic pool (10000 NATIVE_DENOM, 5000 USDT)
        // ADMIN queries simulation swap 100 NATIVE_DENOM_2 from Stable Pool to NATIVE_DENOM in Classic Pool
        // ADMIN swaps 100 NATIVE_DENOM_2 from Stable Pool to NATIVE_DENOM in Classic Pool
        // -> ADMIN should get approximately 50 NATIVE_DENOM
        #[test]
        fn test_swap_native_stable_pair() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            // get the halo factory contract
            let halo_factory_contract = &contracts[0].contract_addr.clone();
            // get the stable factory contract
            let stable_factory_contract = &contracts[1].contract_addr.clone();
            // ger router contract
            let router_contract = &contracts[2].contract_addr.clone();
            // get the USDT contract
            let usdt_token_contract = &contracts[5].contract_addr.clone();
            // get the BUSD contract
            let busd_token_contract = &contracts[6].contract_addr.clone();
            // get current block time
            let current_block_time = app.block_info().time.seconds();

            // mint 1_000_000_000 USDT token to ADMIN
            let mint_msg = Cw20ExecuteMsg::Mint {
                recipient: ADMIN.to_string(),
                amount: MOCK_1_000_000_000_USDT.into(),
            };

            // Execute minting
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(usdt_token_contract.clone()),
                &mint_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // mint 1_000_000_000 BUSD token to ADMIN
            let mint_msg = Cw20ExecuteMsg::Mint {
                recipient: ADMIN.to_string(),
                amount: MOCK_1_000_000_000_BUSD.into(),
            };

            // Execute minting
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(busd_token_contract.clone()),
                &mint_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // add native token decimals for Stable Factory
            let add_native_token_decimals_msg = StableFactoryExecuteMsg::AddNativeTokenDecimals {
                denom: NATIVE_DENOM_2.to_string(),
                decimals: 6,
            };

            // Execute add native token decimals
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(stable_factory_contract.to_string()),
                &add_native_token_decimals_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Assert decimals of native token of NATIVE_DENOM_2
            let response: NativeTokenDecimalsResponse = app
                .wrap()
                .query_wasm_smart(
                    stable_factory_contract.clone(),
                    &StableFactoryQueryMsg::NativeTokenDecimals {
                        denom: NATIVE_DENOM_2.to_string(),
                    },
                )
                .unwrap();
            assert_eq!(response.decimals, 6u8);

            // create stable pool NATIVE_DENOM_2, USDT, BUSD
            let asset_infos = vec![
                AssetInfo::NativeToken {
                    denom: NATIVE_DENOM_2.to_string(),
                },
                AssetInfo::Token {
                    contract_addr: usdt_token_contract.clone(),
                },
                AssetInfo::Token {
                    contract_addr: busd_token_contract.clone(),
                },
            ];

            // create stable pool msg
            let create_stable_pair_msg = StableFactoryExecuteMsg::CreateStablePair {
                asset_infos,
                requirements: CreateStablePairRequirements {
                    whitelist: vec![Addr::unchecked(ADMIN.to_string())],
                    asset_minimum: vec![
                        Uint128::from(1u128),
                        Uint128::from(1u128),
                        Uint128::from(1u128),
                    ],
                },
                commission_rate: None,
                lp_token_info: LPTokenInfo {
                    lp_token_name: "Stable-LP-Token".to_string(),
                    lp_token_symbol: "HALO-SLP".to_string(),
                    lp_token_decimals: None,
                },
                amp_factor_info: AmpFactor {
                    initial_amp_factor: Uint128::from(2000u128),
                    target_amp_factor: Uint128::from(2000u128),
                    current_ts: current_block_time,
                    start_ramp_ts: current_block_time,
                    stop_ramp_ts: current_block_time + 10,
                },
            };

            // Execute create stable pool
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(stable_factory_contract.clone()),
                &create_stable_pair_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // query stable pair info
            let create_stable_pair_response: StablePairInfo = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(stable_factory_contract.clone()),
                    &StableFactoryQueryMsg::StablePair {
                        asset_infos: vec![
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM_2.to_string(),
                            },
                            AssetInfo::Token {
                                contract_addr: usdt_token_contract.clone(),
                            },
                            AssetInfo::Token {
                                contract_addr: busd_token_contract.clone(),
                            },
                        ],
                    },
                )
                .unwrap();

            // Assert stable pair info
            assert_eq!(
                create_stable_pair_response,
                StablePairInfo {
                    contract_addr: create_stable_pair_response.contract_addr.clone(),
                    liquidity_token: create_stable_pair_response.liquidity_token.clone(),
                    asset_infos: vec![
                        AssetInfo::NativeToken {
                            denom: NATIVE_DENOM_2.to_string(),
                        },
                        AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                    ],
                    asset_decimals: vec![6, 18, 18],
                    requirements: CreateStablePairRequirements {
                        whitelist: vec![Addr::unchecked(ADMIN.to_string())],
                        asset_minimum: vec![
                            Uint128::from(1u128),
                            Uint128::from(1u128),
                            Uint128::from(1u128)
                        ],
                    },
                    commission_rate: Decimal256::from_str("0.003").unwrap(),
                }
            );

            // query stable pool info
            let create_stable_pool_response: StablePoolResponse = app
                .wrap()
                .query_wasm_smart(
                    create_stable_pair_response.contract_addr.to_string(),
                    &StablePairQueryMsg::StablePool {},
                )
                .unwrap();

            // Assert stable pool info
            assert_eq!(
                create_stable_pool_response,
                StablePoolResponse {
                    assets_addr: vec![
                        NATIVE_DENOM_2.to_string(),
                        usdt_token_contract.clone(),
                        busd_token_contract.clone()
                    ],
                    total_share: Uint128::zero(),
                }
            );

            // increase allowance for stable pool contract
            let increase_allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: create_stable_pair_response.contract_addr.clone(),
                amount: Uint128::from(1_000_000_000u128 * ONE_UNIT_OF_DECIMAL_18),
                expires: None,
            };

            // Execute increase allowance for USDT
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(usdt_token_contract.clone()),
                &increase_allowance_msg,
                &[],
            );

            assert!(response.is_ok());

            // Execute increase allowance for BUSD
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(busd_token_contract.clone()),
                &increase_allowance_msg,
                &[],
            );

            assert!(response.is_ok());

            // provide liquidity to the pool
            let provide_liquidity_msg = StablePairExecuteMsg::ProvideLiquidity {
                assets: vec![
                    Asset {
                        info: AssetInfo::NativeToken {
                            denom: NATIVE_DENOM_2.to_string(),
                        },
                        amount: Uint128::from(10_000u128 * ONE_UNIT_OF_DECIMAL_6),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(20_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(30_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            // Execute provide liquidity
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(create_stable_pair_response.contract_addr.clone()),
                &provide_liquidity_msg,
                &[Coin {
                    amount: Uint128::from(10_000u128 * ONE_UNIT_OF_DECIMAL_6),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // query stable pool info
            let create_stable_pool_response: StablePoolResponse = app
                .wrap()
                .query_wasm_smart(
                    create_stable_pair_response.contract_addr,
                    &StablePairQueryMsg::StablePool {},
                )
                .unwrap();

            // Assert stable pool info
            assert_eq!(
                create_stable_pool_response,
                StablePoolResponse {
                    assets_addr: vec![
                        NATIVE_DENOM_2.to_string(),
                        usdt_token_contract.clone(),
                        busd_token_contract.clone()
                    ],
                    total_share: Uint128::from(59_999_629_659u128),
                }
            );

            // add native token decimals for Factory Contract
            let add_native_token_decimals_msg = FactoryExecuteMsg::AddNativeTokenDecimals {
                denom: NATIVE_DENOM.to_string(),
                decimals: 6,
            };

            // Execute add native token decimals
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(halo_factory_contract.to_string()),
                &add_native_token_decimals_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Create a pool NATIVE_DENOM, USDT
            let asset_infos = [
                AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                AssetInfo::Token {
                    contract_addr: usdt_token_contract.clone(),
                },
            ];

            // create pool msg
            let create_pool_msg = HaloFactoryExecuteMsg::CreatePair {
                asset_infos,
                requirements: CreatePairRequirements {
                    whitelist: vec![Addr::unchecked(ADMIN.to_string())],
                    first_asset_minimum: Uint128::zero(),
                    second_asset_minimum: Uint128::zero(),
                },
                commission_rate: Some(Decimal256::from_str("0.03").unwrap()),
                lp_token_info: LPTokenInfo {
                    lp_token_name: "aura-USDT".to_string(),
                    lp_token_symbol: "aura-USDT".to_string(),
                    lp_token_decimals: None,
                },
            };

            // Execute create pool
            let create_classic_pool_response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(halo_factory_contract.to_string()),
                &create_pool_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(create_classic_pool_response.is_ok());

            // query pool info
            let create_classic_pool_response: PairInfo = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(halo_factory_contract.to_string()),
                    &HaloFactoryQueryMsg::Pair {
                        asset_infos: [
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM.to_string(),
                            },
                            AssetInfo::Token {
                                contract_addr: usdt_token_contract.clone(),
                            },
                        ],
                    },
                )
                .unwrap();

            // Assert pool info
            assert_eq!(
                create_classic_pool_response,
                PairInfo {
                    contract_addr: create_classic_pool_response.contract_addr.clone(),
                    liquidity_token: create_classic_pool_response.liquidity_token.clone(),
                    asset_infos: [
                        AssetInfo::NativeToken {
                            denom: NATIVE_DENOM.to_string(),
                        },
                        AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                    ],
                    asset_decimals: [6, 18],
                    commission_rate: Decimal256::from_str("0.03").unwrap(),
                    requirements: CreatePairRequirements {
                        whitelist: vec![Addr::unchecked(ADMIN.to_string())],
                        first_asset_minimum: Uint128::zero(),
                        second_asset_minimum: Uint128::zero(),
                    },
                }
            );

            // increase allowance for pool contract for classic pool
            let increase_allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: create_classic_pool_response.contract_addr.clone(),
                amount: Uint128::from(10_000u128 * ONE_UNIT_OF_DECIMAL_18),
                expires: None,
            };

            // Execute increase allowance for USDT for classic pool
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(usdt_token_contract.clone()),
                &increase_allowance_msg,
                &[],
            );

            assert!(response.is_ok());

            // Provide liquidity to the pool
            let provide_liquidity_msg = ExecuteMsg::ProvideLiquidity {
                assets: [
                    Asset {
                        info: AssetInfo::NativeToken {
                            denom: NATIVE_DENOM.to_string(),
                        },
                        amount: Uint128::from(10_000u128 * ONE_UNIT_OF_DECIMAL_6),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(5_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            // Execute provide liquidity
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(create_classic_pool_response.contract_addr),
                &provide_liquidity_msg,
                &[Coin {
                    amount: Uint128::from(10_000u128 * ONE_UNIT_OF_DECIMAL_6),
                    denom: NATIVE_DENOM.to_string(),
                }],
            );

            assert!(response.is_ok());

            // query balance of ADMIN in BUSD before swap
            let req: QueryRequest<Cw20QueryMsg> = QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: busd_token_contract.clone(),
                msg: to_binary(&Cw20QueryMsg::Balance {
                    address: ADMIN.to_string(),
                })
                .unwrap(),
            });

            let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
            let busd_balance_before_swap: BalanceResponse = from_binary(&res).unwrap();

            // Assert balance of ADMIN in BUSD before swap
            assert_eq!(
                busd_balance_before_swap.balance,
                Uint128::from(999_970_000_000_000_000_000_000_000u128),
            );

            // query balance of ADMIN in NATIVE_DENOM before swap
            let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance {
                address: ADMIN.to_string(),
                denom: NATIVE_DENOM.to_string(),
            });

            let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
            let native_denom_balance_before_swap: BankBalanceResponse = from_binary(&res).unwrap();

            // Assert balance of ADMIN in NATIVE_DENOM before swap
            assert_eq!(
                native_denom_balance_before_swap.amount.amount,
                Uint128::from(989999990000u128),
            );

            // query balance of ADMIN in NATIVE_DENOM_2 before swap
            let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance {
                address: ADMIN.to_string(),
                denom: NATIVE_DENOM_2.to_string(),
            });

            let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
            let native_denom_2_balance_before_swap: BankBalanceResponse =
                from_binary(&res).unwrap();

            // Assert balance of ADMIN in NATIVE_DENOM_2 before swap
            assert_eq!(
                native_denom_2_balance_before_swap.amount.amount,
                Uint128::from(489999980000u128),
            );

            // Query Simulate swap 100 NATIVE_DENOM_2 to NATIVE_DENOM via router with operation HaloStableSwap(NATIVE_DENOM_2 -> USDT) and HaloSwap(USDT -> NATIVE_DENOM)
            let simulate_swap_msg = RouterQueryMsg::SimulateSwapOperations {
                operations: vec![
                    SwapOperation::StableSwap {
                        offer_asset_info: AssetInfo::NativeToken {
                            denom: NATIVE_DENOM_2.to_string(),
                        },
                        ask_asset_info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        asset_infos: vec![
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM_2.to_string(),
                            },
                            AssetInfo::Token {
                                contract_addr: usdt_token_contract.clone(),
                            },
                            AssetInfo::Token {
                                contract_addr: busd_token_contract.clone(),
                            },
                        ],
                    },
                    SwapOperation::HaloSwap {
                        offer_asset_info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        ask_asset_info: AssetInfo::NativeToken {
                            denom: NATIVE_DENOM.to_string(),
                        },
                    },
                ],
                offer_amount: Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_6),
            };

            // Query simulate swap
            let response: SimulateSwapOperationsResponse = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(router_contract.to_string()),
                    &simulate_swap_msg,
                )
                .unwrap();

            assert_eq!(
                response,
                SimulateSwapOperationsResponse {
                    amount: Uint128::from(188_344_521u128),
                }
            );

            // Swap 100 NATIVE_DENOM_2 to NATIVE_DENOM via router with operation HaloStableSwap(NATIVE_DENOM_2 -> USDT) and HaloSwap(USDT -> NATIVE_DENOM)
            let swap_msg = RouterExecuteMsg::ExecuteSwapOperations {
                operations: vec![
                    SwapOperation::StableSwap {
                        offer_asset_info: AssetInfo::NativeToken {
                            denom: NATIVE_DENOM_2.to_string(),
                        },
                        ask_asset_info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        asset_infos: vec![
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM_2.to_string(),
                            },
                            AssetInfo::Token {
                                contract_addr: usdt_token_contract.clone(),
                            },
                            AssetInfo::Token {
                                contract_addr: busd_token_contract.clone(),
                            },
                        ],
                    },
                    SwapOperation::HaloSwap {
                        offer_asset_info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        ask_asset_info: AssetInfo::NativeToken {
                            denom: NATIVE_DENOM.to_string(),
                        },
                    },
                ],
                minimum_receive: None,
                to: None,
            };

            // Execute swap
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(router_contract.to_string()),
                &swap_msg,
                &[Coin {
                    amount: Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_6),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // query balance of ADMIN in NATIVE_DENOM_2 after swap
            let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance {
                address: ADMIN.to_string(),
                denom: NATIVE_DENOM_2.to_string(),
            });

            let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
            let native_denom_2_balance_after_swap: BankBalanceResponse = from_binary(&res).unwrap();

            // Assert balance of ADMIN in NATIVE_DENOM_2 after swap
            assert_eq!(
                native_denom_2_balance_after_swap.amount.amount,
                native_denom_2_balance_before_swap.amount.amount
                    - Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_6)
                    + Uint128::from(ONE_UNIT_OF_DECIMAL_6), // platform fee back to ADMIN
            );

            // query balance of ADMIN in NATIVE_DENOM after swap
            let req: QueryRequest<BankQuery> = QueryRequest::Bank(BankQuery::Balance {
                address: ADMIN.to_string(),
                denom: NATIVE_DENOM.to_string(),
            });

            let res = app.raw_query(&to_binary(&req).unwrap()).unwrap().unwrap();
            let native_denom_balance_after_swap: BankBalanceResponse = from_binary(&res).unwrap();

            // Assert balance of ADMIN in NATIVE_DENOM after swap
            assert_eq!(
                native_denom_balance_after_swap.amount.amount,
                native_denom_balance_before_swap.amount.amount + Uint128::from(188_344_521u128),
            );
        }
    }
}
