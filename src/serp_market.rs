use codec::FullCodec;
use crate::DataProvider;
use fixed::{types::extra::U64, FixedU128};
use frame_support::Parameter;
use sp_runtime::{
	traits::{
        AtLeast32Bit, CheckedDiv, MaybeSerializeDeserialize, Member
    }, 
    DispatchResult,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	marker::PhantomData, 
	fmt::Debug,
};
/// A trait to provide relative price for two currencies
pub trait MarketPriceProvider<CurrencyId, Price> {
	fn get_price(base: CurrencyId, quote: CurrencyId) -> Option<Price>;
}

/// Abstraction over a serping market system for the Setheum Elastic Reserve Protocol (SERP) Market.
pub trait Market<CurrencyId, AccountId,  Balance, Price, Source, SerpQuote> {
	/// The price to trade.
	type Balance: AtLeast32Bit + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;
    /// The currency type in trade.
	type CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug;

	/// Called when `expand_supply` is received from the SERP.
	/// Implementation should `deposit` the `amount` to `serpup_to`, 
	/// then `amount` will be slashed from `serpup_from` and update
	/// `new_supply`.
	fn on_expand_supply(
        currency_id: CurrencyId,
		amount: Balance,
		serpup_to: AccountId,
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

	/// Calculate the amount of price change from a fraction given as `numerator` and `denominator`.
	fn calculate_price_change(currency_id: Self::CurrencyId, numerator: u64, denominator: u64, supply: u64) -> u64;
}
