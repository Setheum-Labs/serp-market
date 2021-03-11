//! SerpMarket pallet.
//!
//!This is the Serp Market Pallet that trades with the SERP system 
//!to make stability serping exchange for SettCurrency/Stp258Currency.
//!Trading and price quotation is unique (Serp Quotation), the Setheum way.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use sp_std::prelude::*;

use adapters::{BoundedPriorityQueue, BoundedDeque};
use codec::{Decode, Encode};
use core::cmp::{max, min, Ord, Ordering};
use fixed::{types::extra::U64, FixedU128};
use frame_support::pallet_prelude::*;
use num_rational::Ratio;
use stp258_traits::{
	arithmetic::{Signed, SimpleArithmetic}, 
	DataProvider as SerpMarketProvider,
	price::PriceProvider as MarketPriceProvider,
	serp_market::Market,
	currency::{Stp258Asset, Stp258AssetReservable, Stp258Currency, Stp258CurrencyReservable},
};
use sp_runtime::{
	traits::{AtLeast32Bit, CheckedAdd, CheckedDiv, CheckedMul, MaybeSerializeDeserialize, Member, Zero}, 
	DispatchResult, PerThing, Perbill, RuntimeDebug,
};
use sp_std::collections::vec_
deque::VecDeque;
use frame_system::{self as system, ensure_signed, pallet_prelude::*};

mod mock;
mod tests;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	pub(crate) type BalanceOf<T> =
		<<T as Config>::Stp258Currency as Stp258Currency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type CurrencyIdOf<T> =
		<<T as Config>::Stp258Currency as Stp258Currency<<T as frame_system::Config>::AccountId>>::CurrencyId;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type NativeAsset: Stp258Asset<Self::AccountId>;
		
		type SettCurrency: Stp258Currency<Self::AccountId>;

		type GetSettPayAcc: Get<Self::GetSettPayAcc>;

		type SettPaySupply: Get<Self::SettPaySupply>;
		
		type SerpMarketSupply: Get<Self::SerpMarketSupply>;

		#[pallet::constant]
		type GetNativeAssetId: Get<Self::CurrencyIdOf<Self>>;

		/// The balance of an account.
		#[pallet::constant]
		type GetBaseUnit: Stp258Currency<Self::GetBaseUnit>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Some wrong behavior
		Wrong,
		/// While trying to expand the supply, it overflowed.
		SupplyOverflow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId")]
	pub enum Event<T: Config> {
		/// Serp Expand Supply successful. [currency_id, who, amount]
		SerpedUpSupply(CurrencyIdOf<T>, T::AccountId, BalanceOf<T>),
		/// Serp Contract Supply successful. [currency_id, who, amount]
		SerpedDownSupply(CurrencyIdOf<T>, T::AccountId, BalanceOf<T>),
		/// Serp Withdraw successful. [currency_id, who, amount]
		SerpDeposited(CurrencyIdOf<T>, T::AccountId, BalanceOf<T>),
		/// Serp Deposit Supply successful. [currency_id, who, amount]
		SerpWithdrawn(CurrencyIdOf<T>, T::AccountId, BalanceOf<T>),
	}

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// Serp to SettPay
		fn deposit_serp_to_settpay(
			currency_id: Self::CurrencyId, 
			sett_pay: Self::GetSettPayAcc,
			#[pallet::compact] amount:  Self::Balance,
		) -> DispatchResult {
			let sett_pay_value = amount 
			let to = T::SettPayAcc;
			if amount.is_zero() {
				return Ok(());
			}
			if currency_id == T::GetNativeAssetId::get() {
				debug::warn!("Cannot expand supply for NativeCurrency: {}", currency_id);
				return Err(http::Error::Unknown);
			} else {
				T::Stp258Currency::deposit(currency_id, who, amount)?;
			}
			<Self as Stp258Currency<T::AccountId>>::deposit(currency_id, &to, amount)?;
			Ok(().into())
		}
	}
}

impl<T: Config> Market<T::CurrencyId, T::Price> for Pallet<T> {
	type Balance = BalanceOf<T>;
	type CurrencyId = CurrencyIdOf<T>;


	/// Calculate the amount of currency to be sent to SettPay on Expand Supply from a fraction given as `numerator` and `denominator`.
	fn calculate_serp_distribution(the_supply: u64, the_serp_amount_percentage: u64) -> u64 {
		type Fix = FixedU128<U64>;
		let fraction = Fix::from_num(the_supply) / Fix::from_num(100);
		fraction.saturating_mul_int(the_serp_amount_percentage as u128).to_num::<u64>();
	}

