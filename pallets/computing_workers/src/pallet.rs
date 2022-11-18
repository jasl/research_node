/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, ExistenceRequirement, ReservableCurrency, Get, UnixTime},
	transactional,
};
use scale_codec::Encode;
use sp_runtime::{
	traits::{StaticLookup, Zero},
	DispatchError, SaturatedConversion,
};
use sp_core::{H256, sr25519};
use sp_io::crypto::sr25519_verify;
use sp_std::prelude::*;
use crate::types::{
	Attestation, AttestationMethod, AttestationVerifyMaterial, VerifiedAttestation,
	FlipFlopStage, OnlinePayload, WorkerInfo, WorkerStatus, AttestationError};

type AccountIdLookupOf<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;
type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

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

		/// Time used for verify attestation
		type UnixTime: UnixTime;

		/// Max number of clean up offline workers queue
		#[pallet::constant]
		type MarkingOfflinePerBlockLimit: Get<u32>;

		/// Max number of moving unresponsive workers to pending offline workers queue
		#[pallet::constant]
		type MarkingUnresponsivePerBlockLimit: Get<u32>;

		/// The minimum amount required to keep a worker registration.
		#[pallet::constant]
		type ReservedDeposit: Get<BalanceOf<Self>>;

		/// The duration (blocks) of collecting workers' heartbeats
		#[pallet::constant]
		type CollectingHeartbeatsDuration: Get<u32>;

		/// The duration (blocks) of collecting workers' heartbeats
		#[pallet::constant]
		type AttestationValidityDuration: Get<u32>;

		/// Allow Opt out attestation
		///
		/// SHOULD NOT SET TO FALSE ON PRODUCTION!!!
		#[pallet::constant]
		type DisallowOptOutAttestation: Get<bool>;

		/// Allow Opt out attestation
		///
		/// SHOULD NOT SET TO FALSE ON PRODUCTION!!!
		#[pallet::constant]
		type DisallowNonTEEAttestation: Get<bool>;

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
		/// The worker is online
		Online { worker: T::AccountId },
		/// The worker is requesting offline
		RequestingOffline { worker: T::AccountId },
		/// The worker is offline
		Offline { worker: T::AccountId, force: bool },
		/// The worker send heartbeat successfully
		HeartbeatReceived { worker: T::AccountId },
		/// The work refresh attestation successfully
		AttestationRefreshed { worker: T::AccountId },
		/// The work refresh attestation expired
		AttestationExpired { worker: T::AccountId },
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
		/// The worker is not online
		NotOnline,
		/// The worker must offline before do deregister
		NotOffline,
		/// The worker's status doesn't allow the operation
		WrongStatus,
		/// Attestation required
		MustProvideAttestation,
		/// Attestation expired,
		ExpiredAttestation,
		/// Attestation invalid,
		InvalidAttestation,
		/// Attestation payload invalid
		CanNotVerifyPayload,
		/// Can not downgrade
		CanNotDowngrade,
		/// Worker software changed, it must offline first
		SoftwareChanged,
		/// Can't verify payload
		MismatchedPayloadSignature,
		/// The runtime disallowed NonTEE worker
		DisallowNonTEEAttestation,
		/// Unsupported attestation
		UnsupportedAttestation,
		/// The attestation method must not change
		AttestationMethodChanged,
		/// Attestation expired
		AttestationRefreshNeeded,
		/// Intermission
		Intermission,
		/// AlreadySentHeartbeat
		AlreadySentHeartbeat,
	}

	/// Storage for computing_workers.
	#[pallet::storage]
	#[pallet::getter(fn workers)]
	pub type Workers<T: Config> = CountedStorageMap<_, Identity, T::AccountId, WorkerInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>>;

	/// Storage for pending offline workers
	#[pallet::storage]
	#[pallet::getter(fn pending_offline_workers)]
	pub type PendingOfflineWorkers<T: Config> = CountedStorageMap<_, Identity, T::AccountId, ()>;

	/// Storage for flip set, this is for online checking
	#[pallet::storage]
	#[pallet::getter(fn flip_set)]
	pub type FlipSet<T: Config> = CountedStorageMap<_, Identity, T::AccountId, ()>;

	/// Storage for flop set, this is for online checking
	#[pallet::storage]
	#[pallet::getter(fn flop_set)]
	pub type FlopSet<T: Config> = CountedStorageMap<_, Identity, T::AccountId, ()>;

	/// Storage for stage of flip-flop, this is used for online checking
	#[pallet::storage]
	#[pallet::getter(fn flip_flop_stage)]
	pub type FlipOrFlop<T> = StorageValue<_, FlipFlopStage, ValueQuery>;

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let mut reads: u64 = 2; // read FlipOrFlop & read PendingOfflineWorkers
			let mut writes: u64 = 0;

			let mut flip_or_flop = FlipOrFlop::<T>::get();
			if (n % T::CollectingHeartbeatsDuration::get().into()).is_zero() {
				match flip_or_flop {
					FlipFlopStage::Flip => {
						flip_or_flop = FlipFlopStage::FlipToFlop;
						FlipOrFlop::<T>::set(flip_or_flop);
						writes += 1;
					}
					FlipFlopStage::Flop => {
						flip_or_flop = FlipFlopStage::FlopToFlip;
						FlipOrFlop::<T>::set(flip_or_flop);
						writes += 1;
					},
					_ => {}
				}
			}
			match flip_or_flop {
				FlipFlopStage::FlipToFlop => {
					let unresponsive_workers =
						FlipSet::<T>::iter_keys().take(T::MarkingUnresponsivePerBlockLimit::get() as usize).collect::<Vec<_>>();
					if unresponsive_workers.is_empty() {
						FlipOrFlop::<T>::set(FlipFlopStage::Flop);
						writes += 1;
					} else {
						for worker in &unresponsive_workers {
							FlipSet::<T>::remove(worker);
							PendingOfflineWorkers::<T>::insert(worker, ());
							Workers::<T>::mutate(worker, |worker_info| {
								worker_info.as_mut().map(|mut info| info.status = WorkerStatus::Unresponsive);
							});
						}
						reads += unresponsive_workers.len() as u64;
						writes += unresponsive_workers.len().saturating_mul(3) as u64;
					}
				}
				FlipFlopStage::FlopToFlip => {
					let unresponsive_workers =
						FlopSet::<T>::iter_keys().take(T::MarkingUnresponsivePerBlockLimit::get() as usize).collect::<Vec<_>>();
					if unresponsive_workers.is_empty() {
						FlipOrFlop::<T>::set(FlipFlopStage::Flip);
						writes += 1;
					} else {
						for worker in &unresponsive_workers {
							FlopSet::<T>::remove(worker);
							PendingOfflineWorkers::<T>::insert(worker, ());
							Workers::<T>::mutate(worker, |worker_info| {
								worker_info.as_mut().map(|mut info| info.status = WorkerStatus::Unresponsive);
							});
						}
						reads += unresponsive_workers.len().saturating_mul(2) as u64;
						writes += unresponsive_workers.len().saturating_mul(3) as u64;
					}
				},
				_ => {}
			}

			// TODO: Clean up PendingOfflineWorkers, offline them
			if PendingOfflineWorkers::<T>::count() > 0 {
				let pending_removing_workers =
					PendingOfflineWorkers::<T>::iter_keys().take(T::MarkingOfflinePerBlockLimit::get() as usize).collect::<Vec<_>>();
				assert!(pending_removing_workers.len() > 0); // it shouldn't be zero

				for worker in &pending_removing_workers {
					// Worker who is `RequestingOffline`, `RefreshRegistrationRequired` should answer heartbeat as is `Online`
					FlipSet::<T>::remove(worker);
					FlopSet::<T>::remove(worker);
					PendingOfflineWorkers::<T>::remove(worker);

					// TODO: Apply slash

					Workers::<T>::mutate(worker, |worker_info| {
						worker_info.as_mut().map(|mut info| info.status = WorkerStatus::Offline);
					});
				}
				reads += pending_removing_workers.len().saturating_mul(3) as u64;
				writes += pending_removing_workers.len().saturating_mul(3) as u64;
			}

			T::DbWeight::get().reads_writes(reads, writes)
		}
	}

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
			initial_deposit: BalanceOf<T>,
			worker: T::AccountId
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_register(who, initial_deposit, worker)
		}

		#[pallet::weight(0)]
		#[transactional]
		pub fn refresh_attestation(
			origin: OriginFor<T>,
			payload: OnlinePayload,
			attestation: Option<Attestation>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_refresh_attestation(who, payload, attestation)
		}

		/// Deregister a computing workers.
		#[pallet::weight(0)]
		#[transactional]
		pub fn deregister(origin: OriginFor<T>, worker: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_deregister(who, worker)
		}

		/// The worker claim for online
		#[pallet::weight(0)]
		#[transactional]
		pub fn online(
			origin: OriginFor<T>,
			payload: OnlinePayload,
			attestation: Option<Attestation>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_online(who, payload, attestation)
		}

		/// The worker requesting offline
		#[pallet::weight(0)]
		#[transactional]
		pub fn requesting_offline(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_requesting_offline(who)
		}

		/// The worker force offline, slashing will apply
		#[pallet::weight(0)]
		#[transactional]
		pub fn force_offline(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_force_offline(who)
		}

		/// Worker report it is still online, must called by the worker
		#[pallet::weight(0)]
		#[transactional]
		pub fn heartbeat(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_heartbeat(&who)
		}
	}
}

