use crate::asset::{Asset, PairInfoRaw};
use crate::error::ContractError;
use bignumber::{Decimal256, Uint256};
use cosmwasm_std::{MessageInfo, StdError, Uint128};
use integer_sqrt::IntegerSquareRoot;

/*
* @brief: This function calculates the amount of LP tokens to be minted to the user.
* @param: - info: the message info of the user.
*         - pair_info: the pair information of the pair.
*         - lp_total_supply: the total supply of the LP token.
*         - deposits: the amount of deposits of the user.
*         - pools: the amount of pools of the pair.
* @return: - Result<Uint128, ContractError>:
*             + Ok(Uint128): the amount of LP tokens to be minted to the user.
*             + Err(ContractError): the error message.
* @test: N/A
*/
pub fn calculate_lp_token_amount_to_user(
    info: &MessageInfo,
    pair_info: &PairInfoRaw,
    lp_total_supply: Uint128,
    deposits: [Uint128; 2],
    pools: [Asset; 2],
) -> Result<Uint128, ContractError> {
    if lp_total_supply == Uint128::zero() {
        // when pool is empty
        // if the sender is not in whitelist of requirements, then return error
        if !pair_info.requirements.whitelist.contains(&info.sender) {
            return Err(ContractError::Std(StdError::generic_err(
                "the sender is not in whitelist",
            )));
        }

        // if the minimum amount of deposit is not satisfied, then return error
        if deposits[0] < pair_info.requirements.first_asset_minimum
            || deposits[1] < pair_info.requirements.second_asset_minimum
        {
            return Err(ContractError::Std(StdError::generic_err(
                "the minimum deposit is not satisfied",
            )));
        }

        // if the total supply of the LP token is zero, Initial share = collateral amount
        // hoanm: EQUATION - LP = \sqrt{A * B}
        Ok(Uint128::from(
            (deposits[0].u128() * deposits[1].u128()).integer_sqrt(),
        ))
    } else {
        // hoanm: update these equations by using the formula of Uniswap V2
        // min(1, 2)
        // 1. sqrt(deposit_0 * exchange_rate_0_to_1 * deposit_0) * (lp_total_supply / sqrt(pool_0 * pool_1))
        // == deposit_0 * lp_total_supply / pool_0
        // 2. sqrt(deposit_1 * exchange_rate_1_to_0 * deposit_1) * (lp_total_supply / sqrt(pool_1 * pool_1))
        // == deposit_1 * lp_total_supply / pool_1
        Ok(std::cmp::min(
            deposits[0].multiply_ratio(lp_total_supply, pools[0].amount),
            deposits[1].multiply_ratio(lp_total_supply, pools[1].amount),
        ))
    }
}

/*
* @brief: This function calculates the amount of return, spread and commission based on the formula
*         `return_amount = offer_amount * (1 - spread) * ask_pool / (offer_pool + offer_amount)`
*         in case of user want to swap from 'offer' to 'ask' asset.
* @param: - offer_pool: the amount of the offer pool.
*         - ask_pool: the amount of the ask pool.
*         - offer_amount: the amount of the offer.
* @return: - (Uint128, Uint128, Uint128): the amount of return, spread and commission.
* @test: test_compute_swap_with_huge_pool_variance.
*/
pub fn compute_swap(
    offer_pool: Uint128,
    ask_pool: Uint128,
    offer_amount: Uint128,
    commission_rate: Decimal256,
) -> (Uint128, Uint128, Uint128) {
    let offer_pool: Uint256 = Uint256::from(offer_pool);
    let ask_pool: Uint256 = ask_pool.into();
    let offer_amount: Uint256 = offer_amount.into();

    // Commission rate OR Fee amount for framework
    // let commission_rate = Decimal256::from_str(COMMISSION_RATE).unwrap();

    // offer => ask
    // hoanm: EQUATION - B = (R_B - \frac{K}{R_A + A}) * (1 - F)
    // ask_amount = (ask_pool - cp / (offer_pool + offer_amount)) * (1 - commission_rate)

    // cp (constant product) is K  in the EQUATION
    let cp: Uint256 = offer_pool * ask_pool;

    // calculate the ask_amount without commission
    let return_amount: Uint256 = (Decimal256::from_uint256(ask_pool)
        - Decimal256::from_ratio(cp, offer_pool + offer_amount))
        * Uint256::one();

    // calculate the spread_amount
    // hoanm: EQUATION - SPREAD = (A * \frac{R_B}{R_A}) - B
    let spread_amount: Uint256 = (Decimal256::from_ratio(ask_pool * offer_amount, offer_pool)
        * Uint256::one())
        - return_amount;

    // calculate the commission_amount
    let commission_amount: Uint256 = return_amount * commission_rate;

    // commission will be absorbed to pool and the currency will be the same as the ask currency
    let return_amount: Uint256 = return_amount - commission_amount;
    (
        return_amount.into(),
        spread_amount.into(),
        commission_amount.into(),
    )
}

