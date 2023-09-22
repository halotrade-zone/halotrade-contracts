use bignumber::{Uint256, Decimal256};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

#[cw_serde]
pub struct AmpFactor {
    /// Initial amplification coefficient (A)
    pub initial_amp_factor: Uint128,
    /// Target amplification coefficient (A)
    pub target_amp_factor: Uint128,
    /// Current unix timestamp
    pub current_ts: u64,
    /// Ramp A start timestamp
    pub start_ramp_ts: u64,
    /// Ramp A stop timestamp
    pub stop_ramp_ts: u64,
}

impl AmpFactor{
    pub fn new(
        initial_amp_factor: Uint128,
        target_amp_factor: Uint128,
        current_ts: u64,
        start_ramp_ts: u64,
        stop_ramp_ts: u64,
    ) -> Self {
        AmpFactor {
            initial_amp_factor,
            target_amp_factor,
            current_ts,
            start_ramp_ts,
            stop_ramp_ts,
        }
    }

    /// Comute the amplification coefficient (A)
    pub fn compute_amp_factor(&self) -> Option<Uint128> {
        if self.current_ts < self.stop_ramp_ts {
            let time_range = self.stop_ramp_ts.checked_sub(self.start_ramp_ts)?;
            let time_delta = self.current_ts.checked_sub(self.start_ramp_ts)?;
            // Compute amp factor based on ramp time
            if self.target_amp_factor >= self.initial_amp_factor {
                // Ramp up
                let amp_range = self.target_amp_factor.checked_sub(self.initial_amp_factor).unwrap();
                let amp_delta = amp_range.checked_mul(time_delta.into()).unwrap().checked_div(time_range.into()).unwrap();
                Some(self.initial_amp_factor.checked_add(amp_delta).unwrap())
            } else {
                // Ramp down
                let amp_range = self.initial_amp_factor.checked_sub(self.target_amp_factor).unwrap();
                let amp_delta = amp_range.checked_mul(time_delta.into()).unwrap().checked_div(time_range.into()).unwrap();
                Some(self.initial_amp_factor.checked_sub(amp_delta).unwrap())
            }
        } else {
            // When stop_ramp_ts is 0 or current_ts >= stop_ramp_ts
            Some(self.target_amp_factor)
        }
    }

    /// Compute stable swap invariant (D)
    /// Equation:
    /// A * sum(x_i) * n**n + D = A * D * n**n + D**(n+1) / (n**n * prod(x_i))
    pub fn compute_d(&self, c_amounts: &Vec<Uint128>) -> Option<Decimal256> {
        let n_coins: u32 = (c_amounts.len() as u32).into();
        let sum_x = Uint256::from(c_amounts.iter().fold(Uint128::zero(), |sum, i| sum + i));
        if sum_x.is_zero() {
            return Some(Decimal256::zero());
        } else {
            let amp_factor = self.compute_amp_factor()?;
            let mut d_prev;
            let mut d = Decimal256::from_uint256(sum_x);

            for _ in 0..256 {
                // $ D_{k,prod} = \frac{D_k^{n+1}}{n^n \prod x_{i}} = \frac{D^3}{4xy} $
                let mut d_prod: Decimal256 = d;
                for c_amount in c_amounts {
                    d_prod = (d_prod * d) / Decimal256::from_uint256(Uint256::from(*c_amount * Uint128::from(n_coins)));
                }
                d_prev = d;

                let ann = Uint256::from(amp_factor.checked_mul(n_coins.checked_pow(n_coins)?.into()).unwrap());
                let leverage = (Decimal256::from_uint256(sum_x)) * Decimal256::from_uint256(ann);
                // d = (ann * sum_x + d_prod * n_coins) * d_prev / ((ann - 1) * d_prev + (n_coins + 1) * d_prod)
                println!("ann: {}, sum_x: {}, d_prod: {}, n_coins: {}, d_prev: {}, leverage: {}", ann, sum_x, d_prod, n_coins, d_prev, leverage);
                let numerator = d_prev * (d_prod * Decimal256::from_uint256(Uint256::from(Uint128::from(n_coins))) + leverage);
                let denominator = d_prev * (Decimal256::from_uint256(ann) - Decimal256::one()) + (d_prod * (Decimal256::from_uint256(Uint256::from(Uint128::from(n_coins))) + Decimal256::one())).into();
                d = numerator / denominator;

                // Equality with the precision of 1
                if d > d_prev {
                    if d - d_prev < Decimal256::one() {
                        break;
                    }
                } else if d_prev - d < Decimal256::one() {
                    break;
                }
            }
            Some(d)
        }
    }

