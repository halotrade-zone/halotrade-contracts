use crate::assert::assert_max_spread;
use crate::contract::{
    execute, instantiate, query, query_pool, query_reverse_simulation, query_simulation, reply,
};
use bignumber::Decimal256;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Reply, ReplyOn, Response, StdError,
    SubMsg, SubMsgResponse, SubMsgResult, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg, MinterResponse};
use haloswap::asset::{Asset, AssetInfo, CreatePairRequirements, LPTokenInfo, PairInfo};
use haloswap::error::ContractError;
use haloswap::mock_querier::mock_dependencies;
use haloswap::pair::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, PoolResponse, QueryMsg};
use haloswap::pair::{ReverseSimulationResponse, SimulationResponse};
use haloswap::token::InstantiateMsg as TokenInstantiateMsg;

use std::str::FromStr;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfo::Token {
                contract_addr: "asset0000".to_string(),
            },
        ],
        token_code_id: 10u64,
        asset_decimals: [6u8, 8u8],
        requirements: CreatePairRequirements {
            whitelist: vec![Addr::unchecked("addr0000")],
            first_asset_minimum: Uint128::zero(),
            second_asset_minimum: Uint128::zero(),
        },
        commission_rate: Decimal256::from_str("0.003").unwrap(),
        lp_token_info: LPTokenInfo {
            lp_token_name: "uusd_asset0000_lp".to_string(),
            lp_token_symbol: "uusd_asset0000_lp".to_string(),
            lp_token_decimals: Some(18),
        },
    };

    // we can just call .unwrap() to assert this was a success
    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg {
            msg: WasmMsg::Instantiate {
                code_id: 10u64,
                msg: to_binary(&TokenInstantiateMsg {
                    name: "uusd_asset0000_lp".to_string(),
                    symbol: "uusd_asset0000_lp".to_string(),
                    decimals: 18,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: MOCK_CONTRACT_ADDR.to_string(),
                        cap: None,
                    }),
                })
                .unwrap(),
                funds: vec![],
                label: "lp".to_string(),
                admin: None,
            }
            .into(),
            gas_limit: None,
            id: 1,
            reply_on: ReplyOn::Success,
        }]
    );

    // store liquidity token
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(
                vec![
                    // binary message which is converted from "liquidity0000" string.
                    10, 13, 108, 105, 113, 117, 105, 100, 105, 116, 121, 48, 48, 48, 48,
                ]
                .into(),
            ),
        }),
    };
    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // it worked, let's query the state
    let env = mock_env();
    let pair_info = query(deps.as_ref(), env, QueryMsg::Pair {}).unwrap();
    assert_eq!(
        pair_info,
        to_binary(&PairInfo {
            liquidity_token: "liquidity0000".to_string(),
            asset_infos: [
                AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                AssetInfo::Token {
                    contract_addr: "asset0000".to_string()
                }
            ],
            contract_addr: MOCK_CONTRACT_ADDR.to_string(),
            asset_decimals: [6u8, 8u8],
            requirements: CreatePairRequirements {
                whitelist: vec![Addr::unchecked("addr0000")],
                first_asset_minimum: Uint128::zero(),
                second_asset_minimum: Uint128::zero(),
            },
            commission_rate: Decimal256::from_str("0.003").unwrap(),
        })
        .unwrap()
    );
}

// #[test]
// fn receive() {
//     let offer_amount = Uint128::from(1500000000u128);
//     let mut deps = mock_dependencies(&[Coin {
//         denom: "uusd".to_string(),
//         amount: Uint128::from(200u128),
//     }]);

//     deps.querier.with_token_balances(&[
//         (
//             &"liquidity0000".to_string(),
//             &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::zero())],
//         ),
//         (&"asset0000".to_string(), &[]),
//     ]);

//     let msg = InstantiateMsg {
//         asset_infos: [
//             AssetInfo::NativeToken {
//                 denom: "uusd".to_string(),
//             },
//             AssetInfo::Token {
//                 contract_addr: "asset0000".to_string(),
//             },
//         ],
//         token_code_id: 10u64,
//         asset_decimals: [6u8, 8u8],
//         requirements: CreatePairRequirements {
//             whitelist: vec![Addr::unchecked("addr0000")],
//             first_asset_minimum: Uint128::zero(),
//             second_asset_minimum: Uint128::zero(),
//         },
//     };

//     let env = mock_env();
//     let info = mock_info("addr0000", &[]);
//     // we can just call .unwrap() to assert this was a success
//     let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

//     // store liquidity token
//     let reply_msg = Reply {
//         id: 1,
//         result: SubMsgResult::Ok(SubMsgResponse {
//             events: vec![],
//             data: Some(
//                 vec![
//                     // binary message which is converted from "liquidity0000" string.
//                     10, 13, 108, 105, 113, 117, 105, 100, 105, 116, 121, 48, 48, 48, 48,
//                 ]
//                 .into(),
//             ),
//         }),
//     };

//     let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

