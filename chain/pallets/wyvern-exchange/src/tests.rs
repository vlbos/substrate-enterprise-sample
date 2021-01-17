// Tests to be written here

use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, dispatch};

pub fn store_test_order<T: Trait>(id: OrderId, owner: T::AccountId, registered: T::Moment) {
    let index = 1;
    Orders::<T>::insert(
        index,
        OrderJSONType {
            index,
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
        let index = 1;
        Timestamp::set_timestamp(now);

        let result = Orderbook::post_order(Origin::signed(sender), id.clone(), owner.clone(), None);

        assert_ok!(result);

        assert_eq!(
            Orderbook::order_by_id(&id),
            Some(OrderJSONType {
                index: index,
                id: id.clone(),
                owner: owner,
                registered: now,
                fields: None
            })
        );

        // assert_eq!(<OrdersOfOrganization<Test>>::get(owner), vec![id.clone()]);

        assert_eq!(Orderbook::owner_of(&id), Some(owner));

        // Event is raised
        assert!(System::events()
            .iter()
            .any(|er| er.event
                == TestEvent::orderbook(RawEvent::OrderPosted(sender, id.clone(), owner))));
    });
}

// {
//   "created_date": "2019-01-29T04:04:03.258323",
//   "order_hash": "0x3f8d16507c4d9905815e860324d64b9c9f5933a70e59c2a07a63320459f67826",
//   "metadata": {
//     "asset": {
//       "id": "505",
//       "address": "0x16baf0de678e52367adc69fd067e5edd1d33e3bf"
//     },
//     "schema": "ERC721"
//   },
//   "exchange": "0x5206e78b21ce315ce284fb24cf05e0585a93b1d9",
//   "maker": {
//     "user": {
//       "username": "alex2"
//     },
//     "profile_img_url": "https://storage.googleapis.com/opensea-static/opensea-profile/11.png",
//     "address": "0xe96a1b303a1eb8d04fb973eb2b291b8d591c8f72",
//     "config": "affiliate"
//   },
//   "taker": {
//     "user": null,
//     "profile_img_url": "https://storage.googleapis.com/opensea-static/opensea-profile/1.png",
//     "address": "0x0000000000000000000000000000000000000000",
//     "config": ""
//   },
//   "current_price": "10000000000000000",
//   "current_bounty": "100000000000000.0",
//   "maker_relayer_fee": "100",
//   "taker_relayer_fee": "250",
//   "maker_protocol_fee": "0",
//   "taker_protocol_fee": "0",
//   "maker_referrer_fee": "0",
//   "fee_recipient": {
//     "user": null,
//     "profile_img_url": "https://storage.googleapis.com/opensea-static/opensea-profile/1.png",
//     "address": "0x0000000000000000000000000000000000000000",
//     "config": ""
//   },
//   "fee_method": 1,
//   "side": 1,
//   "sale_kind": 0,
//   "target": "0x16baf0de678e52367adc69fd067e5edd1d33e3bf",
//   "how_to_call": 0,
//   "calldata": "0x23b872dd000000000000000000000000e96a1b303a1eb8d04fb973eb2b291b8d591c8f72000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001f9",
//   "replacement_pattern": "0x000000000000000000000000000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000000000000000000",
//   "static_target": "0x0000000000000000000000000000000000000000",
//   "static_extradata": "0x",
//   "payment_token": "0xc778417e063141139fce010982780140aa0cd5ab",
//   "payment_token_contract": {
//     "address": "0xc778417e063141139fce010982780140aa0cd5ab",
//     "image_url": null,
//     "name": "Wrapped Ether",
//     "symbol": "WETH",
//     "decimals": 18,
//     "eth_price": "1.000000000000000"
//   },
//   "base_price": "10000000000000000",
//   "extra": "0",
//   "listing_time": 1548734810,
//   "expiration_time": 0,
//   "salt": "83006245783548033686093530747847303952463217644495033304999143031082661844460",
//   "v": 28,
//   "r": "0x2a0b0f3b8e6705cdf7894d9f1fb547646c5502a9d1d993c308ed0310620cf660",
//   "s": "0x19211a9a0c3ab3bb94b840774a2f9badf637b95d90b68965a4cf3734d5eaba98",
//   "cancelled": false,
//   "finalized": false,
//   "marked_invalid": false,
//   "prefixed_hash": "0x98a07dfb9e4da7ffc0ad0fb230afc8684dc4a0ac44623eded6a4c42e1df99954"
// }

