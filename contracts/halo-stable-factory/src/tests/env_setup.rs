#[cfg(test)]
pub mod env {
    use cosmwasm_std::{Addr, Coin, Empty, Uint128};
    use cw20::MinterResponse;
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use crate::contract::{
        execute as HaloStableFactoryExecute, instantiate as HaloStableFactoryInstantiate,
        query as HaloStableFactoryQuery, reply as HaloStableFactoryReply,
    };

    use halo_stable_pair::contract::{
        execute as HaloStablePairExecute, instantiate as HaloStablePairInstantiate,
        query as HaloStablePairQuery, reply as HaloStablePairReply,
    };

    use cw20_base::contract::{
        execute as HaloTokenExecute, instantiate as HaloTokenInstantiate, query as HaloTokenQuery,
    };

    use crate::msg::{
        ExecuteMsg as HaloStableFactoryExecuteMsg,
        InstantiateMsg as HaloStableFactoryInstantiateMsg,
    };
    use halo_stable_pair::msg::{
        ExecuteMsg as HaloStablePairExecuteMsg, InstantiateMsg as HaloStablePairInstantiateMsg,
    };
    use haloswap::token::InstantiateMsg as HaloTokenInstantiateMsg;

    // ****************************************
    // You MUST define the constants value here
    // ****************************************
    pub const ADMIN: &str = "aura1uh24g2lc8hvvkaaf7awz25lrh5fptthu2dhq0n";
    pub const USER_1: &str = "aura1fqj2redmssckrdeekhkcvd2kzp9f4nks4fctrt";

    pub const NATIVE_DENOM: &str = "uaura";
    pub const NATIVE_BALANCE: u128 = 1_000_000_000_000u128;

    pub const NATIVE_DENOM_2: &str = "utaura";
    pub const NATIVE_BALANCE_2: u128 = 500_000_000_000u128;

    pub const HALO_TOKEN_SYMBOL: &str = "HALO";
    pub const HALO_TOKEN_NAME: &str = "Halo Token";
    pub const HALO_TOKEN_DECIMALS: u8 = 18;
    pub const HALO_TOKEN_INITIAL_SUPPLY: u128 = 1_000_000_000_000_000_000u128;

    pub struct ContractInfo {
        pub contract_addr: String,
        pub contract_code_id: u64,
    }

