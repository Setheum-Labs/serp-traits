use crate::arithmetic;
use codec::{Codec, FullCodec};
pub use frame_support::{traits::{BalanceStatus, LockIdentifier}, Parameter};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize},
	DispatchError, DispatchResult, 
};
use sp_std::{
	cmp::{Eq, PartialEq},
	convert::{TryFrom, TryInto},
	fmt::Debug,
	result,
};

/// Abstraction over a fungible multi-stable-currency system.
pub trait Stp258Currency<AccountId> {
	/// The currency identifier.
	type CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug;

	/// The balance of an account.
	type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

	// Public immutables

	/// Existential deposit of `currency_id`.
	fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance;


	/// base_unit of `currency_id`.
	fn base_unit(currency_id: Self::CurrencyId) -> Self::Balance;

	/// The total amount of issuance of `currency_id`.
	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance;

	// The combined balance of `who` under `currency_id`.
	fn total_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance;

	// The free balance of `who` under `currency_id`.
	fn free_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance;

	/// A dry-run of `withdraw`. Returns `Ok` iff the account is able to make a
	/// withdrawal of the given amount.
	fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	// Public mutables

	/// Transfer some amount from one account to another.
	fn transfer(
		currency_id: Self::CurrencyId,
		from: &AccountId,
		to: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	/// Add `amount` to the balance of `who` under `currency_id` and increase
	/// total issuance.
	fn deposit(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Remove `amount` from the balance of `who` under `currency_id` and reduce
	/// total issuance.
	fn withdraw(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Same result as `slash(currency_id, who, value)` (but without the
	/// side-effects) assuming there are no balance changes in the meantime and
	/// only the reserved balance is not taken into account.
	fn can_slash(currency_id: Self::CurrencyId, who: &AccountId, value: Self::Balance) -> bool;

	/// Deduct the balance of `who` by up to `amount`.
	///
	/// As much funds up to `amount` will be deducted as possible.  If this is
	/// less than `amount`,then a non-zero value will be returned.
	fn slash(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::Balance) -> Self::Balance;
}

/// Extended `Stp258Currency` with additional helper types and methods.
pub trait Stp258CurrencyExtended<AccountId>: Stp258Currency<AccountId> {
	/// The type for balance related operations, typically signed int.
	type Amount: arithmetic::Signed
		+ TryInto<Self::Balance>
		+ TryFrom<Self::Balance>
		+ arithmetic::SimpleArithmetic
		+ Codec
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ Default;

	/// Add or remove abs(`by_amount`) from the balance of `who` under
	/// `currency_id`. If positive `by_amount`, do add, else do remove.
	fn update_balance(currency_id: Self::CurrencyId, who: &AccountId, by_amount: Self::Amount) -> DispatchResult;
}

/// A fungible multi-stable-currency system whose accounts can have liquidity
/// restrictions.
pub trait Stp258CurrencyLockable<AccountId>: Stp258Currency<AccountId> {
	/// The quantity used to denote time; usually just a `BlockNumber`.
	type Moment;

	/// Create a new balance lock on account `who`.
	///
	/// If the new lock is valid (i.e. not already expired), it will push the
	/// struct to the `Locks` vec in storage. Note that you can lock more funds
	/// than a user has.
	///
	/// If the lock `lock_id` already exists, this will update it.
	fn set_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	/// Changes a balance lock (selected by `lock_id`) so that it becomes less
	/// liquid in all parameters or creates a new one if it does not exist.
	///
	/// Calling `extend_lock` on an existing lock `lock_id` differs from
	/// `set_lock` in that it applies the most severe constraints of the two,
	/// while `set_lock` replaces the lock with the new parameters. As in,
	/// `extend_lock` will set:
	/// - maximum `amount`
	fn extend_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: &AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	/// Remove an existing lock.
	fn remove_lock(lock_id: LockIdentifier, currency_id: Self::CurrencyId, who: &AccountId) -> DispatchResult;
}

/// A fungible multi-stable-currency system where funds can be reserved from the user.
pub trait Stp258CurrencyReservable<AccountId>: Stp258Currency<AccountId> {
	/// Same result as `reserve(who, value)` (but without the side-effects)
	/// assuming there are no balance changes in the meantime.
	fn can_reserve(currency_id: Self::CurrencyId, who: &AccountId, value: Self::Balance) -> bool;

	/// Deducts up to `value` from reserved balance of `who`. This function
	/// cannot fail.
	///
	/// As much funds up to `value` will be deducted as possible. If the reserve
	/// balance of `who` is less than `value`, then a non-zero second item will
	/// be returned.
	fn slash_reserved(currency_id: Self::CurrencyId, who: &AccountId, value: Self::Balance) -> Self::Balance;

	/// The amount of the balance of a given account that is externally
	/// reserved; this can still get slashed, but gets slashed last of all.
	///
	/// This balance is a 'reserve' balance that other subsystems use in order
	/// to set aside tokens that are still 'owned' by the account holder, but
	/// which are suspendable.
	fn reserved_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::Balance;

	/// Moves `value` from balance to reserved balance.
	///
	/// If the free balance is lower than `value`, then no funds will be moved
	/// and an `Err` will be returned to notify of this. This is different
	/// behavior than `unreserve`.
	fn reserve(currency_id: Self::CurrencyId, who: &AccountId, value: Self::Balance) -> DispatchResult;

	/// Moves up to `value` from reserved balance to free balance. This function
	/// cannot fail.
	///
	/// As much funds up to `value` will be moved as possible. If the reserve
	/// balance of `who` is less than `value`, then the remaining amount will be
	/// returned.
	///
	/// # NOTES
	///
	/// - This is different from `reserve`.
	fn unreserve(currency_id: Self::CurrencyId, who: &AccountId, value: Self::Balance) -> Self::Balance;

	/// Moves up to `value` from reserved balance of account `slashed` to
	/// balance of account `beneficiary`. `beneficiary` must exist for this to
	/// succeed. If it does not, `Err` will be returned. Funds will be placed in
	/// either the `free` balance or the `reserved` balance, depending on the
	/// `status`.
	///
	/// As much funds up to `value` will be deducted as possible. If this is
	/// less than `value`, then `Ok(non_zero)` will be returned.
	fn repatriate_reserved(
		currency_id: Self::CurrencyId,
		slashed: &AccountId,
		beneficiary: &AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError>;
}

/// Abstraction over a fungible (single) currency system.
pub trait Stp258Asset<AccountId> {
	/// The balance of an account.
	type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

	// Public immutables

	/// Existential deposit.
	fn minimum_balance() -> Self::Balance;

	/// The total amount of issuance.
	fn total_issuance() -> Self::Balance;

	/// The combined balance of `who`.
	fn total_balance(who: &AccountId) -> Self::Balance;

	/// The free balance of `who`.
	fn free_balance(who: &AccountId) -> Self::Balance;

	/// A dry-run of `withdraw`. Returns `Ok` iff the account is able to make a
	/// withdrawal of the given amount.
	fn ensure_can_withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult;

	// Public mutables

	/// Transfer some amount from one account to another.
	fn transfer(from: &AccountId, to: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Add `amount` to the balance of `who` and increase total issuance.
	fn deposit(who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Remove `amount` from the balance of `who` and reduce total issuance.
	fn withdraw(who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Same result as `slash(who, value)` (but without the side-effects)
	/// assuming there are no balance changes in the meantime and only the
	/// reserved balance is not taken into account.
	fn can_slash(who: &AccountId, value: Self::Balance) -> bool;

	/// Deduct the balance of `who` by up to `amount`.
	///
	/// As much funds up to `amount` will be deducted as possible. If this is
	/// less than `amount`,then a non-zero value will be returned.
	fn slash(who: &AccountId, amount: Self::Balance) -> Self::Balance;
}

/// Extended `Stp258Asset` with additional helper types and methods.
pub trait Stp258AssetExtended<AccountId>: Stp258Asset<AccountId> {
	/// The signed type for balance related operations, typically signed int.
	type Amount: arithmetic::Signed
		+ TryInto<Self::Balance>
		+ TryFrom<Self::Balance>
		+ arithmetic::SimpleArithmetic
		+ Codec
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ Default;

	/// Add or remove abs(`by_amount`) from the balance of `who`. If positive
	/// `by_amount`, do add, else do remove.
	fn update_balance(who: &AccountId, by_amount: Self::Amount) -> DispatchResult;
}

/// A fungible single currency system whose accounts can have liquidity
/// restrictions.
pub trait Stp258AssetLockable<AccountId>: Stp258Asset<AccountId> {
	/// The quantity used to denote time; usually just a `BlockNumber`.
	type Moment;

	/// Create a new balance lock on account `who`.
	///
	/// If the new lock is valid (i.e. not already expired), it will push the
	/// struct to the `Locks` vec in storage. Note that you can lock more funds
	/// than a user has.
	///
	/// If the lock `lock_id` already exists, this will update it.
	fn set_lock(lock_id: LockIdentifier, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Changes a balance lock (selected by `lock_id`) so that it becomes less
	/// liquid in all parameters or creates a new one if it does not exist.
	///
	/// Calling `extend_lock` on an existing lock `lock_id` differs from
	/// `set_lock` in that it applies the most severe constraints of the two,
	/// while `set_lock` replaces the lock with the new parameters. As in,
	/// `extend_lock` will set:
	/// - maximum `amount`
	fn extend_lock(lock_id: LockIdentifier, who: &AccountId, amount: Self::Balance) -> DispatchResult;

	/// Remove an existing lock.
	fn remove_lock(lock_id: LockIdentifier, who: &AccountId) -> DispatchResult;
}

/// A fungible single currency system where funds can be reserved from the user.
pub trait Stp258AssetReservable<AccountId>: Stp258Asset<AccountId> {
	/// Same result as `reserve(who, value)` (but without the side-effects)
	/// assuming there are no balance changes in the meantime.
	fn can_reserve(who: &AccountId, value: Self::Balance) -> bool;

	/// Deducts up to `value` from reserved balance of `who`. This function
	/// cannot fail.
	///
	/// As much funds up to `value` will be deducted as possible. If the reserve
	/// balance of `who` is less than `value`, then a non-zero second item will
	/// be returned.
	fn slash_reserved(who: &AccountId, value: Self::Balance) -> Self::Balance;

	/// The amount of the balance of a given account that is externally
	/// reserved; this can still get slashed, but gets slashed last of all.
	///
	/// This balance is a 'reserve' balance that other subsystems use in order
	/// to set aside tokens that are still 'owned' by the account holder, but
	/// which are suspendable.
	fn reserved_balance(who: &AccountId) -> Self::Balance;

	/// Moves `value` from balance to reserved balance.
	///
	/// If the free balance is lower than `value`, then no funds will be moved
	/// and an `Err` will be returned to notify of this. This is different
	/// behavior than `unreserve`.
	fn reserve(who: &AccountId, value: Self::Balance) -> DispatchResult;

	/// Moves up to `value` from reserved balance to free balance. This function
	/// cannot fail.
	///
	/// As much funds up to `value` will be moved as possible. If the reserve
	/// balance of `who` is less than `value`, then the remaining amount will be
	/// returned.
	///
	/// # NOTES
	///
	/// - This is different from `reserve`.
	fn unreserve(who: &AccountId, value: Self::Balance) -> Self::Balance;

	/// Moves up to `value` from reserved balance of account `slashed` to
	/// balance of account `beneficiary`. `beneficiary` must exist for this to
	/// succeed. If it does not, `Err` will be returned. Funds will be placed in
	/// either the `free` balance or the `reserved` balance, depending on the
	/// `status`.
	///
	/// As much funds up to `value` will be deducted as possible. If this is
	/// less than `value`, then `Ok(non_zero)` will be returned.
	fn repatriate_reserved(
		slashed: &AccountId,
		beneficiary: &AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError>;
}

/// Handler for account which has dust, need to burn or recycle it
pub trait OnDust<AccountId, CurrencyId, Balance> {
	fn on_dust(who: &AccountId, currency_id: CurrencyId, amount: Balance);
}

impl<AccountId, CurrencyId, Balance> OnDust<AccountId, CurrencyId, Balance> for () {
	fn on_dust(_: &AccountId, _: CurrencyId, _: Balance) {}
}

/// Abstraction over a `serp_market` system for the Setheum Elastic Reserve Protocol (SERP) Market for `Stp258Currency` .
pub trait SerpMarket<AccountId>: Stp258Currency<AccountId> {
	/// Called when `expand_supply` is received from the SERP.
	/// Implementation should `deposit` the `amount` to `serpup_to`, 
	/// then `amount` will be slashed from `serpup_from` and update
	/// `new_supply`. `quote_price` is the price ( relative to the settcurrency) of 
	/// the `native_currency` used to expand settcurrency supply.
	fn expand_supply(
		native_currency_id: Self::CurrencyId, 
		stable_currency_id: Self::CurrencyId, 
		expand_by: Self::Balance, 
		quote_price: Self::Balance, 
		// pay_by_quoted: Self::Balance, 
		// serpers: &AccountId
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
		quote_price: Self::Balance, 
		// pay_by_quoted: Self::Balance, 
		// serpers: &AccountId
	) -> DispatchResult;

	/// Quote the amount of currency price quoted as serping fee (serp quoting) for Serpers during serpup, 
	/// the Serp Quote is `new_base_price - quotation` as the amount of native_currency to slash/buy-and-burn from serpers, `base_unit - new_base_price = fractioned`, `fractioned * serp_quote_multiple = quotation`,
	/// and `serp_quoted_price` is the price the SERP will pay for serping in full including the serp_quote, 
	/// the fraction for `serp_quoted_price` is same as `(market_price - (mint_rate * 2))` - where `market-price = new_base_price / quote_price`, 
	/// `(mint_rate * 2) = serp_quote_multiple` as in price balance, `mint_rate = supply/new_supply` that is the ratio of burning/contracting the supply.
	/// Therefore buying the native currency for more than market price.
	///
	/// `quote_price` is the price of expand settcurrency supply.
	/// The quoted amount to pay serpers for serping up supply.
	fn pay_serpup_by_quoted(
		currency_id: Self::CurrencyId, 
		expand_by: Self::Balance, 
		quote_price: Self::Balance, 
	) -> Self::Balance;

	/// Quote the amount of currency price quoted as serping fee (serp quoting) for Serpers during serpdown, 
	/// the Serp Quote is `quotation + new_base_price`, `base_unit - new_base_price = fractioned`, `fractioned * serp_quote_multiple = quotation`,
	/// and `serp_quoted_price` is the price the SERP will pay for serping in full including the serp_quote, 
	/// the fraction for `serp_quoted_price` is same as `(market_price + (burn_rate * 2))` - where `market-price = new_base_price / quote_price`, 
	/// `(burn_rate * 2) = serp_quote_multiple` as in price balance, `burn_rate = supply/new_supply` that is the ratio of burning/contracting the supply.
	/// Therefore buying the stable currency for more than market price.
	///
	/// `quote_price` is the price of `native_currency`.. `quote_price` is the price ( relative to the settcurrency) of 
	/// the `native_currency` used to contract settcurrency supply.
	///
	/// The quoted amount to pay serpers for serping down supply.
	fn pay_serpdown_by_quoted(
		currency_id: Self::CurrencyId, 
		contract_by: Self::Balance, 
		quote_price: Self::Balance, 
	) -> Self::Balance;
}

/// Abstraction over a fungible multi-stable-currency Token Elasticity of Supply system.
pub trait SerpTes<AccountId>: Stp258Currency<AccountId> {
	/// The quantity used to denote time; usually just a `BlockNumber`.
	type Moment;
	/// Contracts or expands the currency supply based on conditions.
	/// Filters through the conditions to see whether it's time to adjust supply or not.
	fn on_serp_block(
		now: Self::Moment, 
		stable_currency_id: Self::CurrencyId,
		stable_currency_price: Self::Balance,
		native_currency_price: Self::Balance, 
	) -> DispatchResult;

	/// Calculate the amount of supply change from a fraction given as `numerator` and `denominator`.
	fn supply_change(currency_id: Self::CurrencyId, new_price: Self::Balance) -> Self::Balance;	

	fn serp_elast(
		stable_currency_id: Self::CurrencyId, 
		stable_currency_price: Self::Balance, 
		native_currency_id: Self::CurrencyId,
		native_currency_price: Self::Balance,
	) -> DispatchResult;

	/// On Expand Supply, this is going to call `expand_supply`.
	/// This is often called by the `serp_elast` from the `SerpTes` trait.
	///
	fn on_expand_supply(
		currency_id: Self::CurrencyId, 
		expand_by: Self::Balance, 
		quote_price: Self::Balance, 
	) -> DispatchResult;

	/// On Contract Supply, this is going to call `contract_supply`.
	/// This is often called by the `serp_elast` from the `SerpTes` trait.
	///
	fn on_contract_supply(
		currency_id: Self::CurrencyId, 
		contract_by: Self::Balance, 
		quote_price: Self::Balance, 
	) -> DispatchResult;
}

/// Expected price oracle interface. `fetch_price` must return the amount of Coins exchanged for the tracked value.
pub trait FetchPrice<Balance> {
	/// The balance of an account.
	type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;

	/// Fetch the current price.
	fn fetch_price() -> Self::Balance;
}

/// A trait to provide relative price for two currencies
pub trait SerpTesPriceProvider<CurrencyId, Price> {
	fn get_price(base: CurrencyId, quote: CurrencyId) -> Option<Price>;
}
