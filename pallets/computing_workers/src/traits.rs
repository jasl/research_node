use crate::types::WorkerInfo;
use frame_support::dispatch::DispatchResult;

pub trait WorkerManageable<AccountId, Balance, BlockNumber> {
	// PUBLIC IMMUTABLES

	/// The worker info of `who`
	fn worker_info(who: &AccountId) -> Option<WorkerInfo<AccountId, Balance, BlockNumber>>;

	/// Deducts up to `value` from the combined balance of `who`, preferring to deduct from the
	/// free balance. This function cannot fail.
	///
	/// The resulting imbalance is the first item of the tuple returned.
	///
	/// As much funds up to `value` will be deducted as possible. If this is less than `value`,
	/// then a non-zero second item will be returned.
	fn slash(who: &AccountId, value: Balance) -> DispatchResult;
}

#[cfg(feature = "std")]
impl<AccountId, Balance, BlockNumber> WorkerManageable<AccountId, Balance, BlockNumber> for () {
	fn worker_info(_: &AccountId) -> Option<WorkerInfo<AccountId, Balance, BlockNumber>> {
		None
	}

	fn slash(_: &AccountId, _: Balance) -> DispatchResult {
		Ok(())
	}
}
