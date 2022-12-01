use frame_support::dispatch::DispatchResult;
use crate::{
	types::{OnlinePayload, WorkerInfo, BalanceOf, NegativeImbalanceOf},
	Config,
};

/// Trait describing something that implements a hook for any operations to perform when a staker is
/// slashed.
pub trait WorkerLifecycleHooks<AccountId, Balance> {
	/// A hook for checking the worker whether can online,
	/// can use for add extra conditions check, if returns error, the worker will not be online
	fn can_online(worker: &AccountId, payload: &OnlinePayload) -> DispatchResult;
	/// A hook after the worker transited to online status,
	/// can use for add additional business logic, e.g. assign job, reserve more money
	fn after_online(worker: &AccountId);

	/// A hook for checking the worker whether can offline,
	/// can use for add extra conditions check,
	/// if returns error (e.g. still have job running), the worker will not be offline
	fn can_offline(worker: &AccountId) -> DispatchResult;
	/// A hook before the worker transited to offline status,
	/// can use for add additional business logic, e.g. un-reserve money
	/// when `force` is true, means it is the user force to do this, it may need to slash
	fn before_offline(worker: &AccountId, force: bool);

	/// A hook after the worker unresponsive,
	/// can use for add additional business logic, e.g. stop assigning job, do slash
	fn after_unresponsive(worker: &AccountId);

	/// A hook after the worker update its attestation,
	/// Can use for if interest in payload's custom field
	fn after_refresh_attestation(worker: &AccountId, payload: &OnlinePayload);

	/// A hook after the worker transited to requesting offline status,
	/// can use for add additional business logic, e.g. stop assigning job
	fn after_requesting_offline(worker: &AccountId);

	/// A hook after the worker force offline by attestation expired,
	/// can use for add additional business logic, e.g. stop assigning job, do slash
	fn after_attestation_expired(worker: &AccountId);

	/// A hook after the worker force offline by worker implementation blocked,
	/// can use for add additional business logic, e.g. stop assigning job, do slash
	fn after_impl_blocked(worker: &AccountId);

	/// A hook after the worker force offline by insufficient reserved funds,
	/// can use for add additional business logic, e.g. stop assigning job
	fn after_insufficient_reserved_funds(worker: &AccountId);
}

impl<AccountId, Balance> WorkerLifecycleHooks<AccountId, Balance> for () {
	fn can_online(_: &AccountId, _: &OnlinePayload) -> DispatchResult {
		Ok(())
	}

	fn after_online(_: &AccountId) {
		// Do nothing
	}

	fn can_offline(_: &AccountId) -> DispatchResult {
		Ok(())
	}

	fn before_offline(_: &AccountId, _: bool) {
		// Do nothing
	}

	fn after_unresponsive(_: &AccountId) {
		// Do nothing
	}

	fn after_refresh_attestation(_: &AccountId, _: &OnlinePayload) {
		// Do nothing
	}

	fn after_requesting_offline(_: &AccountId) {
		// Do nothing
	}

	fn after_attestation_expired(_: &AccountId) {
		// Do nothing
	}

	fn after_impl_blocked(_: &AccountId) {
		// Do nothing
	}

	fn after_insufficient_reserved_funds(_: &AccountId) {
		// Do nothing
	}
}

pub trait WorkerManageable<T: Config> {
	fn worker_info(worker: &T::AccountId) -> Option<WorkerInfo<T>>;

	fn worker_exists(worker: &T::AccountId) -> bool;

	fn reward(worker: &T::AccountId, source: &T::AccountId, value: BalanceOf<T>) -> DispatchResult;

	fn slash(worker: &T::AccountId, value: BalanceOf<T>) -> (NegativeImbalanceOf<T>, BalanceOf<T>);

	fn offline(worker: &T::AccountId) -> DispatchResult;
}

#[cfg(feature = "std")]
use frame_support::traits::Imbalance;
#[cfg(feature = "std")]
use sp_runtime::traits::Zero;

#[cfg(feature = "std")]
impl<T:Config> WorkerManageable<T> for () {
	fn worker_info(_: &T::AccountId) -> Option<WorkerInfo<T>> {
		None
	}

	fn worker_exists(_: &T::AccountId) -> bool {
		false
	}

	fn reward(_: &T::AccountId, _: &T::AccountId, _: BalanceOf<T>) -> DispatchResult {
		Ok(())
	}

	fn slash(_: &T::AccountId, _: BalanceOf<T>) -> (NegativeImbalanceOf<T>, BalanceOf<T>) {
		(NegativeImbalanceOf::<T>::zero(), BalanceOf::<T>::zero())
	}

	fn offline(_: &T::AccountId) -> DispatchResult {
		Ok(())
	}
}
