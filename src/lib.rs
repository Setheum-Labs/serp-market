//! SerpMarket pallet.
//!
//!This is the Serp Market Pallet that trades with the SERP system 
//!to make stability serping exchange for SettCurrency/Stp258Currency.
//!Trading and price quotation is unique (Serp Quotation), the Setheum way.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use sp_std::prelude::*;

use codec::{Decode, Encode};
use core::cmp::{max, min, Ord, Ordering};
use fixed::{types::extra::U64, FixedU128};
use frame_support::pallet_prelude::*;
use stp258_traits::{
	Stp258Currency,
	Stp258CurrencyLockable, Stp258CurrencyReservable, 
	Stp258Asset, Stp258AssetReservable,
};
use sp_runtime::{
	traits::{AtLeast32Bit, CheckedAdd, CheckedDiv, CheckedMul, MaybeSerializeDeserialize, Member, Zero}, 
	DispatchResult, PerThing, Perbill, RuntimeDebug,
};

use frame_system::{self as system, pallet_prelude::*};

mod mock;
mod tests;

pub use module::*;

#[frame_support::pallet]
pub mod module {
	use super::*;

	pub(crate) type BalanceOf<T> =
		<<T as Config>::Stp258StableCurrency as SettCurrency<<T as frame_system::Config>::AccountId>>::Balance;
	pub(crate) type CurrencyIdOf<T> =
		<<T as Config>::Stp258StableCurrency as SettCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;
	pub(crate) type AccountIdOf<T> =
		<<T as Config>::Stp258Currency as SettCurrency<<T as frame_system::Config>::AccountId>>::AccountId;

	

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The balance type
		type Balance: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

		/// The price type
		type Price = Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

		/// The quote type
		type Quote = Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

		/// The Serp ratio type
		type SerpRatio = Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;
		
		/// The base_unit type
		type BaseUnit = Parameter + Member + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

		/// The currency ID type
		type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord;

		/// The native asset (Dinar) type
		type NativeAsset: NativeAsset<Self::AccountId>;
		
		/// The stable currency (SettCurrency) type
		type SettCurrency: SettCurrency<Self::AccountId>;

		/// The SettPay Account type
		#[pallet::constant]
		type GetSettPayAcc: Get<AccountIdOf<Self>>;

		/// The Serpers Account type
		#[pallet::constant]
		type GetSerperAcc: Get<AccountIdOf<Self>>;

		/// The Serp quote multiple type for qUOTE, quoting 
		/// `(mintrate * SERP_QUOTE_MULTIPLE) = SerpQuotedPrice`.
		#[pallet::constant]
		type GetSerpQuoteMultiple: Get<Quote>;

		/// The Serper ratio type getter
		#[pallet::constant]
		type GetSerperRatio: Get<SerpRatio>;

		/// The SettPay ratio type getter
		#[pallet::constant]
		type GetSettPayRatio: Get<SerpRatio>;

		/// The native asset (Dinar) Currency ID type
		#[pallet::constant]
		type GetNativeAssetId: Get<CurrencyIdOf<Self>>;

