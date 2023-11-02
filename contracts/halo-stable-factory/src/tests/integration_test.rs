#[cfg(test)]
mod tests {
    use crate::tests::env_setup::env::{instantiate_contracts, ADMIN, NATIVE_DENOM_2};
    // Mock 1_000_000_000 USDC token amount
    const MOCK_1_000_000_000_USDC: u128 = 1_000_000_000_000_000_000_000_000_000u128;
    // Mock 1_000_000_000 USDT token amount
    const MOCK_1_000_000_000_USDT: u128 = 1_000_000_000_000_000_000_000_000_000u128;
    // Mock 1_000_000_000 BUSD token amount
    const MOCK_1_000_000_000_BUSD: u128 = 1_000_000_000_000_000_000_000_000_000u128;
    // Decimal 18 macro
    const DECIMAL_18: u128 = 1_000_000_000_000_000_000u128;

    const MOCK_TRANSACTION_FEE: u128 = 5000;
    mod execute_contract_native_with_cw20_token {

        use std::str::FromStr;

        use crate::msg::{
            ExecuteMsg as StableFactoryExecuteMsg, QueryMsg as StableFactoryQueryMsg,
        };
        use bignumber::Decimal256;
        use cosmwasm_std::{to_binary, Addr, Coin, Uint128};
        use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
        use cw_multi_test::Executor;
        use halo_stable_pair::msg::ExecuteMsg as StablePairExecuteMsg;
        use halo_stable_pair::{
            math::AmpFactor,
            msg::Cw20StableHookMsg,
            state::{CreateStablePairRequirements, StablePairInfo},
        };
        use haloswap::asset::{Asset, AssetInfo, LPTokenInfo};

        use super::*;
        // Create a stable swap pair with 3 tokens USDC, USDT, BUSD
        // Provide liquidity to the pair (1 USDC, 1 USDT, 1 BUSD)
        // Provide liquidity to the pair one more time (100_000 USDC, 200_000 USDT, 200_000 BUSD)
        // Remove liquidity by Share from the pair by 50% of Share (250_000 LP Token)
        // -> ADMIN should get (50_000.5 USDC, 100_000.5 USDT, 100_000.5 BUSD)
        // Remove liquidity by Token from the pair by 25_000 USDC, 50_000 USDT, 50_000 BUSD
        // -> ADMIN should get (25_000 USDC, 50_000 USDT, 50_000 BUSD) and burn 125_000 LP Token
        #[test]
        fn test_add_liquidity_pair_3_tokens() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            // get the stable factory contract
            let stable_factory_contract = &contracts[0].contract_addr.clone();
            // get the USDC contract
            let usdc_token_contract = &contracts[2].contract_addr.clone();
            // get the USDT contract
            let usdt_token_contract = &contracts[3].contract_addr.clone();
            // get the BUSD contract
            let busd_token_contract = &contracts[4].contract_addr.clone();
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

            // create stable pair USDC, USDT, BUSD
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

            // create stable pair msg
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

            // Execute create stable pair
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
            let response: StablePairInfo = app
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