impl<T: Config> Pallet<T> {
	fn do_register(
		who: T::AccountId,
		initial_deposit: BalanceOf<T>,
		worker: T::AccountId,
	) -> DispatchResult {
		ensure!(who != worker, Error::<T>::InvalidOwner);

		let initial_reserved_deposit = T::ReservedDeposit::get();
		ensure!(initial_deposit >= initial_reserved_deposit, Error::<T>::InitialDepositTooLow);

		ensure!(!Workers::<T>::contains_key(&worker), Error::<T>::AlreadyRegistered);

		let worker_info = WorkerInfo {
			account: worker.clone(),
			owner: who.clone(),
			reserved: initial_reserved_deposit,
			status: WorkerStatus::Registered,
			spec_version: 0,
			attestation_method: None,
			attested_at: T::BlockNumber::default(),
		};

		<T as Config>::Currency::transfer(&who, &worker, initial_deposit, ExistenceRequirement::KeepAlive)?;
		<T as Config>::Currency::reserve(&worker, initial_reserved_deposit)?;

		Workers::<T>::insert(&worker, worker_info);

		Self::deposit_event(Event::<T>::Registered { worker: worker.clone() });
		Ok(())
	}

	fn do_deregister(
		who: T::AccountId,
		worker: T::AccountId
	) -> DispatchResult {
		let worker_info = Workers::<T>::get(&worker).ok_or(Error::<T>::NotExists)?;
		Self::ensure_owner(&who, &worker_info)?;
		ensure!(
			worker_info.status == WorkerStatus::Offline ||
			worker_info.status == WorkerStatus::Registered,
			Error::<T>::NotOffline
		);

		let reserved = worker_info.reserved;
		<T as Config>::Currency::unreserve(&worker, reserved);
		<T as Config>::Currency::transfer(
			&worker,
			&who,
			<T as Config>::Currency::free_balance(&worker),
			ExistenceRequirement::AllowDeath,
		)?;

		Workers::<T>::remove(&worker);

		Self::deposit_event(Event::<T>::Deregistered { worker: worker.clone() });
		Ok(())
	}

