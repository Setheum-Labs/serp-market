//! Unit tests for the SerpMarket module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Event, *};
use sp_runtime::traits::BadOrigin;

#[test]
fn get_price_should_work() {
	assert_eq!(
		MarketPriceProvider::get_price(1, 2),
		Some(Price::saturating_from_rational(1, 2))
	);
	assert_eq!(
		MarketPriceProvider::get_price(2, 1),
		Some(Price::saturating_from_rational(2, 1))
	);
}

#[test]
fn price_is_none_should_not_panic() {
	assert_eq!(MarketPriceProvider::get_price(3, 3), None);
	assert_eq!(MarketPriceProvider::get_price(3, 1), None);
	assert_eq!(MarketPriceProvider::get_price(1, 3), None);
}

#[test]
fn price_is_zero_should_not_panic() {
	assert_eq!(MarketPriceProvider::get_price(0, 0), None);
	assert_eq!(MarketPriceProvider::get_price(1, 0), None);
	assert_eq!(MarketPriceProvider::get_price(0, 1), Some(Price::from_inner(0)));
}

#[test]
fn calculate_serp_price_should_work() {
	assert_eq!(
		MarketPriceProvider::get_price(1, 2),
		Some(Price::saturating_from_rational(1, 2))
	);
	assert_eq!(
		MarketPriceProvider::get_price(2, 1),
		Some(Price::saturating_from_rational(2, 1))
	);
}

#[test]
fn calculate_serp_quote_should_work() {
	assert_eq!(
		MarketPriceProvider::get_price(1, 2),
		Some(Price::saturating_from_rational(1, 2))
	);
	assert_eq!(
		MarketPriceProvider::get_price(2, 1),
		Some(Price::saturating_from_rational(2, 1))
	);
}

#[test]
fn expand_supply_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Currencies::transfer_native_currency(Some(ALICE).into(), BOB, 50));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &BOB), 150 * 1000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &BOB), 150 * 1000);

			assert_ok!(Stp258Currencies::transfer(Some(ALICE).into(), BOB, STP258_TOKEN_ID, 50 * 1000));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &ALICE), 50 * 1000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &BOB), 150 * 1000);
		
		
		

			// payout of 120% of BaseUnit
			let payout = Fixed64::from_rational(20, 100).saturated_multiply_accumulate(BaseUnit::get());
			add_dinar(SettCurrency::new_dinar(2, payout));
			add_dinar(SettCurrency::new_dinar(3, payout));
			add_dinar(SettCurrency::new_dinar(4, payout));
			add_dinar(SettCurrency::new_dinar(5, 7 * payout));

			let prev_supply = SettCurrency::settcurrency_supply();
			let amount = 13 * BaseUnit::get();
			assert_ok!(SettCurrency::expand_supply(prev_supply, amount));

			let amount_per_acc = InitialSupply::get() / 10 + BaseUnit::get() / 10;
			assert_eq!(SettCurrency::get_balance(1), amount_per_acc);
			assert_eq!(SettCurrency::get_balance(2), amount_per_acc + payout);
			assert_eq!(SettCurrency::get_balance(3), amount_per_acc + payout);
			assert_eq!(SettCurrency::get_balance(4), amount_per_acc + payout);
			assert_eq!(SettCurrency::get_balance(5), amount_per_acc + 7 * payout);
			assert_eq!(SettCurrency::get_balance(8), amount_per_acc);
			assert_eq!(SettCurrency::get_balance(10), amount_per_acc);

			assert_eq!(
				SettCurrency::settcurrency_supply(),
				prev_supply + amount,
				"supply should be increased by amount"
			);
		});
}

