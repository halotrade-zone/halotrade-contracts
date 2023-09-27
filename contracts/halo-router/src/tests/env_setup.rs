#[cfg(test)]
pub mod env {
    use cosmwasm_std::{Addr, Coin, Empty, StdError, Uint128};
    use cw20::{Cw20Coin, MinterResponse};
    use cw_multi_test::{App, AppBuilder, Contract, ContractWrapper, Executor};

    use halo_factory::contract::{
        execute as HaloFactoryExecute, instantiate as HaloFactoryInstantiate,
        query as HaloFactoryQuery, reply as HaloFactoryReply,
    };

    use halo_pair::contract::{
        execute as HaloPairExecute, instantiate as HaloPairInstantiate, query as HaloPairQuery,
        reply as HaloPairReply,
    };

    use crate::contract::{
        execute as HaloRouterExecute, instantiate as HaloRouterInstantiate,
        query as HaloRouterQuery,
    };

    use cw20_base::contract::{
        execute as HaloTokenExecute, instantiate as HaloTokenInstantiate, query as HaloTokenQuery,
    };

    use haloswap::factory::InstantiateMsg as HaloFactoryInstantiateMsg;
    use haloswap::router::InstantiateMsg as HaloRouterInstantiateMsg;
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

    pub const MSTR_TOKEN_SYMBOL: &str = "MSTR";
    pub const MSTR_TOKEN_NAME: &str = "MSTR Token";
    pub const MSTR_TOKEN_DECIMALS: u8 = 18;

    pub const USDC_TOKEN_SYMBOL: &str = "USDC";
    pub const USDC_TOKEN_NAME: &str = "USDC Token";
    pub const USDC_TOKEN_DECIMALS: u8 = 18;

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

    fn halo_factory_contract_template() -> Box<dyn Contract<Empty>> {
        let contract =
            ContractWrapper::new(HaloFactoryExecute, HaloFactoryInstantiate, HaloFactoryQuery)
                .with_reply(HaloFactoryReply);
        Box::new(contract)
    }

