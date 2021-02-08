// Tests to be written here

use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, dispatch};

const TEST_ORDER_ID: &str = "00012345600012";
const TEST_ORGANIZATION: &str = "Northwind";
const TEST_SENDER: &str = "Alice";
const TEST_SENDER_1: &str = "Bob";
const LONG_VALUE : &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec aliquam ut tortor nec congue. Pellente";

#[test]
fn change_minimum_taker_protocol_fee() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let id = TEST_ORDER_ID.as_bytes().to_owned();
        let owner = account_key(TEST_ORGANIZATION);
        let now = 42;
        let index = 1;
        let min_taker_protocol_fee = 42;
        Timestamp::set_timestamp(now);

        let result = WyvernExchange::change_minimum_taker_protocol_fee(
            Origin::signed(sender),
            min_taker_protocol_fee,
        );

        assert_ok!(result);

        assert_eq!(
            <MinimumTakerProtocolFee<Test>>::get(),
            min_taker_protocol_fee
        );
    });
}

#[test]
fn transfer_tokens() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let sender1 = account_key(TEST_SENDER_1);

        let id = TEST_ORDER_ID.as_bytes().to_owned();
        let owner = account_key(TEST_ORGANIZATION);
        let now = 42;
        let index = 1;
        let amount = 42;
        Timestamp::set_timestamp(now);
        create_account_test(sender);
        create_account_test(sender1);
        let result = WyvernExchange::transfer_tokens(&sender, &sender, &sender1, amount);

        assert_ok!(result);

        assert_eq!(
            <Test as Trait>::Currency::free_balance(&sender),
            99999999999999958
        );
        assert_eq!(
            <Test as Trait>::Currency::free_balance(&sender1),
            100000000000000042
        );
        // 		assert_eq!(<Test as Config>::Currency::free_balance(&alice()), 100);
        // 		// 10% of the 50 units is unlocked automatically for Alice
        // 		assert_eq!(<Test as Config>::VestingSchedule::vesting_balance(&alice()), Some(45));
        // 		assert_eq!(<Test as Config>::Currency::free_balance(&bob()), 250);
        // 		// A max of 10 units is unlocked automatically for Bob
        // 		assert_eq!(<Test as Config>::VestingSchedule::vesting_balance(&bob()), Some(140));
        // 		// Status is completed.
        // 		assert_eq!(
        // 			Accounts::<Test>::get(alice()),
        // 			AccountStatus {
        // 				validity: AccountValidity::Completed,
        // 				free_balance: 50,
        // 				locked_balance: 50,
        // 				signature: alice_signature().to_vec(),
        // 				vat: Permill::zero(),
        // 			}
        // 		);
    });
}
