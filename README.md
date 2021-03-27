# SERP - Market
## Setheum Elastic Reserve Protocol - Market
SERP-Market Stablecoin SERP Module based on `Stp258Standard` built on top of `Stp258Serp` and `SerpTraits`.

## Overview

This is the `SerpMarket` Pallet that trades with the SERP system 
to make trades for Nativecurrency in this case called Dinar, and Sett-Currencies(Multiple stablecoins).

 ### Implementations

The SERP-Market expands or contract supply/issuance by deposit creating and slashing to and from the accounts associated with the SerpMarket.
It implements the following trait.

 - `SerpMarket` - Abstraction over a stablecoin stability system based on the DS3 (Dinar-Sett Stability System) on the SERP.
 The trait implements `expand_supply` and `contract_supply` SERP functions.
 
The `SerpMarket` module depends on the `Serp-traits` and `Stp258-currencies` modules for the currencies in to adjust the stablecoin supply.

This module is based on the [STP-258 Standard](https://github.com/Setheum-Labs/stp258-standard) built on the [STP-258 SERP](https://github.com/Setheum-Labs/stp258-serp) implementing the [SERP Traits](https://github.com/Setheum-Labs/serp-traits).
 
## Test & Build

Run `cargo build` to build.
Run `cargo test` to test.

    build:

    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v2
    - name: Install toolchain
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: nightly-2021-03-05
        target: wasm32-unknown-unknown
        default: true
    - name: Install Wasm toolchain
      run: rustup target add wasm32-unknown-unknown
    - name: Install clippy
      run: rustup component add clippy
    - name: Build
      run: cargo build --verbose
    - name: Run tests
      run: cargo test --verbose