    fn halo_pair_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(HaloPairExecute, HaloPairInstantiate, HaloPairQuery)
            .with_reply(HaloPairReply);
        Box::new(contract)
    }

    fn halo_router_contract_template() -> Box<dyn Contract<Empty>> {
        let contract =
            ContractWrapper::new(HaloRouterExecute, HaloRouterInstantiate, HaloRouterQuery);
        Box::new(contract)
    }

    fn halo_token_contract_template() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(HaloTokenExecute, HaloTokenInstantiate, HaloTokenQuery);
        Box::new(contract)
    }

    // *********************************************************
    // You MUST store code and instantiate all contracts here
    // Follow the example (2) below:
    // @return App: the mock app
    // @return String: the address of the contract
    // @return u64: the code id of the contract
    //    pub fn instantiate_contracts() -> (App, String, u64) {
    //        // Create a new app instance
    //        let mut app = mock_app();
    //
    //        // store the code of all contracts to the app and get the code ids
    //        let contract_code_id = app.store_code(contract_template());
    //
    //        // create instantiate message for contract
    //        let contract_instantiate_msg = InstantiateMsg {
    //            name: "Contract_A".to_string(),
    //        };
    //
    //        // instantiate contract
    //        let contract_addr = app
    //            .instantiate_contract(
    //                contract_code_id,
    //                Addr::unchecked(ADMIN),
    //                &contract_instantiate_msg,
    //                &[],
    //                "test instantiate contract",
    //                None,
    //            )
    //            .unwrap();
    //
    //        // return the app instance, the addresses and code IDs of all contracts
    //        (app, contract_addr, contract_code_id)
    //    }
    // *********************************************************
    pub fn instantiate_contracts() -> (App, Vec<ContractInfo>) {
        // Create a new app instance
        let mut app = mock_app();
        // Create a vector to store all contract info ([halo factory - [0], halo router - [1], cw20-base token - [2]])
        let mut contract_info_vec: Vec<ContractInfo> = Vec::new();

        // store code of all contracts to the app and get the code ids
        let halo_factory_contract_code_id = app.store_code(halo_factory_contract_template());
        let halo_pair_contract_code_id = app.store_code(halo_pair_contract_template());
        let halo_router_contract_code_id = app.store_code(halo_router_contract_template());
        let halo_token_contract_code_id = app.store_code(halo_token_contract_template());

        // halo factory contract
        // create instantiate message for contract
        let halo_factory_contract_instantiate_msg = HaloFactoryInstantiateMsg {
            pair_code_id: halo_pair_contract_code_id,
            token_code_id: halo_token_contract_code_id,
        };

        // instantiate contract
        let halo_factory_contract_addr = app
            .instantiate_contract(
                halo_factory_contract_code_id,
                Addr::unchecked(ADMIN),
                &halo_factory_contract_instantiate_msg,
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: halo_factory_contract_addr.to_string(),
            contract_code_id: halo_factory_contract_code_id,
        });

        // halo pair contract
        // Not needed to instantiate the pair contract

        // halo router contract

        // create instantiate message for contract
        // instantiate contract
        let halo_router_contract_addr = app
            .instantiate_contract(
                halo_router_contract_code_id,
                Addr::unchecked(ADMIN),
                &HaloRouterInstantiateMsg {
                    halo_factory: halo_factory_contract_addr.to_string(),
                },
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: halo_router_contract_addr.to_string(),
            contract_code_id: halo_router_contract_code_id,
        });

        // cw20-base token contract

        // create instantiate message for contract
        let halo_token_contract_instantiate_msg = HaloTokenInstantiateMsg {
            name: HALO_TOKEN_NAME.to_string(),
            symbol: HALO_TOKEN_SYMBOL.to_string(),
            decimals: 6,
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
        contract_info_vec.push(ContractInfo {
            contract_addr: halo_token_contract_addr.to_string(),
            contract_code_id: halo_token_contract_code_id,
        });

        // create instantiate message for contract
        let mstr_token_contract_instantiate_msg = HaloTokenInstantiateMsg {
            name: MSTR_TOKEN_NAME.to_string(),
            symbol: MSTR_TOKEN_SYMBOL.to_string(),
            decimals: MSTR_TOKEN_DECIMALS,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: ADMIN.to_string(), // the minter of the cw20 token must be the marketplace contract
                cap: None,
            }),
        };

        // instantiate contract
        let mstr_token_contract_addr = app
            .instantiate_contract(
                halo_token_contract_code_id,
                Addr::unchecked(ADMIN),
                &mstr_token_contract_instantiate_msg,
                &[],
                "test instantiate contract",
                None,
            )
            .unwrap();

        // add contract info to the vector
        contract_info_vec.push(ContractInfo {
            contract_addr: mstr_token_contract_addr.to_string(),
            contract_code_id: halo_token_contract_code_id,
        });

        // create instantiate message for contract
        let usdc_token_contract_instantiate_msg = HaloTokenInstantiateMsg {
            name: USDC_TOKEN_NAME.to_string(),
            symbol: USDC_TOKEN_SYMBOL.to_string(),
            decimals: USDC_TOKEN_DECIMALS,
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
        contract_info_vec.push(ContractInfo {
            contract_addr: usdc_token_contract_addr.to_string(),
            contract_code_id: halo_token_contract_code_id,
        });

        // return the app instance, the addresses and code IDs of all contracts
        (app, contract_info_vec)
    }

    // can not instantiate cw20-base token with wrong validate condition (name, symbol, decimals)
    #[test]
    fn cannot_instantiate_with_wrong_validate_condition() {
        let mut app = mock_app();
        let halo_token_contract_code_id = app.store_code(halo_token_contract_template());

        let too_short_token_name_instantiate_msg = HaloTokenInstantiateMsg {
            name: "H".to_string(),
            symbol: HALO_TOKEN_SYMBOL.to_string(),
            decimals: HALO_TOKEN_DECIMALS,
            initial_balances: vec![],
            mint: None,
        };

        let too_long_token_name_instantiate_msg = HaloTokenInstantiateMsg {
            name: "0123456789a123456789b123456789c123456789d123456789e".to_string(), // 51 characters
            symbol: HALO_TOKEN_SYMBOL.to_string(),
            decimals: HALO_TOKEN_DECIMALS,
            initial_balances: vec![],
            mint: None,
        };

        let too_short_token_symbol_instantiate_msg = HaloTokenInstantiateMsg {
            name: HALO_TOKEN_NAME.to_string(),
            symbol: "H".to_string(),
            decimals: HALO_TOKEN_DECIMALS,
            initial_balances: vec![],
            mint: None,
        };

        let too_long_token_symbol_instantiate_msg = HaloTokenInstantiateMsg {
            name: HALO_TOKEN_NAME.to_string(),
            symbol: "0123456789a123456789b123456789c123456789d123456789e".to_string(), // 51 characters
            decimals: HALO_TOKEN_DECIMALS,
            initial_balances: vec![],
            mint: None,
        };

        let too_big_token_decimals_instantiate_msg = HaloTokenInstantiateMsg {
            name: HALO_TOKEN_NAME.to_string(),
            symbol: HALO_TOKEN_SYMBOL.to_string(),
            decimals: 20,
            initial_balances: vec![],
            mint: None,
        };

        let initial_supply_greater_than_cap_msg = HaloTokenInstantiateMsg {
            name: HALO_TOKEN_NAME.to_string(),
            symbol: HALO_TOKEN_SYMBOL.to_string(),
            decimals: HALO_TOKEN_DECIMALS,
            initial_balances: vec![Cw20Coin {
                address: Addr::unchecked(ADMIN).to_string(),
                amount: Uint128::new(100),
            }],
            mint: Some(MinterResponse {
                minter: ADMIN.to_string(),
                cap: Some(Uint128::new(90)),
            }),
        };

        let err = app
            .instantiate_contract(
                halo_token_contract_code_id,
                Addr::unchecked(ADMIN),
                &too_short_token_name_instantiate_msg,
                &[],
                "test wrong token name instantiate contract",
                None,
            )
            .unwrap_err();

        assert_eq!(
            err.source().unwrap().to_string(),
            StdError::generic_err("Name is not in the expected format (3-50 UTF-8 bytes)")
                .to_string()
        );

        let err = app
            .instantiate_contract(
                halo_token_contract_code_id,
                Addr::unchecked(ADMIN),
                &too_long_token_name_instantiate_msg,
                &[],
                "test wrong token name instantiate contract",
                None,
            )
            .unwrap_err();

        assert_eq!(
            err.source().unwrap().to_string(),
            StdError::generic_err("Name is not in the expected format (3-50 UTF-8 bytes)")
                .to_string()
        );

        let err = app
            .instantiate_contract(
                halo_token_contract_code_id,
                Addr::unchecked(ADMIN),
                &too_short_token_symbol_instantiate_msg,
                &[],
                "test wrong token symbol instantiate contract",
                None,
            )
            .unwrap_err();

        assert_eq!(
            err.source().unwrap().to_string(),
            StdError::generic_err("Ticker symbol is not in expected format [a-zA-Z\\-]{3,12}")
                .to_string()
        );

        let err = app
            .instantiate_contract(
                halo_token_contract_code_id,
                Addr::unchecked(ADMIN),
                &too_long_token_symbol_instantiate_msg,
                &[],
                "test wrong token symbol instantiate contract",
                None,
            )
            .unwrap_err();

        assert_eq!(
            err.source().unwrap().to_string(),
            StdError::generic_err("Ticker symbol is not in expected format [a-zA-Z\\-]{3,12}")
                .to_string()
        );

        let err = app
            .instantiate_contract(
                halo_token_contract_code_id,
                Addr::unchecked(ADMIN),
                &too_big_token_decimals_instantiate_msg,
                &[],
                "test wrong token decimals instantiate contract",
                None,
            )
            .unwrap_err();

        assert_eq!(
            err.source().unwrap().to_string(),
            StdError::generic_err("Decimals must not exceed 18").to_string()
        );

        let err = app
            .instantiate_contract(
                halo_token_contract_code_id,
                Addr::unchecked(ADMIN),
                &initial_supply_greater_than_cap_msg,
                &[],
                "test initial supply greater than cap msg",
                None,
            )
            .unwrap_err();

        assert_eq!(
            err.source().unwrap().to_string(),
            StdError::generic_err("Initial supply greater than cap").to_string()
        );
    }
}
