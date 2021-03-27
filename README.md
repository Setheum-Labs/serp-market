# serp-market
SERP-Market Stablecoin Pallet

## Overview

This is the `SerpMarket` Pallet that trades with the SERP system 
to make trades for Nativecurrency in this case called Dinar, and Sett-Currencies(Multiple stablecoins).

The SERP-Market expands or contract supply/issuance by deposit creating and slashing to and from the accounts associated with the SerpMarket.
 
The `SerpMarket` module depends on the `Stp258-traits` and `Stp258-currencies` modules for the currencies in to adjust the stablecoin supply.

This module is based on the [STP-258 Standard](https://github.com/Setheum-Labs/stp258-standard) built on the [STP-258 Tokens](https://github.com/Setheum-Labs/stp258-tokens) implementing the [STP-258 Traits](https://github.com/Setheum-Labs/stp258-traits).