		/// The base_unit getter
		#[pallet::constant]
		type GetBaseUnit: Get<BaseUnit>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Some wrong behavior
		Wrong,
		/// Something went very wrong and the price of the currency is zero.
		ZeroPrice,
		/// While trying to expand the supply, it overflowed.
		SupplyOverflow,
		/// While trying to contract the supply, it underflowed.
		SupplyUnderflow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// Serp Expand Supply successful. [currency_id, who, amount]
		SerpedUpSupply(CurrencyIdOf<T>, BalanceOf<T>),
		/// Serp Contract Supply successful. [currency_id, who, amount]
		SerpedDownSupply(CurrencyIdOf<T>, BalanceOf<T>),
		/// The New Price of Currency. [currency_id, price]
		NewPrice(CurrencyIdOf<T>, Price),
	}

	/// The Price of a currency type.
	#[pallet::storage]
	#[pallet::getter(fn price)]
	pub type Price<T: Config> = StorageMap<
		_, 
		Twox64Concat, 
		T::CurrencyId, 
		T::Price, 
		T::CurrencyId, 
		T::Price, 
		ValueQuery,
	>;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// A trait to provide relative `base_price` of `base_settcurrency_id`. 
		/// The settcurrency `Price` is `base_price * base_unit`.
		/// For example The `Price` of `JUSD` is `base_price: Price = $1.1 * base_unit: BaseUnit = 1_100`.
		/// Therefore, the `Price` is got by checking how much `base_currency_peg` can buy `base_unit`, 
		/// in our example, `1_100` in `base_currency_peg: USD` of `JUSD` can buy `base_unit` of `JUSD` in `USD`.
		#[weight = 0]
		fn get_stable_price(
			base_settcurrency_id: CurrencyId, 
			base_price: Price
		) -> DispatchResult {
			type Fix = FixedU128<U64>;
			let base_unit = T::GetBaseUnit;
			let amount_of_peg_to_buy_base_currency = Fix::from_num(base_price) * Fix::from_num(base_unit);
			Price::put(amount_of_peg_to_buy_base_currency); /// the amount of peg that can buy base currency.
			native::info!("The price of: {} is {} in its peg currency.", base_settcurrency_id, amount_of_peg_to_buy_base_currency);
			Self::deposit_event(RawEvent::NewPrice(base_settcurrency_id, amount_of_peg_to_buy_base_currency));
			Ok(())
		}
		
		/// A trait to provide relative price for two currencies. 
		/// For example, the relative price of `DNAR-JUSD` is `$1_000 / $1.1 = JUSD 1_100`,
		/// meaning the price compared in `USD` as the peg of `JUSD` for example.. or,
		/// the relative price of `DNAR-JUSD` is `DNAR 1 / JUSD 0.001 = JUSD 1_000`,
		/// meaning `DNAR 1` can buy `JUSD 1_000` and therefore `1 DNAR = 0.001 JUSD`.
		/// But tyhe former is preffered and thus used.
		#[weight = 0]
		fn get_relative_price(
			base_currency_id: CurrencyId, 
			base_price: Price, 
			quote_currency_id: CurrencyId, 
			quote_price: Price
		) -> DispatchResult {
			type Fix = FixedU128<U64>;
			let amount_of_quote_to_buy_base = Fix::from_num(base_price) / Fix::from_num(quote_price);
			let amount_of_base_to_buy_quote = Fix::from_num(quote_price) / Fix::from_num(base_price);
			Price::put(amount_of_quote_to_buy_base); /// the amount of quote currency that can buy base currency.
			native::info!(
				"The price of: {} for: {}  is {}, therefore {} {} can buy {} {}", 
				base_currency_id, 
				quote_currency_id, 
				amount_of_quote_to_buy_base, 
				quote_currency_id, 
				amount_of_quote_to_buy_base, 
				base_currency_id, 
				amount_of_base_to_buy_quote,
			);
			Ok(())
		}

		/// Quote the amount of currency price quoted as serping fee (serp quoting) for Serpers, 
		/// the Serp Quote is `price/base_unit = fraction`, `fraction - 1 = fractioned`, `fractioned * serp_quote_multiple = quotation`,
		/// `quotation + fraction = quoted` and `quoted` is the price the SERP will pay for serping in full including the serp_quote,
		///  the fraction is same as `(market_price + (mint_rate * 2))` - where `market-price = price/base_unit`, 
		/// `mint_rate = serp_quote_multiple`, and with `(price/base_unit) - 1 = price_change`.
		///
		/// Calculate the amount of currency price for SerpMarket's SerpQuote from a fraction given as `numerator` and `denominator`.
		#[weight = 0]
		fn quote_serp_price(price: Price) -> Price {
			type Fix = FixedU128<U64>;
			let base_unit = T::GetBaseUnit;
			/// The `serp_quote_multiple` was in place as "[`let serp_quote_multiple = 2;` // (mint_rate * 2) for Serp Quote]".
			let serp_quote_multiple = T::GetSerpQuoteMultiple;
			let fraction = Fix::from_num(price) / Fix::from_num(base_unit);
			let fractioned = Fix::from_num(fraction) - Fix::from_num(1);
			let quotation = fractioned.saturating_mul_int(serp_quote_multiple as u128).to_num::<u64>();
			quoted = Fix::from_num(fraction) + Fix::from_num(quotation);
		}

		/// Calculate the amount of supply change from a fraction given as `numerator` and `denominator`.
		#[weight = 0]
		fn calculate_supply_change(new_price: Price) -> Price {
			type Fix = FixedU128<U64>;
			let base_unit = T::GetBaseUnit; 
			let supply = T::SettCurrency::total_issuance();
			let fraction = Fix::from_num(new_price) / Fix::from_num(base_unit) - Fix::from_num(1);
			fraction.saturating_mul_int(supply as u128).to_num::<u64>()
		}

		/// Called when `expand_supply` is received from the SERP.
		/// Implementation should `deposit` the `amount` to `serpup_to`, 
		/// then `amount` will be slashed from `serpup_from` and update
		/// `new_supply`.
		#[weight = 0]
		fn expand_supply(
			currency_id: Self::CurrencyId, 
			expand_by: Self::Balance,
		) -> DispatchResult{
			let supply = T::SettCurrency::total_issuance();
			// Checking whether the supply will overflow.
			supply
				.checked_add(expand_by)
				.ok_or(Error::<T>::SupplyOverflow)?;
			// ↑ verify ↑
			let native_asset_id = T::GetNativeAssetId;
			let serper = &T::GetSerperAcc; 
			let settpay = &T::GetSettPayAcc;
			let base_currency_id = currency_id;
			let quote_currency_id = native_asset_id;
			let price = Self::get_relative_price(
				native_asset_id,
				base_price: Price, 
				currency_id, 
				quote_price: Price
			);
			let supply_change = Self::calculate_supply_change(price);
			let serp_quoted_price = Self::quote_serp_price(price);
			let settpay_ratio = &T::GetSettPayRatio; // 75% for SettPay. It was statically typed, now moved to runtime and can be set there.
			let serper_ratio = &T::GetSerperRatio; // 25% for Serpers. It was statically typed, now moved to runtime and can be set there.
			let supply_ratio = Fix::from_num(expand_by) / Fix::from_num(100); // Percentage, meaning 1%.
			let settpay_distro = supply_ratio.saturating_mul_int(settpay_ratio as u128).to_num::<u64>(); // 75% distro for SettPay.
			let serper_distro = supply_ratio.saturating_mul_int(serper_ratio as u128).to_num::<u64>(); // 25% distro for Serpers.
			let pay_by_quoted = serper_distro.saturating_div_int(serp_quoted_price as u128).to_num::<u64>();
			if currency_id == native_asset_id::get() {
				debug::warn!("Cannot expand supply for NativeCurrency: {}", currency_id);
				return Err(http::Error::Unknown);
			} else {
				T::SettCurrency::set_free_balance(currency_id, settpay, account.free + settpay_distro);
				T::SettCurrency::set_reserved_balance(currency_id, serper, account.reserved + serper_distro);
				T::NativeAsset::set_reserved_balance(serper, account.reserved - pay_by_quoted);
			}
			// safe to do this late because of the overflow test in the second line of the function
			native::info!("expanded supply by serping settcurrency: {} with amount: {}", currency_id, expand_by);
			T::SettCurrency::<TotalIssuance<T>>::mutate(currency_id, |v| *v += expand_by);
			native::info!("burned native asset: {} by serping settcurrency: {} with amount: {} {}", native_asset_id, currency_id, native_asset_id, pay_by_quoted);
			T::NativeAsset::<TotalIssuance<T>>::mutate(native_asset_id, |v| *v -= pay_by_quoted);
			<Price>::put(new_price);
			native::info!("The new price of: {} is : {}", currency_id, new_price);
			Self::deposit_event(Event::SerpedUpSupply(currency_id, expand_by));
			Self::deposit_event(Event::NewPrice(currency_id, serp_quoted_price));
			Ok(())
		}

		/// Called when `contract_supply` is received from the SERP.
		/// Implementation should `deposit` the `base_currency_id` (The Native Currency) 
		/// of `amount` to `serpup_to`, then `amount` will be slashed from `serpup_from` 
		/// and update `new_supply`.
		#[weight = 0]
		fn contract_supply(
			currency_id: Self::CurrencyId,
			contract_by: Self::Balance
		) -> DispatchResult{
			let supply = T::SettCurrency::total_issuance();
			// Checking whether the supply will overflow.
			supply
				.checked_sub(contract_by)
				.ok_or(Error::<T>::SupplyUnderflow)?;
			// ↑ verify ↑
			let serper = &T::GetSerperAcc;
			let settpay: &T::GetSettPayAcc,
			let base_currency_id = currency_id;
			let quote_currency_id = native_asset_id;
			let price = Self::get_relative_price(
				currency_id, 
				quote_price: Price
				native_asset_id,
				base_price: Price, 
			);
			let supply_change == contract_by;
			let serp_quoted_price = Self::quote_serp_price(price);
			let new_price == serp_quoted_price;
			let pay_by_quoted = serp_quoted_price.saturating_mul_int(supply_change as u128).to_num::<u64>();
			if currency_id == T::GetNativeAssetId::get() {
				debug::warn!("Cannot expand supply for NativeCurrency: {}", currency_id);
				return Err(http::Error::Unknown);
			} else {
				T::SettCurrency::set_reserved_balance(currency_id, serper, account.reserved - contract_by);
				T::NativeAsset::set_reserved_balance(serper, account.reserved + pay_by_quoted);
			}
			// safe to do this late because of the overflow test in the second line of the function
			native::info!("contracted supply by serping settcurrency: {} with amount: {}", currency_id, contract_by);
			T::SettCurrency::<TotalIssuance<T>>::mutate(currency_id, |v| *v -= contract_by);
			native::info!("mined native asset : {} by serping settcurrency: {} with amount: {} {}", native_asset_id, currency_id, native_asset_id, pay_by_quoted);
			T::NativeAsset::<TotalIssuance<T>>::mutate(native_asset_id, |v| *v += pay_by_quoted);
			<Price>::put(new_price);
			native::info!("The price of: {} is: {} in its peg currency.", currency_id, new_price);
			Self::deposit_event(Event::SerpedDownSupply(currency_id, who.clone(), contract_by));
			Self::deposit_event(Event::NewPrice(currency_id, serp_quoted_price));
			Ok(())
		}
	}
}
