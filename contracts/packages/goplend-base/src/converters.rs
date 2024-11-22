use std::str::FromStr;

use cosmwasm_std::{Decimal, Decimal256, Uint128, Uint256};

pub fn str_to_dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

pub fn str_to_dec256(s: &str) -> Decimal256 {
    Decimal256::from_str(s).unwrap()
}

pub fn u128_to_dec<T>(num: T) -> Decimal
where
    Uint128: From<T>,
{
    Decimal::from_ratio(Uint128::from(num), Uint128::one())
}

pub fn u128_to_dec256<T>(num: T) -> Decimal256
where
    Uint128: From<T>,
{
    Decimal256::from_ratio(Uint128::from(num), Uint128::one())
}

pub fn u256_to_dec256<T>(num: T) -> Decimal256
where
    Uint256: From<T>,
{
    Decimal256::from_ratio(Uint256::from(num), Uint256::one())
}

pub fn u256_to_uint128(u256: impl Into<Uint256>) -> Uint128 {
    Uint128::try_from(Into::<Uint256>::into(u256)).unwrap()
}

pub fn dec_to_dec256(dec: Decimal) -> Decimal256 {
    Decimal256::from_str(&dec.to_string()).unwrap()
}

pub fn dec256_to_uint128(dec256: Decimal256) -> Uint128 {
    Uint128::try_from(dec256.to_uint_floor()).unwrap()
}
