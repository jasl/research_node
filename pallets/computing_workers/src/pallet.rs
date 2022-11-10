/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>

pub use pallet::*;

use frame_support::{
	ensure,
	dispatch::DispatchResult,
	traits::{
		Currency,
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

		/// The minimum amount required to keep a worker registration.
		#[pallet::constant]
		type ExistentialDeposit: Get<BalanceOf<Self>>;

		// TODO: type WeightInfo: WeightInfo;
	}

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// The worker registered successfully
		Registered { owner: T::AccountId, controller: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Initial deposit for register a worker must equal or above `ExistentialDeposit`
		InitialDepositTooLow,
		/// Worker already registered
		AlreadyRegistered,
		/// The extrinsic origin isn't the worker's owner
		NotOwner,
		/// The extrinsic origin isn't the worker's controller
		NotController,
		/// The worker not exists
		WorkerNotExists,
	}

	/// Storage for computing_workers.
	#[pallet::storage]
	#[pallet::getter(fn workers)]
	pub type Workers<T: Config> =
		CountedStorageMap<_, Twox64Concat, T::AccountId, WorkerInfo<T::AccountId>>;

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register a computing workers.
		///
		/// ## Arguments
		/// - `origin`: Must be called by a `Signed` origin, it will become the worker's owner.
		/// - `controller`: The account who operate the worker. a controller can only manage one worker.
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
			controller: T::AccountId,
			initial_deposit: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_register(who, controller, initial_deposit)
		}

		/// Deregister a computing workers.
		#[pallet::weight(0)]
		#[transactional]
		pub fn deregister(
			origin: OriginFor<T>,
			controller: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_deregister(who, controller)
		}
	}
}

impl<T: Config> Pallet<T> {
	fn do_register(
		who: T::AccountId,
		controller: T::AccountId,
		initial_deposit: BalanceOf<T>,
	) -> DispatchResult {
		ensure!(
			initial_deposit >= T::ExistentialDeposit::get(),
			Error::<T>::InitialDepositTooLow
		);

		ensure!(
			!Workers::<T>::contains_key(&controller),
			Error::<T>::AlreadyRegistered
		);

		let current_account: T::AccountId = Self::current_account_of(&controller);
		let worker_info = WorkerInfo {
			owner: who.clone(),
			controller: controller.clone(),
			current_account: current_account.clone(),
			status: WorkerStatus::Registered,
		};

		<T as Config>::Currency::transfer(
			&who,
			&current_account,
			initial_deposit,
			ExistenceRequirement::KeepAlive
		)?;

		Workers::<T>::insert(&controller, worker_info);

		Self::deposit_event(
			Event::<T>::Registered { owner: who.clone(), controller: controller.clone() }
		);
		Ok(())
	}

	fn do_deregister(
		who: T::AccountId,
		controller: T::AccountId,
	) -> DispatchResult {
		let worker_info = Workers::<T>::get(&controller).ok_or(Error::<T>::WorkerNotExists)?;
		Self::ensure_owner(&who, &worker_info)?;

		let current_account = worker_info.current_account;
		<T as Config>::Currency::transfer(
			&current_account,
			&who,
			<T as Config>::Currency::free_balance(&current_account),
			ExistenceRequirement::AllowDeath
		)?;

		Workers::<T>::remove(&controller);

		Ok(())
	}

	fn ensure_owner(
		who: &T::AccountId, worker_info: &WorkerInfo<T::AccountId>
	) -> DispatchResult {
		ensure!(*who == worker_info.owner, Error::<T>::NotOwner);
		Ok(())
	}

	fn current_account_of<Encodable>(controller: &T::AccountId) -> Encodable
	where Encodable: Encode + Decode
	{
		(b"CA/", controller)
			.using_encoded(|b| Encodable::decode(&mut TrailingZeroInput::new(b)))
			.expect("Decoding zero-padded account id should always succeed; qed")
	}
}
