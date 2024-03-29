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
pub const LOG_TARGET: &str = "runtime::fake_computing";

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
	sp_runtime::Saturating,
};
use pallet_computing_workers::{
	traits::{WorkerLifecycleHooks, WorkerManageable},
	primitives::{OfflineReason, OnlinePayload, VerifiedAttestation},
};

// pub(crate) type BalanceOf<T> =
// 	<<T as Config>::Currency as WorkerManageable<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber>>::Balance;

pub(crate) type BalanceOf<T> =
	<<T as Config>::WorkerManageable as WorkerManageable<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type WorkerManageable: WorkerManageable<Self::AccountId, Self::BlockNumber>;

		#[pallet::constant]
		type SlashingCardinal: Get<BalanceOf<Self>>;
	}

	#[pallet::storage]
	pub type RunningWorkers<T: Config> = StorageMap<_, Identity, T::AccountId, ()>;

	#[pallet::storage]
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
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		AlreadyStarted,
		AlreadyStopped,
		Blocked,
		NotStarted,
		InsufficientFundsForSlashing,
		NotTheOwner,
		WorkerNotExists,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn start(origin: OriginFor<T>, worker: T::AccountId) -> DispatchResult {
			Self::ensure_owner_or_root(origin, &worker)?;

			ensure!(!<RunningWorkers<T>>::contains_key(&worker), Error::<T>::AlreadyStarted);
			ensure!(!<BlockedWorkers<T>>::contains_key(&worker), Error::<T>::Blocked);

			<RunningWorkers<T>>::insert(&worker, ());

			Self::deposit_event(Event::Started { worker });
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn stop(origin: OriginFor<T>, worker: T::AccountId) -> DispatchResult {
			Self::ensure_owner_or_root(origin, &worker)?;

			ensure!(<RunningWorkers<T>>::contains_key(&worker), Error::<T>::AlreadyStopped);

			<RunningWorkers<T>>::remove(&worker);

			Self::deposit_event(Event::Stopped { worker });
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

	impl<T: Config> WorkerLifecycleHooks<T::AccountId> for Pallet<T> {
		fn can_online(worker: &T::AccountId, _payload: &OnlinePayload, _verified_attestation: &Option<VerifiedAttestation>) -> DispatchResult {
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

		fn can_offline(worker: &T::AccountId) -> bool {
			log!(info, "can_offline: {:?}", worker);

			!<RunningWorkers<T>>::contains_key(worker)
		}

		fn before_offline(worker: &T::AccountId, reason: OfflineReason) {
			log!(info, "before_offline: {:?}", worker);

			if !<RunningWorkers<T>>::contains_key(worker) {
				return
			}

			if reason != OfflineReason::Graceful {
				T::WorkerManageable::slash(
					worker,
					T::SlashingCardinal::get().saturating_mul(10u32.into()),
				);
			}

			<RunningWorkers<T>>::remove(worker);
		}

		fn after_refresh_attestation(worker: &T::AccountId, _payload: &OnlinePayload, _verified_attestation: &Option<VerifiedAttestation>) {
			log!(info, "after_refresh_attestation: {:?}", worker);
		}

		fn after_requesting_offline(worker: &T::AccountId) {
			log!(info, "after_requesting_offline: {:?}", worker);
		}

		fn before_deregister(worker: &T::AccountId) {
			log!(info, "before_deregister: {:?}", worker);
		}
	}
}