// The function to get the offer_amount when user provide ask_amount
pub fn compute_offer_amount(
    offer_pool: Uint128,
    ask_pool: Uint128,
    ask_amount: Uint128,
    commission_rate: Decimal256,
) -> (Uint128, Uint128, Uint128) {
    let offer_pool: Uint256 = offer_pool.into();
    let ask_pool: Uint256 = ask_pool.into();
    let ask_amount: Uint256 = ask_amount.into();

    // let commission_rate = Decimal256::from_str(COMMISSION_RATE).unwrap();

    // EQUATION: A = \frac{K}{R_B - (B * (1-P))} - R_A
    // ask => offer
    // offer_amount = cp / (ask_pool - ask_amount / (1 - commission_rate)) - offer_pool
    let cp: Uint256 = offer_pool * ask_pool;

    let one_minus_commission = Decimal256::one() - commission_rate;
    let inv_one_minus_commission = Decimal256::one() / one_minus_commission;

    let offer_amount: Uint256 = Uint256::one()
        .multiply_ratio(cp, ask_pool - ask_amount * inv_one_minus_commission)
        - offer_pool;

    let before_commission_deduction: Uint256 = ask_amount * inv_one_minus_commission;
    let before_spread_deduction: Uint256 =
        offer_amount * Decimal256::from_ratio(ask_pool, offer_pool);

    let spread_amount = if before_spread_deduction > before_commission_deduction {
        before_spread_deduction - before_commission_deduction
    } else {
        Uint256::zero()
    };

    let commission_amount = before_commission_deduction * commission_rate;

    (
        offer_amount.into(),
        spread_amount.into(),
        commission_amount.into(),
    )
}

// hoanm: EQUATION - \frac{A}{B} * (1-ST) > \frac{R_A}{R_B} \parallel \frac{B}{A} * (1-ST) > \frac{R_B}{R_A}
pub fn calc_price_drop(
    offer_deposits: Uint256,
    ask_deposits: Uint256,
    one_minus_slippage_tolerance: Decimal256,
) -> Decimal256 {
    Decimal256::from_ratio(offer_deposits, ask_deposits) * one_minus_slippage_tolerance
}

pub fn calc_slippage_tolerance(offer_pool: Uint256, ask_pool: Uint256) -> Decimal256 {
    Decimal256::from_ratio(offer_pool, ask_pool)
}

#[test]
fn test_compute_swap_with_huge_pool_variance() {
    use std::str::FromStr;

    let offer_pool = Uint128::from(395451850234u128);
    let ask_pool = Uint128::from(317u128);

    // compute swap return value
    let (return_amount, spread_amount, commission_amount) = compute_swap(
        offer_pool,
        ask_pool,
        Uint128::from(1u128),
        Decimal256::from_str("0.03").unwrap(),
    );

    assert_eq!(return_amount, Uint128::zero());
    assert_eq!(spread_amount, Uint128::zero());
    assert_eq!(commission_amount, Uint128::zero());
}

#[test]
fn test_compute_swap_with_huge_offer_pool() {
    use std::str::FromStr;

    // offer pool is max value of Uint128
    let offer_pool = Uint128::from(340282366920938463463374607431768211455u128);
    // ask pool is 1
    let ask_pool = Uint128::from(1u128);

    let (return_amount, spread_amount, commission_amount) = compute_swap(
        offer_pool,
        ask_pool,
        Uint128::from(1u128),
        Decimal256::from_str("0.03").unwrap(),
    );

    assert_eq!(return_amount, Uint128::zero());
    assert_eq!(spread_amount, Uint128::zero());
    assert_eq!(commission_amount, Uint128::zero());
}

#[test]
fn test_compute_swap_with_huge_ask_pool() {
    use std::str::FromStr;

    // offer pool is 1
    let offer_pool = Uint128::from(1u128);
    // ask pool is max value of Uint128
    let ask_pool = Uint128::from(340282366920938463463374607431768211455u128);

    let (return_amount, spread_amount, commission_amount) = compute_swap(
        offer_pool,
        ask_pool,
        Uint128::from(1u128),
        Decimal256::from_str("0.03").unwrap(),
    );

    assert_eq!(
        return_amount,
        Uint128::from(165036947956655154779736684604407582556u128)
    );
    assert_eq!(
        spread_amount,
        Uint128::from(170141183460469231731687303715884105728u128)
    );
    assert_eq!(
        commission_amount,
        Uint128::from(5104235503814076951950619111476523171u128)
    );
}

// For calculate accurately this contract is using Decimal256 type for division calculations
// So the maximum value when offer_pool and ask_pool are multiplied
// is the maximum value of Decimal256:
// 115792089237316195423570985008687907853269984665640564039457.584007913129639935 (which is (2^128 - 1) / 10^18)
#[test]
fn test_compute_swap_with_huge_ask_pool_and_offer_pool() {
    use std::str::FromStr;

    // offer pool is root of max value of Decimal256
    let offer_pool = Uint128::from(340282366920938463463374607431u128);
    // ask pool is root of max value of Decimal256
    let ask_pool = Uint128::from(340282366920938463463374607431u128);

    let (return_amount, spread_amount, commission_amount) = compute_swap(
        offer_pool,
        ask_pool,
        Uint128::from(1u128),
        Decimal256::from_str("0.03").unwrap(),
    );

    assert_eq!(return_amount, Uint128::one());
    assert_eq!(spread_amount, Uint128::zero());
    assert_eq!(commission_amount, Uint128::zero());
}

#[test]
fn test_compute_swap_with_huge_ask_pool_and_offer_pool_and_offer_amount() {
    use std::str::FromStr;

    // offer pool is root of max value of Decimal256
    let offer_pool = Uint128::from(340282366920938463463374607431u128);
    // ask pool is root of max value of Decimal256
    let ask_pool = Uint128::from(340282366920938463463374607431u128);

    let (return_amount, spread_amount, commission_amount) = compute_swap(
        offer_pool,
        ask_pool,
        Uint128::from(340282366920938463463374607431u128),
        Decimal256::from_str("0.03").unwrap(),
    );

    assert_eq!(
        return_amount,
        Uint128::from(165036947956655154779736684604u128)
    );
    assert_eq!(
        spread_amount,
        Uint128::from(170141183460469231731687303716u128)
    );
    assert_eq!(
        commission_amount,
        Uint128::from(5104235503814076951950619111u128)
    );
}
