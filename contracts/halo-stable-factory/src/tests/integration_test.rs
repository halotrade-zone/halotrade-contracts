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
    const ONE_UNIT_OF_DECIMAL_18: u128 = 1_000_000_000_000_000_000u128;
    // Decimal 6 macro
    const ONE_UNIT_OF_DECIMAL_6: u128 = 1_000_000u128;

    const MOCK_TRANSACTION_FEE: u128 = 5000;
    mod execute_contract_native_with_cw20_token {

        use std::str::FromStr;

        use crate::msg::{
            ExecuteMsg as StableFactoryExecuteMsg, QueryMsg as StableFactoryQueryMsg,
        };
        use crate::tests::env_setup::env::NATIVE_DENOM;
        use bignumber::Decimal256;
        use cosmwasm_std::{to_binary, Addr, Coin, Uint128};
        use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
        use cw_multi_test::Executor;
        use halo_stable_pair::msg::{
            ExecuteMsg as StablePairExecuteMsg, QueryMsg as StablePairQueryMsg,
        };
        use halo_stable_pair::state::StablePairsResponse;
        use halo_stable_pair::{
            math::AmpFactor,
            msg::Cw20StableHookMsg,
            state::{CreateStablePairRequirements, StablePairInfo},
        };
        use haloswap::asset::{Asset, AssetInfo, LPTokenInfo};
        use haloswap::pair::SimulationResponse;

        use super::*;
        // Create a stable swap pair with 3 tokens USDC, USDT, BUSD
        // Provide liquidity to the pair (1 USDC, 1 USDT, 1 BUSD)
        // Provide liquidity to the pair one more time (100_000 USDC, 200_000 USDT, 200_000 BUSD)
        // Withdraw liquidity by Share from the pair by 50% of Share (250_000 LP Token)
        // -> ADMIN should get (50_000.5 USDC, 100_000.5 USDT, 100_000.5 BUSD)
        // Withdraw liquidity by Token from the pair by 25_000 USDC, 50_000 USDT, 50_000 BUSD
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
                spender: response.contract_addr,
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

            // provide liquidity to the pair
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
                        amount: Uint128::from(100_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000u128 * ONE_UNIT_OF_DECIMAL_18),
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
            let withdraw_liquidity_msg = Cw20StableHookMsg::WithdrawLiquidityByShare {
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
                        MOCK_1_000_000_000_USDC
                            - ONE_UNIT_OF_DECIMAL_18
                            - 100_000u128 * ONE_UNIT_OF_DECIMAL_18
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
                        MOCK_1_000_000_000_USDT
                            - ONE_UNIT_OF_DECIMAL_18
                            - 200_000u128 * ONE_UNIT_OF_DECIMAL_18
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
                        MOCK_1_000_000_000_BUSD
                            - ONE_UNIT_OF_DECIMAL_18
                            - 200_000u128 * ONE_UNIT_OF_DECIMAL_18
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
            let withdraw_liquidity_msg = StablePairExecuteMsg::WithdrawLiquidityByToken {
                assets: vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(25_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(50_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(50_000u128 * ONE_UNIT_OF_DECIMAL_18),
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
                        MOCK_1_000_000_000_USDC
                            - ONE_UNIT_OF_DECIMAL_18
                            - 100_000u128 * ONE_UNIT_OF_DECIMAL_18
                            + 50_000_499_999_799_998_617_057u128
                            + 25_000u128 * ONE_UNIT_OF_DECIMAL_18
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
                        MOCK_1_000_000_000_USDT
                            - ONE_UNIT_OF_DECIMAL_18
                            - 200_000u128 * ONE_UNIT_OF_DECIMAL_18
                            + 100_000_499_999_599_999_234_109u128
                            + 50_000 * ONE_UNIT_OF_DECIMAL_18
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
                        MOCK_1_000_000_000_BUSD
                            - ONE_UNIT_OF_DECIMAL_18
                            - 200_000u128 * ONE_UNIT_OF_DECIMAL_18
                            + 100_000_499_999_599_999_234_109u128
                            + 50_000 * ONE_UNIT_OF_DECIMAL_18
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
                spender: response.contract_addr,
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

            // provide liquidity to the pair
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
                        amount: Uint128::from(100_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000u128 * ONE_UNIT_OF_DECIMAL_18),
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
                Uint128::from(
                    MOCK_1_000_000_000_USDC
                        - ONE_UNIT_OF_DECIMAL_18
                        - 100_000u128 * ONE_UNIT_OF_DECIMAL_18
                ),
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
                Uint128::from(
                    MOCK_1_000_000_000_USDT
                        - ONE_UNIT_OF_DECIMAL_18
                        - 200_000u128 * ONE_UNIT_OF_DECIMAL_18
                ),
            );

            // ADMIN swap 1 USDC to USDT
            let swap_msg = StablePairExecuteMsg::StableSwap {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: usdc_token_contract.clone(),
                    },
                    amount: Uint128::from(ONE_UNIT_OF_DECIMAL_18),
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
                amount: Uint128::from(ONE_UNIT_OF_DECIMAL_18),
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
                        - ONE_UNIT_OF_DECIMAL_18
                        - 100_000u128 * ONE_UNIT_OF_DECIMAL_18
                        - ONE_UNIT_OF_DECIMAL_18
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
                    MOCK_1_000_000_000_USDT
                        - ONE_UNIT_OF_DECIMAL_18
                        - 200_000u128 * ONE_UNIT_OF_DECIMAL_18
                        + 1_000_053_000_000_000_000u128
                ), // 1 USDT received from the stable pair
            );

            // ADMIN swap 9 USDT to BUSD
            let swap_msg = StablePairExecuteMsg::StableSwap {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: usdt_token_contract.clone(),
                    },
                    amount: Uint128::from(9u128 * ONE_UNIT_OF_DECIMAL_18),
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
                amount: Uint128::from(9u128 * ONE_UNIT_OF_DECIMAL_18),
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
                    MOCK_1_000_000_000_USDT
                        - ONE_UNIT_OF_DECIMAL_18
                        - 200_000u128 * ONE_UNIT_OF_DECIMAL_18
                        + 1_000_053_000_000_000_000u128
                        - 9 * ONE_UNIT_OF_DECIMAL_18
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
                    MOCK_1_000_000_000_BUSD
                        - ONE_UNIT_OF_DECIMAL_18
                        - 200_000u128 * ONE_UNIT_OF_DECIMAL_18
                        + 8_999_999_000_000_000_000u128
                ), // 9 BUSD received from the stable pair
            );

            // ADMIN swap 100 BUSD to USDC
            let swap_msg = StablePairExecuteMsg::StableSwap {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: busd_token_contract.clone(),
                    },
                    amount: Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_18),
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
                amount: Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_18),
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
                    MOCK_1_000_000_000_BUSD
                        - ONE_UNIT_OF_DECIMAL_18
                        - 200_000u128 * ONE_UNIT_OF_DECIMAL_18
                        + 8_999_999_000_000_000_000u128
                        - 100 * ONE_UNIT_OF_DECIMAL_18
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
                        - ONE_UNIT_OF_DECIMAL_18
                        - 100_000u128 * ONE_UNIT_OF_DECIMAL_18
                        - ONE_UNIT_OF_DECIMAL_18
                        + 99_994_634_000_000_000_000u128
                ), // 100 USDC received from the stable pair
            );

            // ADMIN swap 50_000 USDC to USDT
            let swap_msg = StablePairExecuteMsg::StableSwap {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: usdc_token_contract.clone(),
                    },
                    amount: Uint128::from(50_000u128 * ONE_UNIT_OF_DECIMAL_18),
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
                amount: Uint128::from(50_000u128 * ONE_UNIT_OF_DECIMAL_18),
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
                        - ONE_UNIT_OF_DECIMAL_18
                        - 100_000u128 * ONE_UNIT_OF_DECIMAL_18
                        - ONE_UNIT_OF_DECIMAL_18
                        + 99_994_634_000_000_000_000u128
                        - 50_000 * ONE_UNIT_OF_DECIMAL_18
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
                    MOCK_1_000_000_000_USDT
                        - ONE_UNIT_OF_DECIMAL_18
                        - 200_000u128 * ONE_UNIT_OF_DECIMAL_18
                        + 1_000_053_000_000_000_000u128
                        - 9 * ONE_UNIT_OF_DECIMAL_18
                        + 50_001_194_511_000_000_000_000u128
                ), // 50_000 USDT received from the stable pair
            );

            // provide liquidity to the pair one more time
            let provide_liquidity_msg = StablePairExecuteMsg::ProvideLiquidity {
                assets: vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(100_000_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(150_000_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000_000u128 * ONE_UNIT_OF_DECIMAL_18),
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
                    amount: Uint128::from(10_000_000u128 * ONE_UNIT_OF_DECIMAL_18),
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
                amount: Uint128::from(10_000_000u128 * ONE_UNIT_OF_DECIMAL_18),
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
                        - ONE_UNIT_OF_DECIMAL_18
                        - 100_000u128 * ONE_UNIT_OF_DECIMAL_18
                        - ONE_UNIT_OF_DECIMAL_18
                        + 99_994_634_000_000_000_000u128
                        - 50_000 * ONE_UNIT_OF_DECIMAL_18
                        - 100_000_000u128 * ONE_UNIT_OF_DECIMAL_18
                        - 10_000_000 * ONE_UNIT_OF_DECIMAL_18
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
                    MOCK_1_000_000_000_BUSD
                        - ONE_UNIT_OF_DECIMAL_18
                        - 200_000u128 * ONE_UNIT_OF_DECIMAL_18
                        + 8_999_999_000_000_000_000u128
                        - 100 * ONE_UNIT_OF_DECIMAL_18
                        - 200_000_000u128 * ONE_UNIT_OF_DECIMAL_18
                        + 10_000_403_317_233_000_000_000_000u128 // 10_000_000 BUSD received from the stable pair
                ),
            );
        }

        // Create a stable swap pair with 3 tokens USDC, USDT, BUSD
        // Provide liquidity to the pair (1 USDC, 1 USDT, 1 BUSD)
        // Create a stable swap pair with 3 tokens USDC, USDT, DAI
        // Provide liquidity to the stable pair (1 USDC, 1 USDT, 1 HALO)
        // Query ConfigResponse
        // Query StablePairsResponse
        // Query StablePairInfo for (1 USDC, 1 USDT, 1 BUSD) pair

        #[test]
        fn test_query_pairs_msgs() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            // get the stable factory contract
            let stable_factory_contract = &contracts[0].contract_addr.clone();
            // get the HALO token contract
            let halo_token_contract = &contracts[1].contract_addr.clone();
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

            // create stable pair USDC, USDT, HALO
            let asset_infos = vec![
                AssetInfo::Token {
                    contract_addr: usdc_token_contract.clone(),
                },
                AssetInfo::Token {
                    contract_addr: usdt_token_contract.clone(),
                },
                AssetInfo::Token {
                    contract_addr: halo_token_contract.clone(),
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
                    lp_token_name: "Stable-Halo-LP-Token".to_string(),
                    lp_token_symbol: "SLP-HALO".to_string(),
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
                                contract_addr: halo_token_contract.clone(),
                            },
                        ],
                    },
                )
                .unwrap();

            // Assert stable pair info
            assert_eq!(
                response,
                StablePairInfo {
                    contract_addr: "contract7".to_string(),
                    liquidity_token: "contract8".to_string(),
                    asset_infos: vec![
                        AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        AssetInfo::Token {
                            contract_addr: halo_token_contract.clone(),
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

            // query stable pairs
            let response: StablePairsResponse = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(stable_factory_contract.clone()),
                    &StableFactoryQueryMsg::StablePairs {
                        start_after: None,
                        limit: None,
                    },
                )
                .unwrap();

            // Assert stable pairs
            assert_eq!(
                response,
                StablePairsResponse {
                    pairs: vec![
                        StablePairInfo {
                            contract_addr: "contract7".to_string(),
                            liquidity_token: "contract8".to_string(),
                            asset_infos: vec![
                                AssetInfo::Token {
                                    contract_addr: usdc_token_contract.clone(),
                                },
                                AssetInfo::Token {
                                    contract_addr: usdt_token_contract.clone(),
                                },
                                AssetInfo::Token {
                                    contract_addr: halo_token_contract.clone(),
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
                        },
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
                        },
                    ],
                }
            );
        }

        // Create a stable swap pair with 3 tokens USDC, USDT, BUSD
        // ADMIN query ProvideLiquiditySimulation for (100_000 USDC, 200_000 USDT, 200_000 BUSD)
        // Provide liquidity to the pair (100_000 USDC, 200_000 USDT, 200_000 BUSD)
        // -> The LP token amount should be equal to the simulation result (500_000 LP tokens)
        // ADMIN query StableSimulation for 100 USDC to USDT
        // ADMIN swap 100 USDC to USDT
        // -> The result should be equal to the StableSimulation result (100 USDT)
        // ADMIN query WithdrawLiquidityByShareSimulation for 1000 LP tokens
        // ADMIN remove 100_000 LP tokens
        // -> The result should be equal to the WithdrawLiquidityByShareSimulation result
        // ADMIN query WithdrawLiquidityByTokenSimulation for (100 USDC, 200 USDT, 200 BUSD)
        // ADMIN remove 100 USDC, 200 USDT, 200 BUSD
        // -> The result should be equal to the WithdrawLiquidityByTokenSimulation result
        #[test]
        fn test_query_simulation() {
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
                spender: response.contract_addr,
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

            // ADMIN query ProvideLiquiditySimulation for (100_000 USDC, 200_000 USDT, 200_000 BUSD)
            let provide_liquidity_simulation_msg = StablePairQueryMsg::ProvideLiquiditySimulation {
                assets: vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(100_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                ],
            };

            // Execute query ProvideLiquiditySimulation
            let response_provide_500_000_usd: Uint128 = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked("contract5".to_string()),
                    &provide_liquidity_simulation_msg,
                )
                .unwrap();

            // assert ProvideLiquiditySimulation result
            assert_eq!(
                response_provide_500_000_usd,
                Uint128::from(499_998_542_620u128)
            );

            // provide liquidity to the pair
            let provide_liquidity_msg = StablePairExecuteMsg::ProvideLiquidity {
                assets: vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(100_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(200_000u128 * ONE_UNIT_OF_DECIMAL_18),
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

            // Query LP token balance of ADMIN
            let lp_token_balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    "contract6".to_string(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert LP token balance of ADMIN
            assert_eq!(lp_token_balance.balance, response_provide_500_000_usd);

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
                Uint128::from(MOCK_1_000_000_000_USDC - 100_000u128 * ONE_UNIT_OF_DECIMAL_18),
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
                Uint128::from(MOCK_1_000_000_000_USDT - 200_000u128 * ONE_UNIT_OF_DECIMAL_18),
            );

            // Query BUSD Balance of ADMIN before swap
            let busd_balance_before_swap: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    busd_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert BUSD Balance of ADMIN before swap
            assert_eq!(
                busd_balance_before_swap.balance,
                Uint128::from(MOCK_1_000_000_000_BUSD - 200_000u128 * ONE_UNIT_OF_DECIMAL_18),
            );

            // ADMIN query StableSimulation for 100 USDC to USDT
            let stable_simulation_msg = StablePairQueryMsg::StableSimulation {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: usdc_token_contract.clone(),
                    },
                    amount: Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_18),
                },
                ask_asset: AssetInfo::Token {
                    contract_addr: usdt_token_contract.clone(),
                },
            };

            // Execute query StableSimulation
            let response_stable_simulation_100_usd: SimulationResponse = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked("contract5".to_string()),
                    &stable_simulation_msg,
                )
                .unwrap();

            // assert StableSimulation result
            assert_eq!(
                response_stable_simulation_100_usd.return_amount,
                Uint128::from(100_005_349_000_000_000_000u128)
            );

            // ADMIN swap 100 USDC to USDT
            let swap_msg = StablePairExecuteMsg::StableSwap {
                offer_asset: Asset {
                    info: AssetInfo::Token {
                        contract_addr: usdc_token_contract.clone(),
                    },
                    amount: Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_18),
                },
                ask_asset: AssetInfo::Token {
                    contract_addr: usdt_token_contract.clone(),
                },
                max_spread: None,
                belief_price: None,
                to: None,
            };

            // Send 100 USDC to stable pair contract to swap
            let send_msg = Cw20ExecuteMsg::Send {
                contract: "contract5".to_string(),
                amount: Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_18),
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
                usdc_balance_before_swap.balance - Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_18),
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
                usdt_balance_before_swap.balance + response_stable_simulation_100_usd.return_amount,
            );

            // ADMIN query WithdrawLiquidityByShareSimulation for 1000 LP tokens
            let withdraw_liquidity_by_share_simulation_msg =
                StablePairQueryMsg::WithdrawLiquidityByShareSimulation {
                    share: Uint128::from(100_000u128 * ONE_UNIT_OF_DECIMAL_6),
                };

            // Execute query WithdrawLiquidityByShareSimulation
            let response_withdraw_liquidity_by_share_simulation_1000_lp: Vec<Asset> = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked("contract5".to_string()),
                    &withdraw_liquidity_by_share_simulation_msg,
                )
                .unwrap();

            // assert WithdrawLiquidityByShareSimulation result
            assert_eq!(
                response_withdraw_liquidity_by_share_simulation_1000_lp,
                vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(20_020_058_353_625_246_696_016u128), // 20_000 USDC
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(39_980_115_462_561_385_424_498u128), // 40_000 USDT
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(40_000_116_590_659_833_558_475u128), // 40_000 BUSD
                    },
                ]
            );

            // ADMIN remove 100_000 LP tokens
            let withdraw_liquidity_by_share_msg = Cw20StableHookMsg::WithdrawLiquidityByShare {
                share: Uint128::from(100_000u128 * ONE_UNIT_OF_DECIMAL_6),
                assets_min_amount: None,
            };

            // Send 100_000 LP tokens to stable pair contract to withdraw liquidity
            let send_msg = Cw20ExecuteMsg::Send {
                contract: "contract5".to_string(),
                amount: Uint128::from(100_000u128 * ONE_UNIT_OF_DECIMAL_6),
                msg: to_binary(&withdraw_liquidity_by_share_msg).unwrap(),
            };

            // Execute send
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract6".to_string()),
                &send_msg,
                &[],
            );

            assert!(response.is_ok());

            // Query USDC Balance of ADMIN after withdraw liquidity
            let usdc_balance_after_withdraw_liquidity: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdc_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert USDC Balance of ADMIN after withdraw liquidity
            assert_eq!(
                usdc_balance_after_withdraw_liquidity.balance,
                usdc_balance_before_swap.balance - Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_18)
                    + response_withdraw_liquidity_by_share_simulation_1000_lp[0].amount,
            );

            // Query USDT Balance of ADMIN after withdraw liquidity
            let usdt_balance_after_withdraw_liquidity: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdt_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert USDT Balance of ADMIN after withdraw liquidity
            assert_eq!(
                usdt_balance_after_withdraw_liquidity.balance,
                usdt_balance_after_swap.balance
                    + response_withdraw_liquidity_by_share_simulation_1000_lp[1].amount,
            );

            // Query BUSD Balance of ADMIN after withdraw liquidity
            let busd_balance_after_withdraw_liquidity: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    busd_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert BUSD Balance of ADMIN after withdraw liquidity
            assert_eq!(
                busd_balance_after_withdraw_liquidity.balance,
                busd_balance_before_swap.balance
                    + response_withdraw_liquidity_by_share_simulation_1000_lp[2].amount,
            );

            // ADMIN query WithdrawLiquidityByTokenSimulation for (100 USDC, 200 USDT, 200 BUSD)
            let withdraw_liquidity_by_token_simulation_msg =
                StablePairQueryMsg::WithdrawLiquidityByTokenSimulation {
                    assets: vec![
                        Asset {
                            info: AssetInfo::Token {
                                contract_addr: usdc_token_contract.clone(),
                            },
                            amount: Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_18),
                        },
                        Asset {
                            info: AssetInfo::Token {
                                contract_addr: usdt_token_contract.clone(),
                            },
                            amount: Uint128::from(200u128 * ONE_UNIT_OF_DECIMAL_18),
                        },
                        Asset {
                            info: AssetInfo::Token {
                                contract_addr: busd_token_contract.clone(),
                            },
                            amount: Uint128::from(200u128 * ONE_UNIT_OF_DECIMAL_18),
                        },
                    ],
                };

            // Execute query WithdrawLiquidityByTokenSimulation
            let response_withdraw_liquidity_by_token_simulation_100_usd_200_usdt_200_busd: Uint128 =
                app.wrap()
                    .query_wasm_smart(
                        Addr::unchecked("contract5".to_string()),
                        &withdraw_liquidity_by_token_simulation_msg,
                    )
                    .unwrap();

            // assert WithdrawLiquidityByTokenSimulation result
            assert_eq!(
                response_withdraw_liquidity_by_token_simulation_100_usd_200_usdt_200_busd,
                Uint128::from(499_998_542u128)
            );

            // Increase allowance LP token contract for stable pair contract
            let increase_allowance_msg = Cw20ExecuteMsg::IncreaseAllowance {
                spender: "contract5".to_string(),
                amount: Uint128::from(100_000u128 * ONE_UNIT_OF_DECIMAL_6),
                expires: None,
            };

            // Execute increase allowance for LP token contract
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract6".to_string()),
                &increase_allowance_msg,
                &[],
            );

            assert!(response.is_ok());
            // ADMIN remove 100 USDC, 200 USDT, 200 BUSD
            let withdraw_liquidity_by_token_msg = StablePairExecuteMsg::WithdrawLiquidityByToken {
                assets: vec![
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdc_token_contract.clone(),
                        },
                        amount: Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        amount: Uint128::from(200u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                    Asset {
                        info: AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                        amount: Uint128::from(200u128 * ONE_UNIT_OF_DECIMAL_18),
                    },
                ],
                max_burn_share: None,
            };

            // Execute withdraw liquidity
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked("contract5".to_string()),
                &withdraw_liquidity_by_token_msg,
                &[],
            );

            assert!(response.is_ok());

            // Query USDC Balance of ADMIN after withdraw liquidity
            let usdc_balance_after_withdraw_liquidity: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdc_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert USDC Balance of ADMIN after withdraw liquidity
            assert_eq!(
                usdc_balance_after_withdraw_liquidity.balance,
                usdc_balance_before_swap.balance - Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_18)
                    + response_withdraw_liquidity_by_share_simulation_1000_lp[0].amount
                    + Uint128::from(100u128 * ONE_UNIT_OF_DECIMAL_18),
            );

            // Query USDT Balance of ADMIN after withdraw liquidity
            let usdt_balance_after_withdraw_liquidity: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    usdt_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert USDT Balance of ADMIN after withdraw liquidity
            assert_eq!(
                usdt_balance_after_withdraw_liquidity.balance,
                usdt_balance_after_swap.balance
                    + response_withdraw_liquidity_by_share_simulation_1000_lp[1].amount
                    + Uint128::from(200u128 * ONE_UNIT_OF_DECIMAL_18),
            );

            // Query BUSD Balance of ADMIN after withdraw liquidity
            let busd_balance_after_withdraw_liquidity: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    busd_token_contract.clone(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert BUSD Balance of ADMIN after withdraw liquidity
            assert_eq!(
                busd_balance_after_withdraw_liquidity.balance,
                busd_balance_before_swap.balance
                    + response_withdraw_liquidity_by_share_simulation_1000_lp[2].amount
                    + Uint128::from(200u128 * ONE_UNIT_OF_DECIMAL_18),
            );

            // Query LP token balance of ADMIN after withdraw liquidity
            let lp_token_balance: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    "contract6".to_string(),
                    &Cw20QueryMsg::Balance {
                        address: ADMIN.to_string(),
                    },
                )
                .unwrap();

            // assert LP token balance of ADMIN after withdraw liquidity
            assert_eq!(
                lp_token_balance.balance,
                response_provide_500_000_usd
                    - response_withdraw_liquidity_by_token_simulation_100_usd_200_usdt_200_busd
                    - Uint128::from(100_000u128 * ONE_UNIT_OF_DECIMAL_6)
            );
        }

        // Create a stable swap pair with 2 tokens NATIVE_DENOM, USDT
        // Create a stable swap pair with 3 tokens NATIVE_DENOM, USDT, BUSD
        // Query stable pair info and assert token decimals of NATIVE_DENOM is 6
        // Update native token decimals for stable pairs to 9
        // Query stable pair info and assert token decimals of NATIVE_DENOM is 9
        #[test]
        fn update_native_token_decimals_for_stable_pairs() {
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

            // add native token denom to stable factory contract
            let add_native_token_denom_msg = StableFactoryExecuteMsg::AddNativeTokenDecimals {
                denom: NATIVE_DENOM.to_string(),
                decimals: 6,
            };

            // Execute add native token denom
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(stable_factory_contract.clone()),
                &add_native_token_denom_msg,
                &[
                    Coin {
                        amount: Uint128::from(MOCK_TRANSACTION_FEE),
                        denom: NATIVE_DENOM.to_string(),
                    },
                ],
            );

            assert!(response.is_ok());

            // create stable pair NATIVE_DENOM, USDT
            let asset_infos = vec![
                AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                AssetInfo::Token {
                    contract_addr: usdt_token_contract.clone(),
                },
            ];

            // create stable pair msg
            let create_stable_pair_msg = StableFactoryExecuteMsg::CreateStablePair {
                asset_infos,
                requirements: CreateStablePairRequirements {
                    whitelist: vec![Addr::unchecked(ADMIN.to_string())],
                    asset_minimum: vec![Uint128::from(1u128), Uint128::from(1u128)],
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
                    denom: NATIVE_DENOM.to_string(),
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

            // Assert stable pair info
            assert_eq!(
                response,
                StablePairInfo {
                    contract_addr: "contract5".to_string(),
                    liquidity_token: "contract6".to_string(),
                    asset_infos: vec![
                        AssetInfo::NativeToken {
                            denom: NATIVE_DENOM.to_string(),
                        },
                        AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                    ],
                    asset_decimals: vec![6, 18],
                    requirements: CreateStablePairRequirements {
                        whitelist: vec![Addr::unchecked(ADMIN.to_string())],
                        asset_minimum: vec![Uint128::from(1u128), Uint128::from(1u128)],
                    },
                    commission_rate: Decimal256::from_str("0.003").unwrap(),
                }
            );

            // create stable pair NATIVE_DENOM, USDT, BUSD
            let asset_infos = vec![
                AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
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
                    denom: NATIVE_DENOM.to_string(),
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
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM.to_string(),
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
                    contract_addr: "contract7".to_string(),
                    liquidity_token: "contract8".to_string(),
                    asset_infos: vec![
                        AssetInfo::NativeToken {
                            denom: NATIVE_DENOM.to_string(),
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

            // add native token denom to stable factory contract
            let add_native_token_denom_msg = StableFactoryExecuteMsg::AddNativeTokenDecimals {
                denom: NATIVE_DENOM.to_string(),
                decimals: 9,
            };

            // Execute add native token denom
            let response = app.execute_contract(
                Addr::unchecked(ADMIN.to_string()),
                Addr::unchecked(stable_factory_contract.clone()),
                &add_native_token_denom_msg,
                &[
                    Coin {
                        amount: Uint128::from(MOCK_TRANSACTION_FEE),
                        denom: NATIVE_DENOM.to_string(),
                    },
                ],
            );

            assert!(response.is_ok());

            // query stable pair info
            let response: StablePairInfo = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(stable_factory_contract.clone()),
                    &StableFactoryQueryMsg::StablePair {
                        asset_infos: vec![
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

            // Assert stable pair info
            assert_eq!(
                response,
                StablePairInfo {
                    contract_addr: "contract5".to_string(),
                    liquidity_token: "contract6".to_string(),
                    asset_infos: vec![
                        AssetInfo::NativeToken {
                            denom: NATIVE_DENOM.to_string(),
                        },
                        AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                    ],
                    asset_decimals: vec![9, 18],
                    requirements: CreateStablePairRequirements {
                        whitelist: vec![Addr::unchecked(ADMIN.to_string())],
                        asset_minimum: vec![Uint128::from(1u128), Uint128::from(1u128)],
                    },
                    commission_rate: Decimal256::from_str("0.003").unwrap(),
                }
            );

            // query stable pair info
            let response: StablePairInfo = app
                .wrap()
                .query_wasm_smart(
                    Addr::unchecked(stable_factory_contract.clone()),
                    &StableFactoryQueryMsg::StablePair {
                        asset_infos: vec![
                            AssetInfo::NativeToken {
                                denom: NATIVE_DENOM.to_string(),
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
                    contract_addr: "contract7".to_string(),
                    liquidity_token: "contract8".to_string(),
                    asset_infos: vec![
                        AssetInfo::NativeToken {
                            denom: NATIVE_DENOM.to_string(),
                        },
                        AssetInfo::Token {
                            contract_addr: usdt_token_contract.clone(),
                        },
                        AssetInfo::Token {
                            contract_addr: busd_token_contract.clone(),
                        },
                    ],
                    asset_decimals: vec![9, 18, 18],
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


        }
    }
}