#[test]
fn create_order_with_valid_fields() {
    let fields = vec![
OrderField::new(b"created_date", b"2019-01-29T04:04:03.258323"),
OrderField::new(b"order_hash", b"0x3f8d16507c4d9905815e860324d64b9c9f5933a70e59c2a07a63320459f67826"),
OrderField::new(b"metadata.asset.id", b"505"),
OrderField::new(b"metadata.asset.address", b"0x16baf0de678e52367adc69fd067e5edd1d33e3bf"),
OrderField::new(b"metadata.schema", b"ERC721"),
OrderField::new(b"exchange", b"0x5206e78b21ce315ce284fb24cf05e0585a93b1d9"),
OrderField::new(b"maker.user.username", b"alex2"),
OrderField::new(b"maker.profile_img_url", b"https://storage.googleapis.com/opensea-static/opensea-profile/11.png"),
OrderField::new(b"maker.address", b"0xe96a1b303a1eb8d04fb973eb2b291b8d591c8f72"),
OrderField::new(b"maker.config", b"affiliate"),
OrderField::new(b"taker.user", b"null"),
OrderField::new(b"taker.profile_img_url", b"https://storage.googleapis.com/opensea-static/opensea-profile/1.png"),
OrderField::new(b"taker.address", b"0x0000000000000000000000000000000000000000"),
OrderField::new(b"taker.config", b""),
OrderField::new(b"current_price", b"10000000000000000"),
OrderField::new(b"current_bounty", b"100000000000000.0"),
OrderField::new(b"maker_relayer_fee", b"100"),
OrderField::new(b"taker_relayer_fee", b"250"),
OrderField::new(b"maker_protocol_fee", b"0"),
OrderField::new(b"taker_protocol_fee", b"0"),
OrderField::new(b"maker_referrer_fee", b"0"),
OrderField::new(b"fee_recipient.user", b"null"),
OrderField::new(b"fee_recipient.profile_img_url", b"https://storage.googleapis.com/opensea-static/opensea-profile/1.png"),
OrderField::new(b"fee_recipient.address", b"0x0000000000000000000000000000000000000000"),
OrderField::new(b"fee_recipient.config", b""),
OrderField::new(b"fee_method", b"1"),
OrderField::new(b"side", b"1"),
OrderField::new(b"sale_kind", b"0"),
OrderField::new(b"target", b"0x16baf0de678e52367adc69fd067e5edd1d33e3bf"),
OrderField::new(b"how_to_call", b"0"),
OrderField::new(b"calldata", b"0x23b872dd000000000000000000000000e96a1b303a1eb8d04fb973eb2b291b8d591c8f72000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001f9"),
OrderField::new(b"replacement_pattern", b"0x000000000000000000000000000000000000000000000000000000000000000000000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000000000000000000"),
OrderField::new(b"static_target", b"0x0000000000000000000000000000000000000000"),
OrderField::new(b"static_extradata", b"0x"),
OrderField::new(b"payment_token", b"0xc778417e063141139fce010982780140aa0cd5ab"),
OrderField::new(b"payment_token_contract.address", b"0xc778417e063141139fce010982780140aa0cd5ab"),
OrderField::new(b"payment_token_contract.image_url", b"null"),
OrderField::new(b"payment_token_contract.name", b"Wrapped Ether"),
OrderField::new(b"payment_token_contract.symbol", b"WETH"),
OrderField::new(b"payment_token_contract.decimals", b"18"),
OrderField::new(b"payment_token_contract.eth_price", b"1.000000000000000"),
OrderField::new(b"base_price", b"10000000000000000"),
OrderField::new(b"extra", b"0"),
OrderField::new(b"listing_time", b"1548734810"),
OrderField::new(b"expiration_time", b"0"),
OrderField::new(b"salt", b"83006245783548033686093530747847303952463217644495033304999143031082661844460"),
OrderField::new(b"v", b"28"),
OrderField::new(b"r", b"0x2a0b0f3b8e6705cdf7894d9f1fb547646c5502a9d1d993c308ed0310620cf660"),
OrderField::new(b"s", b"0x19211a9a0c3ab3bb94b840774a2f9badf637b95d90b68965a4cf3734d5eaba98"),
OrderField::new(b"cancelled", b"false"),
OrderField::new(b"finalized", b"false"),
OrderField::new(b"marked_invalid", b"false"),
OrderField::new(b"prefixed_hash", b"0x98a07dfb9e4da7ffc0ad0fb230afc8684dc4a0ac44623eded6a4c42e1df99954"),
            ];
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let id = TEST_ORDER_ID.as_bytes().to_owned();
        let owner = account_key(TEST_ORGANIZATION);
        let now = 42;
        let index = 1;
        Timestamp::set_timestamp(now);

        let result = Orderbook::post_order(
            Origin::signed(sender),
            id.clone(),
            owner.clone(),
            Some(fields),
        );

        assert_ok!(result);

        assert_eq!(
            Orderbook::order_by_index(index),
            Some(OrderJSONType {
                index: index,
                id: id.clone(),
                owner: owner,
                registered: now,
                fields: Some(fields),
            })
        );

        assert_eq!(
            Orderbook::get_orders( Some(OrderQuery {
                fields: Some(fields),
            }),
            Some(OrderJSONType {
                index: index,
                id: id.clone(),
                owner: owner,
                registered: now,
                fields: Some(fields),
            })
        );

        // assert_eq!(<OrdersOfOrganization<Test>>::get(owner), vec![id.clone()]);

        assert_eq!(Orderbook::owner_of(&id), Some(owner));

        // Event is raised
        assert!(System::events()
            .iter()
            .any(|er| er.event
                == TestEvent::orderbook(RawEvent::OrderPosted(sender, id.clone(), owner))));
    });
}

#[test]
fn create_order_with_invalid_sender() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            Orderbook::post_order(Origin::none(), vec!(), account_key(TEST_ORGANIZATION), None),
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

        store_test_order::<Test>(existing_order.clone(), account_key(TEST_ORGANIZATION), now);

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