//     // swap uusd to asset0000
//     let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
//         sender: "addr0000".to_string(),
//         amount: Uint128::zero(),
//         msg:to_binary(&Cw20HookMsg::Swap {
//                 belief_price: None,
//                 max_spread: None,
//                 to: None,
//             })
//             .unwrap(),
//     });

//     let env = mock_env();
//     let info = mock_info(
//         "addr0000",
//         &[Coin {
//             denom: "uusd".to_string(),
//             amount: offer_amount,
//         }],
//     );
//     println!("{:?}", msg);
//     let res = execute(deps.as_mut(), env, info, msg).unwrap();
//     let msg_transfer = res.messages.get(0).expect("no message");

//     assert_eq!(
//         &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
//             contract_addr: "asset0000".to_string(),
//             msg: to_binary(&Cw20ExecuteMsg::Transfer {
//                 recipient: "addr0000".to_string(),
//                 amount: Uint128::from(100u128),
//             })
//             .unwrap(),
//             funds: vec![],
//         })),
//         msg_transfer,
//     );
// }

#[test]
fn provide_liquidity() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(340_282_366_918_000_000_000_000_000_000u128),
    }]);

    deps.querier.with_token_balances(&[
        (
            &"liquidity0000".to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::zero())],
        ),
        (&"asset0000".to_string(), &[]),
    ]);

    let msg = InstantiateMsg {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfo::Token {
                contract_addr: "asset0000".to_string(),
            },
        ],
        token_code_id: 10u64,
        asset_decimals: [6u8, 8u8],
        requirements: CreatePairRequirements {
            whitelist: vec![Addr::unchecked("addr0000")],
            first_asset_minimum: Uint128::zero(),
            second_asset_minimum: Uint128::zero(),
        },
        commission_rate: Decimal256::from_str("0.003").unwrap(),
        lp_token_info: LPTokenInfo {
            lp_token_name: "uusd_asset0000_LP".to_string(),
            lp_token_symbol: "uusd_asset0000_LP".to_string(),
            lp_token_decimals: None,
        },
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    // store liquidity token
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(
                vec![
                    // binary message which is converted from "liquidity0000" string.
                    10, 13, 108, 105, 113, 117, 105, 100, 105, 116, 121, 48, 48, 48, 48,
                ]
                .into(),
            ),
        }),
    };

    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // verify failed provide liquidity with invalid asset amount:
    // providing free token (one of the deposits is zero)
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: Uint128::from(100u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(0u128),
            },
        ],
        slippage_tolerance: None,
        receiver: None,
    };

    let env = mock_env();
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(0u128),
        }],
    );
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match res {
        ContractError::InvalidZeroAmount {} => {}
        _ => panic!("DO NOT ENTER HERE"),
    }

    // successfully provide liquidity in max limit asset amount:
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: Uint128::from(340_282_366_923_000_000_000_000_000_000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(1u128),
            },
        ],
        slippage_tolerance: None,
        receiver: None,
    };

    let env = mock_env();
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1u128),
        }],
    );
    let res = execute(deps.as_mut(), env, info, msg);
    assert!(res.is_ok());

    // successfully provide liquidity for the exist pool
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: Uint128::from(100u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(100u128),
            },
        ],
        slippage_tolerance: None,
        receiver: None,
    };

    let env = mock_env();
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u128),
        }],
    );
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    let transfer_from_msg = res.messages.get(0).expect("no message");
    let mint_for_liquidity0000_msg = res.messages.get(1).expect("no message");
    let mint_for_addr0000_msg = res.messages.get(2).expect("no message");
    assert_eq!(
        transfer_from_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "asset0000".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: "addr0000".to_string(),
                recipient: MOCK_CONTRACT_ADDR.to_string(),
                amount: Uint128::from(100u128),
            })
            .unwrap(),
            funds: vec![],
        }))
    );
    assert_eq!(
        mint_for_liquidity0000_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "liquidity0000".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                // Verify minting 1 reserve LP token to lp token contract address
                recipient: "liquidity0000".to_string(),
                amount: Uint128::from(1u128),
            })
            .unwrap(),
            funds: vec![],
        }))
    );
    assert_eq!(
        mint_for_addr0000_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "liquidity0000".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: "addr0000".to_string(),
                // addr0000 will receive 100 - 1 share of LP token
                // because 1 share of LP token is minted to lp token contract address
                amount: Uint128::from(99u128),
            })
            .unwrap(),
            funds: vec![],
        }))
    );

    // provide liquidity with slippage tolerance greater than 100%
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: Uint128::from(100u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(100u128),
            },
        ],
        slippage_tolerance: Some(Decimal::percent(101)), // slippage tolerance is 101%
        receiver: None,
    };

    let env = mock_env();
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u128),
        }],
    );
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match res {
        ContractError::Std(StdError::GenericErr { msg, .. }) => {
            assert_eq!(msg, "slippage_tolerance cannot bigger than 1".to_string())
        }
        _ => panic!("Must return generic error"),
    }

    // provide more liquidity 1:2, which is not proportional to 1:1,
    // then it must accept 1:1 and treat left amount as donation
    deps.querier.with_balance(&[(
        &MOCK_CONTRACT_ADDR.to_string(),
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(
                200u128 + 200u128, // user deposit must be pre-applied
            ),
        }],
    )]);

    deps.querier.with_token_balances(&[
        (
            &"liquidity0000".to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(100u128))],
        ),
        (
            &"asset0000".to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(200u128))],
        ),
    ]);

    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: Uint128::from(100u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(200u128),
            },
        ],
        slippage_tolerance: None,
        receiver: Some("staking0000".to_string()), // try changing receiver
    };

    let env = mock_env();
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(200u128),
        }],
    );

    // only accept 100, then 50 share will be generated with 100 * (100 / 200)
    let res: Response = execute(deps.as_mut(), env, info, msg).unwrap();
    let transfer_from_msg = res.messages.get(0).expect("no message");
    let mint_msg = res.messages.get(1).expect("no message");
    assert_eq!(
        transfer_from_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "asset0000".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: "addr0000".to_string(),
                recipient: MOCK_CONTRACT_ADDR.to_string(),
                amount: Uint128::from(100u128),
            })
            .unwrap(),
            funds: vec![],
        }))
    );
    assert_eq!(
        mint_msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "liquidity0000".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Mint {
                recipient: "staking0000".to_string(),
                // staking0000 will receive 50 share of LP token (100 * (100 / 200))
                amount: Uint128::from(50u128),
            })
            .unwrap(),
            funds: vec![],
        }))
    );

    // check wrong argument
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: Uint128::from(100u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(50u128),
            },
        ],
        slippage_tolerance: None,
        receiver: None,
    };

    let env = mock_env();
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u128),
        }],
    );
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match res {
        ContractError::Std(StdError::GenericErr { msg, .. }) => assert_eq!(
            msg,
            "Native token balance mismatch between the argument and the transferred".to_string()
        ),
        _ => panic!("Must return generic error"),
    }

    // initialize token balance to 1:1
    deps.querier.with_balance(&[(
        &MOCK_CONTRACT_ADDR.to_string(),
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(
                100u128 + 100u128, /* user deposit must be pre-applied */
            ),
        }],
    )]);

    deps.querier.with_token_balances(&[
        (
            &"liquidity0000".to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(100u128))],
        ),
        (
            &"asset0000".to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(100u128))],
        ),
    ]);

    // failed because the price is under slippage_tolerance
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: Uint128::from(98u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(100u128),
            },
        ],
        slippage_tolerance: Some(Decimal::percent(1)),
        receiver: None,
    };

    let env = mock_env();
    let info = mock_info(
        "addr0001",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u128),
        }],
    );
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match res {
        ContractError::MaxSlippageAssertion {} => {}
        _ => panic!("DO NOT ENTER HERE"),
    }

    // initialize token balance to 1:1
    deps.querier.with_balance(&[(
        &MOCK_CONTRACT_ADDR.to_string(),
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u128 + 98u128 /* user deposit must be pre-applied */),
        }],
    )]);

    // failed because the price is under slippage_tolerance
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: Uint128::from(100u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(98u128),
            },
        ],
        slippage_tolerance: Some(Decimal::percent(1)),
        receiver: None,
    };

    let env = mock_env();
    let info = mock_info(
        "addr0001",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(98u128),
        }],
    );
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match res {
        ContractError::MaxSlippageAssertion {} => {}
        _ => panic!("DO NOT ENTER HERE"),
    }

    // initialize token balance to 1:1
    deps.querier.with_balance(&[(
        &MOCK_CONTRACT_ADDR.to_string(),
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(
                100u128 + 100u128, /* user deposit must be pre-applied */
            ),
        }],
    )]);

    // successfully provides
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: Uint128::from(99u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(100u128),
            },
        ],
        slippage_tolerance: Some(Decimal::percent(1)),
        receiver: None,
    };

    let env = mock_env();
    let info = mock_info(
        "addr0001",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u128),
        }],
    );
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();

    // initialize token balance to 1:1
    deps.querier.with_balance(&[(
        &MOCK_CONTRACT_ADDR.to_string(),
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u128 + 99u128 /* user deposit must be pre-applied */),
        }],
    )]);

    // successfully provides
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: Uint128::from(100u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(99u128),
            },
        ],
        slippage_tolerance: Some(Decimal::percent(1)),
        receiver: None,
    };

    let env = mock_env();
    let info = mock_info(
        "addr0001",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(99u128),
        }],
    );
    let _res = execute(deps.as_mut(), env, info, msg).unwrap();
}

