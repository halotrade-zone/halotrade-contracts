use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::convert::TryFrom;
use std::fmt::{self, Write};
use std::ops;
use std::str::FromStr;

use bigint::U256;
use cosmwasm_std::{Decimal, StdError, Uint128};

/// A fixed-point decimal value with 18 fractional digits, i.e. Decimal256(1_000_000_000_000_000_000) == 1.0
/// The greatest possible value that can be represented is 115792089237316195423570985008687907853269984665640564039457.584007913129639935 (which is (2^128 - 1) / 10^18)
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Decimal256(#[schemars(with = "String")] pub U256);

impl Decimal256 {
    pub const MAX: Decimal256 = Decimal256(U256::MAX);
    pub const DECIMAL_FRACTIONAL: U256 = U256([1_000_000_000_000_000_000u64, 0, 0, 0]);

    /// Create a 1.0 Decimal256
    pub const fn one() -> Decimal256 {
        Decimal256(Decimal256::DECIMAL_FRACTIONAL)
    }

    /// Create a 0.0 Decimal256
    pub const fn zero() -> Decimal256 {
        Decimal256(U256([0, 0, 0, 0]))
    }

    /// Convert x% into Decimal256
    pub fn percent(x: u64) -> Decimal256 {
        Decimal256(U256::from(x) * U256::from(10_000_000_000_000_000u64))
    }

    /// Convert permille (x/1000) into Decimal256
    pub fn permille(x: u64) -> Decimal256 {
        Decimal256(U256::from(x) * U256::from(1_000_000_000_000_000u64))
    }

    /// Returns the ratio (nominator / denominator) as a Decimal256
    pub fn from_ratio<A: Into<U256>, B: Into<U256>>(nominator: A, denominator: B) -> Decimal256 {
        let nominator: U256 = nominator.into();
        let denominator: U256 = denominator.into();
        if denominator.is_zero() {
            panic!("Denominator must not be zero");
        }

        Decimal256(nominator * Decimal256::DECIMAL_FRACTIONAL / denominator)
    }

    pub fn from_uint256<A: Into<Uint256>>(val: A) -> Decimal256 {
        let num: Uint256 = val.into();
        Decimal256(num.0 * Decimal256::DECIMAL_FRACTIONAL)
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl From<Decimal> for Decimal256 {
    fn from(val: Decimal) -> Self {
        Decimal256::from_str(&val.to_string()).unwrap()
    }
}

impl From<Decimal256> for Decimal {
    fn from(n: Decimal256) -> Self {
        let U256(ref arr) = n.0;
        assert!(arr[2] == 0u64);
        assert!(arr[3] == 0u64);
        Decimal::from_str(&n.to_string()).unwrap()
    }
}

impl FromStr for Decimal256 {
    type Err = StdError;

    /// Converts the decimal string to a Decimal256
    /// Possible inputs: "1.23", "1", "000012", "1.123000000"
    /// Disallowed: "", ".23"
    ///
    /// This never performs any kind of rounding.
    /// More than 18 fractional digits, even zeros, result in an error.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = input.split('.').collect();
        match parts.len() {
            1 => {
                let whole = U256::from_dec_str(parts[0])
                    .map_err(|_| StdError::generic_err("Error parsing whole"))?;

                let whole_as_atomics = whole * Decimal256::DECIMAL_FRACTIONAL;
                Ok(Decimal256(whole_as_atomics))
            }
            2 => {
                let whole = U256::from_dec_str(parts[0])
                    .map_err(|_| StdError::generic_err("Error parsing whole"))?;
                let fractional = U256::from_dec_str(parts[1])
                    .map_err(|_| StdError::generic_err("Error parsing fractional"))?;
                let exp = (18usize.checked_sub(parts[1].len())).ok_or_else(|| {
                    StdError::generic_err("Cannot parse more than 18 fractional digits")
                })?;
                let fractional_factor = U256::from(10).pow(exp.into());

                let whole_as_atomics = whole * Decimal256::DECIMAL_FRACTIONAL;
                let atomics = whole_as_atomics + fractional * fractional_factor;
                Ok(Decimal256(atomics))
            }
            _ => Err(StdError::generic_err("Unexpected number of dots")),
        }
    }
}

impl fmt::Display for Decimal256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let whole = (self.0) / Decimal256::DECIMAL_FRACTIONAL;
        let fractional = (self.0) % Decimal256::DECIMAL_FRACTIONAL;

        if fractional.is_zero() {
            write!(f, "{}", whole)
        } else {
            let fractional_string = fractional.to_string();
            let fractional_string = "0".repeat(18 - fractional_string.len()) + &fractional_string;

            f.write_str(&whole.to_string())?;
            f.write_char('.')?;
            f.write_str(fractional_string.trim_end_matches('0'))?;

            Ok(())
        }
    }
}

impl ops::Add for Decimal256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Decimal256(self.0 + rhs.0)
    }
}

impl ops::AddAssign for Decimal256 {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
    }
}

impl ops::Sub for Decimal256 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        assert!(self.0 >= rhs.0);
        Decimal256(self.0 - rhs.0)
    }
}

impl ops::Mul for Decimal256 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Decimal256(self.0 * rhs.0 / Decimal256::DECIMAL_FRACTIONAL)
    }
}

impl ops::Div for Decimal256 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        assert!(!rhs.is_zero());

        Decimal256(self.0 * Decimal256::DECIMAL_FRACTIONAL / rhs.0)
    }
}

/// Serializes as a decimal string
impl Serialize for Decimal256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for Decimal256 {
    fn deserialize<D>(deserializer: D) -> Result<Decimal256, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Decimal256Visitor)
    }
}

