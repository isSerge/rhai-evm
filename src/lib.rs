#![doc = include_str!("../README.md")]

//! EVM token denomination helpers, hashing, address utilities, and primitive
//! type conversions for Rhai.
//!
//! Provides [`EvmPackage`] (via `def_package!`) which exports:
//! - Denomination constructors: `ether`, `gwei`, `wei`, `usdc`, `usdt`,
//!   `wbtc`, and the generic `decimals`.
//! - Hashing: `keccak256`.
//! - Address utilities: `is_address`, `to_checksum`.
//!
//! Also exposes [`u256_to_bigint_dynamic`] and [`i256_to_bigint_dynamic`] for
//! host-side conversion of `alloy-primitives` types into Rhai `Dynamic` values.
//!
//! > **Note:** `EvmPackage` does not bundle `rhai-bigint`. Register
//! > `BigIntPackage` alongside `EvmPackage` in your engine.

use alloy_primitives::{I256, Sign as AlloySign, U256};
use num_bigint::{BigInt, Sign as BigIntSign};
use rhai::{Dynamic, def_package, plugin::*};
use rust_decimal::prelude::*;

/// Converts a `U256` value unconditionally to a Rhai `BigInt` dynamic type.
pub fn u256_to_bigint_dynamic(value: U256) -> Dynamic {
    let (sign, bytes) = if value.is_zero() {
        (BigIntSign::NoSign, vec![])
    } else {
        (BigIntSign::Plus, value.to_be_bytes_vec())
    };
    Dynamic::from(BigInt::from_bytes_be(sign, &bytes))
}

/// Converts an `I256` value unconditionally to a Rhai `BigInt` dynamic type.
pub fn i256_to_bigint_dynamic(value: I256) -> Dynamic {
    let (sign, abs_value) = value.into_sign_and_abs();
    let rhai_sign = match sign {
        AlloySign::Positive => BigIntSign::Plus,
        AlloySign::Negative => BigIntSign::Minus,
    };
    let bytes = abs_value.to_be_bytes_vec();
    Dynamic::from(BigInt::from_bytes_be(rhai_sign, &bytes))
}

fn decimals(value: Dynamic, decimals: i64) -> Result<BigInt, Box<rhai::EvalAltResult>> {
    if decimals >= 0 {
        scale_by_decimals(value, decimals as u32)
    } else {
        Err("Decimals must be non-negative".into())
    }
}

fn ether(value: Dynamic) -> Result<BigInt, Box<rhai::EvalAltResult>> {
    scale_by_decimals(value, 18)
}

fn gwei(value: Dynamic) -> Result<BigInt, Box<rhai::EvalAltResult>> {
    scale_by_decimals(value, 9)
}

fn wei(value: Dynamic) -> Result<BigInt, Box<rhai::EvalAltResult>> {
    scale_by_decimals(value, 0)
}

fn usdc(value: Dynamic) -> Result<BigInt, Box<rhai::EvalAltResult>> {
    scale_by_decimals(value, 6)
}

fn usdt(value: Dynamic) -> Result<BigInt, Box<rhai::EvalAltResult>> {
    scale_by_decimals(value, 6)
}

fn wbtc(value: Dynamic) -> Result<BigInt, Box<rhai::EvalAltResult>> {
    scale_by_decimals(value, 8)
}

/// Scale a Rhai Dynamic value by a given number of decimal places and return a
/// `BigInt`.
fn scale_by_decimals(value: Dynamic, decimals: u32) -> Result<BigInt, Box<rhai::EvalAltResult>> {
    let decimal = dynamic_to_decimal(value)?;
    let multiplier = Decimal::from_u64(10).unwrap().powi(decimals.into());
    let scaled = decimal * multiplier;
    let truncated = scaled.trunc();

    BigInt::from_str(truncated.to_string().as_str())
        .map_err(|_| "Failed to convert Decimal to BigInt".into())
}

/// Convert a Rhai Dynamic value to a Decimal.
fn dynamic_to_decimal(value: Dynamic) -> Result<Decimal, Box<rhai::EvalAltResult>> {
    // Value is Decimal
    if value.is::<Decimal>() {
        return Ok(value.clone_cast::<Decimal>());
    }

    // To convert other types use their string representation:

    // Handle BigInt specifically
    if value.is::<BigInt>() {
        let bigint_value = value.clone_cast::<BigInt>();
        return Decimal::from_str(&bigint_value.to_string()).map_err(|_| {
            format!(
                "Failed to convert BigInt value '{}' to Decimal",
                bigint_value
            )
            .into()
        });
    }

    // Rest numeric type (i32, i64, f64) or string
    let string_repr = value.to_string();
    Decimal::from_str(&string_repr).map_err(|_| {
        format!(
            "Cannot convert value of type {:?} ('{}') to Decimal",
            value.type_name(),
            string_repr
        )
        .into()
    })
}