#[test]
fn contract_supply_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Currencies::transfer_native_currency(Some(ALICE).into(), BOB, 50));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &BOB), 150 * 1000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &BOB), 150 * 1000);

			assert_ok!(Stp258Currencies::transfer(Some(ALICE).into(), BOB, STP258_TOKEN_ID, 50 * 1000));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &ALICE), 50 * 1000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &BOB), 150 * 1000);
			




			let dinar_amount = Ratio::new(125, 100)
			.checked_mul(&BaseUnit::get().into())
			.map(|r| r.to_integer())
			.expect("dinar_amount should not have overflowed");
			NativeCurrency::add_bid(Bid::new(1, Perbill::from_percent(80), dinar_amount));
			NativeCurrency::add_bid(Bid::new(2, Perbill::from_percent(75), 2 * BaseUnit::get()));
			SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(80), dinar_amount));
			SettCurrency::add_bid(Bid::new(2, Perbill::from_percent(75), 2 * BaseUnit::get()));

			let prev_supply = SettCurrency::settcurrency_supply(Coins);
			let amount = 2 * BaseUnit::get();
			assert_ok!(SettCurrency::contract_supply(prev_supply, amount));

			assert_ok!(<Stp258 as ExtendedSettCurrency<AccountId>>::settcurrency_supply(
					NATIVE_SETT_USD_ID,
				));

			let bids = SettCurrency::dinar_bids(NATIVE_CURRENCY_ID);
			assert_eq!(bids.len(), 1, "exactly one bid should have been removed");
			let remainging_bid_quantity = Fixed64::from_rational(667, 1_000)
				.saturated_multiply_accumulate(BaseUnit::get())
				- BaseUnit::get();
			assert_eq!(
				bids[0],
				Bid::new(2, Perbill::from_percent(75), remainging_bid_quantity)
			);

			let (start, _) = SettCurrency::dinar_range();
			assert_eq!(SettCurrency::get_dinar(start).payout, dinar_amount);
			assert_eq!(
				SettCurrency::get_dinar(start + 1).payout,
				Fixed64::from_rational(333, 1_000).saturated_multiply_accumulate(BaseUnit::get())
			);

			assert_eq!(
				SettCurrency::settcurrency_supply(SETT_USD_ID),
				prev_supply - amount,
				"supply should be decreased by amount"
			);
		});
}

#[test]
fn stable_currency_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Currencies::transfer(Some(ALICE).into(), BOB, STP258_TOKEN_ID, 50 * 1000));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &ALICE), 50 * 1000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &BOB), 150 * 1000);
		});
}

#[test]
fn native_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			assert_ok!(Stp258Currencies::transfer_native_currency(Some(ALICE).into(), BOB, 50));
			assert_eq!(Stp258Native::free_balance(&ALICE), 50);
			assert_eq!(Stp258Native::free_balance(&BOB), 150);

			assert_ok!(Stp258Native::transfer(&ALICE, &BOB, 10));
			assert_eq!(Stp258Native::free_balance(&ALICE), 40);
			assert_eq!(Stp258Native::free_balance(&BOB), 160);

			assert_eq!(Stp258Currencies::slash(STP258_NATIVE_ID, &ALICE, 10), 0);
			assert_eq!(Stp258Native::free_balance(&ALICE), 30);
			assert_eq!(Stp258Native::total_issuance(), 190);
		});
}

#[test]
fn call_event_should_work() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.build()
		.execute_with(|| {
			System::set_block_number(1);

			assert_ok!(Stp258Currencies::transfer(Some(ALICE).into(), BOB, STP258_TOKEN_ID, 50 * 1000));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &ALICE), 50 * 1000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &BOB), 150 * 1000);

			let transferred_event = Event::stp258_currencies(crate::Event::Transferred(STP258_TOKEN_ID, ALICE, BOB, 50 * 1000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258Currencies as Stp258Currency<AccountId>>::transfer(
				STP258_TOKEN_ID, &ALICE, &BOB, 10 * 1000
			));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &ALICE), 40 * 1000);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &BOB), 160 * 1000);

			let transferred_event = Event::stp258_currencies(crate::Event::Transferred(STP258_TOKEN_ID, ALICE, BOB, 10 * 1000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258Currencies as Stp258Currency<AccountId>>::deposit(
				STP258_TOKEN_ID, &ALICE, 100 * 1000
			));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &ALICE), 140 * 1000);

			let transferred_event = Event::stp258_currencies(crate::Event::Deposited(STP258_TOKEN_ID, ALICE, 100 * 1000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258Currencies as Stp258Currency<AccountId>>::withdraw(
				STP258_TOKEN_ID, &ALICE, 20 * 1000
			));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &ALICE), 120 * 1000);

			let transferred_event = Event::stp258_currencies(crate::Event::Withdrawn(STP258_TOKEN_ID, ALICE, 20 * 1000));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
		});
}
