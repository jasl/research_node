#![cfg_attr(not(feature = "std"), no_std)]

pub mod traits;
pub mod types;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

/// The log target of this pallet.
pub const LOG_TARGET: &str = "runtime::computing_workers";

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

pub use pallet::*;

use crate::{
	types::{
		Attestation, AttestationError, AttestationMethod, FlipFlopStage, OnlinePayload, VerifiedAttestation, WorkerInfo,
		WorkerStatus,
	},
	traits::{WorkerLifecycleHooks, WorkerManageable},
};
use frame_support::{
	dispatch::{DispatchError, DispatchResult},
	ensure,
	traits::{Currency, ExistenceRequirement, Get, ReservableCurrency, UnixTime},
	transactional,
};
use scale_codec::Encode;
use sp_core::{sr25519, H256};
use sp_io::crypto::sr25519_verify;
use sp_runtime::{traits::Zero, SaturatedConversion, Saturating};
use sp_std::prelude::*;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type PositiveImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::PositiveImbalance;
type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
	<T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

#[frame_support::pallet]
mod pallet {
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

		/// A handler for manging worker slashing
		type WorkerLifecycleHooks: WorkerLifecycleHooks<Self::AccountId, BalanceOf<Self>>;

		/// Max number of moving unresponsive workers to pending offline workers queue
		#[pallet::constant]
		type HandleUnresponsivePerBlockLimit: Get<u32>;

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

