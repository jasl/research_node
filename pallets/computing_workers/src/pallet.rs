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
use sp_core::Get;
use sp_runtime::traits::StaticLookup;
use crate::types::{Attestation, WorkerInfo, WorkerStatus};

type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub(crate) mod pallet {
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
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The system's currency for payment.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The minimum amount required to keep a worker registration.
		#[pallet::constant]
		type ReservedDeposit: Get<BalanceOf<Self>>;

		/// Verify attestation
		///
		/// SHOULD NOT SET TO FALSE ON PRODUCTION!!!
		#[pallet::constant]
		type DisallowNoneAttestation: Get<bool>;

		// TODO: type WeightInfo: WeightInfo;
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The worker registered successfully
		Registered { worker: T::AccountId },
		/// The worker registered successfully
		Deregistered { worker: T::AccountId },
		/// The worker initialized successfully
		Initialized { worker: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The own must not the worker it self
		InvalidOwner,
		/// Initial deposit for register a worker must equal or above `ExistentialDeposit`
		InitialDepositTooLow,
		/// Worker already registered
		AlreadyRegistered,
		/// The extrinsic origin isn't the worker's owner
		NotTheOwner,
		/// The extrinsic origin isn't the worker
		NotTheWorker,
		/// The worker not exists
		NotExists,
		/// The worker must offline before do deregister
		MustOffline,
		/// The worker's status doesn't allow the operation
		WrongStatus,
		/// Attestation required
		MustProvideAttestation,
		/// Unsupported attestation
		UnsupportedAttestation,
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
		/// - `worker`: The worker.
		/// - `initial_deposit`: Initial deposit amount.
		///
		/// ## Deposits/Fees
		/// The origin signed account will transfer `initial_deposit` to worker's current account
		/// that will use for slashing.
		/// If the balance below `ReservedDeposit`, the worker will be removed
		///
		/// ## Events
		/// The `Registered` event is emitted in case of success.
		// TODO: #[pallet::weight(<T as Config>::WeightInfo::register())]
		#[pallet::weight(0)]
		#[transactional]
		pub fn register(
			origin: OriginFor<T>,
			worker: T::AccountId,
			initial_deposit: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_register(who, worker, initial_deposit)
		}

		/// Deregister a computing workers.
		#[pallet::weight(0)]
		#[transactional]
		pub fn deregister(
			origin: OriginFor<T>,
			worker: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_deregister(who, worker)
		}

		/// Initialize a worker, must called by the worker
		#[pallet::weight(0)]
		#[transactional]
		pub fn initialize_worker(
			origin: OriginFor<T>,
			spec_version: u32,
			attestation: Option<Attestation>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_initialize_worker(
				who, spec_version, attestation, frame_system::Pallet::<T>::block_number()
			)
		}
	}
}

impl<T: Config> Pallet<T> {
	fn do_register(
		who: T::AccountId,
		worker: T::AccountId,
		initial_deposit: BalanceOf<T>
	) -> DispatchResult {
		ensure!(
			who != worker,
			Error::<T>::InvalidOwner
		);

		let initial_reserved_deposit = T::ReservedDeposit::get();
		ensure!(
			initial_deposit >= initial_reserved_deposit,
			Error::<T>::InitialDepositTooLow
		);

		ensure!(
			!Workers::<T>::contains_key(&worker),
			Error::<T>::AlreadyRegistered
		);

		let worker_info = WorkerInfo {
			account: worker.clone(),
			owner: who.clone(),
			reserved: initial_reserved_deposit,
			status: WorkerStatus::Registered,
			spec_version: 0,
			attestation_type: None,
			updated_at: T::BlockNumber::default(),
			last_heartbeat_at: T::BlockNumber::default(),
		};

		<T as Config>::Currency::transfer(
			&who,
			&worker,
			initial_deposit,
			ExistenceRequirement::KeepAlive
		)?;
		<T as Config>::Currency::reserve(
			&worker,
			initial_reserved_deposit
		)?;

		Workers::<T>::insert(&worker, worker_info);

		Self::deposit_event(
			Event::<T>::Registered { worker: worker.clone() }
		);
		Ok(())
	}

	fn do_deregister(
		who: T::AccountId,
		worker: T::AccountId,
	) -> DispatchResult {
		let worker_info = Workers::<T>::get(&worker).ok_or(Error::<T>::NotExists)?;
		Self::ensure_owner(&who, &worker_info)?;
		ensure!(
			worker_info.status == WorkerStatus::Offline || worker_info.status == WorkerStatus::Registered,
			Error::<T>::MustOffline
		);

		let reserved = worker_info.reserved;
		<T as Config>::Currency::unreserve(
			&worker,
			reserved
		);
		<T as Config>::Currency::transfer(
			&worker,
			&who,
			<T as Config>::Currency::free_balance(&worker),
			ExistenceRequirement::AllowDeath
		)?;

		Workers::<T>::remove(&worker);

		Self::deposit_event(
			Event::<T>::Deregistered { worker: worker.clone() }
		);
		Ok(())
	}

	pub fn do_initialize_worker(
		who: T::AccountId,
		spec_version: u32,
		attestation: Option<Attestation>,
		block_number: T::BlockNumber,
	) -> DispatchResult {
		ensure!(
			!T::DisallowNoneAttestation::get() || attestation.is_some(),
			Error::<T>::MustProvideAttestation
		);

		let mut worker_info = Workers::<T>::get(&who).ok_or(Error::<T>::NotExists)?;
		Self::ensure_worker(&who, &worker_info)?;
		ensure!(
			worker_info.status == WorkerStatus::Registered,
			Error::<T>::WrongStatus
		);

		worker_info.spec_version = spec_version;
		worker_info.attestation_type =
			match attestation {
				None => None,
				// TODO: verify attestation
				_ => return Err(Error::<T>::UnsupportedAttestation.into())
			};
		worker_info.updated_at = block_number;
		worker_info.last_heartbeat_at = block_number;
		worker_info.status = WorkerStatus::Online;

		Workers::<T>::insert(&who, worker_info);

		Self::deposit_event(
			Event::<T>::Initialized { worker: who.clone() }
		);
		Ok(())
	}

	fn ensure_owner(
		who: &T::AccountId, worker_info: &WorkerInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>
	) -> DispatchResult {
		ensure!(*who == worker_info.owner, Error::<T>::NotTheOwner);
		Ok(())
	}

	fn ensure_worker(
		who: &T::AccountId, worker_info: &WorkerInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>
	) -> DispatchResult {
		ensure!(*who == worker_info.account, Error::<T>::NotTheWorker);
		Ok(())
	}
}