#[test]
#[should_panic(expected = "arithmetic operation overflow")]
fn provide_overflow_liquidity() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(340_282_366_918_000_000_000_000_000_000u128),
    }]);

    deps.querier.with_token_balances(&[
        (
            &"liquidity0000".to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::zero())],
        ),
        (&"asset0000".to_string(), &[]),
    ]);

    let msg = InstantiateMsg {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfo::Token {
                contract_addr: "asset0000".to_string(),
            },
        ],
        token_code_id: 10u64,
        asset_decimals: [6u8, 8u8],
        requirements: CreatePairRequirements {
            whitelist: vec![Addr::unchecked("addr0000")],
            first_asset_minimum: Uint128::zero(),
            second_asset_minimum: Uint128::zero(),
        },
        commission_rate: Decimal256::from_str("0.003").unwrap(),
        lp_token_info: LPTokenInfo {
            lp_token_name: "uusd_asset0000_LP".to_string(),
            lp_token_symbol: "uusd_asset0000_LP".to_string(),
            lp_token_decimals: None,
        },
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    // store liquidity token
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(
                vec![
                    // binary message which is converted from "liquidity0000" string.
                    10, 13, 108, 105, 113, 117, 105, 100, 105, 116, 121, 48, 48, 48, 48,
                ]
                .into(),
            ),
        }),
    };

    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // verify failed provide liquidity with invalid asset amount:
    // providing free token (one of the deposits is zero)
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: Uint128::from(100u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(0u128),
            },
        ],
        slippage_tolerance: None,
        receiver: None,
    };

    let env = mock_env();
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(0u128),
        }],
    );
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match res {
        ContractError::InvalidZeroAmount {} => {}
        _ => panic!("DO NOT ENTER HERE"),
    }

    // verify failed provide liquidity with over limit asset amount:
    let msg = ExecuteMsg::ProvideLiquidity {
        assets: [
            Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: Uint128::from(340_282_366_924_000_000_000_000_000_000u128),
            },
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: Uint128::from(1u128),
            },
        ],
        slippage_tolerance: None,
        receiver: None,
    };

    let env = mock_env();
    let info = mock_info(
        "addr0000",
        &[Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1u128),
        }],
    );
    let _res = execute(deps.as_mut(), env, info, msg);
}

