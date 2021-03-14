use codec::FullCodec;
use crate::DataProvider;
use fixed::{types::extra::U64, FixedU128};
use frame_support::Parameter;
use sp_runtime::{
	traits::{
        AtLeast32Bit, Member, MaybeDisplay, MaybeSerializeDeserialize
    }, 
    DispatchResult,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	fmt::Debug,
};

/// Abstraction over a serping market system for the Setheum Elastic Reserve Protocol (SERP) Market.
pub trait SerpMarket<CurrencyId, AccountId,  Balance> {
	/// The price to trade.
	type Balance: AtLeast32Bit + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;
    /// The currency type in trade.
	type CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug;
	/// The account type in trade.
	type AccountId: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord + Default;

	/// A trait to provide relative `base_price` of `base_settcurrency_id`. 
	/// The settcurrency `Price` is `base_price * base_unit`.
	/// For example The `Price` of `JUSD` is `base_price: Price = $1.1 * base_unit: BaseUnit = 1_100`.
	/// Therefore, the `Price` is got by checking how much `base_currency_peg` can buy `base_unit`, 
	/// in our example, `1_100` in `base_currency_peg: USD` of `JUSD` can buy `base_unit` of `JUSD` in `USD`.
	fn get_stable_price(base_settcurrency_id: CurrencyId, base_price: u64) -> DispatchResult;
	
	/// A trait to provide relative price for two currencies. 
	/// For example, the relative price of `DNAR-JUSD` is `$1_000 / $1.1 = JUSD 1_100`,
	/// meaning the price compared in `USD` as the peg of `JUSD` for example.. or,
	/// the relative price of `DNAR-JUSD` is `DNAR 1 / JUSD 0.001 = JUSD 1_000`,
	/// meaning `DNAR 1` can buy `JUSD 1_000` and therefore `1 DNAR = 0.001 JUSD`.
	/// But tyhe former is preffered and thus used.
	fn get_relative_price(
		base_currency_id: CurrencyId, 
		base_price: u64,  
		quote_currency_id: CurrencyId, 
		quote_price: u64
	) -> DispatchResult;

	/// Quote the amount of currency price quoted as serping fee (serp quoting) for Serpers, 
	/// the Serp Quote is `price/base_unit = fraction`, `fraction - 1 = fractioned`, `fractioned * serp_quote_multiple = quotation`,
	/// `quotation + fraction = quoted` and `quoted` is the price the SERP will pay for serping in full including the serp_quote,
	///  the fraction is same as `(market_price + (mint_rate * 2))` - where `market-price = price/base_unit`, 
	/// `mint_rate = serp_quote_multiple`, and with `(price/base_unit) - 1 = price_change`.
	///
	/// Calculate the amount of currency price for SerpMarket's SerpQuote from a fraction given as `numerator` and `denominator`.
	fn quote_serp_price(price: u64) -> u64;

	/// Calculate the amount of supply change from a fraction given as `numerator` and `denominator`.
	fn calculate_supply_change(new_price: u64) -> u64;

	/// Called when `expand_supply` is received from the SERP.
	/// Implementation should `deposit` the `amount` to `serpup_to`, 
	/// then `amount` will be slashed from `serpup_from` and update
	/// `new_supply`. `quote_price` is the price ( relative to the settcurrency) of 
	/// the `native_currency` used to expand settcurrency supply.
	fn expand_supply(currency_id: CurrencyId, expand_by: Balance, quote_price: Balance) -> DispatchResult;

	/// Called when `contract_supply` is received from the SERP.
	/// Implementation should `deposit` the `base_currency_id` (The Native Currency) 
	/// of `amount` to `serpup_to`, then `amount` will be slashed from `serpup_from` 
	/// and update `new_supply`. `quote_price` is the price ( relative to the settcurrency) of 
	/// the `native_currency` used to contract settcurrency supply.
	fn contract_supply(currency_id: CurrencyId, contract_by: Balance, quote_price: Balance) -> DispatchResult;
}
