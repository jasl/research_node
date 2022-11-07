#[allow(unused)]
use crate::{mock::*, Error};
#[allow(unused)]
use frame_support::{assert_noop, assert_ok, assert_err};

#[test]
fn register_works() {
	new_test_ext().execute_with(|| {
		let owner = 1;
		let controller = 2;

		set_balance(owner, 101 * DOLLARS, 0);

		assert_ok!(
			ComputingWorkers::register(
				RuntimeOrigin::signed(1),
				controller,
				100 * DOLLARS
			)
		);

		let worker_info = ComputingWorkers::workers(controller).unwrap();
		let current_account = worker_info.current_account;

		assert_eq!(Balances::free_balance(owner), 1 * DOLLARS);
		assert_eq!(Balances::free_balance(controller), 0);
		assert_eq!(Balances::free_balance(current_account), 100 * DOLLARS);

		// Ensure the expected error is thrown when no value is present.
		// assert_noop!(ComputingWorkers::cause_error(RuntimeOrigin::signed(1)), Error::<Test>::NoneValue);
	});
}