	/// Storage for computing_workers.
	#[pallet::storage]
	#[pallet::getter(fn workers)]
	pub type Workers<T: Config> =
		CountedStorageMap<_, Identity, T::AccountId, WorkerInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>>;

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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The worker registered successfully
		Registered { worker: T::AccountId },
		/// The worker registered successfully
		Deregistered { worker: T::AccountId, force: bool },
		/// The worker is online
		Online { worker: T::AccountId },
		/// The worker is requesting offline
		RequestingOffline { worker: T::AccountId },
		/// The worker is offline
		Offline { worker: T::AccountId, force: bool },
		/// The worker send heartbeat successfully
		HeartbeatReceived { worker: T::AccountId },
		/// The worker refresh its attestation successfully
		AttestationRefreshed { worker: T::AccountId },
		/// The worker's attestation expired
		AttestationExpired { worker: T::AccountId },
		/// The worker's reserved money below requirement
		InsufficientReservedFunds { worker: T::AccountId },
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
		/// Worker's wallet reserved money smaller than should be reserved
		InsufficientReserved,
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
		AttestationExpired,
		/// Intermission
		Intermission,
		/// AlreadySentHeartbeat
		AlreadySentHeartbeat,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: T::BlockNumber) -> Weight {
			let mut reads: u64 = 1; // Read FlipOrFlop
			let mut writes: u64 = 0;

			let mut flip_or_flop = FlipOrFlop::<T>::get();
			if (n % T::CollectingHeartbeatsDuration::get().into()).is_zero() {
				match flip_or_flop {
					FlipFlopStage::Flip => {
						flip_or_flop = FlipFlopStage::FlipToFlop;
						FlipOrFlop::<T>::set(flip_or_flop);
						writes += 1;
					},
					FlipFlopStage::Flop => {
						flip_or_flop = FlipFlopStage::FlopToFlip;
						FlipOrFlop::<T>::set(flip_or_flop);
						writes += 1;
					},
					_ => {},
				}
			}
			match flip_or_flop {
				FlipFlopStage::FlipToFlop => {
					let iter = FlipSet::<T>::iter_keys().take(T::HandleUnresponsivePerBlockLimit::get() as usize);
					let total_count = FlipSet::<T>::count();

					let mut i: u64 = 0;
					for worker in iter {
						FlipSet::<T>::remove(&worker);
						Self::handle_worker_unresponsive(&worker);
						i += 1;
					}

					reads += i;
					writes += i.saturating_mul(3);

					if i >= total_count as u64 {
						FlipOrFlop::<T>::set(FlipFlopStage::Flop);
						writes += 1;
					}
				},
				FlipFlopStage::FlopToFlip => {
					let iter = FlopSet::<T>::iter_keys().take(T::HandleUnresponsivePerBlockLimit::get() as usize);
					let total_count = FlopSet::<T>::count();

					let mut i: u64 = 0;
					for worker in iter {
						FlopSet::<T>::remove(&worker);
						Self::handle_worker_unresponsive(&worker);
						i += 1;
					}

					reads += i;
					writes += i.saturating_mul(3);

					if i >= total_count as u64 {
						FlipOrFlop::<T>::set(FlipFlopStage::Flip);
						writes += 1;
					}
				},
				_ => {},
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
		pub fn register(origin: OriginFor<T>, initial_deposit: BalanceOf<T>, worker: T::AccountId) -> DispatchResult {
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

		/// The same with balances.transfer_keep_alive(owner, worker, balance)
		#[pallet::weight(0)]
		#[transactional]
		pub fn deposit(origin: OriginFor<T>, worker: T::AccountId, value: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let worker_info = Workers::<T>::get(&worker).ok_or(Error::<T>::NotExists)?;
			Self::ensure_owner(&who, &worker_info)?;

			<T as Config>::Currency::transfer(&who, &worker, value, ExistenceRequirement::KeepAlive)?;
			Ok(())
		}

		/// The same with balances.transfer_keep_alive(worker, owner, balance)
		#[pallet::weight(0)]
		#[transactional]
		pub fn withdraw(origin: OriginFor<T>, worker: T::AccountId, value: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let worker_info = Workers::<T>::get(&worker).ok_or(Error::<T>::NotExists)?;
			Self::ensure_owner(&who, &worker_info)?;

			<T as Config>::Currency::transfer(&worker, &who, value, ExistenceRequirement::KeepAlive)?;
			Ok(())
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
		pub fn request_offline(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_request_offline(who, None)
		}

		/// The owner (or his proxy) requesting a worker to offline
		#[pallet::weight(0)]
		#[transactional]
		pub fn request_offline_for(origin: OriginFor<T>, worker: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_request_offline(worker, Some(who))
		}

		/// The worker force offline, slashing will apply
		#[pallet::weight(0)]
		#[transactional]
		pub fn force_offline(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_force_offline(who, None)
		}

		/// The owner (or his proxy) force a worker to offline, will apply slash
		#[pallet::weight(0)]
		#[transactional]
		pub fn force_offline_for(origin: OriginFor<T>, worker: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_force_offline(worker, Some(who))
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
	fn do_register(owner: T::AccountId, initial_deposit: BalanceOf<T>, worker: T::AccountId) -> DispatchResult {
		ensure!(owner != worker, Error::<T>::InvalidOwner);

		let initial_reserved_deposit = T::ReservedDeposit::get();
		ensure!(initial_deposit >= initial_reserved_deposit, Error::<T>::InitialDepositTooLow);

		ensure!(!Workers::<T>::contains_key(&worker), Error::<T>::AlreadyRegistered);

		let worker_info = WorkerInfo {
			account: worker.clone(),
			owner: owner.clone(),
			reserved: initial_reserved_deposit,
			status: WorkerStatus::Registered,
			spec_version: 0,
			attestation_method: None,
			attested_at: T::BlockNumber::default(),
		};

		<T as Config>::Currency::transfer(&owner, &worker, initial_deposit, ExistenceRequirement::KeepAlive)?;
		if !initial_reserved_deposit.is_zero() {
			<T as Config>::Currency::reserve(&worker, initial_reserved_deposit)?;
		}

		Workers::<T>::insert(&worker, worker_info);

		Self::deposit_event(Event::<T>::Registered { worker });
		Ok(())
	}

	fn do_deregister(owner: T::AccountId, worker: T::AccountId) -> DispatchResult {
		let worker_info = Workers::<T>::get(&worker).ok_or(Error::<T>::NotExists)?;
		Self::ensure_owner(&owner, &worker_info)?;
		ensure!(
			worker_info.status == WorkerStatus::Offline || worker_info.status == WorkerStatus::Registered,
			Error::<T>::NotOffline
		);

		let reserved = worker_info.reserved;
		if !reserved.is_zero() {
			// The upper limit is the actual reserved, so it is OK
			<T as Config>::Currency::unreserve(&worker, reserved);
		}
		<T as Config>::Currency::transfer(
			&worker,
			&owner,
			<T as Config>::Currency::free_balance(&worker),
			ExistenceRequirement::AllowDeath,
		)?;

		Workers::<T>::remove(&worker);

		Self::deposit_event(Event::<T>::Deregistered { worker, force: false });
		Ok(())
	}

	/// Transit a worker to `Online` status
	/// Check following things
	/// 1 Get the worker info by the caller
	/// 2 Check the worker's status (Must be `Registered`, and `Offline`)
	/// 3 Check the payload
	/// 4 Check the reserved (will try complement from free)
	/// 5 Check the attestation (the payload's signature is inside as payload)
	/// 6 Do `can_online` hook, will pass the payload
	/// Then
	/// 2 Update worker's info, persists to storage
	/// 3 Set flipflop
	pub fn do_online(worker: T::AccountId, payload: OnlinePayload, attestation: Option<Attestation>) -> DispatchResult {
		let mut worker_info = Workers::<T>::get(&worker).ok_or(Error::<T>::NotExists)?;
		Self::ensure_worker(&worker, &worker_info)?;
		match worker_info.status {
			WorkerStatus::Registered |
			WorkerStatus::Offline => {}
			_ => {
				return Err(Error::<T>::WrongStatus.into())
			}
		}
		ensure!(
			worker_info.spec_version <= payload.spec_version,
			Error::<T>::CanNotDowngrade
		);

		// Check reserved money
		let reserved = <T as Config>::Currency::reserved_balance(&worker);
		if reserved < worker_info.reserved {
			// Try add reserved from free
			let free = <T as Config>::Currency::free_balance(&worker);
			let should_add_reserve = worker_info.reserved.saturating_sub(reserved);
			ensure!(
				free >= should_add_reserve,
				Error::<T>::InsufficientReserved
			);
			<T as Config>::Currency::reserve(&worker, should_add_reserve)?;
		}

		Self::ensure_attestation_provided(&attestation)?;

		let mut attestation_method: Option<AttestationMethod> = None;
		if let Some(attestation) = attestation {
			attestation_method = Some(attestation.method());
			let verified = Self::verify_attestation(&attestation)?;

			let encode_worker = T::AccountId::encode(&worker);
			let h256_worker = H256::from_slice(&encode_worker);
			let worker_public_key = sr25519::Public::from_h256(h256_worker);

			let encoded_message = Encode::encode(&payload);

			let Some(signature) = sr25519::Signature::from_slice(verified.payload()) else {
				return Err(Error::<T>::CanNotVerifyPayload.into())
			};

			ensure!(
				sr25519_verify(&signature, &encoded_message, &worker_public_key),
				Error::<T>::MismatchedPayloadSignature
			);
		}

		T::WorkerLifecycleHooks::can_online(&worker, &payload)?;

		worker_info.spec_version = payload.spec_version;
		worker_info.attestation_method = attestation_method;
		worker_info.attested_at = frame_system::Pallet::<T>::block_number();
		worker_info.status = WorkerStatus::Online;

		Workers::<T>::insert(&worker, worker_info);

		Self::flipflop_for_online(&worker);

		Self::deposit_event(Event::<T>::Online { worker: worker.clone() });

		T::WorkerLifecycleHooks::after_online(&worker);

		Ok(())
	}

	fn do_refresh_attestation(
		worker: T::AccountId,
		payload: OnlinePayload,
		attestation: Option<Attestation>,
	) -> DispatchResult {
		let mut worker_info = Workers::<T>::get(&worker).ok_or(Error::<T>::NotExists)?;
		Self::ensure_worker(&worker, &worker_info)?;

		if worker_info.attestation_method.is_none() {
			return Ok(())
		}

		ensure!(worker_info.spec_version == payload.spec_version, Error::<T>::SoftwareChanged);

		Self::ensure_attestation_method(&attestation, &worker_info)?;

		if let Some(attestation) = attestation {
			let verified = Self::verify_attestation(&attestation)?;

			let encode_worker = T::AccountId::encode(&worker);
			let h256_worker = H256::from_slice(&encode_worker);
			let worker_public_key = sr25519::Public::from_h256(h256_worker);

			let encoded_message = Encode::encode(&payload);

			let Some(signature) = sr25519::Signature::from_slice(verified.payload()) else {
				return Err(Error::<T>::CanNotVerifyPayload.into())
			};

			ensure!(
				sr25519_verify(&signature, &encoded_message, &worker_public_key),
				Error::<T>::MismatchedPayloadSignature
			);
		}

		worker_info.attested_at = frame_system::Pallet::<T>::block_number();
		Workers::<T>::insert(&worker, worker_info);

		Self::deposit_event(Event::<T>::AttestationRefreshed { worker: worker.clone() });

		T::WorkerLifecycleHooks::after_refresh_attestation(&worker, &payload);

		Ok(())
	}

	/// Transit worker to `Offline` status
	pub fn do_request_offline(worker: T::AccountId, owner: Option<T::AccountId>) -> DispatchResult {
		let worker_info = Workers::<T>::get(&worker).ok_or(Error::<T>::NotExists)?;
		Self::ensure_worker(&worker, &worker_info)?;

		if let Some(owner) = owner {
			Self::ensure_owner(&owner, &worker_info)?;
		}

		ensure!(worker_info.status == WorkerStatus::Online, Error::<T>::NotOnline);

		if T::WorkerLifecycleHooks::can_offline(&worker).is_ok() {
			// Fast path
			T::WorkerLifecycleHooks::before_offline(&worker, false);

			FlipSet::<T>::remove(&worker);
			FlopSet::<T>::remove(&worker);
			Workers::<T>::mutate(&worker, |worker_info| {
				if let Some(mut info) = worker_info.as_mut() {
					info.status = WorkerStatus::Offline;
				}
			});

			Self::deposit_event(Event::<T>::Offline { worker, force: false });
		} else {
			// the worker should keep sending heartbeat until get permission to offline
			Workers::<T>::mutate(&worker, |worker_info| {
				if let Some(mut info) = worker_info.as_mut() {
					info.status = WorkerStatus::RequestingOffline;
				}
			});

			Self::deposit_event(Event::<T>::RequestingOffline { worker: worker.clone() });

			T::WorkerLifecycleHooks::after_requesting_offline(&worker);
		}

		Ok(())
	}

	pub fn do_force_offline(worker: T::AccountId, owner: Option<T::AccountId>) -> DispatchResult {
		let mut worker_info = Workers::<T>::get(&worker).ok_or(Error::<T>::NotExists)?;
		Self::ensure_worker(&worker, &worker_info)?;

		if let Some(owner) = owner {
			Self::ensure_owner(&owner, &worker_info)?;
		}

		match worker_info.status {
			WorkerStatus::Online |
			WorkerStatus::RequestingOffline => {},
			_ => {
				return Err(Error::<T>::WrongStatus.into())
			},
		}

		T::WorkerLifecycleHooks::before_offline(&worker, true);

		worker_info.status = WorkerStatus::Offline;
		Workers::<T>::insert(&worker, worker_info);

		FlipSet::<T>::remove(&worker);
		FlopSet::<T>::remove(&worker);

		Self::deposit_event(Event::<T>::Offline { worker, force: true });
		Ok(())
	}

	pub fn do_heartbeat(worker: &T::AccountId) -> DispatchResult {
		let worker_info = Workers::<T>::get(worker).ok_or(Error::<T>::NotExists)?;
		Self::ensure_worker(worker, &worker_info)?;
		match worker_info.status {
			WorkerStatus::Online |
			WorkerStatus::RequestingOffline => {},
			_ => {
				return Err(Error::<T>::NotOnline.into())
			}
		}

		// Check whether attestation expired, if yes, treat as force offline
		let current_block = frame_system::Pallet::<T>::block_number();
		if current_block - worker_info.attested_at > T::AttestationValidityDuration::get().into() {
			T::WorkerLifecycleHooks::after_attestation_expired(worker);

			FlipSet::<T>::remove(worker);
			FlopSet::<T>::remove(worker);
			Workers::<T>::mutate(worker, |worker_info| {
				if let Some(mut info) = worker_info.as_mut() {
					info.status = WorkerStatus::Offline;
				}
			});

			Self::deposit_event(Event::<T>::AttestationExpired { worker: worker.clone() });
			Self::deposit_event(Event::<T>::Offline { worker: worker.clone(), force: true });
			return Ok(())
		}

		// Check whether can offline now, We ignore error here
		if worker_info.status == WorkerStatus::RequestingOffline &&
			T::WorkerLifecycleHooks::can_offline(worker).is_ok() {
			T::WorkerLifecycleHooks::before_offline(worker, false);

			FlipSet::<T>::remove(worker);
			FlopSet::<T>::remove(worker);
			Workers::<T>::mutate(worker, |worker_info| {
				if let Some(mut info) = worker_info.as_mut() {
					info.status = WorkerStatus::Offline;
				}
			});

			Self::deposit_event(Event::<T>::Offline { worker: worker.clone(), force: false });
			return Ok(())
		}

		// Check the worker's reserved money
		if <T as Config>::Currency::reserved_balance(worker) < T::ReservedDeposit::get() {
			T::WorkerLifecycleHooks::after_insufficient_reserved_funds(worker);

			FlipSet::<T>::remove(worker);
			FlopSet::<T>::remove(worker);
			Workers::<T>::mutate(worker, |worker_info| {
				if let Some(mut info) = worker_info.as_mut() {
					info.status = WorkerStatus::Offline;
				}
			});

			Self::deposit_event(Event::<T>::InsufficientReservedFunds { worker: worker.clone() });
			Self::deposit_event(Event::<T>::Offline { worker: worker.clone(), force: true });
			return Ok(())
		}

		Self::flipflop(worker)?;

		Self::deposit_event(Event::<T>::HeartbeatReceived { worker: worker.clone() });

		Ok(())
	}

	fn flipflop(worker: &T::AccountId) -> DispatchResult {
		let stage = FlipOrFlop::<T>::get();
		match stage {
			FlipFlopStage::Flip => {
				let Some(_flip) = FlipSet::<T>::take(worker) else {
					return Err(Error::<T>::AlreadySentHeartbeat.into())
				};

				FlopSet::<T>::insert(worker, ());

				Self::deposit_event(Event::<T>::HeartbeatReceived { worker: worker.clone() });
				Ok(())
			},
			FlipFlopStage::Flop => {
				let Some(_flop) = FlopSet::<T>::take(worker) else {
					return Err(Error::<T>::AlreadySentHeartbeat.into());
				};

				FlipSet::<T>::insert(worker, ());

				Self::deposit_event(Event::<T>::HeartbeatReceived { worker: worker.clone() });
				Ok(())
			},
			_ => Err(Error::<T>::Intermission.into()),
		}
	}

	fn flipflop_for_online(worker: &T::AccountId) {
		let stage = FlipOrFlop::<T>::get();
		match stage {
			FlipFlopStage::Flip => {
				FlopSet::<T>::insert(worker, ());
			},
			FlipFlopStage::Flop => {
				FlipSet::<T>::insert(worker, ());
			},
			FlipFlopStage::FlipToFlop => {
				FlipSet::<T>::insert(worker, ());
			},
			FlipFlopStage::FlopToFlip => {
				FlopSet::<T>::insert(worker, ());
			},
		}
	}

	fn handle_worker_unresponsive(worker: &T::AccountId) {
		T::WorkerLifecycleHooks::after_unresponsive(worker);

		Workers::<T>::mutate(worker, |worker_info| {
			if let Some(mut info) = worker_info.as_mut() {
				info.status = WorkerStatus::Offline;
			}
		});

		Self::deposit_event(Event::<T>::Offline { worker: worker.clone(), force: true });
	}

	fn verify_attestation(attestation: &Attestation) -> Result<VerifiedAttestation, DispatchError> {
		let now = T::UnixTime::now().as_millis().saturated_into::<u64>();
		let verified = attestation.verify(now);
		match verified {
			Ok(verified) => Ok(verified),
			Err(AttestationError::Expired) => Err(Error::<T>::ExpiredAttestation.into()),
			Err(AttestationError::Invalid) => Err(Error::<T>::InvalidAttestation.into()),
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

	fn ensure_attestation_provided(attestation: &Option<Attestation>) -> DispatchResult {
		let Some(attestation) = attestation else {
			ensure!(
				!T::DisallowOptOutAttestation::get() || attestation.is_some(),
				Error::<T>::MustProvideAttestation
			);
			return Ok(())
		};

		if attestation.method() == AttestationMethod::NonTEE {
			ensure!(!T::DisallowNonTEEAttestation::get(), Error::<T>::DisallowNonTEEAttestation);
		}

		Ok(())
	}

	fn ensure_attestation_method(
		attestation: &Option<Attestation>,
		worker_info: &WorkerInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) -> DispatchResult {
		let Some(worker_attestation_method) = worker_info.attestation_method.clone() else {
			ensure!(attestation.is_none(), Error::<T>::AttestationMethodChanged);
			return Ok(())
		};

		let Some(attestation) = attestation else {
			return Err(Error::<T>::AttestationMethodChanged.into())
		};

		ensure!(attestation.method() == worker_attestation_method, Error::<T>::AttestationMethodChanged);

		Ok(())
	}
}

impl<T: Config> WorkerManageable<T::AccountId, T::BlockNumber> for Pallet<T> {
	type Balance = BalanceOf<T>;
	type PositiveImbalance = PositiveImbalanceOf<T>;
	type NegativeImbalance = NegativeImbalanceOf<T>;

	fn worker_info(worker: &T::AccountId) -> Option<WorkerInfo<T::AccountId, BalanceOf<T>, T::BlockNumber>> {
		Workers::<T>::get(worker)
	}

	fn reward(worker: &T::AccountId, source: &T::AccountId, value: BalanceOf<T>) -> DispatchResult {
		<T as Config>::Currency::transfer(source, worker, value, ExistenceRequirement::KeepAlive)
	}

	fn slash(worker: &T::AccountId, value: BalanceOf<T>) -> (NegativeImbalanceOf<T>, BalanceOf<T>) {
		<T as Config>::Currency::slash(worker, value)
	}

	fn offline(worker: &T::AccountId) -> DispatchResult {
		let mut worker_info = Workers::<T>::get(worker).ok_or(Error::<T>::NotExists)?;
		match worker_info.status {
			WorkerStatus::Online |
			WorkerStatus::RequestingOffline => {},
			_ => {
				return Err(Error::<T>::WrongStatus.into())
			},
		}

		worker_info.status = WorkerStatus::Offline;
		Workers::<T>::insert(worker, worker_info);

		FlipSet::<T>::remove(worker);
		FlopSet::<T>::remove(worker);

		Self::deposit_event(Event::<T>::Offline { worker: worker.clone(), force: true });

		Ok(())
	}
}
