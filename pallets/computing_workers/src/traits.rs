use crate::types::WorkerInfo;
use frame_support::dispatch::DispatchResult;

/// Trait describing something that implements a hook for any operations to perform when a staker is
/// slashed.
pub trait WorkerLifecycleHooks<AccountId, Balance> {
	/// A hook before transit the worker to online status,
	/// can use for add extra conditions check, if returns error, the worker will not be online
	fn before_online(worker: &AccountId) -> DispatchResult;
	/// A hook after the worker transited to online status,
	/// can use for add additional business logic, e.g. assign job, reserve more money
	fn after_online(worker: &AccountId);

	/// A hook before transit the worker to offline status,
	/// can use for add extra conditions check,
	/// if returns error (e.g. still have job running), the worker will not be offline
	fn before_offline(worker: &AccountId) -> DispatchResult;
	/// A hook after the worker transited to offline status,
	/// can use for add additional business logic, e.g. un-reserve money
	fn after_offline(worker: &AccountId);

	/// A hook after the worker transited to unresponsive status,
	/// can use for add additional business logic, e.g. stop assigning job, do slash
	fn after_unresponsive(worker: &AccountId);

	/// A hook after the worker transited to attestation expired status,
	/// can use for add additional business logic, e.g. stop assigning job, do slash
	fn after_attestation_expired(worker: &AccountId);

	/// A hook after the worker transited to requesting offline status,
	/// can use for add additional business logic, e.g. stop assigning job
	fn after_requesting_offline(worker: &AccountId);

	/// A hook for settle the worker whether should be slashed,
	/// returns None means no slash
	fn settle_slash(worker: &AccountId) -> Option<Balance>;
}

impl<AccountId, Balance> WorkerLifecycleHooks<AccountId, Balance> for () {
	fn before_online(_: &AccountId) -> DispatchResult {
		Ok(())
	}

	fn after_online(_: &AccountId) {
		// Do nothing
	}

	fn before_offline(_: &AccountId) -> DispatchResult {
		Ok(())
	}

	fn after_offline(_: &AccountId) {
		// Do nothing
	}

	fn after_unresponsive(_: &AccountId) {
		// Do nothing
	}

	fn after_attestation_expired(_: &AccountId) {
		// Do nothing
	}
	
	fn after_requesting_offline(_: &AccountId) {
		// Do nothing
	}

	fn settle_slash(_: &AccountId) -> Option<Balance> {
		None
	}
}


// TODO: WIP
pub trait WorkerManageable<AccountId, Balance, BlockNumber> {
	/// Get the worker's info
	fn worker_info(who: &AccountId) -> Option<WorkerInfo<AccountId, Balance, BlockNumber>>;

	/// Slash the worker, deducts up to `value` from the combined balance of it,
	/// if free balance doesn't enough, the worker will be forced to deregister,
	/// it can specify whether force to set the worker to offline status
	/// It won't trigger any hooks, so you must do clean up e.g. stop assigning job) before call the function
	fn slash(who: &AccountId, value: Balance, force_offline: bool);
}

#[cfg(feature = "std")]
impl<AccountId, Balance, BlockNumber> WorkerManageable<AccountId, Balance, BlockNumber> for () {
	fn worker_info(_: &AccountId) -> Option<WorkerInfo<AccountId, Balance, BlockNumber>> {
		None
	}

	fn slash(_: &AccountId, _: Balance, _: bool) {
		// Do nothing
	}
}
