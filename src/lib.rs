#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use codec::Codec;
use frame_support::{
	pallet_prelude::*,
	traits::{
		Currency as SetheumCurrency, ExistenceRequirement, Get, 
		LockableCurrency as SetheumLockableCurrency,
		ReservableCurrency as SetheumReservableCurrency, WithdrawReasons,
	},
};
use frame_system::{ensure_root, ensure_signed, pallet_prelude::*};
use stp258_traits::{
	account::MergeAccount,
	arithmetic::{Signed, SimpleArithmetic},
	BalanceStatus, GetByKey, SerpMarket, Stp258Asset, Stp258AssetExtended, Stp258AssetLockable, Stp258AssetReservable,
	LockIdentifier, Stp258Currency, Stp258CurrencyExtended, Stp258CurrencyReservable, Stp258CurrencyLockable,
};
use orml_utilities::with_transaction_result;
use sp_runtime::{
	traits::{CheckedSub,  MaybeSerializeDeserialize, AtLeast32BitUnsigned, StaticLookup, Zero},
	DispatchResult, Perbill
};


mod default_weight;
mod mock;
mod tests;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	pub trait WeightInfo {
		fn transfer_non_native_currency() -> Weight;
		fn transfer_native_currency() -> Weight;
		fn update_balance_non_native_currency() -> Weight;
		fn update_balance_native_currency_creating() -> Weight;
		fn update_balance_native_currency_killing() -> Weight;
	}

	pub(crate) type BalanceOf<T> =
		<<T as Config>::SerpMarket as SerpMarket<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type CurrencyIdOf<T> =
		<<T as Config>::SerpMarket as SerpMarket<<T as frame_system::Config>::AccountId>>::CurrencyId;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);
	
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type SerpMarket: SerpMarket<Self::AccountId>;

		/// The balance type
		type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

		/// The currency ID type
		type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord;

		#[pallet::constant]
		type GetStp258NativeId: Get<CurrencyIdOf<Self>>;

		/// The base unit of a currency
		type GetBaseUnit: GetByKey<CurrencyIdOf<Self>, BalanceOf<Self>>;

		/// The single unit to avoid data loss with mized type arithmetic.
		#[pallet::constant]
		type GetSingleUnit: Get<BalanceOf<Self>>;

		/// The Serpers Account type
		type GetSerperRatio: Get<Perbill>;

		/// The SettPay Account type
		type GetSettPayRatio: Get<Perbill>;

		/// The SettPay Account type
		#[pallet::constant]
		type GetSettPayAcc: Get<Self::AccountId>;

		/// The Serpers Account type
		#[pallet::constant]
		type GetSerperAcc: Get<Self::AccountId>;

		/// The Serp quote multiple type for qUOTE, quoting 
		/// `(mintrate * SERP_QUOTE_MULTIPLE) = SerpQuotedPrice`.
		#[pallet::constant]
		type GetSerpQuoteMultiple: Get<BalanceOf<Self>>;

		/// Weight information for extrinsics in this module.
		type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Unable to convert the Amount type into Balance.
		AmountIntoBalanceFailed,
		/// Balance is too low.
		BalanceTooLow,
		// Cannott expand or contract Native Asset, only SettCurrency	Serping.
		CannotSerpNativeAssetOnlySerpSettCurrency,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(crate) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Serp Expand Supply successful. [currency_id, who, amount]
		SerpedUpSupply(CurrencyIdOf<T>, BalanceOf<T>),
		/// Serp Contract Supply successful. [currency_id, who, amount]
		SerpedDownSupply(CurrencyIdOf<T>, BalanceOf<T>),
	}


	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

impl<T: Config> SerpMarket<T::AccountId> for Pallet<T>{
	type CurrencyId = CurrencyIdOf<T>;
	type Balance = BalanceOf<T>;

    /// Called when `expand_supply` is received from the SERP.
    /// Implementation should `deposit` the `amount` to `serpup_to`, 
    /// then `amount` will be slashed from `serper` and update
    /// `new_supply`. `quote_price` is the price ( relative to the settcurrency) of 
    /// the `native_currency` used to expand settcurrency supply.
    fn expand_supply(
        native_currency_id: CurrencyIdOf<T>, 
        stable_currency_id: CurrencyIdOf<T>, 
        expand_by: BalanceOf<T>,
        pay_by_quoted: BalanceOf<T>, // the price of Dinar, so as to expand settcurrency supply.
		serpers: &T::AccountId,
	) -> DispatchResult {
        if expand_by.is_zero() || stable_currency_id == native_currency_id {
			return Ok(());
		}
        if native_currency_id == T::GetStp258NativeId::get() {
			T::SerpMarket::expand_supply(native_currency_id, stable_currency_id, expand_by, pay_by_quoted, serpers)?;
		}
        Self::deposit_event(Event::SerpedUpSupply(stable_currency_id, expand_by));
        Ok(())
    }

    /// Called when `contract_supply` is received from the SERP.
    /// Implementation should `deposit` the `base_currency_id` (The Native Currency) 
    /// of `amount` to `serper`, then `amount` will be slashed from `serper` 
    /// and update `new_supply`. `quote_price` is the price ( relative to the settcurrency) of 
    /// the `native_currency` used to contract settcurrency supply.
    fn contract_supply(
        native_currency_id: CurrencyIdOf<T>, 
        stable_currency_id: CurrencyIdOf<T>, 
        contract_by: BalanceOf<T>,
        pay_by_quoted: BalanceOf<T>, // the price of Dinar, so as to contract settcurrency supply.
		serpers: &T::AccountId,
    ) -> DispatchResult {
		if contract_by.is_zero() || stable_currency_id == native_currency_id {
			return Ok(());
		}
        if native_currency_id == T::GetStp258NativeId::get() {
			T::SerpMarket::contract_supply(native_currency_id, stable_currency_id, contract_by, pay_by_quoted, serpers)?;
		}
        Self::deposit_event(Event::SerpedDownSupply(stable_currency_id, contract_by));
        Ok(())
    }
}
