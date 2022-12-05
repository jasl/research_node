#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

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
	types::{BalanceOf, OnlinePayload},
};
use crate::types::{Job, JobStatus, JobResult, JobPayloadVec};

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

		type MaxJobPayloadLen: Get<u32>;

		#[pallet::constant]
		type SlashingCardinal: Get<BalanceOf<Self>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn assigned_jobs)]
	pub(crate) type AssignedJobs<T: Config> = StorageMap<
		_,
		Identity, T::AccountId,
		Job<T>
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Slashed { worker: T::AccountId, amount: BalanceOf<T> },
		JobAssigned { worker: T::AccountId },
		JobStarted { worker: T::AccountId },
		JobCompleted { worker: T::AccountId, result: JobResult },
		JobRemoved { worker: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InsufficientFundsForSlashing,
		NoPermission,
		NotTheOwner,
		WorkerNotExists,
		JobNotExists,
		AlreadyAssigned,
		AlreadyStarted,
		CantRemove,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn create_job(
			origin: OriginFor<T>,
			worker: T::AccountId,
			payload: JobPayloadVec<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::ensure_owner(&who, &worker)?;

			ensure!(
				!AssignedJobs::<T>::contains_key(&worker),
				Error::<T>::AlreadyAssigned
			);

			let job = Job {
				status: JobStatus::Created,
				result: None,
				created_by: who.clone(),
				created_at: Some(frame_system::Pallet::<T>::block_number()),
				started_at: None,
				completed_at: None,
				payload,
			};
			AssignedJobs::<T>::insert(&worker, job);

			Self::deposit_event(Event::JobAssigned { worker });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn start_job(origin: OriginFor<T>) -> DispatchResult {
			let worker = ensure_signed(origin)?;
			// ensure worker or owner
			Self::ensure_worker(&worker)?;

			let Some(mut job) = AssignedJobs::<T>::get(&worker) else {
				return Err(Error::<T>::JobNotExists.into())
			};

			ensure!(
				job.status == JobStatus::Created,
				Error::<T>::AlreadyStarted
			);

			job.status = JobStatus::Started;
			job.started_at = Some(frame_system::Pallet::<T>::block_number());

			AssignedJobs::<T>::insert(&worker, job);

			Self::deposit_event(Event::JobStarted { worker });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn complete_job(origin: OriginFor<T>, result: JobResult) -> DispatchResult {
			let worker = ensure_signed(origin)?;
			Self::ensure_worker(&worker)?;

			let Some(mut job) = AssignedJobs::<T>::get(&worker) else {
				return Err(Error::<T>::JobNotExists.into())
			};

			ensure!(
				job.status == JobStatus::Started,
				Error::<T>::AlreadyStarted
			);

			job.status = JobStatus::Completed;
			job.result = Some(result);
			job.completed_at = Some(frame_system::Pallet::<T>::block_number());

			AssignedJobs::<T>::insert(&worker, job);

			Self::deposit_event(Event::JobCompleted { worker, result });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn remove_job(origin: OriginFor<T>, worker: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let Some(worker_info) = T::WorkerManageable::worker_info(&worker) else {
				return Err(Error::<T>::WorkerNotExists.into())
			};

			ensure!(
				who == worker || who == worker_info.owner,
				Error::<T>::NoPermission
			);

			let Some(job) = AssignedJobs::<T>::get(&worker) else {
				return Err(Error::<T>::JobNotExists.into())
			};

			ensure!(
				match job.status {
					JobStatus::Created |
					JobStatus::Completed |
					JobStatus::Timeout |
					JobStatus::Cancelled => true,
					_ => false
				},
				Error::<T>::CantRemove
			);

			AssignedJobs::<T>::remove(&worker);

			// TODO: Do settlement

			Self::deposit_event(Event::JobRemoved { worker });

			Ok(())
		}

		// TODO: Cancel Job (called by the owner)
		// TODO: Heartbeat for the job, or it will be timeout
		// TODO: How to clean up timeout jobs and slash those workers?
	}

	impl<T: Config> Pallet<T> {
		fn ensure_owner(who: &T::AccountId, worker: &T::AccountId) -> DispatchResult {
			if let Some(worker_info) = T::WorkerManageable::worker_info(worker) {
				ensure!(who == &worker_info.owner, Error::<T>::NotTheOwner);
			} else {
				return Err(Error::<T>::WorkerNotExists.into())
			}

			Ok(())
		}

		fn ensure_worker(who: &T::AccountId) -> DispatchResult {
			ensure!(
				T::WorkerManageable::worker_exists(who),
				Error::<T>::WorkerNotExists
			);

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
