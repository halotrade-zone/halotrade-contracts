use bignumber::{Decimal256, Uint256};
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

impl AmpFactor {
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
                let amp_range = self
                    .target_amp_factor
                    .checked_sub(self.initial_amp_factor)
                    .unwrap();
                let amp_delta = amp_range
                    .checked_mul(time_delta.into())
                    .unwrap()
                    .checked_div(time_range.into())
                    .unwrap();
                Some(self.initial_amp_factor.checked_add(amp_delta).unwrap())
            } else {
                // Ramp down
                let amp_range = self
                    .initial_amp_factor
                    .checked_sub(self.target_amp_factor)
                    .unwrap();
                let amp_delta = amp_range
                    .checked_mul(time_delta.into())
                    .unwrap()
                    .checked_div(time_range.into())
                    .unwrap();
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
                    d_prod = (d_prod * d)
                        / Decimal256::from_uint256(Uint256::from(
                            *c_amount * Uint128::from(n_coins),
                        ));
                }
                d_prev = d;

                let ann = Uint256::from(
                    amp_factor
                        .checked_mul(n_coins.checked_pow(n_coins)?.into())
                        .unwrap(),
                );
                let leverage = (Decimal256::from_uint256(sum_x)) * Decimal256::from_uint256(ann);
                // d = (ann * sum_x + d_prod * n_coins) * d_prev / ((ann - 1) * d_prev + (n_coins + 1) * d_prod)
                let numerator = d_prev
                    * (d_prod * Decimal256::from_uint256(Uint256::from(Uint128::from(n_coins)))
                        + leverage);
                let denominator = d_prev * (Decimal256::from_uint256(ann) - Decimal256::one())
                    + (d_prod
                        * (Decimal256::from_uint256(Uint256::from(Uint128::from(n_coins)))
                            + Decimal256::one()))
                    .into();
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
        old_c_amounts: &Vec<Uint128>,     // current in-pool tokens in comparable precision
        pool_token_supply: Uint128,       // current share supply
        _fees: Uint128,                   // fees in decimal
    ) -> Option<(Uint128, Uint128)> {
        if pool_token_supply.is_zero() {
            let invariant = self.compute_d(deposit_c_amounts)? * Uint256::one();
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
                    let ideal_balance =
                        d_1 * Decimal256::from_uint256(Uint256::from(old_c_amounts[i])) / d_0;
                    let difference = if ideal_balance
                        > Decimal256::from_uint256(Uint256::from(new_balances[i]))
                    {
                        ideal_balance - Decimal256::from_uint256(Uint256::from(new_balances[i]))
                    } else {
                        Decimal256::from_uint256(Uint256::from(new_balances[i])) - ideal_balance
                    };
                    // let fee = difference * Decimal256::from_uint256(Uint256::from(fees)) / Decimal256::from_uint256(Uint256::from(10000u128));
                    // new_balances[i] = new_balances[i] - fee;
                }

                let d_2 = self.compute_d(&new_balances)?;
                let mints_shares: Uint256 =
                    (Decimal256::from_uint256(Uint256::from(pool_token_supply)) * (d_2 - d_0)
                        / d_0)
                        * Uint256::one();

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
        old_c_amounts: &Vec<Uint128>,      // current in-pool tokens in comparable precision
        pool_token_supply: Uint128,        // current share supply
        _fees: Uint128,                    // fees in decimal
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
                let ideal_balance =
                    d_1 * Decimal256::from_uint256(Uint256::from(old_c_amounts[i])) / d_0;
                let difference =
                    if ideal_balance > Decimal256::from_uint256(Uint256::from(new_balances[i])) {
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
            let burn_shares: Uint256 =
                (Decimal256::from_uint256(Uint256::from(pool_token_supply)) * (d_0 - d_2) / d_0)
                    * Uint256::one();
            let diff_shares: Uint256 =
                (Decimal256::from_uint256(Uint256::from(pool_token_supply)) * (d_0 - d_1) / d_0)
                    * Uint256::one();

            Some((burn_shares.into(), (burn_shares - diff_shares).into()))
        }
    }

    /// Compute new amount of token 'y' with new amount of token 'x'
    /// return new y_token amount according to the equation
    pub fn compute_y(
        &self,
        x_c_amount: Uint128, // new x_token amount in comparable precision,
        current_c_amounts: &Vec<Uint128>, // current in-pool tokens in comparable precision
        index_x: usize,      // index of x_token
        index_y: usize,      // index of y_token
    ) -> Option<Decimal256> {
        let n_coins: u32 = (current_c_amounts.len() as u32).into();
        let amp_factor = self.compute_amp_factor()?;
        let ann = Uint256::from(
            amp_factor
                .checked_mul(n_coins.checked_pow(n_coins)?.into())
                .unwrap(),
        );
        // invariant
        let d = self.compute_d(current_c_amounts)?;
        let mut s_ = Decimal256::from_uint256(Uint256::from(x_c_amount));
        let mut c = d * d / Decimal256::from_uint256(Uint256::from(x_c_amount));
        for (idx, c_amount) in current_c_amounts.iter().enumerate() {
            if idx != index_x && idx != index_y {
                s_ += Decimal256::from_uint256(Uint256::from(*c_amount));
                c = c * d / Decimal256::from_uint256(Uint256::from(*c_amount));
            }
        }
        c = c * d
            / Decimal256::from_uint256(
                ann * Uint256::from(Uint128::from(n_coins.checked_pow(n_coins)?)),
            );

        let b = d / Decimal256::from_uint256(Uint256::from(ann)) + s_;

        // Solve for y by approximating: y**2 + b*y = c
        let mut y_prev;
        let mut y = d;
        for _ in 0..256 {
            y_prev = y;
            // $ y_{k+1} = \frac{y_k^2 + c}{2y_k + b - D} $
            let y_numerator = y * y + c;
            let y_denominator = y * Decimal256::from_uint256(Uint256::from(2u128)) + b - d;
            y = y_numerator / y_denominator;
            if y > y_prev {
                if y - y_prev <= Decimal256::one() {
                    break;
                }
            } else if y_prev - y <= Decimal256::one() {
                break;
            }
        }
        Some(y.into())
    }

    /// Compute amount of token user will receive after swap
    pub fn swap_to(
        &self,
        token_in_idx: usize,              // index of token in token vector
        token_in_amount: Uint128,         // amount of token in comparable precision
        token_out_idx: usize,             // index of token out token vector
        current_c_amounts: &Vec<Uint128>, // current in-pool tokens in comparable precision
        _swap_fee: Decimal256,            // swap fee in decimal
    ) -> Option<Uint128> {
        let y: Uint256 = self
            .compute_y(
                token_in_amount + current_c_amounts[token_in_idx],
                current_c_amounts,
                token_in_idx,
                token_out_idx,
            )
            .unwrap()
            * Uint256::one();

        let dy: Uint128 = current_c_amounts[token_out_idx]
            .checked_sub(y.into())
            .unwrap()
            .checked_sub(Uint128::one())
            .unwrap_or(Uint128::zero());
        Some(dy)
    }
}
