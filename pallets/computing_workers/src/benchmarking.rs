//! Benchmarking setup for pallet-computing_workers

use super::*;

#[allow(unused)]
use crate::Pallet as ComputingWorkers;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::{RawOrigin, Account};
use frame_support::{
	sp_runtime::{SaturatedConversion, Saturating},
	assert_ok,
};
use crate::types::{
	AttestationMethod, AttestationPayload, ExtraOnlinePayload, NonTEEAttestation, OnlinePayload,
};

const SEED: u32 = 0;

type Balance = u128;
const MILLI_CENTS: Balance = 1_000_000;
const CENTS: Balance = 1_000 * MILLI_CENTS;
const DOLLARS: Balance = 100 * CENTS;

fn mock_online_payload_and_attestation() -> (OnlinePayload, Option<Attestation>) {
	let payload = OnlinePayload {
		spec_version: 1,
		extra: ExtraOnlinePayload::default(),
	};

	let attestation = Attestation::NonTEE(
		NonTEEAttestation {
			issued_at: 1669297549807,
			payload: AttestationPayload::truncate_from(
				hex_literal::hex!(
					"a6ab1b925f50c4a6a8def5125050730356930ff988eec941e632a14f98f7a01ef56d339e5e314c473f0a15adc1f374856d476e4de8227991ea2ff6d03056e08b"
				).to_vec()
			)
		}
	);

	(payload, Some(attestation))
}

fn add_mock_worker<T: Config>(owner: &T::AccountId) -> T::AccountId {
	let mock_worker_public = hex_literal::hex!("1efe052c6f591f3c6884ce9f7a60c8e73a869e1940569b284f4d3e904897dd30");
	let worker = T::AccountId::decode(&mut &mock_worker_public.encode()[..]).unwrap();
	let reserved_deposit = T::ReservedDeposit::get();

	let owner_balance = reserved_deposit.saturating_add((50 * DOLLARS).saturated_into::<BalanceOf<T>>());
	let _ = T::Currency::make_free_balance_be(&owner, owner_balance);

	let initial_deposit = reserved_deposit.saturating_add((10 * DOLLARS).saturated_into::<BalanceOf<T>>());

	assert_ok!(
		ComputingWorkers::<T>::register(
			RawOrigin::Signed(owner.clone()).into(),
			worker.clone(),
			initial_deposit
		)
	);
	assert_eq!(Workers::<T>::contains_key(&worker), true);

	worker
}

fn add_mock_online_worker<T: Config>(owner: &T::AccountId) -> T::AccountId {
	let worker = add_mock_worker(owner);

	let (payload, attestation) = mock_online_payload_and_attestation();
	assert_ok!(
		ComputingWorkers::<T>::register(
			RawOrigin::Signed(worker.clone()).into(),
			payload,
			attestation
		)
	);

	let worker_info = Workers::<T>::get(&worker).expect("WorkerInfo should has value");
	assert_eq!(worker_info.attestation_method, Some(AttestationMethod::NonTEE));

	worker
}

benchmarks! {
	register {
		let owner: T::AccountId = whitelisted_caller();
		let worker = account::<T::AccountId>("worker", 0, SEED);

		let reserved_deposit = T::ReservedDeposit::get();
		let balance = reserved_deposit.saturating_add((1 * DOLLARS).saturated_into::<BalanceOf<T>>());
		let _ = T::Currency::make_free_balance_be(&owner, balance);
	}: _(RawOrigin::Signed(owner.clone()), worker.clone(), reserved_deposit)
	verify {
		let worker_info = Workers::<T>::get(&worker).expect("WorkerInfo should has value");
		assert_eq!(worker_info.owner, owner);
		assert_eq!(T::Currency::reserved_balance(&worker), reserved_deposit);
	}

	deregister {
		let owner: T::AccountId = whitelisted_caller();
		let worker = add_mock_worker::<T>(&owner);
	}: _(RawOrigin::Signed(owner.clone()), worker.clone())
	verify {
		assert_eq!(Workers::<T>::contains_key(&worker), false);
		assert_eq!(Account::<T>::contains_key(&worker), false);
	}

	deposit {
		let owner: T::AccountId = whitelisted_caller();
		let worker = add_mock_worker::<T>(&owner);

		let worker_balance = T::Currency::free_balance(&worker);
		let amount = (10 * DOLLARS).saturated_into::<BalanceOf<T>>();
	}: _(RawOrigin::Signed(owner.clone()), worker.clone(), amount)
	verify {
		assert_eq!(
			T::Currency::free_balance(&worker),
			worker_balance.saturating_add(amount)
		);
	}

	withdraw {
		let owner: T::AccountId = whitelisted_caller();
		let worker = add_mock_worker::<T>(&owner);

		let worker_balance = T::Currency::free_balance(&worker);
		let amount = (10 * DOLLARS).saturated_into::<BalanceOf<T>>();
	}: _(RawOrigin::Signed(owner.clone()), worker.clone(), amount)
	verify {
		assert_eq!(
			T::Currency::free_balance(&worker),
			worker_balance.saturating_sub(amount)
		);
	}

	online {
		let owner: T::AccountId = whitelisted_caller();
		let worker = add_mock_worker::<T>(&owner);
		let (payload, attestation) = mock_online_payload_and_attestation();
	}: _(RawOrigin::Signed(worker.clone()), payload, attestation)
	verify {
		let worker_info = Workers::<T>::get(&worker).expect("WorkerInfo should has value");
		assert_eq!(worker_info.attestation_method, Some(AttestationMethod::NonTEE));
	}

	impl_benchmark_test_suite!(ComputingWorkers, crate::mock::new_test_ext(), crate::mock::Test);
}