/// Rhai-registered EVM denomination functions — delegates to the private
/// helpers above so tests can call them directly via `use super::*`.
#[export_module]
mod evm_functions {
    use num_bigint::BigInt;
    use rhai::{Dynamic, EvalAltResult};

    /// Scales a value by a given number of decimal places, returning a `BigInt`.
    ///
    /// The `value` argument can be an integer, float, or string. `d` must be
    /// non-negative.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let amount = decimals(1.23, 2); // 123
    /// ```
    #[rhai_fn(return_raw)]
    pub fn decimals(value: Dynamic, d: i64) -> Result<BigInt, Box<EvalAltResult>> {
        super::decimals(value, d)
    }

    /// Converts a value denominated in ether to its wei equivalent (×10¹⁸).
    ///
    /// The `value` argument can be an integer, float, or string.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let one_ether_in_wei = ether(1);   // 1000000000000000000
    /// let half_ether      = ether(0.5);  // 500000000000000000
    /// ```
    #[rhai_fn(return_raw)]
    pub fn ether(value: Dynamic) -> Result<BigInt, Box<EvalAltResult>> {
        super::ether(value)
    }

    /// Converts a value denominated in gwei to its wei equivalent (×10⁹).
    ///
    /// The `value` argument can be an integer, float, or string.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let one_gwei_in_wei = gwei(1); // 1000000000
    /// ```
    #[rhai_fn(return_raw)]
    pub fn gwei(value: Dynamic) -> Result<BigInt, Box<EvalAltResult>> {
        super::gwei(value)
    }

    /// Parses a value as a whole number of wei, truncating any fractional part.
    ///
    /// The `value` argument can be an integer, float, or string.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let amount = wei(1.9); // 1
    /// ```
    #[rhai_fn(return_raw)]
    pub fn wei(value: Dynamic) -> Result<BigInt, Box<EvalAltResult>> {
        super::wei(value)
    }

    /// Converts a value denominated in USDC to its atomic unit equivalent (×10⁶).
    ///
    /// The `value` argument can be an integer, float, or string.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let one_usdc = usdc(1); // 1000000
    /// ```
    #[rhai_fn(return_raw)]
    pub fn usdc(value: Dynamic) -> Result<BigInt, Box<EvalAltResult>> {
        super::usdc(value)
    }

    /// Converts a value denominated in USDT to its atomic unit equivalent (×10⁶).
    ///
    /// The `value` argument can be an integer, float, or string.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let one_usdt = usdt(1); // 1000000
    /// ```
    #[rhai_fn(return_raw)]
    pub fn usdt(value: Dynamic) -> Result<BigInt, Box<EvalAltResult>> {
        super::usdt(value)
    }

    /// Converts a value denominated in WBTC to its atomic unit equivalent (×10⁸).
    ///
    /// The `value` argument can be an integer, float, or string.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let one_wbtc = wbtc(1); // 100000000
    /// ```
    #[rhai_fn(return_raw)]
    pub fn wbtc(value: Dynamic) -> Result<BigInt, Box<EvalAltResult>> {
        super::wbtc(value)
    }

    /// Returns the Keccak-256 hash of a UTF-8 string as a `0x`-prefixed hex string.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let hash = keccak256("hello"); // "0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8"
    /// ```
    #[rhai_fn(name = "keccak256")]
    pub fn keccak256_str(value: String) -> String {
        alloy_primitives::keccak256(value.as_bytes()).to_string()
    }

    /// Returns `true` if the string is a valid EVM address (with or without checksum).
    ///
    /// # Example
    ///
    /// ```rhai
    /// is_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")  // true
    /// is_address("not-an-address")                              // false
    /// ```
    #[rhai_fn(name = "is_address")]
    pub fn is_address(value: String) -> bool {
        value.parse::<alloy_primitives::Address>().is_ok()
    }

    /// Parses an EVM address string and returns its EIP-55 checksum form.
    ///
    /// Returns an error if the input is not a valid EVM address.
    ///
    /// # Example
    ///
    /// ```rhai
    /// let addr = to_checksum("0xd8da6bf26964af9d7eed9e03e53415d37aa96045");
    /// // "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
    /// ```
    #[rhai_fn(name = "to_checksum", return_raw)]
    pub fn to_checksum(value: String) -> Result<String, Box<rhai::EvalAltResult>> {
        value
            .parse::<alloy_primitives::Address>()
            .map(|a| a.to_checksum(None))
            .map_err(|_| format!("Invalid EVM address: '{value}'").into())
    }
}

