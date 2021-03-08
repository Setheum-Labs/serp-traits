use codec::FullCodec;
use sp_runtime::{
	traits::{
        AtLeast32Bit, MaybeSerializeDeserialize,
    }, 
    DispatchResult,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	fmt::Debug,
};

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
}
