use codec::FullCodec;
use crate::DataProvider;
pub use frame_support::{traits::{BalanceStatus, LockIdentifier}, Parameter};
use sp_runtime::{
	traits::{
		AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member, CheckedDiv,
	}, 
	DispatchResult
};
use sp_std::{marker::PhantomData, fmt::Debug};

/// A trait to provide relative price for two currencies
pub trait SerpTesPriceProvider<CurrencyId, Price> {
	fn get_price(base: CurrencyId, quote: CurrencyId) -> Option<Price>;
}

/// The frequency of adjustments for the Currency supply.
pub struct ElastAdjustmentFrequency<BlockNumber> {
	/// Number of blocks for adjustment frequency.
	pub adjustment_frequency: BlockNumber,
}

/// Abstraction over a fungible multi-stable-currency Token Elasticity of Supply system.
pub trait SerpTes<AccountId, BlockNumber, CurrencyId, Price> {
	/// The currency identifier.
	type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize;

	/// The balance of an account.
	type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

	fn adjustment_frequency(
		adjustment_frequency: BlockNumber,
    ) -> DispatchResult;

	// Public immutables

	/// The total amount of issuance of `currency_id`.
	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance;

	// Public mutables

	/// Contract the currency supply.
	fn contract_supply(
		currency_id: Self::CurrencyId,
		total_issuance: Self::Balance,
		amount: Self::Balance,
		to: &AccountId,
	) -> DispatchResult;

	/// Expand the currency supply.
	fn expand_supply(
		currency_id: Self::CurrencyId,
		total_issuance: Self::Balance,
		amount: Self::Balance,
		to: &AccountId,
	) -> DispatchResult;

	/// Contracts or expands the currency supply based on conditions.
	fn on_block_with_price(block: BlockNumber, currency_id: Self::CurrencyId, price: Price) -> DispatchResult;

	/// Expands (if the price is high) or contracts (if the price is low) the currency supply.
	fn serp_elast(currency_id: Self::CurrencyId, price: Price) -> DispatchResult;

	/// Calculate the amount of supply change from a fraction given as `numerator` and `denominator`.
	fn supply_change(currency_id: Self::CurrencyId, numerator: u64, denominator: u64, supply: u64) -> u64;
}

/// A `PriceProvider` implementation based on price data from a `DataProvider`
pub struct DefaultSerpTesPriceProvider<CurrencyId, Source>(PhantomData<(CurrencyId, Source)>);

impl<CurrencyId, Source, Price> SerpTesPriceProvider<CurrencyId, Price> for DefaultSerpTesPriceProvider<CurrencyId, Source>
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

fn get_serpup_price(base_currency_id: CurrencyId, quote_currency_id: CurrencyId) -> Option<Price> {
		let base_price = Source::get(&base_currency_id)?; // base currency price compared to currency (native currency could work best)
		let quote_price = Source::get(&quote_currency_id)?;
        let market_price = base_price.checked_div(&quote_price); // market_price of the currency.
        let mint_rate = Perbill::from_percent(); // supply change of the currency.
        let serp_quote = market_price.checked_add(Perbill::from_percent(&mint_rate * 2)); // serping_price of the currency.
        serp_quote.checked_add(Perbill::from_percent(&mint_rate * 2)); 
	}