def_package! {
    /// Rhai package bundling all EVM scripting utilities.
    ///
    /// Exports denomination constructors (`ether`, `gwei`, `wei`, `usdc`,
    /// `usdt`, `wbtc`, `decimals`), the `keccak256` hash function, and address
    /// helpers (`is_address`, `to_checksum`).
    ///
    /// Register this alongside [`rhai_bigint::BigIntPackage`] for a complete
    /// EVM scripting environment.
    pub EvmPackage(lib) {
        combine_with_exported_module!(lib, "evm", evm_functions);
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::U256;

    use super::*;

    #[test]
    fn test_keccak256() {
        // keccak256("") is a well-known constant
        let empty = evm_functions::keccak256_str(String::new());
        assert_eq!(
            empty,
            "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );

        // keccak256("hello")
        let hello = evm_functions::keccak256_str("hello".to_string());
        assert_eq!(
            hello,
            "0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8"
        );
    }

    #[test]
    fn test_is_address() {
        assert!(evm_functions::is_address(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string()
        ));
        // lowercase (no checksum) is also valid
        assert!(evm_functions::is_address(
            "0xd8da6bf26964af9d7eed9e03e53415d37aa96045".to_string()
        ));
        assert!(!evm_functions::is_address("not-an-address".to_string()));
        assert!(!evm_functions::is_address(String::new()));
    }

    #[test]
    fn test_to_checksum() {
        let checksummed =
            evm_functions::to_checksum("0xd8da6bf26964af9d7eed9e03e53415d37aa96045".to_string())
                .unwrap();
        assert_eq!(checksummed, "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045");

        let err = evm_functions::to_checksum("not-an-address".to_string());
        assert!(err.is_err());
    }

    #[test]
    fn test_dynamic_to_decimal() {
        let decimal = Decimal::new(123456789, 6);
        let dynamic_decimal = Dynamic::from(decimal);
        let dynamic_int = Dynamic::from(42);
        let dynamic_float = Dynamic::from(3.14);
        let large_bigint_str = "1234567890123456789012345"; // large, but can fit in a Decimal
        let dynamic_bigint = Dynamic::from(BigInt::from_str(large_bigint_str).unwrap());
        let dynamic_wrong_type = Dynamic::from("not a number");

        let decimal_decimal = dynamic_to_decimal(dynamic_decimal).unwrap();
        let decimal_int = dynamic_to_decimal(dynamic_int).unwrap();
        let decimal_float = dynamic_to_decimal(dynamic_float).unwrap();
        let decimal_bigint = dynamic_to_decimal(dynamic_bigint).unwrap();
        let decimal_wrong_type = dynamic_to_decimal(dynamic_wrong_type);

        assert_eq!(decimal_decimal, decimal);
        assert_eq!(decimal_int, Decimal::from(42));
        assert_eq!(decimal_float, Decimal::from_f64(3.14).unwrap());
        assert_eq!(
            decimal_bigint,
            Decimal::from_str(&large_bigint_str).unwrap()
        );
        assert!(decimal_wrong_type.is_err());
    }

    #[test]
    fn test_scale_by_decimals() {
        let dynamic_value = Dynamic::from(42);
        let scaled_value = scale_by_decimals(dynamic_value, 6).unwrap();
        assert_eq!(scaled_value, BigInt::from(42000000));

        let dynamic_value_float = Dynamic::from(3.14);
        let scaled_value_float = scale_by_decimals(dynamic_value_float, 6).unwrap();
        assert_eq!(scaled_value_float, BigInt::from(3140000));

        let dynamic_value_bigint = u256_to_bigint_dynamic(U256::from(1000));
        let scaled_value_bigint = scale_by_decimals(dynamic_value_bigint, 3).unwrap();
        assert_eq!(scaled_value_bigint, BigInt::from(1000000));
    }

    #[test]
    fn test_scale_by_decimals_truncation() {
        let dynamic_value = Dynamic::from(3.141592653589793);
        let scaled_value = scale_by_decimals(dynamic_value, 2).unwrap();
        assert_eq!(scaled_value, BigInt::from(314)); // Should truncate to 3.14
    }

    #[test]
    fn test_decimals_constructor_decimals_should_be_positive() {
        let dynamic_value = Dynamic::from(42);
        let neg_decimals = -6; // should be positive
        let result = decimals(dynamic_value, neg_decimals);
        assert!(result.is_err());
    }

    #[test]
    fn test_wei_constructor() {
        let dynamic_value = Dynamic::from(1.2);
        let wei_value = wei(dynamic_value).unwrap();
        assert_eq!(wei_value, BigInt::from(1)); // 1.2 wei should scale to 1 wei
    }

    #[test]
    fn test_gwei_constructor() {
        let dynamic_float = Dynamic::from(1.5);
        let dynamic_integer = Dynamic::from(2);
        let dynamic_string = Dynamic::from("3.0");
        let gwei_float = gwei(dynamic_float).unwrap();
        let gwei_integer = gwei(dynamic_integer).unwrap();
        let gwei_string = gwei(dynamic_string).unwrap();
        assert_eq!(gwei_float, BigInt::from(1500000000u64)); // 1.5 gwei should scale to 1.5 * 10^9 wei
        assert_eq!(gwei_integer, BigInt::from(2000000000u64)); // 2 gwei should scale to 2 * 10^9 wei
        assert_eq!(gwei_string, BigInt::from(3000000000u64)); // 3.0 gwei should scale to 3.0 * 10^9 wei
    }

    #[test]
    fn test_ether_constructor() {
        let dynamic_float = Dynamic::from(1.5);
        let dynamic_integer = Dynamic::from(2);
        let dynamic_string = Dynamic::from("3.0");
        let ether_float = ether(dynamic_float).unwrap();
        let ether_integer = ether(dynamic_integer).unwrap();
        let ether_string = ether(dynamic_string).unwrap();
        assert_eq!(ether_float, BigInt::from(1500000000000000000u64)); // 1.5 ether should scale to 1.5 * 10^18 wei
        assert_eq!(ether_integer, BigInt::from(2000000000000000000u64)); // 2 ether should scale to 2 * 10^18 wei
        assert_eq!(ether_string, BigInt::from(3000000000000000000u64)); // 3.0 ether should scale to 3.0 * 10^18 wei
    }

    #[test]
    fn test_usdc_constructor() {
        let dynamic_float = Dynamic::from(1.2345);
        let dynamic_integer = Dynamic::from(1000000000); // one billion
        let dynamic_string = Dynamic::from("1.2345");
        let usdc_float = usdc(dynamic_float).unwrap();
        let usdc_integer = usdc(dynamic_integer).unwrap();
        let usdc_string = usdc(dynamic_string).unwrap();
        assert_eq!(usdc_float, BigInt::from(1234500)); // 1.2345 USDC should scale to 1.2345 * 10^6
        assert_eq!(usdc_integer, BigInt::from(1000000000000000_i64)); // 1000000000 USDC should scale to 1000000000 * 10^6
        assert_eq!(usdc_string, BigInt::from(1234500)); // "1.2345" USDC should scale to 1.2345 * 10^6
    }

    #[test]
    fn test_usdt_constructor() {
        let dynamic_float = Dynamic::from(1.2345);
        let dynamic_integer = Dynamic::from(1000000000); // one billion
        let dynamic_string = Dynamic::from("1.2345");
        let usdt_float = usdt(dynamic_float).unwrap();
        let usdt_integer = usdt(dynamic_integer).unwrap();
        let usdt_string = usdt(dynamic_string).unwrap();
        assert_eq!(usdt_float, BigInt::from(1234500)); // 1.2345 USDT should scale to 1.2345 * 10^6
        assert_eq!(usdt_integer, BigInt::from(1000000000000000_i64)); // 1000000000 USDT should scale to 1000000000 * 10^6
        assert_eq!(usdt_string, BigInt::from(1234500)); // "1.2345" USDT should scale to 1.2345 * 10^6
    }

    #[test]
    fn test_wbtc_constructor() {
        let dynamic_float = Dynamic::from(0.12345678);
        let dynamic_integer = Dynamic::from(100000000); // one hundred million
        let dynamic_string = Dynamic::from("0.12345678");
        let wbtc_float = wbtc(dynamic_float).unwrap();
        let wbtc_integer = wbtc(dynamic_integer).unwrap();
        let wbtc_string = wbtc(dynamic_string).unwrap();
        assert_eq!(wbtc_float, BigInt::from(12345678)); // 0.12345678 WBTC should scale to 0.12345678 * 10^8
        assert_eq!(wbtc_integer, BigInt::from(10000000000000000_i64)); // 100000000 WBTC should scale to 100000000 * 10^8
        assert_eq!(wbtc_string, BigInt::from(12345678)); // "0.12345678" WBTC should scale to 0.12345678 * 10^8
    }
}
