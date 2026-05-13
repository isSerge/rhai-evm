# rhai-evm

[![Crates.io](https://img.shields.io/crates/v/rhai-evm.svg)](https://crates.io/crates/rhai-evm)
[![Docs.rs](https://docs.rs/rhai-evm/badge.svg)](https://docs.rs/rhai-evm)
[![License](https://img.shields.io/crates/l/rhai-evm.svg)](https://crates.io/crates/rhai-evm)
[![Build Status](https://img.shields.io/github/actions/workflow/status/isSerge/rhai-evm/ci.yml?branch=main)](https://github.com/isSerge/rhai-evm/actions)

A [Rhai](https://rhai.rs/) scripting engine plugin providing EVM token denomination helpers and primitive type conversions between `alloy-primitives` and `num_bigint::BigInt`.

## Why this exists

Working with EVM-based applications requires handling high-precision integers (e.g., 1 ETH = $10^{18}$ Wei). Rhai's default integer types (64-bit) are often insufficient, and floating-point math leads to precision loss.

`rhai-evm` complements [rhai-bigint](https://crates.io/crates/rhai-bigint) by providing:
1.  **Denomination Helpers**: Ergonomic functions like `ether()`, `gwei()`, and `usdc()` to handle scaling and precision.
2.  **Type Conversions**: Lossless conversion from `alloy-primitives` types (`U256`, `I256`) into `BigInt` values.

## Features

- **Denomination Constructors**: `ether`, `gwei`, `wei`, `usdc`, `usdt`, `wbtc`, and generic `decimals`.
- **Hashing**: `keccak256(string)` returns a `0x`-prefixed hex string of the Keccak-256 digest.
- **Address Utilities**: `is_address` validates an address string; `to_checksum` returns its EIP-55 checksum form.
- **Alloy Interop**: Convert `U256` and `I256` directly to Rhai `Dynamic` values.

## Installation

### Add via Cargo

```bash
cargo add rhai-evm rhai-bigint
```

### Manual Configuration

Add the following to your `Cargo.toml`:

```toml
[dependencies]
rhai = "1.22.2"
rhai-bigint = "0.1.0"
rhai-evm = "0.1.0"
```

### Feature Flags

*   `sync`: Enables `rhai/sync` and `rhai-bigint/sync` support. Turn this on if your Rhai engine requires thread-safe types (e.g., when evaluating scripts across a Tokio thread pool).

## Usage

### 1. Registering the Package in Rust

Using the plugin requires registering both the `BigIntPackage` (which provides the math operations) and the `EvmPackage` (which provides the denomination helpers).

```rust
use rhai::{Engine, packages::Package};
use rhai_bigint::BigIntPackage;
use rhai_evm::EvmPackage;

fn main() {
    let mut engine = Engine::new();
    
    // Register BOTH packages into the engine
    BigIntPackage::new().register_into_engine(&mut engine);
    EvmPackage::new().register_into_engine(&mut engine);

    let script = r#"
        let price = ether(1.5); // 1.5 ETH in Wei
        let gas = gwei(30);     // 30 Gwei in Wei
        
        let threshold = parse_bigint("1000000000"); 

        price > gas && price > threshold
    "#;

    let result: bool = engine.eval(script).unwrap();
    assert!(result);
}
```

### 2. Scripting Examples

#### Denomination Helpers
```js
let amount = ether(1.23);      // 1230000000000000000
let small = gwei(10);          // 10000000000
let stable = usdc(500);        // 500000000
let custom = decimals(1.5, 2); // 150
```

#### Hashing
```js
let hash = keccak256("hello"); // "0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8"
```

#### Address Utilities
```js
is_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045") // true
is_address("not-an-address")                              // false

let addr = to_checksum("0xd8da6bf26964af9d7eed9e03e53415d37aa96045");
// "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
```

#### Alloy Interop (Host Side)
```rust
use alloy_primitives::U256;
use rhai::{Engine, Scope};
use rhai_evm::u256_to_bigint_dynamic;

let mut engine = Engine::new();
let mut scope = Scope::new();
let balance = U256::from(1000000000000000000u128);
scope.push("balance", u256_to_bigint_dynamic(balance));
```

## License

* MIT license (http://opensource.org/licenses/MIT)
