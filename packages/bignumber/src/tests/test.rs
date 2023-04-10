use crate::math::{Decimal256, Uint256};
use bigint::U256;
use cosmwasm_std::StdError;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{from_slice, to_vec, StdResult};
    use std::convert::TryInto;

    #[test]
    fn decimal_one() {
        let value = Decimal256::one();
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL);
    }

    #[test]
    fn decimal_zero() {
        let value = Decimal256::zero();
        assert_eq!(value.0, U256::zero());
    }

    #[test]
    fn decimal_percent() {
        let value = Decimal256::percent(50);
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL / 2.into());
    }

    #[test]
    fn decimal_permille() {
        let value = Decimal256::permille(125);
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL / 8.into());
    }

    #[test]
    fn decimal_from_ratio_works() {
        // 1.0
        assert_eq!(Decimal256::from_ratio(1, 1), Decimal256::one());
        assert_eq!(Decimal256::from_ratio(53, 53), Decimal256::one());
        assert_eq!(Decimal256::from_ratio(125, 125), Decimal256::one());

        // 1.5
        assert_eq!(Decimal256::from_ratio(3, 2), Decimal256::percent(150));
        assert_eq!(Decimal256::from_ratio(150, 100), Decimal256::percent(150));
        assert_eq!(Decimal256::from_ratio(333, 222), Decimal256::percent(150));

        // 0.125
        assert_eq!(Decimal256::from_ratio(1, 8), Decimal256::permille(125));
        assert_eq!(Decimal256::from_ratio(125, 1000), Decimal256::permille(125));

        // 1/3 (result floored)
        assert_eq!(
            Decimal256::from_ratio(1, 3),
            Decimal256(333_333_333_333_333_333u64.into())
        );

        // 2/3 (result floored)
        assert_eq!(
            Decimal256::from_ratio(2, 3),
            Decimal256(666_666_666_666_666_666u64.into())
        );
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn decimal_from_ratio_panics_for_zero_denominator() {
        Decimal256::from_ratio(1, 0);
    }

    #[test]
    fn decimal_from_str_works() {
        // Integers
        assert_eq!(Decimal256::from_str("").unwrap(), Decimal256::percent(0));
        assert_eq!(Decimal256::from_str("0").unwrap(), Decimal256::percent(0));
        assert_eq!(Decimal256::from_str("1").unwrap(), Decimal256::percent(100));
        assert_eq!(Decimal256::from_str("5").unwrap(), Decimal256::percent(500));
        assert_eq!(
            Decimal256::from_str("42").unwrap(),
            Decimal256::percent(4200)
        );
        assert_eq!(Decimal256::from_str("000").unwrap(), Decimal256::percent(0));
        assert_eq!(
            Decimal256::from_str("001").unwrap(),
            Decimal256::percent(100)
        );
        assert_eq!(
            Decimal256::from_str("005").unwrap(),
            Decimal256::percent(500)
        );
        assert_eq!(
            Decimal256::from_str("0042").unwrap(),
            Decimal256::percent(4200)
        );

        // Decimal256s
        assert_eq!(
            Decimal256::from_str("1.").unwrap(),
            Decimal256::percent(100)
        );
        assert_eq!(
            Decimal256::from_str("1.0").unwrap(),
            Decimal256::percent(100)
        );
        assert_eq!(
            Decimal256::from_str("1.5").unwrap(),
            Decimal256::percent(150)
        );
        assert_eq!(
            Decimal256::from_str("0.5").unwrap(),
            Decimal256::percent(50)
        );
        assert_eq!(
            Decimal256::from_str("0.123").unwrap(),
            Decimal256::permille(123)
        );

        assert_eq!(
            Decimal256::from_str("40.00").unwrap(),
            Decimal256::percent(4000)
        );
        assert_eq!(
            Decimal256::from_str("04.00").unwrap(),
            Decimal256::percent(400)
        );
        assert_eq!(
            Decimal256::from_str("00.40").unwrap(),
            Decimal256::percent(40)
        );
        assert_eq!(
            Decimal256::from_str("00.04").unwrap(),
            Decimal256::percent(4)
        );

        // Can handle 18 fractional digits
        assert_eq!(
            Decimal256::from_str("7.123456789012345678").unwrap(),
            Decimal256(7123456789012345678u64.into())
        );
        assert_eq!(
            Decimal256::from_str("7.999999999999999999").unwrap(),
            Decimal256(7999999999999999999u64.into())
        );

        // Works for documented max value
        assert_eq!(
            Decimal256::from_str(
                "115792089237316195423570985008687907853269984665640564039457.584007913129639935"
            )
            .unwrap(),
            Decimal256::MAX
        );
    }

    #[test]
    fn decimal_from_str_errors_for_broken_whole_part() {
        match Decimal256::from_str(" ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("-1").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_broken_fractinal_part() {
        match Decimal256::from_str("1. ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("1.e").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("1.2e3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_more_than_18_fractional_digits() {
        match Decimal256::from_str("7.1234567890123456789").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 18 fractional digits")
            }
            e => panic!("Unexpected error: {:?}", e),
        }

        // No special rules for trailing zeros. This could be changed but adds gas cost for the happy path.
        match Decimal256::from_str("7.1230000000000000000").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 18 fractional digits")
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_invalid_number_of_dots() {
        match Decimal256::from_str("1.2.3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal256::from_str("1.2.3.4").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    #[should_panic(expected = "arithmetic operation overflow")]
    fn decimal_from_str_errors_for_more_than_max_value_integer_part() {
        let _ =
            Decimal256::from_str("115792089237316195423570985008687907853269984665640564039458");
    }

    #[test]
    #[should_panic(expected = "arithmetic operation overflow")]
    fn decimal_from_str_errors_for_more_than_max_value_integer_part_with_decimal() {
        let _ =
            Decimal256::from_str("115792089237316195423570985008687907853269984665640564039458.0");
    }
    #[test]
    #[should_panic(expected = "arithmetic operation overflow")]
    fn decimal_from_str_errors_for_more_than_max_value_decimal_part() {
        let _ = Decimal256::from_str(
            "115792089237316195423570985008687907853269984665640564039457.584007913129639936",
        );
    }

    #[test]
    fn decimal_is_zero_works() {
        assert!(Decimal256::zero().is_zero());
        assert!(Decimal256::percent(0).is_zero());
        assert!(Decimal256::permille(0).is_zero());

        assert!(!Decimal256::one().is_zero());
        assert!(!Decimal256::percent(123).is_zero());
        assert!(!Decimal256::permille(1234).is_zero());
    }

    #[test]
    fn decimal_add() {
        let value = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(
            value.0,
            Decimal256::DECIMAL_FRACTIONAL * U256::from(3) / U256::from(2)
        );
    }

    #[test]
    fn decimal_sub() {
        assert_eq!(
            Decimal256::percent(50),
            Decimal256::one() - Decimal256::percent(50)
        );
    }

    #[test]
    fn decimal_mul() {
        assert_eq!(
            Decimal256::percent(25),
            Decimal256::percent(50) * Decimal256::percent(50)
        );
    }

    #[test]
    fn decimal_div() {
        assert_eq!(
            Decimal256::one() + Decimal256::one(),
            Decimal256::percent(50) / Decimal256::percent(25)
        );
    }

    #[test]
    fn decimal_to_string() {
        // Integers
        assert_eq!(Decimal256::zero().to_string(), "0");
        assert_eq!(Decimal256::one().to_string(), "1");
        assert_eq!(Decimal256::percent(500).to_string(), "5");

        // Decimal256s
        assert_eq!(Decimal256::percent(125).to_string(), "1.25");
        assert_eq!(Decimal256::percent(42638).to_string(), "426.38");
        assert_eq!(Decimal256::percent(1).to_string(), "0.01");
        assert_eq!(Decimal256::permille(987).to_string(), "0.987");

        assert_eq!(Decimal256(1u64.into()).to_string(), "0.000000000000000001");
        assert_eq!(Decimal256(10u64.into()).to_string(), "0.00000000000000001");
        assert_eq!(Decimal256(100u64.into()).to_string(), "0.0000000000000001");
        assert_eq!(Decimal256(1000u64.into()).to_string(), "0.000000000000001");
        assert_eq!(Decimal256(10000u64.into()).to_string(), "0.00000000000001");
        assert_eq!(Decimal256(100000u64.into()).to_string(), "0.0000000000001");
        assert_eq!(Decimal256(1000000u64.into()).to_string(), "0.000000000001");
        assert_eq!(Decimal256(10000000u64.into()).to_string(), "0.00000000001");
        assert_eq!(Decimal256(100000000u64.into()).to_string(), "0.0000000001");
        assert_eq!(Decimal256(1000000000u64.into()).to_string(), "0.000000001");
        assert_eq!(Decimal256(10000000000u64.into()).to_string(), "0.00000001");
        assert_eq!(Decimal256(100000000000u64.into()).to_string(), "0.0000001");
        assert_eq!(Decimal256(10000000000000u64.into()).to_string(), "0.00001");
        assert_eq!(Decimal256(100000000000000u64.into()).to_string(), "0.0001");
        assert_eq!(Decimal256(1000000000000000u64.into()).to_string(), "0.001");
        assert_eq!(Decimal256(10000000000000000u64.into()).to_string(), "0.01");
        assert_eq!(Decimal256(100000000000000000u64.into()).to_string(), "0.1");
    }

    #[test]
    fn decimal_serialize() {
        assert_eq!(to_vec(&Decimal256::zero()).unwrap(), br#""0""#);
        assert_eq!(to_vec(&Decimal256::one()).unwrap(), br#""1""#);
        assert_eq!(to_vec(&Decimal256::percent(8)).unwrap(), br#""0.08""#);
        assert_eq!(to_vec(&Decimal256::percent(87)).unwrap(), br#""0.87""#);
        assert_eq!(to_vec(&Decimal256::percent(876)).unwrap(), br#""8.76""#);
        assert_eq!(to_vec(&Decimal256::percent(8765)).unwrap(), br#""87.65""#);
    }

    #[test]
    fn decimal_deserialize() {
        assert_eq!(
            from_slice::<Decimal256>(br#""0""#).unwrap(),
            Decimal256::zero()
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""1""#).unwrap(),
            Decimal256::one()
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""000""#).unwrap(),
            Decimal256::zero()
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""001""#).unwrap(),
            Decimal256::one()
        );

        assert_eq!(
            from_slice::<Decimal256>(br#""0.08""#).unwrap(),
            Decimal256::percent(8)
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""0.87""#).unwrap(),
            Decimal256::percent(87)
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""8.76""#).unwrap(),
            Decimal256::percent(876)
        );
        assert_eq!(
            from_slice::<Decimal256>(br#""87.65""#).unwrap(),
            Decimal256::percent(8765)
        );
    }

    #[test]
    fn to_and_from_uint256() {
        let a: Uint256 = 12345u64.into();
        assert_eq!(U256::from(12345), a.0);
        assert_eq!("12345", a.to_string());

        let a: Uint256 = "34567".try_into().unwrap();
        assert_eq!(U256::from(34567), a.0);
        assert_eq!("34567", a.to_string());

        let a: StdResult<Uint256> = "1.23".try_into();
        assert!(a.is_err());
    }

    #[test]
    fn uint256_is_zero_works() {
        assert!(Uint256::zero().is_zero());
        assert!(Uint256::from(0u64).is_zero());

        assert!(!Uint256::from(1u64).is_zero());
        assert!(!Uint256::from(123u64).is_zero());
    }

    #[test]
    fn uint256_json() {
        let orig = Uint256::from(1234567890987654321u64);
        let serialized = to_vec(&orig).unwrap();
        assert_eq!(serialized.as_slice(), b"\"1234567890987654321\"");
        let parsed: Uint256 = from_slice(&serialized).unwrap();
        assert_eq!(parsed, orig);
    }

    #[test]
    fn uint256_compare() {
        let a = Uint256::from(12345u64);
        let b = Uint256::from(23456u64);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, Uint256::from(12345u64));
    }

    #[test]
    fn uint256_math() {
        let a = Uint256::from(12345u64);
        let b = Uint256::from(23456u64);

        // test + and - for valid values
        assert_eq!(a + b, Uint256::from(35801u64));
        assert_eq!(b - a, Uint256::from(11111u64));

        // test +=
        let mut c = Uint256::from(300000u64);
        c += b;
        assert_eq!(c, Uint256::from(323456u64));
    }
    #[test]
    #[should_panic]
    fn uint256_math_sub_underflow() {
        let _ = Uint256::from(12345u64) - Uint256::from(23456u64);
    }

    #[test]
    #[should_panic]
    fn uint256_math_overflow_panics() {
        // almost_max is 2^256 - 10
        let almost_max = Uint256::from(U256([
            18446744073709551615,
            18446744073709551615,
            18446744073709551615,
            18446744073709551615,
        ]));
        let _ = almost_max + Uint256::from(12u64);
    }

    #[test]
    // in this test the Decimal256 is on the right
    fn uint256_decimal_multiply() {
        // a*b
        let left = Uint256::from(300u64);
        let right = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(left * right, Uint256::from(450u64));

        // a*0
        let left = Uint256::from(300u64);
        let right = Decimal256::zero();
        assert_eq!(left * right, Uint256::from(0u64));

        // 0*a
        let left = Uint256::zero();
        let right = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(left * right, Uint256::zero());
    }

    #[test]
    fn u256_multiply_ratio_works() {
        let base = Uint256::from(500u64);

        // factor 1/1
        assert_eq!(base.multiply_ratio(1, 1), Uint256::from(500u64));
        assert_eq!(base.multiply_ratio(3, 3), Uint256::from(500u64));
        assert_eq!(base.multiply_ratio(654321, 654321), Uint256::from(500u64));

        // factor 3/2
        assert_eq!(base.multiply_ratio(3, 2), Uint256::from(750u64));
        assert_eq!(base.multiply_ratio(333333, 222222), Uint256::from(750u64));

        // factor 2/3 (integer devision always floors the result)
        assert_eq!(base.multiply_ratio(2, 3), Uint256::from(333u64));
        assert_eq!(base.multiply_ratio(222222, 333333), Uint256::from(333u64));

        // factor 5/6 (integer devision always floors the result)
        assert_eq!(base.multiply_ratio(5, 6), Uint256::from(416u64));
        assert_eq!(base.multiply_ratio(100, 120), Uint256::from(416u64));
    }

    #[test]
    fn u256_from_u128() {
        assert_eq!(Uint256::from(100u64), Uint256::from(100u128));
        let num = Uint256::from(1_000_000_000_000_000_000_000_000u128);
        assert_eq!(num.to_string(), "1000000000000000000000000");
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn u256_multiply_ratio_panics_for_zero_denominator() {
        Uint256::from(500u64).multiply_ratio(1, 0);
    }

    #[test]
    fn u256_zero_one() {
        assert_eq!(Uint256::zero().0, U256::zero());
        assert_eq!(Uint256::one().0, U256::one());
    }

    #[test]
    fn u256_into_u128() {
        let val: u128 = Uint256::from(1234556700000000000999u128).into();
        assert_eq!(val, 1234556700000000000999u128);
    }

    #[test]
    #[should_panic]
    fn u256_into_u128_panics_for_overflow() {
        let _: u128 = Uint256::from_str("2134982317498312749832174923184732198471983247")
            .unwrap()
            .into();
    }

    #[test]
    // in this test the Decimal256 is on the left
    fn decimal_uint256_multiply() {
        // a*b
        let left = Decimal256::one() + Decimal256::percent(50); // 1.5
        let right = Uint256::from(300u64);
        assert_eq!(left * right, Uint256::from(450u64));

        // 0*a
        let left = Decimal256::zero();
        let right = Uint256::from(300u64);
        assert_eq!(left * right, Uint256::from(0u64));

        // a*0
        let left = Decimal256::one() + Decimal256::percent(50); // 1.5
        let right = Uint256::from(0u64);
        assert_eq!(left * right, Uint256::from(0u64));
    }
}
