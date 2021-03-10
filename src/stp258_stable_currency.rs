use crate::arithmetic;
use codec::{Codec, FullCodec};
use frame_support::traits::{BalanceStatus, LockIdentifier};
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
pub trait Stp258StableCurrency<AccountId> {
	/// The currency identifier.
	type CurrencyId: FullCodec + Eq + PartialEq + Copy + MaybeSerializeDeserialize + Debug;

	/// The balance of an account.
	type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug + Default;
	
	/// The amount of Stp258Currency that are meant to track the value.
	type BaseUnit: AtLeast32BitUnsigned 
		+ TryInto<Self::Balance> 
		+ TryFrom<Self::Balance> 
		+ FullCodec 
		+ Copy 
		+ MaybeSerializeDeserialize 
		+ Debug 
		+ Default;

	// Public immutables

	/// Existential deposit of `currency_id`.
	fn minimum_balance(currency_id: Self::CurrencyId) -> Self::BaseUnit;

	/// The total amount of issuance of `currency_id`.
	fn total_issuance(currency_id: Self::CurrencyId) -> Self::BaseUnit;

	// The combined balance of `who` under `currency_id`.
	fn total_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::BaseUnit;

	// The free balance of `who` under `currency_id`.
	fn free_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::BaseUnit;

	/// A dry-run of `withdraw`. Returns `Ok` iff the account is able to make a
	/// withdrawal of the given amount.
	fn ensure_can_withdraw(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::BaseUnit) -> DispatchResult;

	// Public mutables

	/// Transfer some amount from one account to another.
	fn transfer(
		currency_id: Self::CurrencyId,
		from: &AccountId,
		to: &AccountId,
		amount: Self::BaseUnit,
	) -> DispatchResult;

	/// Add `amount` to the balance of `who` under `currency_id` and increase
	/// total issuance.
	fn deposit(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::BaseUnit) -> DispatchResult;

	/// Remove `amount` from the balance of `who` under `currency_id` and reduce
	/// total issuance.
	fn withdraw(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::BaseUnit) -> DispatchResult;

	/// Same result as `slash(currency_id, who, value)` (but without the
	/// side-effects) assuming there are no balance changes in the meantime and
	/// only the reserved balance is not taken into account.
	fn can_slash(currency_id: Self::CurrencyId, who: &AccountId, value: Self::BaseUnit) -> bool;

	/// Deduct the balance of `who` by up to `amount`.
	///
	/// As much funds up to `amount` will be deducted as possible.  If this is
	/// less than `amount`,then a non-zero value will be returned.
	fn slash(currency_id: Self::CurrencyId, who: &AccountId, amount: Self::BaseUnit) -> Self::BaseUnit;
}

/// Extended `Stp258Currency` with additional helper types and methods.
pub trait Stp258StableCurrencyExtended<AccountId>: Stp258StableCurrency<AccountId> {
	/// The type for balance related operations, typically signed int.
	type Amount: arithmetic::Signed
		+ TryInto<Self::BaseUnit>
		+ TryFrom<Self::BaseUnit>
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
pub trait Stp258StableCurrencyLockable<AccountId>: Stp258StableCurrency<AccountId> {
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
		amount: Self::BaseUnit,
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
		amount: Self::BaseUnit,
	) -> DispatchResult;

	/// Remove an existing lock.
	fn remove_lock(lock_id: LockIdentifier, currency_id: Self::CurrencyId, who: &AccountId) -> DispatchResult;
}

/// A fungible multi-stable-currency system where funds can be reserved from the user.
pub trait Stp258StableCurrencyReservable<AccountId>: Stp258StableCurrency<AccountId> {
	/// Same result as `reserve(who, value)` (but without the side-effects)
	/// assuming there are no balance changes in the meantime.
	fn can_reserve(currency_id: Self::CurrencyId, who: &AccountId, value: Self::BaseUnit) -> bool;

	/// Deducts up to `value` from reserved balance of `who`. This function
	/// cannot fail.
	///
	/// As much funds up to `value` will be deducted as possible. If the reserve
	/// balance of `who` is less than `value`, then a non-zero second item will
	/// be returned.
	fn slash_reserved(currency_id: Self::CurrencyId, who: &AccountId, value: Self::BaseUnit) -> Self::BaseUnit;

	/// Burns `value` from reserved balance of `who`. This function 
	/// returns `TotalIssuanceUnderflow` as error if underflow. Increases `total_issuance`.
	/// It changes `TotalIssuance`.
	/// Is a no-op if the `value` to be deposited is zero.
	fn burn_reserved(currency_id: Self::CurrencyId, who: &AccountId, value: Self::BaseUnit,) -> DispatchResult;

	/// Mints `value` to reserved balance of `who`. This function 
	/// returns `TotalIssuanceOverflow` as error if overflow. Increases `total_issuance`.
	/// It changes `TotalIssuance`.
	/// Is a no-op if the `value` to be deposited is zero.
	fn create_reserved(currency_id: Self::CurrencyId, who: &AccountId, value: Self::BaseUnit) -> DispatchResult;

	/// The amount of the balance of a given account that is externally
	/// reserved; this can still get slashed, but gets slashed last of all.
	///
	/// This balance is a 'reserve' balance that other subsystems use in order
	/// to set aside tokens that are still 'owned' by the account holder, but
	/// which are suspendable.
	fn reserved_balance(currency_id: Self::CurrencyId, who: &AccountId) -> Self::BaseUnit;

	/// Moves `value` from balance to reserved balance.
	///
	/// If the free balance is lower than `value`, then no funds will be moved
	/// and an `Err` will be returned to notify of this. This is different
	/// behavior than `unreserve`.
	fn reserve(currency_id: Self::CurrencyId, who: &AccountId, value: Self::BaseUnit) -> DispatchResult;

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
	fn unreserve(currency_id: Self::CurrencyId, who: &AccountId, value: Self::BaseUnit) -> Self::BaseUnit;

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
		value: Self::BaseUnit,
		status: BalanceStatus,
	) -> result::Result<Self::BaseUnit, DispatchError>;
}

/// Handler for account which has dust, need to burn or recycle it
pub trait OnStp258Dust<AccountId, CurrencyId, BaseUnit> {
	fn on_stp258_dust(who: &AccountId, currency_id: CurrencyId, amount: BaseUnit);
}

impl<AccountId, CurrencyId, BaseUnit> OnStp258Dust<AccountId, CurrencyId, BaseUnit> for () {
	fn on_stp258_dust(_: &AccountId, _: CurrencyId, _: BaseUnit) {}
}
