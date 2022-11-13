/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>

pub use pallet::*;

use frame_support::{
	ensure,
	dispatch::DispatchResult,
	traits::{
		Currency,
		ReservableCurrency,
		ExistenceRequirement,
	},
	transactional,
};
use scale_codec::{Decode, Encode};
use sp_core::Get;
use sp_runtime::{
	traits::{StaticLookup, TrailingZeroInput},
};
use crate::types::{WorkerInfo, WorkerStatus};

type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub(crate) mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use crate::types::Attestation;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The system's currency for payment.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The minimum amount required to keep a worker registration.
		#[pallet::constant]
		type ReservedDeposit: Get<BalanceOf<Self>>;

		/// Verify attestation
		///
		/// SHOULD NOT SET TO TRUE ON PRODUCTION!!!
		#[pallet::constant]
		type AllowNoneAttestation: Get<bool>;

		// TODO: type WeightInfo: WeightInfo;
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The worker registered successfully
		Registered { identity: T::AccountId },
		/// The worker registered successfully
		Deregistered { identity: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Initial deposit for register a worker must equal or above `ExistentialDeposit`
		InitialDepositTooLow,
		/// Worker already registered
		AlreadyRegistered,
		/// The extrinsic origin isn't the worker's owner
		NotTheOwner,
		/// The extrinsic origin isn't the worker's identity
		NotTheWorker,
		/// The worker not exists
		WorkerNotExists,
	}

	/// Storage for computing_workers.
	#[pallet::storage]
	#[pallet::getter(fn workers)]
	pub type Workers<T: Config> =
		CountedStorageMap<_, Twox64Concat, T::AccountId, WorkerInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a computing workers.
		///
		/// ## Arguments
		/// - `origin`: Must be called by a `Signed` origin, it will become the worker's owner.
		/// - `identity`: The account who operate the worker. a identity can only manage one worker.
		/// - `initial_deposit`: Initial deposit amount.
		///
		/// ## Deposits/Fees
		/// The origin signed account will transfer `initial_deposit` to worker's current account
		/// that will use for slashing.
		/// If the balance below `ExistentialDeposit`, the worker will be removed
		///
		/// ## Events
		/// The `Registered` event is emitted in case of success.
		// TODO: #[pallet::weight(<T as Config>::WeightInfo::register())]
		#[pallet::weight(0)]
		#[transactional]
		pub fn register(
			origin: OriginFor<T>,
			identity: T::AccountId,
			initial_deposit: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_register(who, identity, initial_deposit)
		}

		/// Deregister a computing workers.
		#[pallet::weight(0)]
		#[transactional]
		pub fn deregister(
			origin: OriginFor<T>,
			identity: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_deregister(who, identity)
		}

		/// Initialize a worker, must called by the worker
		#[pallet::weight(0)]
		#[transactional]
		pub fn initialize_worker(
			origin: OriginFor<T>,
			spec_version: u32,
			attestation: Attestation
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			// TODO:
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn do_register(
		who: T::AccountId,
		identity: T::AccountId,
		initial_deposit: BalanceOf<T>
	) -> DispatchResult {
		let initial_reserved_deposit = T::ReservedDeposit::get();
		ensure!(
			initial_deposit >= initial_reserved_deposit,
			Error::<T>::InitialDepositTooLow
		);

		ensure!(
			!Workers::<T>::contains_key(&identity),
			Error::<T>::AlreadyRegistered
		);

		let worker_info = WorkerInfo {
			owner: who.clone(),
			identity: identity.clone(),
			reserved: initial_reserved_deposit,
			status: WorkerStatus::Registered,
			spec_version: 0,
			attestation_type: None,
			updated_at: T::BlockNumber::default(),
			expiring_at: T::BlockNumber::default(),
		};

		<T as Config>::Currency::transfer(
			&who,
			&identity,
			initial_deposit,
			ExistenceRequirement::KeepAlive
		)?;
		<T as Config>::Currency::reserve(
			&identity,
			initial_reserved_deposit
		)?;

		Workers::<T>::insert(&identity, worker_info);

		Self::deposit_event(
			Event::<T>::Registered { identity: identity.clone() }
		);
		Ok(())
	}

	fn do_deregister(
		who: T::AccountId,
		identity: T::AccountId,
	) -> DispatchResult {
		let worker_info = Workers::<T>::get(&identity).ok_or(Error::<T>::WorkerNotExists)?;
		Self::ensure_owner(&who, &worker_info)?;

		let reserved = worker_info.reserved;
		<T as Config>::Currency::unreserve(
			&identity,
			reserved
		);
		<T as Config>::Currency::transfer(
			&identity,
			&who,
			<T as Config>::Currency::free_balance(&identity),
			ExistenceRequirement::AllowDeath
		)?;

		Workers::<T>::remove(&identity);

		Self::deposit_event(
			Event::<T>::Deregistered { identity: identity.clone() }
		);
		Ok(())
	}

	fn ensure_owner(
		who: &T::AccountId, worker_info: &WorkerInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>
	) -> DispatchResult {
		ensure!(*who == worker_info.owner, Error::<T>::NotTheOwner);
		Ok(())
	}
}
