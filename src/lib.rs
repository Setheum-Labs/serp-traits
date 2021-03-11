#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::{
	cmp::{Eq, PartialEq},
	prelude::Vec,
};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub use auction::{Auction, AuctionHandler, AuctionInfo, OnNewBidResult};

pub use data_provider::{DataFeeder, DataProvider, DataProviderExtended};
pub use get_by_key::GetByKey;
pub use nft::NFT;
pub use price::{DefaultPriceProvider, PriceProvider};
pub use rewards::RewardHandler;
pub use serp_market::Market;
pub use serp_tes::{SerpTes, ElastAdjustmentFrequency};
pub use setheum_currency::{Currency, LockableCurrency, ReservableCurrency};
pub use stp258_currency::{
	BalanceStatus, LockIdentifier,
	OnDust, Stp258Asset, Stp258AssetExtended, Stp258AssetLockable, 
	Stp258AssetReservable, Stp258Currency, Stp258CurrencyExtended, 
	Stp258CurrencyLockable, Stp258CurrencyReservable,
};
pub use stp258_stable_currency::{
	Stp258StableCurrency, Stp258StableCurrencyExtended, 
	Stp258StableCurrencyLockable, Stp258StableCurrencyReservable,
};

pub mod account;
pub mod arithmetic;
pub mod auction;
pub mod data_provider;
pub mod get_by_key;
pub mod nft;
pub mod price;
pub mod rewards;
pub mod serp_market;
pub mod serp_tes;
pub mod setheum_currency;
pub mod stp258_currency;
pub mod stp258_stable_currency;

/// New data handler
#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait OnNewData<AccountId, Key, Value> {
	/// New data is available
	fn on_new_data(who: &AccountId, key: &Key, value: &Value);
}

/// Combine data provided by operators
pub trait CombineData<Key, TimestampedValue> {
	/// Combine data provided by operators
	fn combine_data(
		key: &Key,
		values: Vec<TimestampedValue>,
		prev_value: Option<TimestampedValue>,
	) -> Option<TimestampedValue>;
}

/// Indicate if should change a value
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum Change<Value> {
	/// No change.
	NoChange,
	/// Changed to new value.
	NewValue(Value),
}

#[derive(Encode, Decode, RuntimeDebug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TimestampedValue<Value: Ord + PartialOrd, Moment> {
	pub value: Value,
	pub timestamp: Moment,
}

#[impl_trait_for_tuples::impl_for_tuples(30)]
pub trait Happened<T> {
	fn happened(t: &T);
}
