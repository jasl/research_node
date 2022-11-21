#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// The log target of this pallet.
pub const LOG_TARGET: &str = "runtime::fake_computing";

// syntactic sugar for logging.
#[macro_export]
macro_rules! log {
	($level:tt, $patter:expr $(, $values:expr)* $(,)?) => {
		log::$level!(
			target: $crate::LOG_TARGET,
			concat!("[{:?}] ", $patter), <frame_system::Pallet<T>>::block_number() $(, $values)*
		)
	};
}

use frame_support::traits::Currency;
type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

pub const MILLI_CENTS: u128 = 1_000_000;
pub const CENTS: u128 = 1_000 * MILLI_CENTS;
pub const UNITS: u128 = 1_000 * CENTS;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, sp_std::prelude::*, traits::Currency, sp_runtime::SaturatedConversion};
	use frame_system::pallet_prelude::*;

	use pallet_computing_workers::{
		traits::{WorkerLifecycleHooks, WorkerManageable},
		types::OnlinePayload,
	};
	use crate::{BalanceOf, UNITS, log};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The system's currency for payment.
		type Currency: Currency<Self::AccountId>;

		type WorkerManageable: WorkerManageable<Self::AccountId, Self::BlockNumber>;
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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn start(origin: OriginFor<T>, worker: T::AccountId) -> DispatchResult {
			let _who = ensure_signed(origin)?; // Maybe `ensure_root` ?

			ensure!(
				!<RunningWorkers<T>>::contains_key(&worker),
				Error::<T>::AlreadyStarted
			);
			ensure!(
				!<BlockedWorkers<T>>::contains_key(&worker),
				Error::<T>::Blocked
			);

			<RunningWorkers<T>>::insert(&worker, ());

			Self::deposit_event(
				Event::Started { worker }
			);

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn stop(origin: OriginFor<T>, worker: T::AccountId) -> DispatchResult {
			let _who = ensure_signed(origin)?; // Maybe `ensure_root` ?

			ensure!(
				<RunningWorkers<T>>::contains_key(&worker),
				Error::<T>::AlreadyStopped
			);

			<RunningWorkers<T>>::remove(&worker);

			Self::deposit_event(
				Event::Stopped { worker }
			);

			Ok(())
		}
	}

	impl<T: Config> WorkerLifecycleHooks<T::AccountId, BalanceOf<T>> for Pallet<T> {
		fn can_online(worker: &T::AccountId, _payload: &OnlinePayload) -> DispatchResult {
			log!(info, "can_online: {:?}", worker);

			ensure!(
				!<BlockedWorkers<T>>::contains_key(worker),
				Error::<T>::Blocked
			);
			ensure!(
				!<RunningWorkers<T>>::contains_key(worker),
				Error::<T>::AlreadyStarted
			);

			Ok(())
		}

		fn after_online(worker: &T::AccountId) {
			log!(info, "after_online: {:?}", worker);

			<RunningWorkers<T>>::insert(worker, ());

			Self::deposit_event(
				Event::Started { worker: worker.clone() }
			);
		}

		fn can_offline(worker: &T::AccountId) -> DispatchResult {
			log!(info, "can_offline: {:?}", worker);

			ensure!(
				!<RunningWorkers<T>>::contains_key(worker),
				Error::<T>::Computing
			);

			Ok(())
		}

		fn before_offline(worker: &T::AccountId, force: bool) {
			log!(info, "before_offline: {:?}", worker);

			if !<RunningWorkers<T>>::contains_key(worker) {
				return
			}

			if force {
				<T::WorkerManageable as WorkerManageable<_, _>>::slash(worker, (10 * UNITS).saturated_into());
			}

			<RunningWorkers<T>>::remove(worker);
		}

		fn after_unresponsive(worker: &T::AccountId) {
			log!(info, "after_unresponsive: {:?}", worker);

			if !<RunningWorkers<T>>::contains_key(worker) {
				return
			}

			<T::WorkerManageable as WorkerManageable<_, _>>::slash(worker, (10 * UNITS).saturated_into());
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

			<T::WorkerManageable as WorkerManageable<_, _>>::slash(worker, (10 * UNITS).saturated_into());
			<RunningWorkers<T>>::remove(worker);
		}

		fn after_insufficient_reserved_funds(worker: &T::AccountId) {
			log!(info, "after_insufficient_reserved_funds: {:?}", worker);

			if !<RunningWorkers<T>>::contains_key(worker) {
				return
			}

			<T::WorkerManageable as WorkerManageable<_, _>>::slash(worker, (10 * UNITS).saturated_into());
			<RunningWorkers<T>>::remove(worker);
		}
	}
}