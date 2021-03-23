//! Unit tests for the SerpMarket module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;


#[test]
fn expand_supply_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Tokens::reserve(DNAR, &SERPER, 100));
			assert_ok!(Stp258Tokens::reserve(JUSD, &SERPER, 100 * 1_000));
			assert_eq!(Stp258Tokens::reserved_balance(DNAR, &SERPER), 100);
			assert_eq!(Stp258Tokens::reserved_balance(JUSD, &SERPER), 100 * 1_000);
			assert_eq!(Stp258Tokens::total_issuance(DNAR), 400);
			assert_eq!(Stp258Tokens::total_issuance(JUSD), 400 * 1_000);
			assert_ok!(SerpMarket::expand_supply(DNAR, JUSD, 40 * 1_000, 2, &SERPER)); 
			assert_eq!(Stp258Tokens::reserved_balance(JUSD, &SERPER), 140 * 1_000);
			assert_eq!(Stp258Tokens::reserved_balance(DNAR, &SERPER), 98);
			assert_eq!(Stp258Tokens::total_issuance(JUSD), 440 * 1_000);
		});
}

#[test]
fn contract_supply_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob_n_serper_n_settpay()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Tokens::reserve(DNAR, &SERPER, 100));
			assert_ok!(Stp258Tokens::reserve(JUSD, &SERPER, 100 * 1_000));
			assert_eq!(Stp258Tokens::reserved_balance(DNAR, &SERPER), 100);
			assert_eq!(Stp258Tokens::reserved_balance(JUSD, &SERPER), 100 * 1_000);
			assert_eq!(Stp258Tokens::total_issuance(DNAR), 400);
			assert_eq!(Stp258Tokens::total_issuance(JUSD), 400 * 1_000);
			assert_ok!(SerpMarket::contract_supply(DNAR, JUSD, 40 * 1_000, 4, &SERPER)); 
			assert_eq!(Stp258Tokens::reserved_balance(JUSD, &SERPER), 60 * 1_000);
			assert_eq!(Stp258Tokens::reserved_balance(DNAR, &SERPER), 104);
			assert_eq!(Stp258Tokens::total_issuance(JUSD), 360 * 1_000);
			assert_eq!(Stp258Tokens::total_issuance(DNAR), 404);
		});
}
