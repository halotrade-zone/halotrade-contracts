#[cfg(test)]
mod tests {
    use crate::tests::env_setup::env::{
        instantiate_contracts, ADMIN, NATIVE_DENOM, NATIVE_DENOM_2, USER_1,
    };
    // Mock 1_000_000_000 USDC token amount
    const MOCK_1_000_000_000_USDC: u128 = 1_000_000_000_000_000_000_000_000_000u128;
    // Mock 1_000_000_000 USDT token amount
    const MOCK_1_000_000_000_USDT: u128 = 1_000_000_000_000_000_000_000_000_000u128;
    // Mock 1_000_000_000 BUSD token amount
    const MOCK_1_000_000_000_BUSD: u128 = 1_000_000_000_000_000_000_000_000_000u128;

    const MOCK_TRANSACTION_FEE: u128 = 5000;
    mod execute_contract_native_with_cw20_token {

        use cosmwasm_std::{Addr, Coin, Uint128};
        use cw20::Cw20ExecuteMsg;
        use cw_multi_test::Executor;
        use halo_stable_pool::{state::CreateStablePoolRequirements, math::AmpFactor};
        use haloswap::asset::{AssetInfo, LPTokenInfo};
        use crate::msg::{ExecuteMsg as FactoryExecuteMsg, QueryMsg as FactoryQueryMsg};

        use super::*;
        // Create a stable swap pool with 3 tokens USDC, USDT, BUSD
        // Add liquidity to the pool
        #[test]
        fn test_add_liquidity_pool_3_tokens() {
            // get integration test app and contracts
            let (mut app, contracts) = instantiate_contracts();
            // get the stable factory contract
            let stable_factory_contract = &contracts[0].contract_addr.clone();
            // get the cw20 token contract
            let cw20_token_contract = &contracts[1].contract_addr.clone();
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
            let create_stable_pool_msg = FactoryExecuteMsg::CreateStablePool {
                asset_infos,
                requirements: CreateStablePoolRequirements {
                    whitelist: vec![Addr::unchecked(ADMIN.to_string())],
                    asset_minimum: vec![Uint128::from(1u128), Uint128::from(1u128), Uint128::from(1u128)],
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
                &create_stable_pool_msg,
                &[Coin {
                    amount: Uint128::from(MOCK_TRANSACTION_FEE),
                    denom: NATIVE_DENOM_2.to_string(),
                }],
            );

            println!("responseeeee: {:?}", response);
            assert!(response.is_ok());

        }
    }
}