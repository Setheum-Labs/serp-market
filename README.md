# SERP - Market
## Setheum Elastic Reserve Protocol - Market
SERP-Market Stablecoin SERP Module based on `Stp258Standard` built on top of `Stp258Serp` and `Stp258Traits`.

## Overview

This is the `SerpMarket` Pallet that trades with the SERP system 
to make trades for Nativecurrency in this case called Dinar, and Sett-Currencies(Multiple stablecoins).

 ### Implementations

The SERP-Market expands or contract supply/issuance by deposit creating and slashing to and from the accounts associated with the SerpMarket.
It implements the following trait.

 - `SerpMarket` - Abstraction over a stablecoin stability system based on the DS3 (Dinar-Sett Stability System) on the SERP.
 The trait implements `expand_supply` and `contract_supply` SERP functions.
 
The `SerpMarket` module depends on the `Stp258-traits` and `Stp258-currencies` modules for the currencies in to adjust the stablecoin supply.

This module is based on the [STP-258 Standard](https://github.com/Setheum-Labs/stp258-standard) built on the [STP-258 Serp](https://github.com/Setheum-Labs/stp258-serp) implementing the [STP-258 Traits](https://github.com/Setheum-Labs/stp258-traits).
