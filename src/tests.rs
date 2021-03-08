use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
    new_test_ext().execute_with(|| {
        // Dispatch a signed extrinsic.
        assert_ok!(SerpMarket::do_something(Origin::signed(1), 42));
        // Read pallet storage and assert an expected result.
        assert_eq!(SerpMarket::something(), Some(42));
    });
}

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
fn get_serpup_price_should_work() {
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
fn get_serpdown_price_should_work() {
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
			assert_ok!(Stp258Currencies::transfer(Some(ALICE).into(), BOB, STP258_TOKEN_ID, 50));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &ALICE), 50);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &BOB), 150);
		});
}

#[test]
fn contract_supply_should_work() {
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

			assert_ok!(Stp258Currencies::transfer(Some(ALICE).into(), BOB, STP258_TOKEN_ID, 50));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &ALICE), 50);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &BOB), 150);

			let transferred_event = Event::stp258_currencies(crate::Event::Transferred(STP258_TOKEN_ID, ALICE, BOB, 50));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258Currencies as Stp258Currency<AccountId>>::transfer(
				STP258_TOKEN_ID, &ALICE, &BOB, 10
			));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &ALICE), 40);
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &BOB), 160);

			let transferred_event = Event::stp258_currencies(crate::Event::Transferred(STP258_TOKEN_ID, ALICE, BOB, 10));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258Currencies as Stp258Currency<AccountId>>::deposit(
				STP258_TOKEN_ID, &ALICE, 100
			));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &ALICE), 140);

			let transferred_event = Event::stp258_currencies(crate::Event::Deposited(STP258_TOKEN_ID, ALICE, 100));
			assert!(System::events().iter().any(|record| record.event == transferred_event));

			assert_ok!(<Stp258Currencies as Stp258Currency<AccountId>>::withdraw(
				STP258_TOKEN_ID, &ALICE, 20
			));
			assert_eq!(Stp258Currencies::free_balance(STP258_TOKEN_ID, &ALICE), 120);

			let transferred_event = Event::stp258_currencies(crate::Event::Withdrawn(STP258_TOKEN_ID, ALICE, 20));
			assert!(System::events().iter().any(|record| record.event == transferred_event));
		});
}

#[test]
fn cancel_selected_bids_test() {
	new_test_ext().execute_with(|| {
		let bid_amount = 5 * BaseUnit::get();
		SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(25), bid_amount));
		SettCurrency::add_bid(Bid::new(2, Perbill::from_percent(33), bid_amount));
		SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(45), bid_amount));
		SettCurrency::add_bid(Bid::new(1, Perbill::from_percent(50), bid_amount));
		SettCurrency::add_bid(Bid::new(3, Perbill::from_percent(55), bid_amount));
		assert_eq!(SettCurrency::dinar_bids().len(), 5);

		assert_ok!(SettCurrency::cancel_bids_at_or_below(
			Origin::signed(1),
			Perbill::from_percent(45)
		));

		let bids = SettCurrency::dinar_bids();
		assert_eq!(bids.len(), 3);
		let bids: Vec<(_, _)> = bids
			.into_iter()
			.map(|Bid { account, price, .. }| (account, price))
			.collect();
		// highest bid is last so we can pop
		assert_eq!(
			bids,
			vec![
				(2, Perbill::from_percent(33)),
				(1, Perbill::from_percent(50)),
				(3, Perbill::from_percent(55)),
			]
		);
	});
}