    fn mock_app() -> App {
        AppBuilder::new().build(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(ADMIN),
                    vec![
                        Coin {
                            denom: NATIVE_DENOM.to_string(),
                            amount: Uint128::new(NATIVE_BALANCE),
                        },
                        Coin {
                            denom: NATIVE_DENOM_2.to_string(),
                            amount: Uint128::new(NATIVE_BALANCE_2),
                        },
                    ],
                )
                .unwrap();
        })
    }

    fn halo_stable_factory_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            HaloStableFactoryExecute,
            HaloStableFactoryInstantiate,
            HaloStableFactoryQuery,
        )
        .with_reply(HaloStableFactoryReply);
        Box::new(contract)
    }

    fn halo_stable_pair_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            HaloStablePairExecute,
            HaloStablePairInstantiate,
            HaloStablePairQuery,
        )
        .with_reply(HaloStablePairReply);
        Box::new(contract)
    }

    fn halo_token_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(HaloTokenExecute, HaloTokenInstantiate, HaloTokenQuery);
        Box::new(contract)
    }

    pub fn instantiate_contracts() -> (App, Vec<ContractInfo>) {
        // Create a new app instance
        let mut app = mock_app();
        // Create a vector to store all contract info
        let mut contracts: Vec<ContractInfo> = Vec::new();

        // store code of all contracts to the app and get the code ids
        let halo_stable_factory_code_id = app.store_code(halo_stable_factory_template());
        let halo_stable_pair_code_id = app.store_code(halo_stable_pair_template());
        let halo_token_contract_code_id = app.store_code(halo_token_contract_template());

        // halo stable factory contract
        // create instantiate msg for contract
        let halo_stable_factory_instantiate_msg = HaloStableFactoryInstantiateMsg {
            stable_pair_code_id: halo_stable_pair_code_id,
            token_code_id: halo_token_contract_code_id,
        };

        // instantiate the contract
        let halo_stable_factory_contract_addr = app
            .instantiate_contract(
                halo_stable_factory_code_id,
                Addr::unchecked(ADMIN),
                &halo_stable_factory_instantiate_msg,
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contracts.push(ContractInfo {
            contract_addr: halo_stable_factory_contract_addr.to_string(),
            contract_code_id: halo_stable_factory_code_id,
        });

        // cw20 token contract
        // create instantiate msg for contract
        // create instantiate message for contract
        let halo_token_contract_instantiate_msg = HaloTokenInstantiateMsg {
            name: HALO_TOKEN_NAME.to_string(),
            symbol: HALO_TOKEN_SYMBOL.to_string(),
            decimals: 18,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: ADMIN.to_string(), // the minter of the cw20 token must be the marketplace contract
                cap: Some(Uint128::new(HALO_TOKEN_INITIAL_SUPPLY)),
            }),
        };

        // instantiate contract
        let halo_token_contract_addr = app
            .instantiate_contract(
                halo_token_contract_code_id,
                Addr::unchecked(ADMIN),
                &halo_token_contract_instantiate_msg,
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contracts.push(ContractInfo {
            contract_addr: halo_token_contract_addr.to_string(),
            contract_code_id: halo_token_contract_code_id,
        });

        // cw20 USDC token contract
        // create instantiate msg for contract
        // create instantiate message for contract
        let usdc_token_contract_instantiate_msg = HaloTokenInstantiateMsg {
            name: "USDC".to_string(),
            symbol: "USDC".to_string(),
            decimals: 18,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: ADMIN.to_string(), // the minter of the cw20 token must be the marketplace contract
                cap: None,
            }),
        };

        // instantiate contract
        let usdc_token_contract_addr = app
            .instantiate_contract(
                halo_token_contract_code_id,
                Addr::unchecked(ADMIN),
                &usdc_token_contract_instantiate_msg,
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contracts.push(ContractInfo {
            contract_addr: usdc_token_contract_addr.to_string(),
            contract_code_id: halo_token_contract_code_id,
        });

        // cw20 USDT token contract
        // create instantiate msg for contract
        // create instantiate message for contract
        let usdt_token_contract_instantiate_msg = HaloTokenInstantiateMsg {
            name: "USDT".to_string(),
            symbol: "USDT".to_string(),
            decimals: 18,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: ADMIN.to_string(), // the minter of the cw20 token must be the marketplace contract
                cap: None,
            }),
        };

        // instantiate contract
        let usdt_token_contract_addr = app
            .instantiate_contract(
                halo_token_contract_code_id,
                Addr::unchecked(ADMIN),
                &usdt_token_contract_instantiate_msg,
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contracts.push(ContractInfo {
            contract_addr: usdt_token_contract_addr.to_string(),
            contract_code_id: halo_token_contract_code_id,
        });

        // cw20 BUSD token contract
        // create instantiate msg for contract
        // create instantiate message for contract
        let busd_token_contract_instantiate_msg = HaloTokenInstantiateMsg {
            name: "BUSD".to_string(),
            symbol: "BUSD".to_string(),
            decimals: 18,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: ADMIN.to_string(), // the minter of the cw20 token must be the marketplace contract
                cap: None,
            }),
        };

        // instantiate contract
        let busd_token_contract_addr = app
            .instantiate_contract(
                halo_token_contract_code_id,
                Addr::unchecked(ADMIN),
                &busd_token_contract_instantiate_msg,
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contracts.push(ContractInfo {
            contract_addr: busd_token_contract_addr.to_string(),
            contract_code_id: halo_token_contract_code_id,
        });

        // return the app and contracts
        (app, contracts)
    }
}
