/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>

pub use pallet::*;

use frame_support::{
	ensure,
	dispatch::DispatchResult,
	traits::Currency,
};
use frame_support::traits::ExistenceRequirement::KeepAlive;
use scale_codec::{Decode, Encode};
use sp_core::Get;
use sp_runtime::{
	traits::{StaticLookup, TrailingZeroInput},
};
use crate::primitives::{
	WorkerInfo,
};

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
		/// Register head data and validation code for a reserved Para Id.
		///
		/// ## Arguments
		/// - `origin`: Must be called by a `Signed` origin.
		/// - `id`: The para ID. Must be owned/managed by the `origin` signing account.
		/// - `genesis_head`: The genesis head data of the parachain/thread.
		/// - `validation_code`: The initial validation code of the parachain/thread.
		///
		/// ## Deposits/Fees
		/// The origin signed account must reserve a corresponding deposit for the registration. Anything already
		/// reserved previously for this para ID is accounted for.
		///
		/// ## Events
		/// The `Registered` event is emitted in case of success.
		/// #[pallet::weight(<T as Config>::WeightInfo::register())]
		#[pallet::weight(0)]
		pub fn register(
			origin: OriginFor<T>,
			controller: T::AccountId,
			initial_deposit: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::do_register(who, controller, initial_deposit)
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

		let current_account: T::AccountId = Self::assign_current_account_for(&controller);
		let worker_info = WorkerInfo {
			owner: who.clone(),
			controller: controller.clone(),
			current_account: current_account.clone(),
		};

		<T as Config>::Currency::transfer(
			&who,
			&current_account,
			initial_deposit,
			KeepAlive
		)?;

		Workers::<T>::insert(controller.clone(), worker_info);

		Self::deposit_event(
			Event::<T>::Registered { owner: who.clone(), controller: controller.clone() }
		);
		Ok(())
	}

	fn assign_current_account_for<Encodable>(controller: &T::AccountId) -> Encodable
	where Encodable: Encode + Decode
	{
		(b"current_account/", controller)
			.using_encoded(|b| Encodable::decode(&mut TrailingZeroInput::new(b)))
			.expect("Decoding zero-padded account id should always succeed; qed")
	}
}
