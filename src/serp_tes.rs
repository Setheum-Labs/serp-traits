use codec::FullCodec;
pub use frame_support::{traits::{BalanceStatus, LockIdentifier}, Parameter};
use sp_runtime::{
	traits::{
		AtLeast32BitUnsigned, MaybeSerializeDeserialize, Member,
	}, 
	DispatchResult
};
use sp_std::fmt::Debug;

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
pub trait SerpTes<BlockNumber> {
	/// The currency identifier.
	type CurrencyId: Parameter + Member + Copy + MaybeSerializeDeserialize;

	/// The balance of an account.
	type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

	fn adjustment_frequency(adjustment_frequency: BlockNumber) -> DispatchResult;

	/// Contracts or expands the currency supply based on conditions.
	fn on_block_with_price(currency_id: Self::CurrencyId, block: BlockNumber, price: Self::Balance) -> DispatchResult;

	/// Expands (if the price is high) or contracts (if the price is low) the currency supply.
	fn serp_elast(currency_id: Self::CurrencyId, price: Self::Balance) -> DispatchResult;

	/// Calculate the amount of supply change from a fraction given as `numerator` and `denominator`.
	fn supply_change(currency_id: Self::CurrencyId, price: Self::Balance) -> Self::Balance;
}
