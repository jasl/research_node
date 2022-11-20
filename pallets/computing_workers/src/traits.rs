use crate::types::{OnlinePayload, WorkerInfo};
use scale_codec::MaxEncodedLen;
use sp_std::fmt::Debug;
use sp_runtime::{
	traits::MaybeSerializeDeserialize,
	FixedPointOperand,
};
use frame_support::{
	dispatch::DispatchResult,
	traits::{
		Imbalance,
		tokens::Balance
	}
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

	/// A hook after the worker transited to unresponsive status,
	/// can use for add additional business logic, e.g. stop assigning job, do slash
	fn after_unresponsive(worker: &AccountId);

	/// A hook after the worker update its attestation,
	/// Can use for if interest in payload's custom field
	fn after_refresh_attestation(worker: &AccountId, payload: &OnlinePayload);

	/// A hook after the worker transited to attestation expired status,
	/// can use for add additional business logic, e.g. stop assigning job, do slash
	fn after_attestation_expired(worker: &AccountId);

	/// A hook after the worker transited to requesting offline status,
	/// can use for add additional business logic, e.g. stop assigning job
	fn after_requesting_offline(worker: &AccountId);
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

	fn after_refresh_attestation(_: &AccountId, _: &OnlinePayload) {
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
}

pub trait WorkerManageable<AccountId, BlockNumber> {
	/// The balance of an account.
	type Balance: Balance + MaybeSerializeDeserialize + Debug + MaxEncodedLen + FixedPointOperand;

	/// The opaque token type for an imbalance. This is returned by unbalanced operations
	/// and must be dealt with. It may be dropped but cannot be cloned.
	type PositiveImbalance: Imbalance<Self::Balance, Opposite = Self::NegativeImbalance>;

	/// The opaque token type for an imbalance. This is returned by unbalanced operations
	/// and must be dealt with. It may be dropped but cannot be cloned.
	type NegativeImbalance: Imbalance<Self::Balance, Opposite = Self::PositiveImbalance>;

	fn worker_info(worker: &AccountId) -> Option<WorkerInfo<AccountId, Self::Balance, BlockNumber>>;

	fn reward(worker: &AccountId, source: &AccountId, value: Self::Balance) -> DispatchResult;

	fn slash(worker: &AccountId, value: Self::Balance) -> (Self::NegativeImbalance, Self::Balance);

	fn offline(worker: &AccountId) -> DispatchResult;
}

#[cfg(feature = "std")]
impl<AccountId, BlockNumber> WorkerManageable<AccountId, BlockNumber> for () {
	type Balance = u128;
	type PositiveImbalance = ();
	type NegativeImbalance = ();

	fn worker_info(_: &AccountId) -> Option<WorkerInfo<AccountId, Self::Balance, BlockNumber>> {
		None
	}

	fn reward(_: &AccountId, _: &AccountId, _: Self::Balance) -> DispatchResult {
		Ok(())
	}

	fn slash(_: &AccountId, _: Self::Balance) -> (Self::NegativeImbalance, Self::Balance) {
		((), 0)
	}

	fn offline(_: &AccountId) -> DispatchResult {
		Ok(())
	}
}