	pub fn do_online(
		who: T::AccountId,
		payload: OnlinePayload,
		attestation: Option<Attestation>,
	) -> DispatchResult {
		let mut worker_info = Workers::<T>::get(&who).ok_or(Error::<T>::NotExists)?;
		Self::ensure_worker(&who, &worker_info)?;

		ensure!(
			worker_info.status == WorkerStatus::Registered ||
			worker_info.status == WorkerStatus::Offline ||
			worker_info.status == WorkerStatus::Unresponsive,
			Error::<T>::WrongStatus
		);

		if worker_info.spec_version > payload.spec_version {
			return Err(Error::<T>::CanNotDowngrade.into())
		}

		Self::ensure_attestation_provided(&attestation)?;

		let mut attestation_method: Option<AttestationMethod> = None;
		if let Some(attestation) = attestation {
			attestation_method = Some(attestation.method());
			let verified = Self::verify_attestation(&attestation)?;

			let encode_worker = T::AccountId::encode(&who);
			let h256_worker = H256::from_slice(&encode_worker);
			let worker_public_key = sr25519::Public::from_h256(h256_worker);

			let encoded_message = Encode::encode(&payload);

			if let Some(signature) =
				sr25519::Signature::from_slice(verified.payload()) {
				if !sr25519_verify(&signature, &encoded_message, &worker_public_key) {
					return Err(Error::<T>::MismatchedPayloadSignature.into())
				}
			} else {
				return Err(Error::<T>::CanNotVerifyPayload.into())
			}
		}

		if worker_info.status == WorkerStatus::Unresponsive {
			PendingOfflineWorkers::<T>::remove(&who);
		}

		worker_info.spec_version = payload.spec_version;
		worker_info.attestation_method = attestation_method;
		worker_info.attested_at = frame_system::Pallet::<T>::block_number();
		worker_info.status = WorkerStatus::Online;
		Workers::<T>::insert(&who, worker_info);

		Self::flipflop_for_online(&who);

		Self::deposit_event(Event::<T>::Online { worker: who });
		Ok(())
	}