#[test]
fn withdraw_liquidity() {
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: Uint128::from(100u128),
    }]);

    deps.querier.with_token_balances(&[
        (
            &"liquidity0000".to_string(),
            &[(&"addr0000".to_string(), &Uint128::from(100u128))],
        ),
        (
            &"asset0000".to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(100u128))],
        ),
    ]);

    let msg = InstantiateMsg {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfo::Token {
                contract_addr: "asset0000".to_string(),
            },
        ],
        token_code_id: 10u64,
        asset_decimals: [6u8, 8u8],
        requirements: CreatePairRequirements {
            whitelist: vec![Addr::unchecked("addr0000")],
            first_asset_minimum: Uint128::zero(),
            second_asset_minimum: Uint128::zero(),
        },
        commission_rate: Decimal256::from_str("0.003").unwrap(),
        lp_token_info: LPTokenInfo {
            lp_token_name: "uusd_asset0000_LP".to_string(),
            lp_token_symbol: "uusd_asset0000_LP".to_string(),
            lp_token_decimals: None,
        },
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    // store liquidity token
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(
                vec![
                    // binary message which is converted from "liquidity0000" string.
                    10, 13, 108, 105, 113, 117, 105, 100, 105, 116, 121, 48, 48, 48, 48,
                ]
                .into(),
            ),
        }),
    };

    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // withdraw successfully liquidity
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        msg: to_binary(&Cw20HookMsg::WithdrawLiquidity {}).unwrap(),
        amount: Uint128::from(100u128),
    });

    let env = mock_env();
    let info = mock_info("liquidity0000", &[]);
    let res = execute(deps.as_mut(), env.clone(), info, msg.clone()).unwrap();
    let log_withdrawn_share = res.attributes.get(2).expect("no log");
    let log_refund_assets = res.attributes.get(3).expect("no log");
    let msg_refund_0 = res.messages.get(0).expect("no message");
    let msg_refund_1 = res.messages.get(1).expect("no message");
    let msg_burn_liquidity = res.messages.get(2).expect("no message");
    assert_eq!(
        msg_refund_0,
        &SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "addr0000".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(100u128),
            }],
        }))
    );
    assert_eq!(
        msg_refund_1,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "asset0000".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0000".to_string(),
                amount: Uint128::from(100u128),
            })
            .unwrap(),
            funds: vec![],
        }))
    );
    assert_eq!(
        msg_burn_liquidity,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "liquidity0000".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Burn {
                amount: Uint128::from(100u128),
            })
            .unwrap(),
            funds: vec![],
        }))
    );

    assert_eq!(
        log_withdrawn_share,
        &attr("withdrawn_share", 100u128.to_string())
    );
    assert_eq!(
        log_refund_assets,
        &attr("refund_assets", "100uusd, 100asset0000")
    );

    // withdraw failed with incorrect sender address and liquidity token
    let info = mock_info("liquidity0001", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match res {
        ContractError::Unauthorized {} => (),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

// #[test]
// fn try_native_to_token() {
//     let total_share = Uint128::from(30_000_000_000u128);
//     let asset_pool_amount = Uint128::from(20_000_000_000u128);
//     let collateral_pool_amount = Uint128::from(30_000_000_000u128);
//     let exchange_rate: Decimal = Decimal::from_ratio(asset_pool_amount, collateral_pool_amount);
//     let offer_amount = Uint128::from(1_500_000_000u128);

//     let mut deps = mock_dependencies(&[Coin {
//         denom: "uusd".to_string(),
//         amount: collateral_pool_amount + offer_amount, /* user deposit must be pre-applied */
//     }]);

//     deps.querier.with_token_balances(&[
//         (
//             &"liquidity0000".to_string(),
//             &[(&MOCK_CONTRACT_ADDR.to_string(), &total_share)],
//         ),
//         (
//             &"asset0000".to_string(),
//             &[(&MOCK_CONTRACT_ADDR.to_string(), &asset_pool_amount)],
//         ),
//     ]);

//     let msg = InstantiateMsg {
//         asset_infos: [
//             AssetInfo::NativeToken {
//                 denom: "uusd".to_string(),
//             },
//             AssetInfo::Token {
//                 contract_addr: "asset0000".to_string(),
//             },
//         ],
//         token_code_id: 10u64,
//         asset_decimals: [6u8, 8u8],
//         requirements: CreatePairRequirements {
//             whitelist: vec![Addr::unchecked("addr0000")],
//             first_asset_minimum: Uint128::zero(),
//             second_asset_minimum: Uint128::zero(),
//         },
//     };

//     let env = mock_env();
//     let info = mock_info("addr0000", &[]);
//     // we can just call .unwrap() to assert this was a success
//     let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

//     // store liquidity token
//     let reply_msg = Reply {
//         id: 1,
//         result: SubMsgResult::Ok(SubMsgResponse {
//             events: vec![],
//             data: Some(
//                 vec![
//                     // binary message which is converted from "liquidity0000" string.
//                     10, 13, 108, 105, 113, 117, 105, 100, 105, 116, 121, 48, 48, 48, 48,
//                 ]
//                 .into(),
//             ),
//         }),
//     };

//     let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

//     // swap uusd to asset0000
//     let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
//         sender: "addr0000".to_string(),
//         amount: offer_amount,
//         msg:to_binary(&Cw20HookMsg::Swap {
//                 belief_price: None,
//                 max_spread: None,
//                 to: None,
//             })
//             .unwrap(),
//     });
//     // normal swap
//     // let msg = ExecuteMsg::Swap {
//     //     offer_asset: Asset {
//     //         info: AssetInfo::NativeToken {
//     //             denom: "uusd".to_string(),
//     //         },
//     //         amount: offer_amount,
//     //     },
//     //     belief_price: None,
//     //     max_spread: None,
//     //     to: None,
//     // };
//     let env = mock_env();
//     let info = mock_info(
//         "addr0000",
//         &[Coin {
//             denom: "uusd".to_string(),
//             amount: offer_amount,
//         }],
//     );
//     let res = execute(deps.as_mut(), env, info, msg).unwrap();
//     let msg_transfer = res.messages.get(0).expect("no message");

//     // current price is 1.5, so expected return without spread is 1000
//     // 952.380952 = 20000 - 20000 * 30000 / (30000 + 1500)
//     let expected_ret_amount = Uint128::from(952_380_952u128);
//     let expected_spread_amount = (offer_amount * exchange_rate)
//         .checked_sub(expected_ret_amount)
//         .unwrap();
//     let expected_commission_amount = expected_ret_amount.multiply_ratio(3u128, 1000u128); // 0.3%
//     let expected_return_amount = expected_ret_amount
//         .checked_sub(expected_commission_amount)
//         .unwrap();
//     // check simulation res
//     deps.querier.with_balance(&[(
//         &MOCK_CONTRACT_ADDR.to_string(),
//         vec![Coin {
//             denom: "uusd".to_string(),
//             amount: collateral_pool_amount, /* user deposit must be pre-applied */
//         }],
//     )]);

//     let simulation_res: SimulationResponse = query_simulation(
//         deps.as_ref(),
//         Asset {
//             info: AssetInfo::NativeToken {
//                 denom: "uusd".to_string(),
//             },
//             amount: offer_amount,
//         },
//     )
//     .unwrap();
//     assert_eq!(expected_return_amount, simulation_res.return_amount);
//     assert_eq!(expected_commission_amount, simulation_res.commission_amount);
//     assert_eq!(expected_spread_amount, simulation_res.spread_amount);

//     // check reverse simulation res
//     let reverse_simulation_res: ReverseSimulationResponse = query_reverse_simulation(
//         deps.as_ref(),
//         Asset {
//             info: AssetInfo::Token {
//                 contract_addr: "asset0000".to_string(),
//             },
//             amount: expected_return_amount,
//         },
//     )
//     .unwrap();

//     assert!(
//         (offer_amount.u128() as i128 - reverse_simulation_res.offer_amount.u128() as i128).abs()
//             < 3i128
//     );
//     assert!(
//         (expected_commission_amount.u128() as i128
//             - reverse_simulation_res.commission_amount.u128() as i128)
//             .abs()
//             < 3i128
//     );
//     assert!(
//         (expected_spread_amount.u128() as i128
//             - reverse_simulation_res.spread_amount.u128() as i128)
//             .abs()
//             < 3i128
//     );

//     assert_eq!(
//         res.attributes,
//         vec![
//             attr("action", "swap"),
//             attr("sender", "addr0000"),
//             attr("receiver", "addr0000"),
//             attr("offer_asset", "uusd"),
//             attr("ask_asset", "asset0000"),
//             attr("offer_amount", offer_amount.to_string()),
//             attr("return_amount", expected_return_amount.to_string()),
//             attr("spread_amount", expected_spread_amount.to_string()),
//             attr("commission_amount", expected_commission_amount.to_string()),
//         ]
//     );

//     assert_eq!(
//         &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
//             contract_addr: "asset0000".to_string(),
//             msg: to_binary(&Cw20ExecuteMsg::Transfer {
//                 recipient: "addr0000".to_string(),
//                 amount: expected_return_amount,
//             })
//             .unwrap(),
//             funds: vec![],
//         })),
//         msg_transfer,
//     );
// }

#[test]
fn try_token_to_native() {
    let total_share = Uint128::from(20_000_000_000u128);
    let asset_pool_amount = Uint128::from(30_000_000_000u128);
    let collateral_pool_amount = Uint128::from(20_000_000_000u128);
    let offer_amount = Uint128::from(1_500_000_000u128);

    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: collateral_pool_amount,
    }]);
    deps.querier.with_token_balances(&[
        (
            &"liquidity0000".to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &total_share)],
        ),
        (
            &"asset0000".to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &(asset_pool_amount + offer_amount),
            )],
        ),
    ]);

    let msg = InstantiateMsg {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfo::Token {
                contract_addr: "asset0000".to_string(),
            },
        ],
        token_code_id: 10u64,
        asset_decimals: [8u8, 8u8],
        requirements: CreatePairRequirements {
            whitelist: vec![Addr::unchecked("addr0000")],
            first_asset_minimum: Uint128::zero(),
            second_asset_minimum: Uint128::zero(),
        },
        commission_rate: Decimal256::from_str("0.003").unwrap(),
        lp_token_info: LPTokenInfo {
            lp_token_name: "uusd_asset0000_LP".to_string(),
            lp_token_symbol: "uusd_asset0000_LP".to_string(),
            lp_token_decimals: None,
        },
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    // store liquidity token
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(
                vec![
                    // binary message which is converted from "liquidity0000" string.
                    10, 13, 108, 105, 113, 117, 105, 100, 105, 116, 121, 48, 48, 48, 48,
                ]
                .into(),
            ),
        }),
    };

    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    // unauthorized access; can not execute swap directly for token swap
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: offer_amount,
        msg: to_binary(&Cw20HookMsg::Swap {
            offer_asset: Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: offer_amount,
            },
            belief_price: None,
            max_spread: None,
            to: None,
        })
        .unwrap(),
    });
    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();

    match res {
        ContractError::Unauthorized {} => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // asset mismatch; mismatch amount of asset token in Cw20ReceiveMsg and Cw20HookMsg::Swap
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: Uint128::zero(),
        msg: to_binary(&Cw20HookMsg::Swap {
            offer_asset: Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: offer_amount,
            },
            belief_price: None,
            max_spread: None,
            to: None,
        })
        .unwrap(),
    });
    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();

    match res {
        ContractError::AssetMismatch {} => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    // swap to another address
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: offer_amount,
        msg: to_binary(&Cw20HookMsg::Swap {
            offer_asset: Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: offer_amount,
            },
            belief_price: None,
            max_spread: None,
            to: Some("addr0001".to_string()),
        })
        .unwrap(),
    });

    let env = mock_env();
    let info = mock_info("asset0000", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    let msg_transfer = res.messages.get(0).expect("no message");

    assert_eq!(
        &SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "addr0001".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                // `949523810u128` is expected_return_amount
                // that is calculated by the formula in `normal sell`test case below.
                // In this test case just uses to verify swap to another address.
                amount: Uint128::from(949523810u128)
            }],
        })),
        msg_transfer,
    );

    // normal sell
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: offer_amount,
        msg: to_binary(&Cw20HookMsg::Swap {
            offer_asset: Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: offer_amount,
            },
            belief_price: None,
            max_spread: None,
            to: None,
        })
        .unwrap(),
    });
    let env = mock_env();
    let info = mock_info("asset0000", &[]);

    let res = execute(deps.as_mut(), env, info, msg).unwrap();
    let msg_transfer = res.messages.get(0).expect("no message");

    // current price is 1.5, so expected return without spread is 1000
    // 952.380952 = 20000 - 20000 * 30000 / (30000 + 1500)
    let expected_ret_amount = Uint128::from(952_380_952u128);
    let expected_spread_amount = (collateral_pool_amount * offer_amount / asset_pool_amount)
        .checked_sub(expected_ret_amount)
        .unwrap();
    let expected_commission_amount = expected_ret_amount.multiply_ratio(3u128, 1000u128); // 0.3%
    let expected_return_amount = expected_ret_amount
        .checked_sub(expected_commission_amount)
        .unwrap();
    // check simulation res
    // return asset token balance as normal
    deps.querier.with_token_balances(&[
        (
            &"liquidity0000".to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &total_share)],
        ),
        (
            &"asset0000".to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &(asset_pool_amount))],
        ),
    ]);

    let simulation_res: SimulationResponse = query_simulation(
        deps.as_ref(),
        Asset {
            amount: offer_amount,
            info: AssetInfo::Token {
                contract_addr: "asset0000".to_string(),
            },
        },
    )
    .unwrap();
    assert_eq!(expected_return_amount, simulation_res.return_amount);
    assert_eq!(expected_commission_amount, simulation_res.commission_amount);
    assert_eq!(expected_spread_amount, simulation_res.spread_amount);

    // check reverse simulation res
    let reverse_simulation_res: ReverseSimulationResponse = query_reverse_simulation(
        deps.as_ref(),
        Asset {
            amount: expected_return_amount,
            info: AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
        },
    )
    .unwrap();
    assert!(
        (offer_amount.u128() as i128 - reverse_simulation_res.offer_amount.u128() as i128).abs()
            < 3i128
    );
    assert!(
        (expected_commission_amount.u128() as i128
            - reverse_simulation_res.commission_amount.u128() as i128)
            .abs()
            < 3i128
    );
    assert!(
        (expected_spread_amount.u128() as i128
            - reverse_simulation_res.spread_amount.u128() as i128)
            .abs()
            < 3i128
    );

    assert_eq!(
        res.attributes,
        vec![
            attr("action", "swap"),
            attr("sender", "addr0000"),
            attr("receiver", "addr0000"),
            attr("offer_asset", "asset0000"),
            attr("ask_asset", "uusd"),
            attr("offer_amount", offer_amount.to_string()),
            attr("return_amount", expected_return_amount.to_string()),
            attr("spread_amount", expected_spread_amount.to_string()),
            attr("commission_amount", expected_commission_amount.to_string()),
        ]
    );

    assert_eq!(
        &SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
            to_address: "addr0000".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: expected_return_amount
            }],
        })),
        msg_transfer,
    );

    // failed due to non asset token contract try to execute sell
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: "addr0000".to_string(),
        amount: offer_amount,
        msg: to_binary(&Cw20HookMsg::Swap {
            offer_asset: Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: offer_amount,
            },
            belief_price: None,
            max_spread: None,
            to: None,
        })
        .unwrap(),
    });
    let env = mock_env();
    let info = mock_info("liquidity0000", &[]);
    let res = execute(deps.as_mut(), env, info, msg).unwrap_err();
    match res {
        ContractError::Unauthorized {} => (),
        _ => panic!("DO NOT ENTER HERE"),
    }
}

