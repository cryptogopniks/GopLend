pub const DECIMALS_DEFAULT: u8 = 6;

pub const MINS_PER_DAY: u128 = 24 * 60;
pub const NANOS_PER_MIN: u128 = 60 * 1_000_000_000;

pub const P6: u128 = 1_000_000; // 1 asset with 6 decimals
pub const P12: u128 = P6.pow(2); // 1_000_000 of assets with 6 decimals
pub const P18: u128 = P6 * P12; // 1 of asset with 18 decimals
pub const P24: u128 = P12.pow(2); // 1_000_000 of assets with 18 decimals