	fn do_refresh_attestation(
		who: T::AccountId,
		payload: OnlinePayload,
		attestation: Option<Attestation>,
	) -> DispatchResult {
		let mut worker_info = Workers::<T>::get(&who).ok_or(Error::<T>::NotExists)?;
		Self::ensure_worker(&who, &worker_info)?;

		if worker_info.attestation_method.is_none() {
			return Ok(())
		}

		ensure!(
			worker_info.spec_version == payload.spec_version,
			Error::<T>::SoftwareChanged
		);

		Self::ensure_attestation_method(&attestation, &worker_info)?;

		if let Some(attestation) = attestation {
			let verified = Self::verify_attestation(&attestation)?;

			let encode_worker = T::AccountId::encode(&who);
			let h256_worker = H256::from_slice(&encode_worker);
			let worker_public_key = sr25519::Public::from_h256(h256_worker);

			let encoded_message = Encode::encode(&payload);

			if let Some(signature) =
				sr25519::Signature::from_slice(verified.payload()) {
				if !sr25519_verify(&signature, &encoded_message, &worker_public_key) {
					return Err(Error::<T>::MismatchedPayloadSignature.into())
				}
			} else {
				return Err(Error::<T>::CanNotVerifyPayload.into())
			}
		}

		worker_info.attested_at = frame_system::Pallet::<T>::block_number();
		Workers::<T>::insert(&who, worker_info);

		Self::deposit_event(Event::<T>::AttestationRefreshed { worker: who });
		Ok(())
	}

	pub fn do_requesting_offline(who: T::AccountId) -> DispatchResult {
		let mut worker_info = Workers::<T>::get(&who).ok_or(Error::<T>::NotExists)?;
		Self::ensure_worker(&who, &worker_info)?;

		ensure!(
			worker_info.status == WorkerStatus::Online,
			Error::<T>::NotOnline
		);

		// The fast path, the worker can safely offline if no workload
		worker_info.status = WorkerStatus::Offline;
		Workers::<T>::insert(&who, worker_info);

		FlipSet::<T>::remove(&who);
		FlopSet::<T>::remove(&who);
		Self::deposit_event(Event::<T>::Offline { worker: who, force: false });

		// TODO: enable this path when we have real workload
		// worker_info.status = WorkerStatus::RequestingOffline;
		// Workers::<T>::insert(&who, worker_info);
		//
		// PendingOfflineWorkers::<T>::insert(&who, ());
		// // It should keep sending heartbeat until really offline
		//
		// Self::deposit_event(Event::<T>::RequestingOffline { worker: who });

		Ok(())
	}

	pub fn do_force_offline(who: T::AccountId) -> DispatchResult {
		let mut worker_info = Workers::<T>::get(&who).ok_or(Error::<T>::NotExists)?;
		Self::ensure_worker(&who, &worker_info)?;

		ensure!(
			worker_info.status == WorkerStatus::Online ||
			worker_info.status == WorkerStatus::RefreshAttestationRequired ||
			worker_info.status == WorkerStatus::RequestingOffline ||
			worker_info.status == WorkerStatus::RequestingOffline ||
			worker_info.status == WorkerStatus::Unresponsive,
			Error::<T>::WrongStatus
		);

		worker_info.status = WorkerStatus::Offline;
		Workers::<T>::insert(&who, worker_info);

		FlipSet::<T>::remove(&who);
		FlopSet::<T>::remove(&who);
		PendingOfflineWorkers::<T>::remove(&who);

		// TODO: Apply slash

		Self::deposit_event(Event::<T>::Offline { worker: who, force: true });
		Ok(())
	}

	pub fn do_heartbeat(who: &T::AccountId) -> DispatchResult {
		let mut worker_info = Workers::<T>::get(&who).ok_or(Error::<T>::NotExists)?;
		Self::ensure_worker(&who, &worker_info)?;
		ensure!(
			worker_info.status == WorkerStatus::Online ||
			worker_info.status == WorkerStatus::RequestingOffline ||
			worker_info.status == WorkerStatus::RefreshAttestationRequired,
			Error::<T>::NotOnline
		);

		Self::flipflop(who)?;

		Self::deposit_event(Event::<T>::HeartbeatReceived { worker: who.clone() });

		let current_block = frame_system::Pallet::<T>::block_number();
		if current_block - worker_info.attested_at > T::AttestationValidityDuration::get().into() {
			worker_info.status = WorkerStatus::RefreshAttestationRequired;
			Workers::<T>::insert(&who, worker_info);

			PendingOfflineWorkers::<T>::insert(&who, ());

			Self::deposit_event(Event::<T>::AttestationExpired { worker: who.clone() });
		}

		Ok(())
	}