#[test]
fn test_max_spread() {
    let offer_asset_info = AssetInfo::NativeToken {
        denom: "offer_asset".to_string(),
    };
    let ask_asset_info = AssetInfo::NativeToken {
        denom: "ask_asset_info".to_string(),
    };

    assert_max_spread(
        Some(Decimal::from_ratio(1200u128, 1u128)),
        Some(Decimal::percent(1)),
        Asset {
            info: offer_asset_info.clone(),
            amount: Uint128::from(1_200_000_000u128),
        },
        Asset {
            info: ask_asset_info.clone(),
            amount: Uint128::from(989999u128),
        },
        Uint128::zero(),
        6u8,
        6u8,
    )
    .unwrap_err();

    assert_max_spread(
        Some(Decimal::from_ratio(1200u128, 1u128)),
        Some(Decimal::percent(1)),
        Asset {
            info: offer_asset_info.clone(),
            amount: Uint128::from(1_200_000_000u128),
        },
        Asset {
            info: ask_asset_info.clone(),
            amount: Uint128::from(990_000u128),
        },
        Uint128::zero(),
        6u8,
        6u8,
    )
    .unwrap();

    assert_max_spread(
        None,
        Some(Decimal::percent(1)),
        Asset {
            info: offer_asset_info.clone(),
            amount: Uint128::zero(),
        },
        Asset {
            info: ask_asset_info.clone(),
            amount: Uint128::from(989999u128),
        },
        Uint128::from(10001u128),
        6u8,
        6u8,
    )
    .unwrap_err();

    assert_max_spread(
        None,
        Some(Decimal::percent(1)),
        Asset {
            info: offer_asset_info,
            amount: Uint128::zero(),
        },
        Asset {
            info: ask_asset_info,
            amount: Uint128::from(990_000u128),
        },
        Uint128::from(10000u128),
        6u8,
        6u8,
    )
    .unwrap();
}