            // Assert stable pair info
            assert_eq!(
                response,
                StablePairInfo {
                    contract_addr: "contract5".to_string(),
                    liquidity_token: "contract6".to_string(),
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

            // increase allowance for stable pair contract
            let increase_allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: response.contract_addr.clone(),
                amount: Uint128::from(1_000_000_000u128 * DECIMAL_18),
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

            // provide liquidity to the pair
            let provide_liquidity_msg = StablePairExecuteMsg::ProvideLiquidity {
                assets: vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(1u128 * DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(1u128 * DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(1u128 * DECIMAL_18),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            // Execute provide liquidity
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract5".to_string()),
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
                    "contract6".to_string(),
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

            // provide liquidity to the pair one more time
            let provide_liquidity_msg = StablePairExecuteMsg::ProvideLiquidity {
                assets: vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(100_000u128 * DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000u128 * DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000u128 * DECIMAL_18),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            // Execute provide liquidity
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract5".to_string()),
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
                    "contract6".to_string(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // Assert LP Balance of ADMIN
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(500_001_542_633u128),
                }
            );

            // Increase allowance of LP Token for stable pair contract
            let increase_allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: "contract5".to_string(),
                amount: Uint128::from(500_001_542_633u128),
                expires: None,
            };

            // Execute increase allowance for LP Token
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract6".to_string()),
                &increase_allowance_msg,
                &[],
            );

            assert!(response.is_ok());

            // Withdraw liquidity by share from the pair
            let withdraw_liquidity_msg = Cw20StableHookMsg::RemoveLiquidityByShare {
                share: Uint128::from(250_000_771_316u128),
                assets_min_amount: None,
            };

            // Send withdraw liquidity msg to stable pair contract
            let send_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Send {
                contract: "contract5".to_string(),
                amount: Uint128::from(250_000_771_316u128),
                msg: to_binary(&withdraw_liquidity_msg).unwrap(),
            };

            // Execute send msg
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract6".to_string()), // LP Token contract
                &send_msg,
                &[],
            );

            assert!(response.is_ok());

            // Query USDC Balance of ADMIN
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdc_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // Assert USDC Balance of ADMIN
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(
                        MOCK_1_000_000_000_USDC - 1 * DECIMAL_18 - 100_000u128 * DECIMAL_18
                            + 50_000_499_999_799_998_617_057u128
                    ),
                }
            );

