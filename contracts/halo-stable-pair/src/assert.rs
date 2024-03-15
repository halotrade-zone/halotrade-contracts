use bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Decimal, StdError, Uint128};
use haloswap::{asset::Asset, error::ContractError};

pub fn assert_stable_slippage_tolerance(
    slippage_tolerance: &Option<Decimal>,
    deposits: &[Uint128],
    pairs: &[Asset],
) -> Result<(), ContractError> {
    if let Some(slippage_tolerance) = *slippage_tolerance {
        let slippage_tolerance: Decimal256 = slippage_tolerance.into();
        // the slippage tolerance cannot be greater than 100%
        if slippage_tolerance > Decimal256::one() {
            return Err(StdError::generic_err("slippage_tolerance cannot bigger than 1").into());
        }

        let _one_minus_slippage_tolerance = Decimal256::one() - slippage_tolerance;
        let _deposits: [Uint256; 2] = [deposits[0].into(), deposits[1].into()];
        let _pairs: [Uint256; 2] = [pairs[0].amount.into(), pairs[1].amount.into()];

        // Ensure each prices are not dropped as much as slippage tolerance rate
        // if calc_price_drop(deposits[0], deposits[1], one_minus_slippage_tolerance)
        //     > calc_slippage_tolerance(pairs[0], pairs[1])
        //     || calc_price_drop(deposits[1], deposits[0], one_minus_slippage_tolerance)
        //         > calc_slippage_tolerance(pairs[1], pairs[0])
        // {
        //     return Err(ContractError::MaxSlippageAssertion {});
        // }
    }
    Ok(())
}