#[test]
fn test_max_spread_with_diff_decimal() {
    let token_addr = "ask_asset_info".to_string();

    let mut deps = mock_dependencies(&[]);
    deps.querier.with_token_balances(&[(
        &token_addr,
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(10_000_000_000u64),
        )],
    )]);
    let offer_asset_info = AssetInfo::NativeToken {
        denom: "offer_asset".to_string(),
    };
    let ask_asset_info = AssetInfo::Token {
        contract_addr: token_addr.to_string(),
    };

    assert_max_spread(
        Some(Decimal::from_ratio(1200u128, 1u128)),
        Some(Decimal::percent(1)),
        Asset {
            info: offer_asset_info.clone(),
            amount: Uint128::from(1_200_000_000u128),
        },
        Asset {
            info: ask_asset_info.clone(),
            amount: Uint128::from(100_000_000u128),
        },
        Uint128::zero(),
        6u8,
        8u8,
    )
    .unwrap();

    assert_max_spread(
        Some(Decimal::from_ratio(1200u128, 1u128)),
        Some(Decimal::percent(1)),
        Asset {
            info: offer_asset_info,
            amount: Uint128::from(1_200_000_000u128),
        },
        Asset {
            info: ask_asset_info,
            amount: Uint128::from(98_999_999u128),
        },
        Uint128::zero(),
        6u8,
        8u8,
    )
    .unwrap_err();

    let offer_asset_info = AssetInfo::Token {
        contract_addr: token_addr,
    };
    let ask_asset_info = AssetInfo::NativeToken {
        denom: "offer_asset".to_string(),
    };

    assert_max_spread(
        Some(Decimal::from_ratio(1200u128, 1u128)),
        Some(Decimal::percent(1)),
        Asset {
            info: offer_asset_info.clone(),
            amount: Uint128::from(120_000_000_000u128),
        },
        Asset {
            info: ask_asset_info.clone(),
            amount: Uint128::from(1_000_000u128),
        },
        Uint128::zero(),
        8u8,
        6u8,
    )
    .unwrap();

    assert_max_spread(
        Some(Decimal::from_ratio(1200u128, 1u128)),
        Some(Decimal::percent(1)),
        Asset {
            info: offer_asset_info,
            amount: Uint128::from(120_000_000_000u128),
        },
        Asset {
            info: ask_asset_info,
            amount: Uint128::from(989_999u128),
        },
        Uint128::zero(),
        8u8,
        6u8,
    )
    .unwrap_err();
}

