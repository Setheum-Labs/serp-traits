use codec::FullCodec;
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
pub trait SerpMarket<AccountId> {
	/// The price to trade.
	type Balance: AtLeast32Bit + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;
    /// The currency type in trade.
	type CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug;
	
	/// Quote the amount of currency price quoted as serping fee (serp quoting) for Serpers during serpdown, 
	/// the Serp Quote is `quotation + new_base_price`, `base_unit - new_base_price = fractioned`, `fractioned * serp_quote_multiple = quotation`,
	/// and `serp_quoted_price` is the price the SERP will pay for serping in full including the serp_quote, 
	/// the fraction for `serp_quoted_price` is same as `(market_price + (burn_rate * 2))` - where `market-price = new_base_price / quote_price`, 
	/// `(burn_rate * 2) = serp_quote_multiple` as in price balance, `burn_rate = supply/new_supply` that is the ratio of burning/contracting the supply.
	/// Therefore buying the stable currency for more than market price.
	///
	/// The quoted amount to pay serpers for serping down supply.
	fn pay_serpup_by_quoted(currency_id: Self::CurrencyId, expand_by: Self::Balance, quote_price: Self::Balance) -> DispatchResult;
	
	/// Quote the amount of currency price quoted as serping fee (serp quoting) for Serpers during serpdown, 
	/// the Serp Quote is `quotation + new_base_price`, `base_unit - new_base_price = fractioned`, `fractioned * serp_quote_multiple = quotation`,
	/// and `serp_quoted_price` is the price the SERP will pay for serping in full including the serp_quote, 
	/// the fraction for `serp_quoted_price` is same as `(market_price + (burn_rate * 2))` - where `market-price = new_base_price / quote_price`, 
	/// `(burn_rate * 2) = serp_quote_multiple` as in price balance, `burn_rate = supply/new_supply` that is the ratio of burning/contracting the supply.
	/// Therefore buying the stable currency for more than market price.
	///
	/// The quoted amount to pay serpers for serping down supply.
	fn pay_serpdown_by_quoted(currency_id: Self::CurrencyId, contract_by: Self::Balance, quote_price: Self::Balance) -> DispatchResult;


	/// Called when `expand_supply` is received from the SERP.
	/// Implementation should `deposit` the `amount` to `serpup_to`, 
	/// then `amount` will be slashed from `serpup_from` and update
	/// `new_supply`. `quote_price` is the price ( relative to the settcurrency) of 
	/// the `native_currency` used to expand settcurrency supply.
	fn expand_supply(currency_id: Self::CurrencyId, expand_by: Self::Balance, quote_price: Self::Balance) -> DispatchResult;

	/// Called when `contract_supply` is received from the SERP.
	/// Implementation should `deposit` the `base_currency_id` (The Native Currency) 
	/// of `amount` to `serpup_to`, then `amount` will be slashed from `serpup_from` 
	/// and update `new_supply`. `quote_price` is the price ( relative to the settcurrency) of 
	/// the `native_currency` used to contract settcurrency supply.
	fn contract_supply(currency_id: Self::CurrencyId, contract_by: Self::Balance, quote_price: Self::Balance) -> DispatchResult;

	//// A trait to provide relative `base_price` of `base_settcurrency_id`. 
	//// The settcurrency `Price` is `base_price * base_unit`.
	//// For example The `Price` of `JUSD` is `base_price: Price = $1.1 * base_unit: BaseUnit = 1_100`.
	//// Therefore, the `Price` is got by checking how much `base_currency_peg` can buy `base_unit`, 
	//// in our example, `1_100` in `base_currency_peg: USD` of `JUSD` can buy `base_unit` of `JUSD` in `USD`.
	//// `fn get_stable_price(base_price: Self::Balance) -> Self::Balance;`
	
	//// A trait to provide relative price for two currencies. 
	//// For example, the relative price of `DNAR-JUSD` is `$1_000 / $1.1 = JUSD 1_100`,
	//// meaning the price compared in `USD` as the peg of `JUSD` for example.. or,
	//// the relative price of `DNAR-JUSD` is `DNAR 1 / JUSD 0.001 = JUSD 1_000`,
	//// meaning `DNAR 1` can buy `JUSD 1_000` and therefore `1 DNAR = 0.001 JUSD`.
	//// But tyhe former is preffered and thus used.
	//// `fn get_relative_price(`
	//// 	`base_price: Self::Balance, ` 
	//// 	`quote_price: Self::Balance`
	//// `) -> Self::Balance;`

	//// Quote the amount of currency price quoted as serping fee (serp quoting) for Serpers during serpdown, 
	//// the Serp Quote is `price/base_unit = fraction`, `fraction - 1 = fractioned`, `fractioned * serp_quote_multiple = quotation`,
	//// `quotation + fraction = quoted` and `quoted` is the price the SERP will pay for serping in full including the serp_quote,
	////  the fraction is same as `(market_price + (mint_rate * 2))` - where `market-price = price/base_unit`, 
	//// `mint_rate = serp_quote_multiple`, and with `(price/base_unit) - 1 = price_change`.
	////
	//// Calculate the amount of currency price for SerpMarket's SerpQuote from a fraction given as `numerator` and `denominator`.
	//// `fn quote_serpdown_price( price: Self::Balance) -> Self::Balance;`

	//// Quote the amount of currency price quoted as serping fee (serp quoting) for Serpers during serpup, 
	//// the Serp Quote is `price/base_unit = fraction`, `fraction - 1 = fractioned`, `fractioned * serp_quote_multiple = quotation`,
	//// `quotation - fraction = quoted` and `quoted` is the price the SERP will pay for serping in full including the serp_quote,
	////  the fraction is same as `(market_price + (mint_rate * 2))` - where `market-price = price/base_unit`, 
	//// `mint_rate = serp_quote_multiple`, and with `(price/base_unit) - 1 = price_change`.
	////
	//// Calculate the amount of currency price for SerpMarket's SerpQuote from a fraction given as `numerator` and `denominator`.
	//// `fn quote_serpup_price(price: Self::Balance) -> Self::Balance;`

	//// Calculate the amount of supply change from a fraction given as `numerator` and `denominator`.
	//// `fn calculate_supply_change(currency_id: Self::CurrencyId, new_price: Self::Balance) -> Self::Balance;`
}
