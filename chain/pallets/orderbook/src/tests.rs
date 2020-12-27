// Tests to be written here

use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, dispatch};

pub fn store_test_order<T: Trait>(id: OrderId, owner: T::AccountId, registered: T::Moment) {
    Orders::<T>::insert(
        id.clone(),
        OrderJSONType {
            id,
            owner,
            registered,
            fields: None,
        },
    );
}

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
        Timestamp::set_timestamp(now);

        let result = Orderbook::post_order(
            Origin::signed(sender),
            id.clone(),
            owner.clone(),
            None,
        );

        assert_ok!(result);

        assert_eq!(
            Orderbook::order_by_id(&id),
            Some(OrderJSONType {
                id: id.clone(),
                owner: owner,
                registered: now,
                fields: None
            })
        );

        assert_eq!(<OrdersOfOrganization<Test>>::get(owner), vec![id.clone()]);

        assert_eq!(Orderbook::owner_of(&id), Some(owner));

        // Event is raised
        assert!(System::events().iter().any(|er| er.event
            == TestEvent::orderbook(RawEvent::OrderPosted(
                sender,
                id.clone(),
                owner
            ))));
    });
}

#[test]
fn create_order_with_valid_fields() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let id = TEST_ORDER_ID.as_bytes().to_owned();
        let owner = account_key(TEST_ORGANIZATION);
        let now = 42;
        Timestamp::set_timestamp(now);

        let result = Orderbook::post_order(
            Origin::signed(sender),
            id.clone(),
            owner.clone(),
            Some(vec![
                OrderField::new(b"field1", b"val1"),
                OrderField::new(b"field2", b"val2"),
                OrderField::new(b"field3", b"val3"),
            ]),
        );

        assert_ok!(result);

        assert_eq!(
            Orderbook::order_by_id(&id),
            Some(OrderJSONType {
                id: id.clone(),
                owner: owner,
                registered: now,
                fields: Some(vec![
                    OrderField::new(b"field1", b"val1"),
                    OrderField::new(b"field2", b"val2"),
                    OrderField::new(b"field3", b"val3"),
                ]),
            })
        );

        assert_eq!(<OrdersOfOrganization<Test>>::get(owner), vec![id.clone()]);

        assert_eq!(Orderbook::owner_of(&id), Some(owner));

        // Event is raised
        assert!(System::events().iter().any(|er| er.event
            == TestEvent::orderbook(RawEvent::OrderPosted(
                sender,
                id.clone(),
                owner
            ))));
    });
}

#[test]
fn create_order_with_invalid_sender() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Orderbook::post_order(
                Origin::none(),
                vec!(),
                account_key(TEST_ORGANIZATION),
                None
            ),
            dispatch::DispatchError::BadOrigin
        );
    });
}

#[test]
fn create_order_with_missing_id() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Orderbook::post_order(
                Origin::signed(account_key(TEST_SENDER)),
                vec!(),
                account_key(TEST_ORGANIZATION),
                None
            ),
            Error::<Test>::OrderIdMissing
        );
    });
}

#[test]
fn create_order_with_long_id() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Orderbook::post_order(
                Origin::signed(account_key(TEST_SENDER)),
                LONG_VALUE.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                None
            ),
            Error::<Test>::OrderIdTooLong
        );
    })
}

#[test]
fn create_order_with_existing_id() {
    new_test_ext().execute_with(|| {
        let existing_order = TEST_ORDER_ID.as_bytes().to_owned();
        let now = 42;

        store_test_order::<Test>(
            existing_order.clone(),
            account_key(TEST_ORGANIZATION),
            now,
        );

        assert_noop!(
            Orderbook::post_order(
                Origin::signed(account_key(TEST_SENDER)),
                existing_order,
                account_key(TEST_ORGANIZATION),
                None
            ),
            Error::<Test>::OrderIdExists
        );
    })
}

#[test]
fn create_order_with_too_many_fields() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Orderbook::post_order(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_ORDER_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                Some(vec![
                    OrderField::new(b"field1", b"val1"),
                    OrderField::new(b"field2", b"val2"),
                    OrderField::new(b"field3", b"val3"),
                    OrderField::new(b"field4", b"val4")
                ])
            ),
            Error::<Test>::OrderTooManyFields
        );
    })
}

#[test]
fn create_order_with_invalid_field_name() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Orderbook::post_order(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_ORDER_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                Some(vec![
                    OrderField::new(b"field1", b"val1"),
                    OrderField::new(b"field2", b"val2"),
                    OrderField::new(&LONG_VALUE.as_bytes().to_owned(), b"val3"),
                ])
            ),
            Error::<Test>::OrderInvalidFieldName
        );
    })
}

#[test]
fn create_order_with_invalid_field_value() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Orderbook::post_order(
                Origin::signed(account_key(TEST_SENDER)),
                TEST_ORDER_ID.as_bytes().to_owned(),
                account_key(TEST_ORGANIZATION),
                Some(vec![
                    OrderField::new(b"field1", b"val1"),
                    OrderField::new(b"field2", b"val2"),
                    OrderField::new(b"field3", &LONG_VALUE.as_bytes().to_owned()),
                ])
            ),
            Error::<Test>::OrderInvalidFieldValue
        );
    })
}
