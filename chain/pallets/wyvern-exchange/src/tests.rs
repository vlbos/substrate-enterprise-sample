// Tests to be written here

use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, dispatch};

const TEST_ORDER_ID: &str = "00012345600012";
const TEST_ORGANIZATION: &str = "Northwind";
const TEST_SENDER: &str = "Alice";
const TEST_SENDER_1: &str = "Bob";
const LONG_VALUE : &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Donec aliquam ut tortor nec congue. Pellente";

//   const makeOrder = (exchange, isMaker) => ({
//     exchange: exchange,
//     maker: accounts[0],
//     taker: accounts[0],
//     makerRelayerFee: 0,
//     takerRelayerFee: 0,
//     makerProtocolFee: 0,
//     takerProtocolFee: 0,
//     feeRecipient: isMaker ? accounts[0] : '0x0000000000000000000000000000000000000000',
//     feeMethod: 0,
//     side: 0,
//     saleKind: 0,
//     target: proxy,
//     howToCall: 0,
//     calldata: '0x',
//     replacementPattern: '0x',
//     staticTarget: '0x0000000000000000000000000000000000000000',
//     staticExtradata: '0x',
//     paymentToken: accounts[0],
//     basePrice: new BigNumber(0),
//     extra: 0,
//     listingTime: 0,
//     expirationTime: 0,
//     salt: new BigNumber(0)
//   })
type AccountId = <Test as system::Trait>::AccountId;
type Moment = <Test as timestamp::Trait>::Moment;
type Balance = <Test as balances::Trait>::Balance;
fn make_order(
    maker: AccountId,
    taker: AccountId,
    fee_recipient: AccountId,
    side: u8,
) -> OrderType<AccountId, Moment, Balance> {
    let sender = account_key(TEST_SENDER);
    let fee: u64 = 0;
    let bytes = vec![0x0];
    let time = Moment::default();
    OrderType::<AccountId, Moment, Balance> {
        index: 0,
        exchange: sender,

        maker: maker,

        taker: taker,

        maker_relayer_fee: fee,

        taker_relayer_fee: fee,

        maker_protocol_fee: fee,

        taker_protocol_fee: fee,

        fee_recipient: fee_recipient,

        fee_method: FeeMethod::from(0),

        side: Side::from(side),

        sale_kind: SaleKind::from(0),

        target: sender,

        how_to_call: HowToCall::from(0),

        calldata: bytes.clone(),

        replacement_pattern: bytes.clone(),

        static_target: sender,

        static_extradata: bytes.clone(),

        payment_token: sender,

        base_price: fee,

        extra: time,

        listing_time: Zero::zero(),

        expiration_time: Zero::zero(),

        salt: 0,
        registered: time,
    }
}

#[test]
fn change_minimum_maker_protocol_fee() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let new_minimum_maker_protocol_fee = 42;

        let result = WyvernExchange::change_minimum_maker_protocol_fee(
            Origin::signed(sender),
            new_minimum_maker_protocol_fee,
        );

        assert_ok!(result);

        assert_eq!(
            <MinimumMakerProtocolFee<Test>>::get(),
            new_minimum_maker_protocol_fee
        );
    });
}

#[test]
fn change_minimum_taker_protocol_fee() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let min_taker_protocol_fee = 42;

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
fn change_protocol_fee_recipient() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let sender1 = account_key(TEST_SENDER_1);

        let result = WyvernExchange::change_protocol_fee_recipient(Origin::signed(sender), sender1);

        assert_ok!(result);

        assert_eq!(<ProtocolFeeRecipient<Test>>::get(), sender1);
    });
}

// [order.exchange, order.maker, order.taker, order.feerecipient, order.target, order.statictarget, order.paymenttoken],
//               [order.makerRelayerFee, order.takerRelayerFee, order.makerProtocolFee, order.takerProtocolFee, order.basePrice, order.extra, order.listingTime, order.expirationTime, order.salt],
//               order.feeMethod,
//               order.side,
//               order.saleKind,
//               order.howToCall,
//               order.calldata,
//               order.replacementPattern,
//               order.staticExtradata,
//               true
//             ).then(res => {
//
//                 return exchangeInstance.cancelOrder_(
//                   [order.exchange, order.maker, order.taker, order.feerecipient, order.target, order.statictarget, order.paymenttoken],
//                   [order.makerRelayerFee, order.takerRelayerFee, order.makerProtocolFee, order.takerProtocolFee, order.basePrice, order.extra, order.listingTime, order.expirationTime, order.salt],
//                   order.feeMethod,
//                   order.side,
//                   order.saleKind,
//                   order.howToCall,
//                   order.calldata,
//                   order.replacementPattern,
//                   order.staticExtradata,
//                   0, '0x', '0x'

