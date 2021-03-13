//! Unit tests for the SerpMarket module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn get_stable_price_should_work() {
	assert_eq!(SerpMarket::get_stable_price(STP258_TOKEN_ID, 1.1) 1.1 * 1_000);
	assert_eq!(SerpMarket::get_stable_price(STP258_JCHF_ID, 1) 1 * 1_000);
	assert_eq!(SerpMarket::get_stable_price(STP258_JUSD_ID, 0.9) 0.9 * 1_000);
}

#[test]
fn get_relative_price_should_work() {
	assert_eq!(SerpMarket::get_price(STP258_TOKEN_ID, 1.3, STP258_NATIVE_ID, 0.065 * 1_000) 20);
	assert_eq!(SerpMarket::get_price(STP258_NATIVE_ID, 2_000, STP258_JCHF_ID, 1) 1);
	assert_eq!(SerpMarket::get_price(STP258_JUSD_ID, 1.1599, STP258_JCHF_ID, 0.92) 1.260);
}

#[test]
fn quote_serp_price_should_work() {
	assert_eq!(SerpMarket::get_price(1.1)1.3 * 1_000);
	assert_eq!(SerpMarket::get_price(1.3)1.9 * 1_000);
	assert_eq!(SerpMarket::get_price(0.9)1.1 * 1_000);
}

#[test]
fn expand_supply_should_work() {
	ExtBuilder::default()
		.five_hundred_thousand_for_sett_pay_n_serper()
		.build()
		.execute_with(|| {
			assert_ok!(SettCurrency::reserve(STP258_NATIVE_ID, &SERPER_ACC, 150_000));
			assert_eq!(Stp258Currencies::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 500_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 500_000 * 1_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 500_000 * 1_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 0);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 0 * 1_000);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 150_000);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 0 * 1_000);
			assert_ok!(SettCurrency::reserve(STP258_TOKEN_ID, &SERPER_ACC, 150_000 * 1_000));
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 150_000 * 1_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 350_000 * 1_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);

			assert_ok!(SerpMarket::expand_supply(STP258_TOKEN_ID, 100_000 * 1000)); 
			assert_eq!(Stp258Currencies::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 500_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 575_000 * 1_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 350_000 * 1_000);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 0);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 0 * 1_000);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 169_230.769);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 175_000 * 1_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_NATIVE_ID), 980_769.231);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_100_000 * 1_000);
		});
}

#[test]
fn contract_supply_should_work() {
	ExtBuilder::default()
		.five_hundred_thousand_for_sett_pay_n_serper()
		.build()
		.execute_with(|| {
			assert_ok!(SettCurrency::reserve(STP258_NATIVE_ID, &SERPER_ACC, 150_000));
			assert_eq!(Stp258Currencies::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 500_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 500_000 * 1_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 500_000 * 1_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 0);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 0 * 1_000);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 150_000);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 0 * 1_000);
			assert_ok!(SettCurrency::reserve(STP258_TOKEN_ID, &SERPER_ACC, 150_000 * 1_000));
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 150_000 * 1_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 350_000 * 1_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);

			assert_ok!(SerpMarket::contract_supply(STP258_TOKEN_ID, 10_000 * 1000)); 
			assert_eq!(Stp258Currencies::free_balance(STP258_NATIVE_ID, &SERPER_ACC), 350_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 500_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 500_000 * 1_000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &SERPER_ACC), 350_000 * 1_000);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_NATIVE_ID, &SETT_PAY_ACC), 0);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 0 * 1_000);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 170_600);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 140_000 * 1_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_NATIVE_ID), 980_769.231);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_100_000 * 1_000);
		});
}

#[test]
fn call_event_should_work() {
	ExtBuilder::default()
		.five_hundred_thousand_for_sett_pay_n_serper()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_eq!(SerpMarket::get_stable_price(STP258_TOKEN_ID, 1.1) 1.1 * 1_000);

			let transferred_event = Event::serp_market(crate::Event::NewPrice(STP258_TOKEN_ID,1.1 * 1_000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
		
			assert_ok!(SettCurrency::reserve(STP258_NATIVE_ID, &SERPER_ACC, 150_000));
			assert_ok!(SettCurrency::reserve(STP258_TOKEN_ID, &SERPER_ACC, 150_000 * 1_000));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 500_000 * 1_000);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 150_000);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 150_000 * 1_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_NATIVE_ID), 1_000_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_000_000 * 1_000);
			assert_ok!(SerpMarket::expand_supply(STP258_TOKEN_ID, 100_000 * 1000)); 
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &SETT_PAY_ACC), 575_000 * 1_000);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 169_230.769);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 175_000 * 1_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_NATIVE_ID), 980_769.231);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_100_000 * 1_000);

			let transferred_event = Event::serp_market(crate::Event::SerpedUpSupply(STP258_TOKEN_ID, 100_000 * 1000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
			
			let transferred_event = Event::serp_market(crate::Event::NewPrice(STP258_TOKEN_ID,1.3 * 1_000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));


			assert_ok!(SettCurrency::reserve(STP258_NATIVE_ID, &SERPER_ACC, 150_000));
			assert_eq!(Stp258Currencies::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 150_000);
			assert_ok!(SettCurrency::reserve(STP258_TOKEN_ID, &SERPER_ACC, 150_000 * 1_000));
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 150_000 * 1_000);
			assert_ok!(SerpMarket::contract_supply(STP258_TOKEN_ID, 10_000 * 1000)); 
			assert_eq!(Stp258Currencies::reserved_balance(STP258_NATIVE_ID, &SERPER_ACC), 170_600);
			assert_eq!(Stp258Currencies::reserved_balance(STP258_TOKEN_ID, &SERPER_ACC), 140_000 * 1_000);
			assert_eq!(Stp258Tokens::total_issuance(STP258_NATIVE_ID), 980_769.231);
			assert_eq!(Stp258Tokens::total_issuance(STP258_TOKEN_ID), 1_100_000 * 1_000);

			let transferred_event = Event::serp_market(crate::Event::SerpedDownSupply(STP258_TOKEN_ID, 10_000 * 1000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
			
			let transferred_event = Event::serp_market(crate::Event::NewPrice(STP258_TOKEN_ID,1.03 * 1_000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
		});
}
