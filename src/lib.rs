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
	pub(crate) type AmountOf<T> =
		<<T as Config>::Stp258Currency as Stp258CurrencyExtended<<T as frame_system::Config>::AccountId>>::Amount;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		
		type Balance: Parameter + codec::HasCompact + From<u32> + Into<Weight> + Default + MaybeSerializeDeserialize;
		
		type Stp258Currency: MergeAccount<Self::AccountId>
			+ Stp258CurrencyExtended<Self::AccountId>
			+ Stp258CurrencyLockable<Self::AccountId>
			+ Stp258CurrencyReservable<Self::AccountId>;

		type Stp258Native: Stp258AssetExtended<Self::AccountId, Balance = BalanceOf<Self>, Amount = AmountOf<Self>>
			+ Stp258AssetLockable<Self::AccountId, Balance = BalanceOf<Self>>
			+ Stp258AssetReservable<Self::AccountId, Balance = BalanceOf<Self>>;

		#[pallet::constant]
		type GetStp258NativeId: Get<CurrencyIdOf<Self>>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Some wrong behavior
		Wrong,
		/// Something went very wrong and the price of the currency is zero.
		ZeroPrice,
		/// An arithmetic operation caused an overflow.
		GenericOverflow,
		/// An arithmetic operation caused an underflow.
		GenericUnderflow,
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
		#[pallet::weight(<T::Balance as Into<Weight>>::into(new_value.clone()))]
		pub fn set_dummy(origin: OriginFor<T>, #[pallet::compact] new_value: T::Balance) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			Dummy::<T>::put(&new_value);
			Self::deposit_event(Event::Dummy(new_value));

			Ok(().into())
		}]

		// TODO: REPLACE WITH `serp_up_supply`.
		/// Transfer some balance to another account under `currency_id`.
		///
		/// The dispatch origin for this call must be `Signed` by the
		/// transactor.
		#[pallet::weight(T::WeightInfo::transfer_non_native_currency())]
		pub fn transfer(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			currency_id: CurrencyIdOf<T>,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			<Self as Stp258Currency<T::AccountId>>::transfer(currency_id, &from, &to, amount)?;
			Ok(().into())
		}

		// TODO: REPLACE WITH `serp_down_supply`.
		/// Transfer some native currency to another account.
		///
		/// The dispatch origin for this call must be `Signed` by the
		/// transactor.
		#[pallet::weight(T::WeightInfo::transfer_native_currency())]
		pub fn transfer_native_currency(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] amount: BalanceOf<T>,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
			let to = T::Lookup::lookup(dest)?;
			T::Stp258Native::transfer(&from, &to, amount)?;

			Self::deposit_event(Event::Transferred(T::GetStp258NativeId::get(), from, to, amount));
			Ok(().into())
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
		amount: Balance,
		serpup_to: AccountId, AccountId,
		serpup_from: AccountId,
		new_supply: Balance,
	) -> DispatchResult;

	/// Called when `contract_supply` is received from the SERP.
	/// Implementation should `deposit` the `base_currency_id` (The Native Currency) 
	/// of `amount` to `serpup_to`, then `amount` will be slashed from `serpup_from` 
	/// and update `new_supply`.
	fn on_contract_supply(
		currency_id: CurrencyId,
		amount: Balance,
		serpdown_to: AccountId,
		serpdown_from: AccountId,
		new_supply: Balance,
	) -> DispatchResult;
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

	/// Provide relative `serping_price` for two currencies
    /// with additional `serp_quote`.
	fn get_serpup_price(base_currency_id: CurrencyId, quote_currency_id: CurrencyId) -> Option<Price> {
		let base_price = Source::get(&base_currency_id)?; // base currency price compared to currency (native currency could work best)
		let quote_price = Source::get(&quote_currency_id)?;
        let market_price = base_price.checked_div(&quote_price); // market_price of the currency.
        let mint_rate = Perbill::from_percent(); // supply change of the currency.
        let serp_quote = market_price.checked_add(Perbill::from_percent(&mint_rate * 2)); // serping_price of the currency.
        serp_quote.checked_add(Perbill::from_percent(&mint_rate * 2)); 
	}
}