    /// Compute the amount of LP tokens to mint after providing liquidity
    /// return <lp_amount_to_mint, lp_fees_part>
    pub fn compute_lp_amount_for_deposit(
        &self,
        deposit_c_amounts: &Vec<Uint128>, // deposit tokens in comparable precision
        old_c_amounts: &Vec<Uint128>, // current in-pool tokens in comparable precision
        pool_token_supply: Uint128, // current share supply
        _fees: Uint128, // fees in decimal
    ) -> Option<(Uint128, Uint128)> {

        if pool_token_supply.is_zero() {
            let invariant = self.compute_d(deposit_c_amounts)? * Uint256::one();
            println!("invariant: {}", invariant);
            return Some((invariant.into(), Uint128::zero()));
        } else {
            let n_coins = old_c_amounts.len();
            // Initial invariant
            let d_0 = self.compute_d(old_c_amounts)?;
            let mut new_balances = vec![Uint128::zero(); n_coins];

            for (index, value) in deposit_c_amounts.iter().enumerate() {
                new_balances[index] = old_c_amounts[index].checked_add(*value).unwrap();
            }

            // Invariant after change
            let d_1 = self.compute_d(&new_balances)?;
            if d_1 <= d_0 {
                return None;
            } else {
                // Recalculate the invariant accounting for fees
                for i in 0..new_balances.len() {
                    let ideal_balance = d_1 * Decimal256::from_uint256(Uint256::from(old_c_amounts[i])) / d_0;
                    let difference = if ideal_balance > Decimal256::from_uint256(Uint256::from(new_balances[i])) {
                        ideal_balance - Decimal256::from_uint256(Uint256::from(new_balances[i]))
                    } else {
                        Decimal256::from_uint256(Uint256::from(new_balances[i])) - ideal_balance
                    };
                    // let fee = difference * Decimal256::from_uint256(Uint256::from(fees)) / Decimal256::from_uint256(Uint256::from(10000u128));
                    // new_balances[i] = new_balances[i] - fee;
                }

                let d_2 = self.compute_d(&new_balances)?;
                let mints_shares: Uint256 = (Decimal256::from_uint256(Uint256::from(pool_token_supply)) * (d_2 - d_0) / d_0) * Uint256::one();

                Some((mints_shares.into(), Uint128::zero()))
            }
        }
    }

    /// Compute the amount of LP tokens to burn after withdrawing liquidity
    /// given token_out user want get and total tokens in pool and lp token supply
    /// all amounts are in comparable precision
    pub fn compute_lp_amount_for_withdraw(
        &self,
        withdraw_c_amounts: &Vec<Uint128>, // withdraw tokens in comparable precision
        old_c_amounts: &Vec<Uint128>, // current in-pool tokens in comparable precision
        pool_token_supply: Uint128, // current share supply
        _fees: Uint128, // fees in decimal
    ) -> Option<(Uint128, Uint128)> {
        let n_coins = old_c_amounts.len();
        // Initial invariant, D0
        let d_0 = self.compute_d(old_c_amounts)?;

        // Real invariant after withdraw, D1
        let mut new_balances = vec![Uint128::zero(); n_coins];
        for (index, value) in withdraw_c_amounts.iter().enumerate() {
            new_balances[index] = old_c_amounts[index].checked_sub(*value).unwrap();
        }

        let d_1 = self.compute_d(&new_balances)?;

        // compare ideal token portion from D1 with withdraws, to calculate diff fee.
        if d_1 >= d_0 {
            None
        } else {
            // Recalculate the invariant accounting for fees
            for i in 0..new_balances.len() {
                let ideal_balance = d_1 * Decimal256::from_uint256(Uint256::from(old_c_amounts[i])) / d_0;
                let difference = if ideal_balance > Decimal256::from_uint256(Uint256::from(new_balances[i])) {
                    ideal_balance - Decimal256::from_uint256(Uint256::from(new_balances[i]))
                } else {
                    Decimal256::from_uint256(Uint256::from(new_balances[i])) - ideal_balance
                };
                // let fee = difference * Decimal256::from_uint256(Uint256::from(fees)) / Decimal256::from_uint256(Uint256::from(10000u128));
                // new_balances[i] = new_balances[i] - fee;
            }

            let d_2 = self.compute_d(&new_balances)?;

            // d0 > d1 > d2
            // (d0 - d2) => burn_shares (plus fee),
            // (d0 - d1) => burn_shares (without fee),
            // (d1 - d2) => fee part,
            // burn_shares = diff_shares + fee part
            let burn_shares: Uint256 = (Decimal256::from_uint256(Uint256::from(pool_token_supply)) * (d_0 - d_2) / d_0) * Uint256::one();
            let diff_shares: Uint256 = (Decimal256::from_uint256(Uint256::from(pool_token_supply)) * (d_0 - d_1) / d_0) * Uint256::one();

            Some((burn_shares.into(), (burn_shares - diff_shares).into()))
        }

    }


}