            // Query USDT Balance of ADMIN
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdt_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // Assert USDT Balance of ADMIN
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(
                        MOCK_1_000_000_000_USDT - 1 * DECIMAL_18 - 200_000u128 * DECIMAL_18
                            + 100_000_499_999_599_999_234_109u128
                    ),
                }
            );

            // Query BUSD Balance of ADMIN
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    busd_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // Assert BUSD Balance of ADMIN
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(
                        MOCK_1_000_000_000_BUSD - 1 * DECIMAL_18 - 200_000u128 * DECIMAL_18
                            + 100_000_499_999_599_999_234_109u128
                    ),
                }
            );

            // Query LP Balance of ADMIN
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    "contract6".to_string(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // Assert LP Balance of ADMIN
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(250_000_771_317u128),
                }
            );

            // Withdraw liquidity by token from the pair
            let withdraw_liquidity_msg = StablePairExecuteMsg::RemoveLiquidityByToken {
                assets: vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(25_000u128 * DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(50_000u128 * DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(50_000u128 * DECIMAL_18),
                    },
                ],
                max_burn_share: None,
            };

            // Execute withdraw liquidity
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract5".to_string()),
                &withdraw_liquidity_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Query USDC Balance of ADMIN
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdc_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // Assert USDC Balance of ADMIN
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(
                        MOCK_1_000_000_000_USDC - 1 * DECIMAL_18 - 100_000u128 * DECIMAL_18
                            + 50_000_499_999_799_998_617_057u128
                            + 25_000u128 * DECIMAL_18
                    ),
                }
            );

            // Query USDT Balance of ADMIN
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdt_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // Assert USDT Balance of ADMIN
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(
                        MOCK_1_000_000_000_USDT - 1 * DECIMAL_18 - 200_000u128 * DECIMAL_18
                            + 100_000_499_999_599_999_234_109u128
                            + 50_000 * DECIMAL_18
                    ),
                }
            );

            // Query BUSD Balance of ADMIN
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    busd_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // Assert BUSD Balance of ADMIN
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(
                        MOCK_1_000_000_000_BUSD - 1 * DECIMAL_18 - 200_000u128 * DECIMAL_18
                            + 100_000_499_999_599_999_234_109u128
                            + 50_000 * DECIMAL_18
                    ),
                }
            );

            // Query LP Balance of ADMIN
            let response: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    "contract6".to_string(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // Assert LP Balance of ADMIN
            assert_eq!(
                response,
                BalanceResponse {
                    balance: Uint128::from(125_001_135_662u128),
                }
            );
        }

        // Create a stable swap pair with 3 tokens USDC, USDT, BUSD
        // Provide liquidity to the pair (1 USDC, 1 USDT, 1 BUSD)
        // Provide liquidity to the pair one more time (100_000 USDC, 200_000 USDT, 200_000 BUSD)
        // ADMIN swap 1 USDC to USDT
        // -> ADMIN should get 1 USDT
        // ADMIN swap 9 USDT to BUSD
        // -> ADMIN should get 9 BUSD
        // ADMIN swap 100 BUSD to USDC
        // -> ADMIN should get 100 USDC
        // ADMIN swap 50_000 USDC to USDT
        // -> ADMIN should get 50_000 USDT
        // Provide liquidity to the pair one more time (100_000_000 USDC, 150_000_000 USDT, 200_000_000 BUSD)
        // ADMIN swap 10_000_000 USDC to BUSD
        // -> ADMIN should get 10_000_000 BUSD

        #[test]
        fn test_swap_independently_without_rounter() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            // get the stable factory contract
            let stable_factory_contract = &contracts[0].contract_addr.clone();
            // get the USDC contract
            let usdc_token_contract = &contracts[2].contract_addr.clone();
            // get the USDT contract
            let usdt_token_contract = &contracts[3].contract_addr.clone();
            // get the BUSD contract
            let busd_token_contract = &contracts[4].contract_addr.clone();
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

            // create stable pair USDC, USDT, BUSD
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

            // create stable pair msg
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

            // Execute create stable pair
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
            let response: StablePairInfo = app
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

            // Assert stable pair info
            assert_eq!(
                response,
                StablePairInfo {
                    contract_addr: "contract5".to_string(),
                    liquidity_token: "contract6".to_string(),
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

            // increase allowance for stable pair contract
            let increase_allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: response.contract_addr.clone(),
                amount: Uint128::from(1_000_000_000u128 * DECIMAL_18),
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

            // provide liquidity to the pair
            let provide_liquidity_msg = StablePairExecuteMsg::ProvideLiquidity {
                assets: vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(1u128 * DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(1u128 * DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(1u128 * DECIMAL_18),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            // Execute provide liquidity
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract5".to_string()),
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
                    "contract6".to_string(),
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

            // provide liquidity to the pair one more time
            let provide_liquidity_msg = StablePairExecuteMsg::ProvideLiquidity {
                assets: vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(100_000u128 * DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000u128 * DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000u128 * DECIMAL_18),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            // Execute provide liquidity
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract5".to_string()),
                &provide_liquidity_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // Query USDC Balance of ADMIN before swap
            let usdc_balance_before_swap: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdc_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert USDC Balance of ADMIN before swap
            assert_eq!(
                usdc_balance_before_swap.balance,
                Uint128::from(MOCK_1_000_000_000_USDC - 1 * DECIMAL_18 - 100_000u128 * DECIMAL_18),
            );

            // Query USDT Balance of ADMIN before swap
            let usdt_balance_before_swap: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdt_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert USDT Balance of ADMIN before swap
            assert_eq!(
                usdt_balance_before_swap.balance,
                Uint128::from(MOCK_1_000_000_000_USDT - 1 * DECIMAL_18 - 200_000u128 * DECIMAL_18),
            );

            // ADMIN swap 1 USDC to USDT
            let swap_msg = StablePairExecuteMsg::StableSwap {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: usdc_token_contract.clone(),
                    },
                    amount: Uint128::from(1u128 * DECIMAL_18),
                },
                ask_asset: AssetInfo::Token {
                    contract_addr: usdt_token_contract.clone(),
                },
                max_spread: None,
                belief_price: None,
                to: None,
            };

            // Send 1 USDC to stable pair contract to swap
            let send_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Send {
                contract: "contract5".to_string(),
                amount: Uint128::from(1u128 * DECIMAL_18),
                msg: to_binary(&swap_msg).unwrap(),
            };

            // Execute send
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(usdc_token_contract.clone()),
                &send_msg,
                &[],
            );

            assert!(response.is_ok());

            // Query USDC Balance of ADMIN after swap
            let usdc_balance_after_swap: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdc_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert USDC Balance of ADMIN after swap
            assert_eq!(
                usdc_balance_after_swap.balance,
                Uint128::from(
                    MOCK_1_000_000_000_USDC
                        - 1 * DECIMAL_18
                        - 100_000u128 * DECIMAL_18
                        - 1 * DECIMAL_18
                ), // 1 USDC transferred to the stable pair
            );

            // Query USDT Balance of ADMIN after swap
            let usdt_balance_after_swap: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdt_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert USDT Balance of ADMIN after swap
            assert_eq!(
                usdt_balance_after_swap.balance,
                Uint128::from(
                    MOCK_1_000_000_000_USDT - 1 * DECIMAL_18 - 200_000u128 * DECIMAL_18
                        + 1_000_053_000_000_000_000u128
                ), // 1 USDT received from the stable pair
            );

            // ADMIN swap 9 USDT to BUSD
            let swap_msg = StablePairExecuteMsg::StableSwap {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: usdt_token_contract.clone(),
                    },
                    amount: Uint128::from(9u128 * DECIMAL_18),
                },
                ask_asset: AssetInfo::Token {
                    contract_addr: busd_token_contract.clone(),
                },
                max_spread: None,
                belief_price: None,
                to: None,
            };

            // Send 9 USDT to stable pair contract to swap
            let send_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Send {
                contract: "contract5".to_string(),
                amount: Uint128::from(9u128 * DECIMAL_18),
                msg: to_binary(&swap_msg).unwrap(),
            };

            // Execute send
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(usdt_token_contract.clone()),
                &send_msg,
                &[],
            );

            assert!(response.is_ok());

            // Query USDT Balance of ADMIN after swap
            let usdt_balance_after_swap: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdt_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert USDT Balance of ADMIN after swap
            assert_eq!(
                usdt_balance_after_swap.balance,
                Uint128::from(
                    MOCK_1_000_000_000_USDT - 1 * DECIMAL_18 - 200_000u128 * DECIMAL_18
                        + 1_000_053_000_000_000_000u128
                        - 9 * DECIMAL_18
                ), // 9 USDT transferred to the stable pair
            );

            // Query BUSD Balance of ADMIN after swap
            let busd_balance_after_swap: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    busd_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert BUSD Balance of ADMIN after swap
            assert_eq!(
                busd_balance_after_swap.balance,
                Uint128::from(
                    MOCK_1_000_000_000_BUSD - 1 * DECIMAL_18 - 200_000u128 * DECIMAL_18
                        + 8_999_999_000_000_000_000u128
                ), // 9 BUSD received from the stable pair
            );

            // ADMIN swap 100 BUSD to USDC
            let swap_msg = StablePairExecuteMsg::StableSwap {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: busd_token_contract.clone(),
                    },
                    amount: Uint128::from(100u128 * DECIMAL_18),
                },
                ask_asset: AssetInfo::Token {
                    contract_addr: usdc_token_contract.clone(),
                },
                max_spread: None,
                belief_price: None,
                to: None,
            };

            // Send 100 BUSD to stable pair contract to swap
            let send_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Send {
                contract: "contract5".to_string(),
                amount: Uint128::from(100u128 * DECIMAL_18),
                msg: to_binary(&swap_msg).unwrap(),
            };

            // Execute send
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(busd_token_contract.clone()),
                &send_msg,
                &[],
            );

            assert!(response.is_ok());

            // Query BUSD Balance of ADMIN after swap
            let busd_balance_after_swap: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    busd_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert BUSD Balance of ADMIN after swap
            assert_eq!(
                busd_balance_after_swap.balance,
                Uint128::from(
                    MOCK_1_000_000_000_BUSD - 1 * DECIMAL_18 - 200_000u128 * DECIMAL_18
                        + 8_999_999_000_000_000_000u128
                        - 100 * DECIMAL_18
                ), // 100 BUSD transferred to the stable pair
            );

            // Query USDC Balance of ADMIN after swap
            let usdc_balance_after_swap: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdc_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert USDC Balance of ADMIN after swap
            assert_eq!(
                usdc_balance_after_swap.balance,
                Uint128::from(
                    MOCK_1_000_000_000_USDC
                        - 1 * DECIMAL_18
                        - 100_000u128 * DECIMAL_18
                        - 1 * DECIMAL_18
                        + 99_994_630_000_000_000_000u128
                ), // 100 USDC received from the stable pair
            );

            // ADMIN swap 50_000 USDC to USDT
            let swap_msg = StablePairExecuteMsg::StableSwap {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: usdc_token_contract.clone(),
                    },
                    amount: Uint128::from(50_000u128 * DECIMAL_18),
                },
                ask_asset: AssetInfo::Token {
                    contract_addr: usdt_token_contract.clone(),
                },
                max_spread: None,
                belief_price: None,
                to: None,
            };

            // Send 50_000 USDC to stable pair contract to swap
            let send_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Send {
                contract: "contract5".to_string(),
                amount: Uint128::from(50_000u128 * DECIMAL_18),
                msg: to_binary(&swap_msg).unwrap(),
            };

            // Execute send
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(usdc_token_contract.clone()),
                &send_msg,
                &[],
            );

            assert!(response.is_ok());

            // Query USDC Balance of ADMIN after swap
            let usdc_balance_after_swap: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdc_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert USDC Balance of ADMIN after swap
            assert_eq!(
                usdc_balance_after_swap.balance,
                Uint128::from(
                    MOCK_1_000_000_000_USDC
                        - 1 * DECIMAL_18
                        - 100_000u128 * DECIMAL_18
                        - 1 * DECIMAL_18
                        + 99_994_630_000_000_000_000u128
                        - 50_000 * DECIMAL_18
                ), // 50_000 USDC transferred to the stable pair
            );

            // Query USDT Balance of ADMIN after swap
            let usdt_balance_after_swap: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdt_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert USDT Balance of ADMIN after swap
            assert_eq!(
                usdt_balance_after_swap.balance,
                Uint128::from(
                    MOCK_1_000_000_000_USDT - 1 * DECIMAL_18 - 200_000u128 * DECIMAL_18
                        + 1_000_053_000_000_000_000u128
                        - 9 * DECIMAL_18
                        + 50_000_001_866_000_000_000_000u128
                ), // 50_000 USDT received from the stable pair
            );

            // provide liquidity to the pair one more time
            let provide_liquidity_msg = StablePairExecuteMsg::ProvideLiquidity {
                assets: vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(100_000_000u128 * DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(150_000_000u128 * DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000_000u128 * DECIMAL_18),
                    },
                ],
                slippage_tolerance: None,
                receiver: None,
            };

            // Execute provide liquidity
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract5".to_string()),
                &provide_liquidity_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            assert!(response.is_ok());

            // ADMIN swap 10_000_000 USDC to BUSD
            let swap_msg = StablePairExecuteMsg::StableSwap {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: usdc_token_contract.clone(),
                    },
                    amount: Uint128::from(10_000_000u128 * DECIMAL_18),
                },
                ask_asset: AssetInfo::Token {
                    contract_addr: busd_token_contract.clone(),
                },
                max_spread: None,
                belief_price: None,
                to: None,
            };

            // Send 10_000_000 USDC to stable pair contract to swap
            let send_msg: Cw20ExecuteMsg = Cw20ExecuteMsg::Send {
                contract: "contract5".to_string(),
                amount: Uint128::from(10_000_000u128 * DECIMAL_18),
                msg: to_binary(&swap_msg).unwrap(),
            };

            // Execute send
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(usdc_token_contract.clone()),
                &send_msg,
                &[],
            );

            assert!(response.is_ok());

            // Query USDC Balance of ADMIN after swap
            let usdc_balance_after_swap: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdc_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert USDC Balance of ADMIN after swap
            assert_eq!(
                usdc_balance_after_swap.balance,
                Uint128::from(
                    MOCK_1_000_000_000_USDC
                        - 1 * DECIMAL_18
                        - 100_000u128 * DECIMAL_18
                        - 1 * DECIMAL_18
                        + 99_994_630_000_000_000_000u128
                        - 50_000 * DECIMAL_18
                        - 100_000_000u128 * DECIMAL_18
                        - 10_000_000 * DECIMAL_18
                ), // 100_000_000 USDC transferred to the stable pair
            );

            // Query BUSD Balance of ADMIN after swap
            let busd_balance_after_swap: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    busd_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert BUSD Balance of ADMIN after swap
            assert_eq!(
                busd_balance_after_swap.balance,
                Uint128::from(
                    MOCK_1_000_000_000_BUSD - 1 * DECIMAL_18 - 200_000u128 * DECIMAL_18
                        + 8_999_999_000_000_000_000u128
                        - 100 * DECIMAL_18
                        - 200_000_000u128 * DECIMAL_18
                        + 10_000_326_281_165_000_000_000_000u128
                ), // 10_000_000 BUSD received from the stable pair
            );
        }
    }
}