	fn flipflop(who: &T::AccountId) -> DispatchResult {
		let stage = FlipOrFlop::<T>::get();
		match stage {
			FlipFlopStage::Flip => {
				if let Some(_flip) = FlipSet::<T>::take(who) {
					FlopSet::<T>::insert(who, ());

					Self::deposit_event(Event::<T>::HeartbeatReceived { worker: who.clone() });
					Ok(())
				} else {
					Err(Error::<T>::AlreadySentHeartbeat.into())
				}
			},
			FlipFlopStage::Flop => {
				if let Some(_flop) = FlopSet::<T>::take(who) {
					FlipSet::<T>::insert(who, ());

					Self::deposit_event(Event::<T>::HeartbeatReceived { worker: who.clone() });
					Ok(())
				} else {
					Err(Error::<T>::AlreadySentHeartbeat.into())
				}
			},
			_ => {
				Err(Error::<T>::Intermission.into())
			}
		}
	}

	fn flipflop_for_online(who: &T::AccountId) {
		let stage = FlipOrFlop::<T>::get();
		match stage {
			FlipFlopStage::Flip => {
				FlopSet::<T>::insert(who, ());
			},
			FlipFlopStage::Flop => {
				FlipSet::<T>::insert(who, ());
			},
			FlipFlopStage::FlipToFlop => {
				FlipSet::<T>::insert(who, ());
			},
			FlipFlopStage::FlopToFlip => {
				FlopSet::<T>::insert(who, ());
			}
		}
	}

	fn verify_attestation(attestation: &Attestation) -> Result<VerifiedAttestation, DispatchError> {
		let verify_material = AttestationVerifyMaterial {
			now: T::UnixTime::now().as_millis().saturated_into::<u64>(),
		};

		let verified = attestation.verify(&verify_material);
		return match verified {
			Ok(verified) => {
				Ok(verified)
			},
			Err(AttestationError::Expired) => {
				Err(Error::<T>::ExpiredAttestation.into())
			},
			Err(AttestationError::Invalid) => {
				Err(Error::<T>::InvalidAttestation.into())
			}
		}
	}

	fn ensure_owner(
		who: &T::AccountId,
		worker_info: &WorkerInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) -> DispatchResult {
		ensure!(*who == worker_info.owner, Error::<T>::NotTheOwner);
		Ok(())
	}

	fn ensure_worker(
		who: &T::AccountId,
		worker_info: &WorkerInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) -> DispatchResult {
		ensure!(*who == worker_info.account, Error::<T>::NotTheWorker);
		Ok(())
	}

	fn ensure_worker_attestation_validity(
		worker_info: &WorkerInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) -> DispatchResult {
		if !T::DisallowOptOutAttestation::get() && worker_info.attested_at.is_zero() {
			return Ok(())
		}

		let current_block = frame_system::Pallet::<T>::block_number();
		if current_block - worker_info.attested_at > T::AttestationValidityDuration::get().into() {
			Err(Error::<T>::AttestationRefreshNeeded.into())
		} else {
			Ok(())
		}
	}

	fn ensure_attestation_provided(attestation: &Option<Attestation>) -> DispatchResult {
		if let Some(attestation) = attestation {
			if attestation.method() == AttestationMethod::NonTEE {
				ensure!(!T::DisallowNonTEEAttestation::get(), Error::<T>::DisallowNonTEEAttestation);
			}
		} else {
			ensure!(!T::DisallowOptOutAttestation::get() || attestation.is_some(), Error::<T>::MustProvideAttestation);
		}

		Ok(())
	}

	fn ensure_attestation_method(
		attestation: &Option<Attestation>,
		worker_info: &WorkerInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) -> DispatchResult {
		if let Some(worker_attestation_method) = worker_info.clone().attestation_method {
			if let Some(attestation) = attestation {
				ensure!(attestation.method() == worker_attestation_method, Error::<T>::AttestationMethodChanged);
			} else {
				return Err(Error::<T>::AttestationMethodChanged.into())
			}
		} else {
			ensure!(attestation.is_none(), Error::<T>::AttestationMethodChanged)
		}

		Ok(())
	}
}