struct Decimal256Visitor;

impl<'de> de::Visitor<'de> for Decimal256Visitor {
    type Value = Decimal256;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Decimal256::from_str(v) {
            Ok(d) => Ok(d),
            Err(e) => Err(E::custom(format!("Error parsing decimal '{}': {}", v, e))),
        }
    }
}

//*** Uint256 ***/
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Uint256(#[schemars(with = "String")] pub U256);

impl Uint256 {
    /// Creates a Uint256(0)
    pub const fn zero() -> Self {
        Uint256(U256([0, 0, 0, 0]))
    }

    /// Create a 1.0 Decimal256
    pub const fn one() -> Self {
        Uint256(U256([1, 0, 0, 0]))
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl From<U256> for Uint256 {
    fn from(val: U256) -> Self {
        Uint256(val)
    }
}

impl From<Uint256> for U256 {
    fn from(val: Uint256) -> Self {
        val.0
    }
}

#[inline(always)]
fn split_u128(a: u128) -> (u64, u64) {
    ((a >> 64) as _, (a & 0xFFFFFFFFFFFFFFFF) as _)
}

impl From<Uint128> for Uint256 {
    fn from(val: Uint128) -> Self {
        Uint256::from(val.u128())
    }
}

impl From<u128> for Uint256 {
    fn from(val: u128) -> Self {
        let (hi, low) = split_u128(val);
        Uint256(U256([low, hi, 0, 0]))
    }
}

impl From<u64> for Uint256 {
    fn from(val: u64) -> Self {
        Uint256(val.into())
    }
}

impl TryFrom<&str> for Uint256 {
    type Error = StdError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match U256::from_dec_str(val) {
            Ok(u) => Ok(Uint256(u)),
            Err(_e) => Err(StdError::generic_err(format!("invalid Uint256 '{}'", val))),
        }
    }
}

impl FromStr for Uint256 {
    type Err = StdError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let number =
            U256::from_dec_str(input).map_err(|_| StdError::generic_err("Error parsing number"))?;
        Ok(Uint256(number))
    }
}

impl From<Uint256> for String {
    fn from(n: Uint256) -> Self {
        n.0.to_string()
    }
}

impl From<Uint256> for u128 {
    fn from(n: Uint256) -> Self {
        let U256(ref arr) = n.0;
        assert!(arr[2] == 0u64);
        assert!(arr[3] == 0u64);

        let (hi, low) = (arr[1], arr[0]);
        ((hi as u128) << 64) + (low as u128)
    }
}

impl From<Uint256> for Uint128 {
    fn from(n: Uint256) -> Self {
        let num: u128 = n.into();
        Uint128::from(num)
    }
}

impl fmt::Display for Uint256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ops::Add for Uint256 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Uint256(self.0 + rhs.0)
    }
}

impl ops::AddAssign for Uint256 {
    fn add_assign(&mut self, other: Self) {
        self.0 = self.0 + other.0;
    }
}

impl ops::Sub for Uint256 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        assert!(self.0 >= rhs.0);
        Uint256(self.0 - rhs.0)
    }
}

impl ops::Mul<Uint256> for Uint256 {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Uint256) -> Self::Output {
        // 0*a and b*0 is always 0
        if self.is_zero() || rhs.is_zero() {
            return Uint256::zero();
        }

        Uint256(self.0 * rhs.0)
    }
}

/// Both d*u and u*d with d: Decimal256 and u: Uint256 returns an Uint256. There is no
/// specific reason for this decision other than the initial use cases we have. If you
/// need a Decimal256 result for the same calculation, use Decimal256(d*u) or Decimal256(u*d).
impl ops::Mul<Decimal256> for Uint256 {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Decimal256) -> Self::Output {
        // 0*a and b*0 is always 0
        if self.is_zero() || rhs.is_zero() {
            return Uint256::zero();
        }

        self.multiply_ratio(rhs.0, Decimal256::DECIMAL_FRACTIONAL)
    }
}

impl ops::Div<Decimal256> for Uint256 {
    type Output = Self;

    fn div(self, rhs: Decimal256) -> Self::Output {
        assert!(!rhs.is_zero());

        if self.is_zero() {
            return Uint256::zero();
        }

        self.multiply_ratio(Decimal256::DECIMAL_FRACTIONAL, rhs.0)
    }
}

impl ops::Mul<Uint256> for Decimal256 {
    type Output = Uint256;

    fn mul(self, rhs: Uint256) -> Self::Output {
        rhs * self
    }
}

impl Uint256 {
    /// returns self * nom / denom
    pub fn multiply_ratio<A: Into<U256>, B: Into<U256>>(&self, nom: A, denom: B) -> Uint256 {
        let nominator: U256 = nom.into();
        let denominator: U256 = denom.into();
        if denominator.is_zero() {
            panic!("Denominator must not be zero");
        }

        // TODO: minimize rounding that takes place (using gcd algorithm)
        let val = self.0 * nominator / denominator;
        Uint256::from(val)
    }
}

/// Serializes as a base64 string
impl Serialize for Uint256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for Uint256 {
    fn deserialize<D>(deserializer: D) -> Result<Uint256, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Uint256Visitor)
    }
}

struct Uint256Visitor;

impl<'de> de::Visitor<'de> for Uint256Visitor {
    type Value = Uint256;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match U256::from_dec_str(v) {
            Ok(u) => Ok(Uint256(u)),
            Err(_e) => Err(E::custom(format!("invalid Uint256 '{}'", v))),
        }
    }
}