	/// Called when `expand_supply` is received from the SERP.
	/// Implementation should `deposit` the `amount` to `serpup_to`, 
	/// then `amount` will be slashed from `serpup_from` and update
	/// `new_supply`.
	fn expand_supply(
		currency_id: CurrencyId,
		supply: TotalIssuance,
		expand_by: Balance,
	) -> DispatchResult{
		let supply = T::SettCurrency::total_issuance();;
		let to_settpay = Self::calculate_serp_distribution(expand_by, T::SettPaySupply)
		let to_market = Self::calculate_serp_distribution(expand_by, T::SerpMarketSupply)
		let base_unit = Self::BaseUnit;
		let price_quote = Self::quote_serp_price(price: Price, base_unit: Self::BaseUnit, 2)
		if currency_id == T::GetNativeAssetId::get() {
			debug::warn!("Cannot expand supply for NativeCurrency: {}", currency_id);
			return Err(http::Error::Unknown);
		} else {
			T::SettCurrency::deposit(currency_id, to_settpay)?;
			for T::SettCurrency::deposit(currency_id, to_market)?;
				T::NativeAsset::slash(currency_id, to_market)?;
		}
		
		// Checking whether the supply will overflow.
		total_issuance
			.checked_add(currency_id, expand_by)
			.ok_or(Error::<T>::SupplyOverflow)?;
		let 
		let new_supply = total_issuance + expand_by;
		native::info!("expanded supply by minting {} {} sett currency", currency_id, expand_by);
		<SettCurrencySupply>::put(new_supply);
		Self::deposit_event(RawEvent::ExpandedSupply(currency_id, expand_by));
		let        
		Ok(())
	}

	/// Called when `contract_supply` is received from the SERP.
	/// Implementation should `deposit` the `base_currency_id` (The Native Currency) 
	/// of `amount` to `serpup_to`, then `amount` will be slashed from `serpup_from` 
	/// and update `new_supply`.
	fn contract_supply(currency_id: Self::CurrencyId, serper: &T::GetSerperAcc, contract_by: Self::Balance) -> DispatchResult{
		T::Stp258Currency::slash(currency_id, serper, contract_by)?;
		let contract_by = Self::calculate_supply_change(currency_id: CurrencyId, price, base_unit: T::GetBaseUnit, supply);
		if currency_id == T::GetStp258NativeId::get() {
			debug::warn!("Cannot expand supply for NativeCurrency: {}", currency_id);
			return Err(http::Error::Unknown);
		} else {
			T::Stp258Currency::slash(currency_id, who, contract_by)
		}
		Self::deposit_event(Event::Deposited(currency_id, who.clone(), contract_by));
		Ok(())
	}

	fn get_price(base_currency_id: CurrencyId, quote_currency_id: CurrencyId) -> Option<Price> {
		let base_price = Source::get(&base_currency_id)?;
		let quote_price = Source::get(&quote_currency_id)?;

		base_price.checked_div(&quote_price)
	}

	/// Calculate the amount of currency price to bequoted as serping fee (serp quoting) for Serpers, 
	/// the Serp Quote is `((price/base_unit) - 1) * serp_quote_multiple)`,
	/// the fraction is same as `(market_price + (mint_rate * 2))` - where `market-price = price/base_unit`, 
	/// `mint_rate = serp_quote_multiple`, and with `(price/base_unit) - 1 = price_change`.
	/// 
	/// SerpMarket's SerpQuote from a fraction given as `numerator` and `denominator`.
	fn calculate_serp_quote(numerator: u64, denominator: u64, serp_quote_multiple: u64) -> u64 {
		type Fix = FixedU128<U64>;
		let fraction = Fix::from_num(numerator) / Fix::from_num(denominator) - Fix::from_num(1);
		fraction.saturating_mul_int(supply as u128).to_num::<u64>()
	}

	/// Quote the amount of currency price quoted as serping fee (serp quoting) for Serpers, 
	/// the Serp Quote is `price/base_unit = fraction`, `fraction - 1 = fractioned`, `fractioned * serp_quote_multiple = quotation`,
	/// `quotation + fraction = quoted` and `quoted` is the price the SERP will pay for serping in full including the serp_quote,
	///  the fraction is same as `(market_price + (mint_rate * 2))` - where `market-price = price/base_unit`, 
	/// `mint_rate = serp_quote_multiple`, and with `(price/base_unit) - 1 = price_change`.
	///
	/// 
	/// Calculate the amount of currency price for SerpMarket's SerpQuote from a fraction given as `numerator` and `denominator`.
	fn quote_serp_price(price: u64, base_unit: u64, serp_quote_multiple: u64) -> u64 {
		type Fix = FixedU128<U64>;
		let fraction = Fix::from_num(price) / Fix::from_num(base_unit);
		let fractioned = Fix::from_num(fraction) - Fix::from_num(1);
		let quotation = fractioned.saturating_mul_int(serp_quote_multiple as u128).to_num::<u64>();
		quoted = Fix::from_num(fraction) + Fix::from_num(quotation);
	}

	/// Calculate the amount of supply change from a fraction given as `numerator` and `denominator`.
	fn calculate_supply_change(numerator: u64, denominator: u64, supply: u64) -> u64 {
		type Fix = FixedU128<U64>;
		let fraction = Fix::from_num(numerator) / Fix::from_num(denominator) - Fix::from_num(1);
		fraction.saturating_mul_int(supply as u128).to_num::<u64>()
	}
}

/// A `PriceProvider` implementation based on price data from a `DataProvider`.
pub struct SerpMarketPriceProvider<CurrencyId, Source>(PhantomData<(CurrencyId, Source)>);

impl<CurrencyId, Source, Price> MarketPriceProvider<CurrencyId, Price> for SerpMarketPriceProvider<CurrencyId, Source>
where
	CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize,
	Source: SerpMarketProvider,<CurrencyId, Price>,
	Price: CheckedDiv,
{
	fn get_price(base_currency_id: CurrencyId, quote_currency_id: CurrencyId) -> Option<Price> {
		let base_price = Source::get(&base_currency_id)?;
		let quote_price = Source::get(&quote_currency_id)?;

		base_price.checked_div(&quote_price)
	}

}