use crate::{Change, DataProvider};
use codec::FullCodec;
use codec::{Decode, Encode};
use frame_support::Parameter;
use sp_runtime::{
	traits::{AtLeast32Bit, Bounded, CheckedDiv, MaybeSerializeDeserialize, Member},
	DispatchError, DispatchResult, RuntimeDebug,
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
pub struct SerpingInfo<AccountId, Balance, BlockNumber> {
	/// Current trader, trade price and the blocknumber of trade.
	pub trade: Option<(AccountId, Balance, BlockNumber)>,
}

/// Abstraction over a simple serping system.
pub trait SerpMarket<AccountId,  Balance, BlockNumber, SerpingId> {
	/// The id of a SerpingInfo
	type SerpingId: FullCodec + Default + Copy + Eq + PartialEq + MaybeSerializeDeserialize + Bounded + Debug;
	/// The price to trade.
	type Balance: AtLeast32Bit + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;
    /// The currency type in trade.
	type CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug;
    /// The currency price
	type Price = FixedU128;

	/// The serping info of `id`
	fn serping_info(id: Self::SerpingId) -> Option<SerpingInfo<AccountId, Self::Balance, BlockNumber>>;

    /// The serping info of `id`
	fn mint_rate(id: Self::SerpingId) -> Option<SerpingInfo<AccountId, Self::Balance, BlockNumber>>;

    /// Add `amount` to the balance of `who` under `currency_id` and increase
	/// total issuance.
	fn on_(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Remove `amount` from the balance of `who` under `currency_id` and reduce
	/// total issuance.
	fn withdraw(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

    /// Called when supply expansion is received from the SERP.
	/// The return value determines if the trade should be accepted and update
	/// serping end time. Implementation should reserve money from current
	/// winner and refund previous winner.
	fn on_expand_supply(
		now: BlockNumber,
		id: SerpingId,
		new_trade: (AccountId, Balance),
		last_trade: Option<(AccountId, Balance)>,
	) -> OnNewTradeResult<BlockNumber>;

	/// Called when new trade is received.
	/// The return value determines if the trade should be accepted and update
	/// serping end time. Implementation should reserve money from current
	/// winner and refund previous winner.
	fn on_contract_supply(
		now: BlockNumber,
		id: SerpingId,
		new_trade: (AccountId, Balance),
		last_trade: Option<(AccountId, Balance)>,
	) -> OnNewTradeResult<BlockNumber>;

}

/// A trait to provide relative price for two currencies
pub trait MarketPriceProvider<CurrencyId, Price> {
	fn get_price(base: CurrencyId, quote: CurrencyId) -> Option<Price>;
}

/// A `MarketPriceProvider` implementation based on price data from a `DataProvider`
pub struct DefaultMarketPriceProvider<CurrencyId, Source>(PhantomData<(CurrencyId, Source)>);

impl<CurrencyId, Source, Price> PriceProvider<CurrencyId, Price> for DefaultPriceProvider<CurrencyId, Source>
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
}

/// Hooks for serping to handle trades.
pub trait MarketHandler<AccountId, Balance, BlockNumber, SerpingId> {
	/// Called when supply expansion is received from the SERP.
	/// The return value determines where to `deposit` the `new_supply` to `serpup_to` and update
	/// supply. Implementation should deposit `amount` to `serpup_to`, and in the case of Setheum,
	/// `amount` will be slashed from `serpup_from`.
	fn on_expand_supply(
        currency_id: CurrencyId,
		amount: Balance,
		serpup_to: AccountId,
		serpup_from: AccountId,
		new_supply: Balance,
		id: SerpingId,
	) -> DispatchResult;


	/// Called when new trade is received.
	/// The return value determines if the trade should be accepted and update
	/// serping end time. Implementation should reserve money from current
	/// winner and refund previous winner.
	fn on_contract_supply(
		now: BlockNumber,
		id: SerpingId,
		new_trade: (AccountId, Balance),
		last_trade: Option<(AccountId, Balance)>,
	) -> OnNewTradeResult<BlockNumber>;
}