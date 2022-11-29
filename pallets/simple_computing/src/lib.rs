#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

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

use frame_support::{
	traits::ConstU32,
	BoundedVec
};
use pallet_computing_workers::BalanceOf;

type JobId = BoundedVec<u8, ConstU32<64>>;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		sp_runtime::Saturating,
		sp_std::prelude::*,
	};
	use frame_system::pallet_prelude::*;

	use crate::{log, BalanceOf, JobId};
	use pallet_computing_workers::{
		traits::{WorkerLifecycleHooks, WorkerManageable},
		types::OnlinePayload,
	};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_computing_workers::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type WorkerManageable: WorkerManageable<Self>;

		#[pallet::constant]
		type SlashingCardinal: Get<BalanceOf<Self>>;
	}

	#[pallet::storage]
	#[pallet::getter(fn running_workers)]
	pub type RunningWorkers<T: Config> = StorageMap<_, Identity, T::AccountId, ()>;

	#[pallet::storage]
	#[pallet::getter(fn blocked_workers)]
	pub type BlockedWorkers<T: Config> = StorageMap<_, Identity, T::AccountId, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Started { worker: T::AccountId },
		Stopped { worker: T::AccountId },
		Slashed { worker: T::AccountId, amount: BalanceOf<T> },
		Offline { worker: T::AccountId },
		Blocked { worker: T::AccountId },
		Unblocked { worker: T::AccountId },
		JobAssigned { worker: T::AccountId, job_id: JobId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AlreadyStarted,
		AlreadyStopped,
		Computing,
		Blocked,
		NotStarted,
		InsufficientFundsForSlashing,
		NotTheOwner,
		WorkerNotExists,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn start(origin: OriginFor<T>, worker: T::AccountId) -> DispatchResult {
			Self::ensure_owner_or_root(origin, &worker)?;

			ensure!(!<RunningWorkers<T>>::contains_key(&worker), Error::<T>::AlreadyStarted);
			ensure!(!<BlockedWorkers<T>>::contains_key(&worker), Error::<T>::Blocked);

			<RunningWorkers<T>>::insert(&worker, ());

			Self::deposit_event(Event::Started { worker });
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn stop(origin: OriginFor<T>, worker: T::AccountId) -> DispatchResult {
			Self::ensure_owner_or_root(origin, &worker)?;

			ensure!(<RunningWorkers<T>>::contains_key(&worker), Error::<T>::AlreadyStopped);

			<RunningWorkers<T>>::remove(&worker);

			Self::deposit_event(Event::Stopped { worker });
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn assign_job(origin: OriginFor<T>, worker: T::AccountId, job_id: JobId) -> DispatchResult {
			Self::ensure_owner_or_root(origin, &worker)?;

			ensure!(<RunningWorkers<T>>::contains_key(&worker), Error::<T>::AlreadyStopped);

			Self::deposit_event(Event::JobAssigned { worker, job_id });
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
		fn can_online(worker: &T::AccountId, _payload: &OnlinePayload) -> DispatchResult {
			log!(info, "can_online: {:?}", worker);

			ensure!(!<BlockedWorkers<T>>::contains_key(worker), Error::<T>::Blocked);
			ensure!(!<RunningWorkers<T>>::contains_key(worker), Error::<T>::AlreadyStarted);

			Ok(())
		}

		fn after_online(worker: &T::AccountId) {
			log!(info, "after_online: {:?}", worker);

			<RunningWorkers<T>>::insert(worker, ());

			Self::deposit_event(Event::Started { worker: worker.clone() });
		}

		fn can_offline(worker: &T::AccountId) -> DispatchResult {
			log!(info, "can_offline: {:?}", worker);

			ensure!(!<RunningWorkers<T>>::contains_key(worker), Error::<T>::Computing);

			Ok(())
		}

		fn before_offline(worker: &T::AccountId, force: bool) {
			log!(info, "before_offline: {:?}", worker);

			if !<RunningWorkers<T>>::contains_key(worker) {
				return
			}

			if force {
				<T::WorkerManageable as WorkerManageable<_>>::slash(worker, T::SlashingCardinal::get().saturating_mul(10u32.into()));
			}

			<RunningWorkers<T>>::remove(worker);
		}

		fn after_unresponsive(worker: &T::AccountId) {
			log!(info, "after_unresponsive: {:?}", worker);

			if !<RunningWorkers<T>>::contains_key(worker) {
				return
			}

			<T::WorkerManageable as WorkerManageable<_>>::slash(worker, T::SlashingCardinal::get().saturating_mul(10u32.into()));
			<RunningWorkers<T>>::remove(worker);
		}

		fn after_refresh_attestation(worker: &T::AccountId, _: &OnlinePayload) {
			log!(info, "after_refresh_attestation: {:?}", worker);
		}

		fn after_requesting_offline(worker: &T::AccountId) {
			log!(info, "after_requesting_offline: {:?}", worker);
		}

		fn after_attestation_expired(worker: &T::AccountId) {
			log!(info, "after_attestation_expired: {:?}", worker);

			if !<RunningWorkers<T>>::contains_key(worker) {
				return
			}

			<T::WorkerManageable as WorkerManageable<_>>::slash(worker, T::SlashingCardinal::get().saturating_mul(10u32.into()));
			<RunningWorkers<T>>::remove(worker);
		}

		fn after_impl_blocked(worker: &T::AccountId) {
			log!(info, "after_impl_blocked: {:?}", worker);

			if !<RunningWorkers<T>>::contains_key(worker) {
				return
			}

			<T::WorkerManageable as WorkerManageable<_>>::slash(worker, T::SlashingCardinal::get().saturating_mul(10u32.into()));
			<RunningWorkers<T>>::remove(worker);
		}

		fn after_insufficient_reserved_funds(worker: &T::AccountId) {
			log!(info, "after_insufficient_reserved_funds: {:?}", worker);

			if !<RunningWorkers<T>>::contains_key(worker) {
				return
			}

			<T::WorkerManageable as WorkerManageable<_>>::slash(worker, T::SlashingCardinal::get().saturating_mul(10u32.into()));
			<RunningWorkers<T>>::remove(worker);
		}
	}
}