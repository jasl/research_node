//! Benchmarking setup for pallet-computing_workers

use super::*;

#[allow(unused)]
use crate::Pallet as ComputingWorkers;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use frame_support::{
	sp_runtime::{SaturatedConversion, Saturating}
};

const SEED: u32 = 0;

pub(crate) type Balance = u128;
pub(crate) const MILLI_CENTS: Balance = 1_000_000;
pub(crate) const CENTS: Balance = 1_000 * MILLI_CENTS;
pub(crate) const DOLLARS: Balance = 100 * CENTS;

benchmarks! {
	register {
		let caller: T::AccountId = whitelisted_caller();

		let worker = account::<T::AccountId>("worker", 0, SEED);
		let reserved_deposit = T::ReservedDeposit::get();

		let balance = reserved_deposit.saturating_add((1 * DOLLARS).saturated_into::<BalanceOf<T>>());
		let _ = T::Currency::make_free_balance_be(&caller, balance);
	}: _(RawOrigin::Signed(caller.clone()), worker.clone(), reserved_deposit)
	verify {
		let worker_info = Workers::<T>::get(&worker).expect("WorkerInfo should has value");
		assert_eq!(worker_info.owner, caller);
	}

	impl_benchmark_test_suite!(ComputingWorkers, crate::mock::new_test_ext(), crate::mock::Test);
}