#[test]
fn test_query_pool() {
    let total_share_amount = Uint128::from(111u128);
    let asset_0_amount = Uint128::from(222u128);
    let asset_1_amount = Uint128::from(333u128);
    let mut deps = mock_dependencies(&[Coin {
        denom: "uusd".to_string(),
        amount: asset_0_amount,
    }]);

    deps.querier.with_token_balances(&[
        (
            &"asset0000".to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &asset_1_amount)],
        ),
        (
            &"liquidity0000".to_string(),
            &[(&MOCK_CONTRACT_ADDR.to_string(), &total_share_amount)],
        ),
    ]);

    let msg = InstantiateMsg {
        asset_infos: [
            AssetInfo::NativeToken {
                denom: "uusd".to_string(),
            },
            AssetInfo::Token {
                contract_addr: "asset0000".to_string(),
            },
        ],
        token_code_id: 10u64,
        asset_decimals: [6u8, 8u8],
        requirements: CreatePairRequirements {
            whitelist: vec![Addr::unchecked("addr0000")],
            first_asset_minimum: Uint128::zero(),
            second_asset_minimum: Uint128::zero(),
        },
        commission_rate: Decimal256::from_str("0.003").unwrap(),
        lp_token_info: LPTokenInfo {
            lp_token_name: "uusd_asset0000_LP".to_string(),
            lp_token_symbol: "uusd_asset0000_LP".to_string(),
            lp_token_decimals: None,
        },
    };

    let env = mock_env();
    let info = mock_info("addr0000", &[]);
    // we can just call .unwrap() to assert this was a success
    let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

    // store liquidity token
    let reply_msg = Reply {
        id: 1,
        result: SubMsgResult::Ok(SubMsgResponse {
            events: vec![],
            data: Some(
                vec![
                    // binary message which is converted from "liquidity0000" string.
                    10, 13, 108, 105, 113, 117, 105, 100, 105, 116, 121, 48, 48, 48, 48,
                ]
                .into(),
            ),
        }),
    };

    let _res = reply(deps.as_mut(), mock_env(), reply_msg).unwrap();

    let res: PoolResponse = query_pool(deps.as_ref()).unwrap();

    assert_eq!(
        res.assets,
        [
            Asset {
                info: AssetInfo::NativeToken {
                    denom: "uusd".to_string(),
                },
                amount: asset_0_amount
            },
            Asset {
                info: AssetInfo::Token {
                    contract_addr: "asset0000".to_string(),
                },
                amount: asset_1_amount
            }
        ]
    );
    assert_eq!(res.total_share, total_share_amount);
}