#[test]
fn approve_order_ex() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let sender1 = account_key(TEST_SENDER_1);

        create_account_test(sender);
        create_account_test(sender1);
        let order = make_order(sender, sender, sender, 0);
        let addrs = vec![
            order.exchange,
            order.maker,
            order.taker,
            order.fee_recipient,
            order.target,
            order.static_target,
            order.payment_token,
        ]
        .to_vec();
        let uints = vec![
            order.maker_relayer_fee,
            order.taker_relayer_fee,
            order.maker_protocol_fee,
            order.taker_protocol_fee,
            order.base_price,
            order.extra,
            order.listing_time,
            order.expiration_time,
            order.salt,
        ]
        .to_vec();
        // let addrs =  Vec::<<Test as system::Trait>::AccountId>::new();
        //    let     uints =  Vec::<u32>::new();
        let fee_method = FeeMethod::from(0);
        let side = Side::from(0);
        let sale_kind = SaleKind::from(0);
        let how_to_call = HowToCall::from(0);
        let calldata = Vec::<u8>::new();
        let replacement_pattern = Vec::<u8>::new();
        let static_extradata = Vec::<u8>::new();
        let orderbook_inclusion_desired: bool = false;

        let result = WyvernExchange::approve_order_ex(
            Origin::signed(sender),
            addrs,
            uints,
            fee_method,
            side,
            sale_kind,
            how_to_call,
            calldata,
            replacement_pattern,
            static_extradata,
            orderbook_inclusion_desired,
        );

        assert_ok!(result);
    });
}

#[test]
fn cancel_order_ex() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let sender1 = account_key(TEST_SENDER_1);

        create_account_test(sender);
        create_account_test(sender1);
        <ContractSelf<Test>>::put(sender);
        let order = make_order(sender, sender, sender, 0);
        let addrs = vec![
            order.exchange,
            order.maker,
            order.taker,
            order.fee_recipient,
            order.target,
            order.static_target,
            order.payment_token,
        ]
        .to_vec();
        let uints = vec![
            order.maker_relayer_fee,
            order.taker_relayer_fee,
            order.maker_protocol_fee,
            order.taker_protocol_fee,
            order.base_price,
            order.extra,
            order.listing_time,
            order.expiration_time,
            order.salt,
        ]
        .to_vec();

        let fee_method = FeeMethod::from(0);
        let side = Side::from(0);
        let sale_kind = SaleKind::from(0);
        let how_to_call = HowToCall::from(0);
        let calldata = Vec::<u8>::new();
        let replacement_pattern = Vec::<u8>::new();
        let static_extradata = Vec::<u8>::new();
        let sig = Signature::default();

        let result = WyvernExchange::cancel_order_ex(
            Origin::signed(sender),
            addrs,
            uints,
            fee_method,
            side,
            sale_kind,
            how_to_call,
            calldata,
            replacement_pattern,
            static_extradata,
            sig,
        );

        assert_ok!(result);
    });
}

#[test]
fn atomic_match_ex() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let sender1 = account_key(TEST_SENDER_1);

        create_account_test(sender);
        create_account_test(sender1);
        <ContractSelf<Test>>::put(sender);
        let buy = make_order(sender, sender, sender, 0);
        let sell = make_order(sender1, sender, sender1, 1);
        let addrs = vec![
            buy.exchange,
            buy.maker,
            buy.taker,
            buy.fee_recipient,
            buy.target,
            buy.static_target,
            buy.payment_token,
            sell.exchange,
            sell.maker,
            sell.taker,
            sell.fee_recipient,
            sell.target,
            sell.static_target,
            sell.payment_token,
        ]
        .to_vec();
        let uints = vec![
            buy.maker_relayer_fee,
            buy.taker_relayer_fee,
            buy.maker_protocol_fee,
            buy.taker_protocol_fee,
            buy.base_price,
            buy.extra,
            buy.listing_time,
            buy.expiration_time,
            buy.salt,
            sell.maker_relayer_fee,
            sell.taker_relayer_fee,
            sell.maker_protocol_fee,
            sell.taker_protocol_fee,
            sell.base_price,
            sell.extra,
            sell.listing_time,
            sell.expiration_time,
            sell.salt,
        ]
        .to_vec();

        let fee_methods_sides_kinds_how_to_calls: Vec<u8> = vec![
            buy.fee_method.value(),
            buy.side.value(),
            buy.sale_kind.value(),
            buy.how_to_call.value(),
            sell.fee_method.value(),
            sell.side.value(),
            sell.sale_kind.value(),
            sell.how_to_call.value(),
        ]
        .to_vec();

        let calldata_buy = Vec::<u8>::new();
        let calldata_sell = Vec::<u8>::new();
        let replacement_pattern_buy = Vec::<u8>::new();
        let replacement_pattern_sell = Vec::<u8>::new();
        let static_extradata_buy = Vec::<u8>::new();
        let static_extradata_sell = Vec::<u8>::new();
        let sig = vec![Signature::default(), Signature::default()];
        let rss_metadata = Vec::<u8>::new();

        let result = WyvernExchange::atomic_match_ex(
            Origin::signed(sender),
            addrs,
            uints,
            fee_methods_sides_kinds_how_to_calls,
            calldata_buy,
            calldata_sell,
            replacement_pattern_buy,
            replacement_pattern_sell,
            static_extradata_buy,
            static_extradata_sell,
            sig,
            rss_metadata,
        );

        assert_ok!(result);
    });
}

#[test]
fn transfer_tokens() {
    new_test_ext().execute_with(|| {
        let sender = account_key(TEST_SENDER);
        let sender1 = account_key(TEST_SENDER_1);

        let amount = 42;
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
