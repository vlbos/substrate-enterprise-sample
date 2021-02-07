// Tests to be written here

use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, dispatch};

const TEST_ORDER_ID: &str = "00012345600012";
const TEST_ORGANIZATION: &str = "Northwind";
const TEST_SENDER: &str = "Alice";
const LONG_VALUE : &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec aliquam ut tortor nec congue. Pellente";

#[test]
fn create_order_without_fields() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let id = TEST_ORDER_ID.as_bytes().to_owned();
        let owner = account_key(TEST_ORGANIZATION);
        let now = 42;
        let index = 1;
        Timestamp::set_timestamp(now);

        let result = WyvernExchange::change_minimum_taker_protocol_fee(Origin::signed(sender), 10);

        assert_ok!(result);

        assert_eq!(<MinimumTakerProtocolFee<Test>>::get(), 10);
    });
}
