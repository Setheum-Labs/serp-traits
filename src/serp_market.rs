use codec::FullCodec;
use frame_support::Parameter;
use sp_runtime::{
	traits::{AtLeast32Bit, MaybeSerializeDeserialize}, 
    DispatchResult,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	fmt::Debug,
};

/// Abstraction over a serping market system for the Setheum Elastic Reserve Protocol (SERP) Market.
pub trait SerpMarket<AccountId> {
	/// The price to trade.
	type Balance: AtLeast32Bit + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;
    /// The currency type in trade.
	type CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug;

	/// Called when `expand_supply` is received from the SERP.
	/// Implementation should `deposit` the `amount` to `serpup_to`, 
	/// then `amount` will be slashed from `serpup_from` and update
	/// `new_supply`. `quote_price` is the price ( relative to the settcurrency) of 
	/// the `native_currency` used to expand settcurrency supply.
	fn expand_supply(
		native_currency_id: Self::CurrencyId, 
		stable_currency_id: Self::CurrencyId, 
		expand_by: Self::Balance, 
		pay_by_quoted: Self::Balance, 
		serpers: &AccountId
	) -> DispatchResult;

	/// Called when `contract_supply` is received from the SERP.
	/// Implementation should `deposit` the `base_currency_id` (The Native Currency) 
	/// of `amount` to `serpup_to`, then `amount` will be slashed from `serpup_from` 
	/// and update `new_supply`. `quote_price` is the price ( relative to the settcurrency) of 
	/// the `native_currency` used to contract settcurrency supply.
	fn contract_supply(
		native_currency_id: Self::CurrencyId, 
		stable_currency_id: Self::CurrencyId, 
		contract_by: Self::Balance, 
		pay_by_quoted: Self::Balance, 
		serpers: &AccountId
	) -> DispatchResult;
}
