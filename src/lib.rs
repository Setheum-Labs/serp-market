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
use stp258::{
	account::MergeAccount,
	arithmetic::{Signed, SimpleArithmetic}, 
	Stp258Asset, Stp258AssetReservable, Stp258Currency, Stp258CurrencyReservable,
};
use stp258_traits::{
	arithmetic::{Signed, SimpleArithmetic}, 
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

		type SettPayAccountId: AccountId;

		type SettPaySerpAmount: Balance;

		#[pallet::constant]
		type GetStp258NativeId: Get<CurrencyIdOf<Self>>;

		/// The balance of an account.
		#[pallet::constant]
		type GetBaseUnit: Get<u64>;
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
			sett_pay_account_id: &T::SettPayAccountId,
			#[pallet::compact] amount: &T::SettPaySerpAmount,
		) -> DispatchResult {
			let SettPaySerpAmount = Perbil;
			if amount.is_zero() {
				return Ok(());
			}
			if currency_id == T::GetStp258NativeId::get() {
				debug::warn!("Cannot expand supply for NativeCurrency: {}", currency_id);
				return Err(http::Error::Unknown);
			} else {
				T::Stp258Currency::deposit(currency_id, who, amount)?;
			}
			Self::deposit_event(Event::Deposited(currency_id, who.clone(), amount));
			Ok(())
		}

		fn trade_serpup(
			currency_id: CurrencyIdOf<T>,
			to: &T::AccountId
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let to = T::SettPayAccountId;
			<Self as Stp258Currency<T::AccountId>>::transfer(currency_id, &from, &to, amount)?;
			Ok(().into())
		}

		fn trade_serpdown(
			currency_id: CurrencyIdOf<T>,
			to: &T::AccountId
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let to = T::SettPayAccountId;
			<Self as Stp258Currency<T::AccountId>>::transfer(currency_id, &from, &to, amount)?;
			Ok(().into())
		}
	}
}

impl<T: Config> Market<T::CurrencyId, T::Price> for Pallet<T> {
	type Balance = BalanceOf<T>;
	type CurrencyId = CurrencyIdOf<T>;

	// Public mutables
	/// Called when `expand_supply` is received from the SERP.
	/// Implementation should `deposit` the `amount` to `serpup_to`, 
	/// then `amount` will be slashed from `serpup_from` and update
	/// `new_supply`.
	fn on_expand_supply(
		currency_id: CurrencyId,
		expand_amount: Balance,
		serpup_to: AccountId, AccountId,
		serpup_from: AccountId,
		new_supply: Balance,
		total_issuance: TotalIssuance,
	) -> DispatchResult{
		if currency_id == T::GetNativeCurrencyId::get() {
			debug::warn!("Cannot expand supply for NativeCurrency: {}", currency_id);
			return Err(http::Error::Unknown);
		} else {
			T::SettCurrency::expand_supply(currency_id, expand_amount)?;
		}
		// Checking whether the supply will overflow.
		total_issuance
			.checked_add(currency_id, expand_amount)
			.ok_or(Error::<T>::SupplyOverflow)?;
		let 
		let new_supply = total_issuance + expand_amount;
		native::info!("expanded supply by minting {} {} sett currency", currency_id, expand_amount);
		<SettCurrencySupply>::put(new_supply);
		Self::deposit_event(RawEvent::ExpandedSupply(currency_id, expand_amount));
		let        
		Ok(())
	}

	fn deposit(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::deposit(who, amount)?;
		} else {
			T::Stp258Currency::deposit(currency_id, who, amount)?;
		}
		Self::deposit_event(Event::Deposited(currency_id, who.clone(), amount));
		Ok(())
	}

	fn slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::slash(who, amount)
		} else {
			T::Stp258Currency::slash(currency_id, who, amount)
		}
	}

	/// Called when `contract_supply` is received from the SERP.
	/// Implementation should `deposit` the `base_currency_id` (The Native Currency) 
	/// of `amount` to `serpup_to`, then `amount` will be slashed from `serpup_from` 
	/// and update `new_supply`.
	fn on_contract_supply(
		currency_id: CurrencyId,
		contract_amount: Balance,
		serpdown_to: AccountId,
		serpdown_from: AccountId,
		new_supply: Balance,
	) -> DispatchResult;

	fn deposit(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> DispatchResult {
		if amount.is_zero() {
			return Ok(());
		}
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::deposit(who, amount)?;
		} else {
			T::Stp258Currency::deposit(currency_id, who, amount)?;
		}
		Self::deposit_event(Event::Deposited(currency_id, who.clone(), amount));
		Ok(())
	}

	fn slash(currency_id: Self::CurrencyId, who: &T::AccountId, amount: Self::Balance) -> Self::Balance {
		if currency_id == T::GetStp258NativeId::get() {
			T::Stp258Native::slash(who, amount)
		} else {
			T::Stp258Currency::slash(currency_id, who, amount)
		}
	}

	fn get_price(base_currency_id: CurrencyId, quote_currency_id: CurrencyId) -> Option<Price> {
		let base_price = Source::get(&base_currency_id)?;
		let quote_price = Source::get(&quote_currency_id)?;

		base_price.checked_div(&quote_price)
	}

	/// Calculate the amount of currency price quoted as serping fee for Serpers, 
	/// the Serp Quote is `((price/base_unit) - 1) * serp_quote_multiple)`,
	/// the fraction is same as `(market_price + (mint_rate * 2))` - where `market-price = price/base_unit`, 
	/// `mint_rate = serp_quote_multiple`, and with `(price/base_unit) - 1 = price_change`.
	/// 
	/// SerpMarket's SerpQuote from a fraction given as `numerator` and `denominator`.
	fn calculate_serp_quote(numerator: u64, denominator: u64, serp_quote_multiple: u64) -> u64 {
		type Fix = FixedU128<U64>;
		let fraction = Fix::from_num(numerator) / Fix::from_num(denominator) - Fix::from_num(1);
		fraction.saturating_mul_int(serp_quote_multiple as u128).to_num::<u64>()
	}

	/// Calculate the amount of currency price for SerpMarket's SerpQuote from a fraction given as `numerator` and `denominator`.
	fn calculate_serp_price(numerator: u64, denominator: u64, serp_quote_multiple: u64) -> u64 {
		type Fix = FixedU128<U64>;
		let fraction = Fix::from_num(numerator) / Fix::from_num(denominator) - Fix::from_num(1);
		fraction.saturating_mul_int(serp_quote_multiple as u128).to_num::<u64>()
	}
}

/// A `PriceProvider` implementation based on price data from a `DataProvider`.
pub struct SerpMarketPriceProvider<CurrencyId, Source>(PhantomData<(CurrencyId, Source)>);

impl<CurrencyId, Source, Price> MarketPriceProvider<CurrencyId, Price> for SerpMarketPriceProvider<CurrencyId, Source>
where
	CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize,
	Source: DataProvider<CurrencyId, Price>,
	Price: CheckedDiv,
{
	fn get_price(base_currency_id: CurrencyId, quote_currency_id: CurrencyId) -> Option<Price> {
		let base_price = Source::get(&base_currency_id)?;
		let quote_price = Source::get(&quote_currency_id)?;

		base_price.checked_div(&quote_price)
	}

}