# serp-market
SERP-Market Stablecoin Pallet

## Overview

This is the `SerpMarket` Pallet that trades with the SERP system 
to make trades for Nativecurrency in this case called Dinar, and Sett-Currencies(Multiple stablecoins).

The SERP-Market expands or contract supply/issuance by deposit creating and slashing to and from the accounts associated with the SerpMarket.
 
The `SerpMarket` module depends on the `Stp258-traits` and `Stp258-currencies` modules for the currencies in to adjust the stablecoin supply.

## Acknowledgement

This Pallet is inspired by the [Stablecoin](https://github.com/apopiak/stablecoin) Pallet originally developed by [Alexander Popiak](https://github.com/apopiak), for reference check [The Apopiak/Stablecoin Repo](https://github.com/apopiak/stablecoin.
