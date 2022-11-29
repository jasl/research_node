#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

pub mod macros;
pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The log target of this pallet.
pub const LOG_TARGET: &str = "runtime::simple_computing";

// Syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: $crate::LOG_TARGET,
			concat!("[{:?}] ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

use pallet_computing_workers::{
	traits::{WorkerLifecycleHooks, WorkerManageable},
	types::OnlinePayload,
	BalanceOf,
};
use crate::{
	types::{AutoIncremental, Job}
};

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_computing_workers::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type WorkerManageable: WorkerManageable<Self>;

		type JobId: Member + Parameter + MaxEncodedLen + Copy + AutoIncremental;

		#[pallet::constant]
		type SlashingCardinal: Get<BalanceOf<Self>>;
	}

	#[pallet::storage]
	pub(crate) type NextJobId<T: Config> = StorageMap<_, Identity, T::AccountId, T::JobId>;

	#[pallet::storage]
	pub(crate) type WorkingJobs<T: Config> = StorageDoubleMap<
		_,
		Identity, T::AccountId,
		Identity, T::JobId,
		Job<T::AccountId, T::BlockNumber>
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Slashed { worker: T::AccountId, amount: BalanceOf<T> },
		JobAssigned { worker: T::AccountId, job_id: T::JobId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InsufficientFundsForSlashing,
		NotTheOwner,
		WorkerNotExists,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn assign_job(
			origin: OriginFor<T>,
			worker: T::AccountId,
			job: Job<T::AccountId, T::BlockNumber>
		) -> DispatchResult {
			Self::ensure_owner_or_root(origin, &worker)?;

			// TODO:

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn ensure_owner_or_root(origin: OriginFor<T>, worker: &T::AccountId) -> DispatchResult {
			let who = ensure_signed_or_root(origin)?;
			if let Some(worker_info) = T::WorkerManageable::worker_info(worker) {
				if let Some(owner) = who {
					ensure!(owner == worker_info.owner, Error::<T>::NotTheOwner)
				}
			} else {
				return Err(Error::<T>::WorkerNotExists.into())
			}

			Ok(())
		}
	}

	impl<T: Config> WorkerLifecycleHooks<T::AccountId, BalanceOf<T>> for Pallet<T> {
		fn can_online(_worker: &T::AccountId, _payload: &OnlinePayload) -> DispatchResult {
			Ok(())
		}

		fn after_online(_worker: &T::AccountId) {
			// Nothing to do
		}

		fn can_offline(_worker: &T::AccountId) -> DispatchResult {
			// TODO:

			Ok(())
		}

		fn before_offline(_worker: &T::AccountId, _force: bool) {
			// TODO:
		}

		fn after_unresponsive(_worker: &T::AccountId) {
			// TODO:
		}

		fn after_refresh_attestation(_worker: &T::AccountId, _: &OnlinePayload) {
			// Nothing to do
		}

		fn after_requesting_offline(_worker: &T::AccountId) {
			// TODO:
		}

		fn after_attestation_expired(_worker: &T::AccountId) {
			// TODO:
		}

		fn after_impl_blocked(_worker: &T::AccountId) {
			// TODO:
		}

		fn after_insufficient_reserved_funds(_worker: &T::AccountId) {
			// TODO:
		}
	}
}
