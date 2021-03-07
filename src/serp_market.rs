use crate::{Change, DataProvider};
use codec::FullCodec;
use codec::{Decode, Encode};
use frame_support::Parameter;
use sp_runtime::{
	traits::{AtLeast32Bit, Bounded, CheckedDiv, MaybeSerializeDeserialize, Member},
	DispatchError, DispatchResult, RuntimeDebug, PerThing, Perbill,
};
use sp_std::{
	cmp::{Eq, PartialEq},
	fmt::Debug,
    marker::PhantomData,
	result,
};

/// Serping info.
#[cfg_attr(feature = "std", derive(PartialEq, Eq))]
#[derive(Encode, Decode, RuntimeDebug)]
/// A `SerpMarketPriceProvider` implementation based on price data from a `DataProvider`
pub struct DefaultSerpMarketPriceProvider<CurrencyId, Source>(PhantomData<(CurrencyId, Source)>);

/// Abstraction over a serping market system for the Setheum Elastic Reserve Protocol (SERP) Market.
pub trait SerpMarket<CurrencyId, AccountId,  Balance, Price, BlockNumber> {
	/// The price to trade.
	type Balance: AtLeast32Bit + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;
    /// The currency type in trade.
	type CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug;
    /// The currency price type.
	type Price = FixedU128;

    // Public immutables
    
    /// Provide relative `price` for two currencies
    fn get_price(
        base: CurrencyId, 
        quote: CurrencyId,
    ) -> Option<Price>;

    /// Provide relative `serping_price` for two currencies
    /// with additional `serp_quote`.
	fn get_serping_price(
        base: CurrencyId, 
        quote: CurrencyId,
        mint_rate: Perbill,
    ) -> Option<Price>;

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

/// A trait to provide relative markey price for two currencies
pub trait SerpMarketPriceProvider<CurrencyId, Price> {
	fn get_price(base: CurrencyId, quote: CurrencyId) -> Option<Price>;
	fn get_serping_price(base: CurrencyId, quote: CurrencyId) -> Option<Price>;
}


/// A `MarketPriceProvider` implementation based on price data from a `DataProvider`
pub struct DefaultMarketPriceProvider<CurrencyId, Source>(PhantomData<(CurrencyId, Source)>);

impl<CurrencyId, Source, Price> MarketPriceProvider<CurrencyId, Price> for DefaultMarketPriceProvider<CurrencyId, Source>
where
	CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize,
	Source: DataProvider<CurrencyId, Price>,
	Price: CheckedDiv,
    SerpingQuote: CheckedAdd,
    MintRate: Perbill
{
	fn get_serpup_price(base_currency_id: CurrencyId, quote_currency_id: CurrencyId) -> Option<Price> {
		let base_price = Source::get(&base_currency_id)?; // base currency price compared to currency (native currency could work best)
		let quote_price = Source::get(&quote_currency_id)?;
        let market_price = base_price.checked_div(&quote_price); // market_price of the currency.
        let mint_rate = Perbill::from_percent(); // supply change of the currency.
        let serp_quote = market_price.checked_add(Perbill::from_percent(&mint_rate * 2)); // serping_price of the currency.
        serp_quote.checked_add(Perbill::from_percent(&mint_rate * 2)); 
	}
}

/// Hooks for serping to handle trades.
pub trait SerpMarketHandler<AccountId, Balance, BlockNumber, SerpingId> {
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