//! # Substrate Enterprise Sample - OrderType Post example pallet

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "512"]

use codec::{Decode, Encode};
use core::result::Result;
use core::convert::TryInto;
// use sp_std::convert::{TryFrom, TryInto};

use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult, DispatchResultWithPostInfo},
    ensure,
    sp_io::hashing::keccak_256,
    sp_runtime::{
        print,
        traits::{
            DispatchInfoOf, Dispatchable, IdentifyAccount, Member, PostDispatchInfoOf,
            SaturatedConversion, Saturating, SignedExtension, Verify, Zero,
        },
        MultiSignature, RuntimeDebug,
    },
    sp_std::collections::btree_set::BTreeSet,
    sp_std::prelude::*,
    traits::{
        Currency, ExistenceRequirement::AllowDeath, Get, LockableCurrency, Randomness,
        ReservableCurrency,
    },
};


// use sp_runtime::{generic, MultiSignature, traits::{Verify, BlakeTwo256, IdentifyAccount}};

// traits::EnsureOrigin,
use balances::Call as BalancesCall;
use frame_system::{self as system, ensure_signed};

// use sp_core::H256;
// use sp_io::hashing;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
////ETH  BEGIN

////ETH END
// General constraints to limit data size
// Note: these could also be passed as trait config parameters
pub const ORDER_ID_MAX_LENGTH: usize = 36;
pub const ORDER_FIELD_NAME_MAX_LENGTH: usize = 10;
pub const ORDER_FIELD_VALUE_MAX_LENGTH: usize = 20;
pub const ORDER_MAX_FIELDS: usize = 3;
// // Inverse basis point.

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

pub const INVERSE_BASIS_POINT: u32 = 10000;

// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

// Some way of identifying an account on the chain. We intentionally make it equivalent
// to the public key of our transaction signing scheme.
// pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

// Custom types
// pub type AccountId =Vec<u8>;
pub type OrderId = Vec<u8>;
pub type FieldName = Vec<u8>;
pub type FieldValue = Vec<u8>;

pub type Bytes = Vec<u8>;

//sale kind interface
#[derive(Encode, Decode, Debug, Clone, Eq, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Encode, Decode, Debug, Clone, Eq, PartialEq)]
pub enum SaleKind {
    FixedPrice,
    DutchAuction,
}

// // Fee method: protocol fee or split fee.
// enum FeeMethod { ProtocolFee, SplitFee }
#[derive(Encode, Decode, Debug, Clone, Eq, PartialEq)]
pub enum FeeMethod {
    ProtocolFee,
    SplitFee,
}

#[derive(Encode, Decode, Debug, Clone, Eq, PartialEq)]
pub enum HowToCall {
    Call,
    DelegateCall,
}

impl Default for Side {
    fn default() -> Self {
        Self::Buy
    }
}

impl Default for SaleKind {
    fn default() -> Self {
        Self::FixedPrice
    }
}
impl Default for FeeMethod {
    fn default() -> Self {
        Self::ProtocolFee
    }
}

impl Default for HowToCall {
    fn default() -> Self {
        Self::Call
    }
}

impl HowToCall {
    pub fn value(&self) -> u8 {
        match *self {
            HowToCall::Call => 0x0,
            HowToCall::DelegateCall => 0x1,
        }
    }
}

impl From<u8> for HowToCall {
    fn from(orig: u8) -> Self {
        match orig {
            0x0 => return HowToCall::Call,
            _ => return HowToCall::DelegateCall,
        };
    }
}

impl FeeMethod {
    pub fn value(&self) -> u8 {
        match *self {
            FeeMethod::ProtocolFee => 0x0,
            FeeMethod::SplitFee => 0x1,
        }
    }
}

impl From<u8> for FeeMethod {
    fn from(orig: u8) -> Self {
        match orig {
            0x0 => return FeeMethod::ProtocolFee,
            _ => return FeeMethod::SplitFee,
        };
    }
}

impl SaleKind {
    pub fn value(&self) -> u8 {
        match *self {
            SaleKind::FixedPrice => 0x0,
            SaleKind::DutchAuction => 0x1,
        }
    }
}

impl From<u8> for SaleKind {
    fn from(orig: u8) -> Self {
        match orig {
            0x0 => return SaleKind::FixedPrice,
            _ => return SaleKind::DutchAuction,
        };
    }
}

impl Side {
    pub fn value(&self) -> u8 {
        match *self {
            Side::Buy => 0x0,
            Side::Sell => 0x1,
        }
    }
}

impl From<u8> for Side {
    fn from(orig: u8) -> Self {
        match orig {
            0x0 => return Side::Buy,
            _ => return Side::Sell,
        };
    }
}

//exchange core begin

// OrderType contains master data (aka class-level) about a trade item.
// This data is typically registered once by the order's manufacturer / supplier,
// to be shared with other network participants, and remains largely static.
// It can also be used for instance-level (lot) master data.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct OrderType<AccountId, Moment, Balance> {
    // // An order on the exchange.
    pub index: u64,
    // Exchange AccountId, intended as a versioning mechanism.
    pub exchange: AccountId,
    // OrderType maker AccountId.
    pub maker: AccountId,
    // OrderType taker AccountId, if specified.
    pub taker: AccountId,
    // Maker relayer fee of the order, unused for taker order.
    pub maker_relayer_fee: Balance,
    // Taker relayer fee of the order, or maximum taker fee for a taker order.
    pub taker_relayer_fee: Balance,
    // Maker protocol fee of the order, unused for taker order.
    pub maker_protocol_fee: Balance,
    // Taker protocol fee of the order, or maximum taker fee for a taker order.
    pub taker_protocol_fee: Balance,
    // OrderType fee recipient or zero AccountId for taker order.
    pub fee_recipient: AccountId,
    // Fee method (protocol token or split fee).
    pub fee_method: FeeMethod,
    // Side (buy/sell).
    pub side: Side,
    // Kind of sale.
    pub sale_kind: SaleKind,
    // Target.
    pub target: AccountId,
    // Vec<u8>.
    pub how_to_call: HowToCall,
    // Calldata.
    pub calldata: Bytes,
    // Calldata replacement pattern, or an empty byte array for no replacement.
    pub replacement_pattern: Bytes,
    // Static call target, zero-AccountId for no static call.
    pub static_target: AccountId,
    // Static call extra data.
    pub static_extradata: Bytes,
    // Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether.
    pub payment_token: AccountId,
    // Base price of the order (in paymentTokens).
    pub base_price: Balance,
    // Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference.
    pub extra: Moment,
    // Listing timestamp.
    pub listing_time: Moment,
    // Expiration timestamp - 0 for no expiry.
    pub expiration_time: Moment,
    // OrderType salt, used to prevent duplicate hashes.
    pub salt: u64,
    pub registered: Moment,
}

//exchange core

// Add new types to the trait:

// pub trait Trait: system::Trait {
//     type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
//     type Public: IdentifyAccount<AccountId = > + Clone;
//     type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
// }

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    // type Public: IdentifyAccount<AccountId = Self::AccountId> + Clone;
    // type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
    // // Currency type for this module.
    type Currency: ReservableCurrency<Self::AccountId>
        + LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
    // type CreateRoleOrigin: EnsureOrigin<Self::Origin>;
    // type Balance;
}

decl_storage! {
    trait Store for Module<T: Trait> as WyvernExchange {
        NextOrderIndex: BalanceOf<T>;
pub ContractSelf:T::AccountId;
 // // The token used to pay exchange fees.
    // ERC20 public ExchangeToken;
pub ExchangeToken:T::AccountId;
    // // User registry.
    // ProxyRegistry public registry;
pub Registry:T::AccountId;
    // // Token transfer proxy.
    // TokenTransferProxy public TokenTransferProxy;
pub TokenTransferProxy:T::AccountId;
    // // Cancelled / finalized orders, by hash.
    // mapping(Vec<u8> => bool) public CancelledOrFinalized;
  pub CancelledOrFinalized get(fn cancelled_or_finalized): map hasher(blake2_128_concat) Vec<u8> => bool;
    // // Orders verified by on-chain approval (alternative to ECDSA signatures so that smart contracts can place orders directly).
    // mapping(Vec<u8> => bool) public ApprovedOrders;
  pub ApprovedOrders get(fn approved_orders): map hasher(blake2_128_concat) Vec<u8> => bool;
    // // For split fee orders, minimum required protocol maker fee, in basis points. Paid to owner (who can change it).
    // BalanceOf<T> public MinimumMakerProtocolFee = 0;
pub MinimumMakerProtocolFee:BalanceOf<T>;
    // // For split fee orders, minimum required protocol taker fee, in basis points. Paid to owner (who can change it).
    // BalanceOf<T> public MinimumTakerProtocolFee = 0;
pub MinimumTakerProtocolFee:BalanceOf<T>;
    // // Recipient of protocol fees.
    // AccountId public ProtocolFeeRecipient;
pub ProtocolFeeRecipient:T::AccountId;


 }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = BalanceOf<T>,
        Moment = <T as timestamp::Trait>::Moment,
    {
        // event OrderApprovedPartOne    (Vec<u8> indexed hash, AccountId exchange, AccountId indexed maker, AccountId taker,
        // BalanceOf<T> maker_relayer_fee, BalanceOf<T> taker_relayer_fee, BalanceOf<T> maker_protocol_fee, BalanceOf<T> taker_protocol_fee,
        // AccountId indexed fee_recipient, FeeMethod fee_method, SaleKindInterface.Side side, SaleKindInterface.SaleKind sale_kind, AccountId target);
        // event OrderApprovedPartTwo    (Vec<u8> indexed hash, AuthenticatedProxy.Vec<u8> how_to_call, Vec<u8> calldata, Vec<u8> replacement_pattern,
        // AccountId static_target, Vec<u8> static_extradata, AccountId payment_token, BalanceOf<T> base_price,
        // BalanceOf<T> extra, BalanceOf<T> listing_time, BalanceOf<T> expiration_time, BalanceOf<T> salt, bool orderbook_inclusion_desired);
        // event OrderCancelled          (Vec<u8> indexed hash);
        // event OrdersMatched           (Vec<u8> buy_hash, Vec<u8> sell_hash, AccountId indexed maker, AccountId indexed taker, BalanceOf<T> price, Vec<u8> indexed metadata);
        OrderApprovedPartOne(
            Vec<u8>,
            AccountId,
            AccountId,
            AccountId,
            Balance,
            Balance,
            Balance,
            Balance,
            AccountId,
            FeeMethod,
            Side,
            SaleKind,
            AccountId,
        ),
        OrderApprovedPartTwo(
            Vec<u8>,
            HowToCall,
            Vec<u8>,
            Vec<u8>,
            AccountId,
            Vec<u8>,
            AccountId,
            Balance,
            Moment,
            Moment,
            Moment,
            u64,
            bool,
        ),
        OrderCancelled(Vec<u8>),
        OrdersMatched(Vec<u8>, Vec<u8>, AccountId, AccountId, Balance, Vec<u8>),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        OrderIdMissing,
        OrderIdTooLong,
        OrderIdExists,
        OrdersCannotMatch,
  OrdersCannotMatch1,
        OrderInvalidFieldName,
ArraySizeNotAsSameAsDesired,
ArraySizeNotAsSameAsMask,
BuyTakerProtocolFeeGreaterThanSellTakerProtocolFee,
BuyTakerRelayerFeeGreaterThanSellTakerRelayerFee,
SellPaymentTokenEqualPaymentToken,
SellTakerProtocolFeeGreaterThanBuyTakerProtocolFee,
SellTakerRelayerFeeGreaterThanBuyTakerRelayerFee,
ValueLessThanRequiredAmount,
ValueNotZero,
BuyPriceLessThanSellPrice,
        OrderHashMissing,
        OnlyMaker,
        OrderHashInvalid,
OrderHashInvalid1,
        OrderHashInvalid2,
OrderHashInvalid3,
        OrderHashInvalid4,
OrderHashInvalid5,
        OrderHashInvalid6,
OrderHashInvalid7,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;


//
//#dev Call approve_order - Solidity ABI encoding workaround:limitation, hopefully temporary.
//
#[weight = 10_000]
    pub fn approve_order_ex(origin,
         addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
        orderbook_inclusion_desired: bool,
    ) -> DispatchResult {
let _user = ensure_signed(origin.clone())?;
        let order: OrderType<T::AccountId, T::Moment, BalanceOf<T>> = Self::build_order_type_arr(
            addrs,
            uints,
            fee_method,
            side,
            sale_kind,
            how_to_call,
            &calldata,
            &replacement_pattern,
            &static_extradata,
        );
        Self::approve_order(origin, &order, orderbook_inclusion_desired)
    }

//
//#dev Call cancel_order - Solidity ABI encoding workaround:limitation, hopefully temporary.
//
#[weight = 10_000]
    pub fn cancel_order_ex(
        origin,
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
        sig: Signature,
    ) -> DispatchResult {
let _user = ensure_signed(origin.clone())?;
        Self::cancel_order(
            origin,
            &Self::build_order_type_arr(
                addrs,
                uints,
                fee_method,
                side,
                sale_kind,
                how_to_call,
                &calldata,
                &replacement_pattern,
                &static_extradata,
            ),
            &sig,
        )
    }

//
//#dev Call atomic_match - Solidity ABI encoding workaround:limitation, hopefully temporary.
//
#[weight = 10_000]
    pub fn atomic_match_ex(
        origin,
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        fee_methods_sides_kinds_how_to_calls: Vec<u8>,
        calldata_buy: Vec<u8>,
        calldata_sell: Vec<u8>,
        replacement_pattern_buy: Vec<u8>,
        replacement_pattern_sell: Vec<u8>,
        static_extradata_buy: Vec<u8>,
        static_extradata_sell: Vec<u8>,
        sig: Vec<Signature>,
        rss_metadata: Vec<u8>,
    ) -> DispatchResult {
        let _user = ensure_signed(origin)?;

        let bs = Self::build_order_type_arr2(
            addrs,
            uints,
            &fee_methods_sides_kinds_how_to_calls,
            &calldata_buy,
            &calldata_sell,
            &replacement_pattern_buy,
            &replacement_pattern_sell,
            &static_extradata_buy,
            &static_extradata_sell,
        );
        Self::atomic_match(
            _user,
            Zero::zero(),
            bs[0].clone(),
            sig[0].clone(),
            bs[1].clone(),
            sig[1].clone(),
            &rss_metadata,
        )?;
        Ok(())
    }

    //exchange core
//
//#dev Change the minimum maker fee paid to the protocol (only:owner)
//#param newMinimumMakerProtocolFee New fee to set in basis points
//
#[weight = 10_000]
    pub fn change_minimum_maker_protocol_fee(
  origin,
        new_minimum_maker_protocol_fee: BalanceOf<T>,
    ) -> DispatchResult
    {
// onlyOwner

let _user = ensure_signed(origin)?;
        MinimumMakerProtocolFee::<T>::put(new_minimum_maker_protocol_fee);
        Ok(())
    }

//
//#dev Change the minimum taker fee paid to the protocol (only:owner)
//#param new_minimum_taker_protocol_fee New fee to set in basis points
//
#[weight = 10_000]
    pub fn change_minimum_taker_protocol_fee(
  origin,
        new_minimum_taker_protocol_fee: BalanceOf<T>,
    ) -> DispatchResult {
        // onlyOwner
let _user = ensure_signed(origin)?;

        MinimumTakerProtocolFee::<T>::put(new_minimum_taker_protocol_fee);
        Ok(())
    }

//
//#dev Change the protocol fee recipient (only:owner)
//#param new_protocol_fee_recipient New protocol fee recipient AccountId
//
#[weight = 10_000]
    pub fn change_protocol_fee_recipient(
  origin,
        new_protocol_fee_recipient: T::AccountId,
    ) -> DispatchResult {

        // onlyOwner
let _user = ensure_signed(origin)?;

        ProtocolFeeRecipient::<T>::put(new_protocol_fee_recipient.clone());
        Ok(())
    }


 }
}

impl<T: Trait> Module<T> {
    // exchange
    //     fn from(acc:Vec<u8>) ->T::AccountId
    // {
    // T::AccountId::from(acc)
    // }

    // #dev Call calculate_final_price - library exposed for testing.
    pub fn calculate_final_price_ex(
        side: Side,
        sale_kind: SaleKind,
        base_price: BalanceOf<T>,
        extra: T::Moment,
        listing_time: T::Moment,
        expiration_time: T::Moment,
    ) -> BalanceOf<T> {
        Self::calculate_final_price(
            &side,
            &sale_kind,
            base_price,
            extra,
            listing_time,
            expiration_time,
        ).unwrap()
    }

    //
    //#dev Call hash_order - Solidity ABI encoding workaround:limitation, hopefully temporary.
    //
    pub fn hash_order_ex(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    ) -> Vec<u8> {
        Self::hash_order(&Self::build_order_type_arr(
            addrs,
            uints,
            fee_method,
            side,
            sale_kind,
            how_to_call,
            &calldata,
            &replacement_pattern,
            &static_extradata,
        )).unwrap()
    }

    //#dev Call hash_to_sign - Solidity ABI encoding workaround:limitation, hopefully temporary.

    pub fn hash_to_sign_ex(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    ) -> Vec<u8> {
        Self::hash_to_sign(&Self::build_order_type_arr(
            addrs,
            uints,
            fee_method,
            side,
            sale_kind,
            how_to_call,
            &calldata,
            &replacement_pattern,
            &static_extradata,
        )).unwrap()
    }

    //
    //#dev Call validate_order_parameters - Solidity ABI encoding workaround:limitation, hopefully temporary.
    //

    pub fn validate_order_parameters_ex(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    ) -> bool {
        let order: OrderType<T::AccountId, T::Moment, BalanceOf<T>> = Self::build_order_type_arr(
            addrs,
            uints,
            fee_method,
            side,
            sale_kind,
            how_to_call,
            &calldata,
            &replacement_pattern,
            &static_extradata,
        );
        Self::validate_order_parameters(&order).unwrap()
    }

    //
    //#dev Call validate_order - Solidity ABI encoding workaround:limitation, hopefully temporary.
    //

    pub fn validate_order_ex(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
        sig: Signature,
    ) -> bool {
        let order: OrderType<T::AccountId, T::Moment, BalanceOf<T>> = Self::build_order_type_arr(
            addrs,
            uints,
            fee_method,
            side,
            sale_kind,
            how_to_call,
            &calldata,
            &replacement_pattern,
            &static_extradata,
        );
        Self::validate_order(&Self::hash_to_sign(&order).unwrap(), &order, &sig).unwrap()
    }

    //
    //#dev Call calculate_current_price - Solidity ABI encoding workaround:limitation, hopefully temporary.
    //

    pub fn calculate_current_price_ex(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    ) -> BalanceOf<T> {
        Self::calculate_current_price(&Self::build_order_type_arr(
            addrs,
            uints,
            fee_method,
            side,
            sale_kind,
            how_to_call,
            &calldata,
            &replacement_pattern,
            &static_extradata,
        )).unwrap()
    }

    //
    //#dev Call orders_can_match - Solidity ABI encoding workaround:limitation, hopefully temporary.
    //

    pub fn orders_can_match_ex(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        fee_methods_sides_kinds_how_to_calls: Vec<u8>,
        calldata_buy: Vec<u8>,
        calldata_sell: Vec<u8>,
        replacement_pattern_buy: Vec<u8>,
        replacement_pattern_sell: Vec<u8>,
        static_extradata_buy: Vec<u8>,
        static_extradata_sell: Vec<u8>,
    ) -> bool {
        let bs = Self::build_order_type_arr2(
            addrs,
            uints,
            &fee_methods_sides_kinds_how_to_calls,
            &calldata_buy,
            &calldata_sell,
            &replacement_pattern_buy,
            &replacement_pattern_sell,
            &static_extradata_buy,
            &static_extradata_sell,
        );
        Self::orders_can_match(&bs[0], &bs[1]).unwrap()
    }

    //
    //#dev Return whether or not two orders' calldata specifications can match
    //#param buy_calldata Buy-side order calldata
    //#param buy_replacement_pattern Buy-side order calldata replacement mask
    //#param sell_calldata Sell-side order calldata
    //#param sell_replacement_pattern Sell-side order calldata replacement mask
    //#return Whether the orders' calldata can be matched
    //

    pub fn order_calldata_can_match(
        buy_calldata: Vec<u8>,
        buy_replacement_pattern: Vec<u8>,
        sell_calldata: Vec<u8>,
        sell_replacement_pattern: Vec<u8>,
    ) -> Result<bool, Error<T>> {
        let mut tmpbuy_calldata = buy_calldata.clone();
        let mut tmpsell_calldata = sell_calldata.clone();
        if buy_replacement_pattern.len() > 0 {
            let _r = Self::guarded_array_replace(
                &mut tmpbuy_calldata,
                &sell_calldata,
                &buy_replacement_pattern,
            );
            // ensure!(r.is_ok(),Error::<T>::OrderIdMissing);
        }
        if sell_replacement_pattern.len() > 0 {
            let _r = Self::guarded_array_replace(
                &mut tmpsell_calldata,
                &buy_calldata,
                &sell_replacement_pattern,
            );
            // ensure!(r.is_ok(),Error::<T>::OrderIdMissing);
        }

        Self::array_eq(&tmpbuy_calldata, &tmpsell_calldata)
    }

    //
    //#dev Call calculate_match_price - Solidity ABI encoding workaround:limitation, hopefully temporary.
    //

    pub fn calculate_match_price_ex(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        fee_methods_sides_kinds_how_to_calls: Vec<u8>,
        calldata_buy: Vec<u8>,
        calldata_sell: Vec<u8>,
        replacement_pattern_buy: Vec<u8>,
        replacement_pattern_sell: Vec<u8>,
        static_extradata_buy: Vec<u8>,
        static_extradata_sell: Vec<u8>,
    ) -> BalanceOf<T> {
        let bs = Self::build_order_type_arr2(
            addrs,
            uints,
            &fee_methods_sides_kinds_how_to_calls,
            &calldata_buy,
            &calldata_sell,
            &replacement_pattern_buy,
            &replacement_pattern_sell,
            &static_extradata_buy,
            &static_extradata_sell,
        );
        Self::calculate_match_price(&bs[0], &bs[1]).unwrap()
    }

    //
    //#dev Transfer tokens
    //#param token Token to transfer
    //#param from AccountId to charge fees
    //#param to AccountId to receive fees
    //#param amount Amount of protocol tokens to charge
    //
    pub fn transfer_tokens(
        _token: &T::AccountId,
        _from: &T::AccountId,
        _to: &T::AccountId,
        _amount: BalanceOf<T>,
    ) -> Result<(), Error<T>> {
        if _amount > Zero::zero() {
            // ensure!(TokenTransferProxy.transferFrom(token, from, to, amount), Error::<T>::OrderIdMissing);
            // let _a = _amount as u128;
            // let balance_amount = BalanceOf::<T>::zero();// _a.try_into().map_err(|_| ());
            let _ = T::Currency::transfer(
                &_from,
                &_to,
                _amount,
                frame_support::traits::ExistenceRequirement::AllowDeath,
            );
        }
        Ok(())
    }

    pub fn transfer_tokens_fee(
        _token: &T::AccountId,
        _from: &T::AccountId,
        _to: &T::AccountId,
        _amount: BalanceOf<T>,
        _price: &BalanceOf<T>,
    ) -> Result<(), Error<T>> {
        if _amount > Zero::zero() {
            let _amount = _amount * *_price / INVERSE_BASIS_POINT.into();
            Self::transfer_tokens(_token, _from, _to, _amount)?;
        }
        Ok(())
    }

    pub fn transfer_tokens_fee_sell(
        _token: &T::AccountId,
        _from: &T::AccountId,
        _to: &T::AccountId,
        _amount: BalanceOf<T>,
        _price: &BalanceOf<T>,
        receive_or_required_amount: &mut BalanceOf<T>,
        is_maker: bool,
    ) -> Result<(), Error<T>> {
        if _amount > Zero::zero() {
            let _fee = _amount * *_price / INVERSE_BASIS_POINT.into();
            let mut _from_ = (*_from).clone();
            if *_token == ContractSelf::<T>::get() {
                if is_maker {
                    *receive_or_required_amount -= _fee;
                } else {
                    *receive_or_required_amount += _fee;
                };

                _from_ = ContractSelf::<T>::get();
            }

            Self::transfer_tokens(_token, _from, _to, _amount)?;
        }
        Ok(())
    }

    //#dev Charge a fee in protocol tokens
    //#param from AccountId to charge fees
    //#param to AccountId to receive fees
    //#param amount Amount of protocol tokens to charge
    //
    pub fn charge_protocol_fee(
        from: &T::AccountId,
        to: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<(), Error<T>> {
        Self::transfer_tokens(&ExchangeToken::<T>::get(), &from, &to, amount)
    }

    //
    //#dev Hash an order, returning the canonical order hash, without the message prefix
    //#param order OrderType to hash
    //#return Hash of order
    //
    pub fn hash_order(
        order: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
    ) -> Result<Vec<u8>, Error<T>> {
        // hash := keccak256(add(array, 0x20), size)
        //    sp_io::hashing::blake2_256(&h).into()
        Ok(keccak_256(&order.encode()).into())
        // }
        // }
        // return hash;
    }

    //
    //#dev Hash an order, returning the hash that a client must sign, including the standard message prefix
    //#param order OrderType to hash
    //#return Hash of message prefix and order hash per Ethereum format
    //
    pub fn hash_to_sign(
        order: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
    ) -> Result<Vec<u8>, Error<T>> {
        Ok(keccak_256(&Self::hash_order(&order)?).to_vec())
    }

    //
    //#dev Assert an order is valid and return its hash
    //#param order OrderType to validate
    //#param sig ECDSA signature
    //
    pub fn require_valid_order(
        order: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
        sig: &Signature,
    ) -> Result<Vec<u8>, Error<T>> {
        let hash: Vec<u8> = Self::hash_to_sign(&order)?;
        ensure!(
            Self::validate_order(&hash, order, sig)?,
            Error::<T>::OrderHashInvalid
        );
        Ok(hash)
    }

    //
    //#dev Validate order parameters (does *not* check validity:signature)
    //#param order OrderType to validate
    //
    pub fn validate_order_parameters(
        order: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
    ) -> Result<bool, Error<T>> {
        // OrderType must be targeted at this protocol version (this contract:Exchange).
        //TODO
        if order.exchange != ContractSelf::<T>::get() {
            frame_support::debug::RuntimeLogger::init();
            debug::error!("exchange is contract self.");
            ensure!(false, Error::<T>::OrderHashInvalid1);
            return Ok(false);
        }

        // OrderType must possess valid sale kind parameter combination.
        if !Self::validate_parameters(&order.sale_kind, order.expiration_time)? {
            ensure!(false, Error::<T>::OrderHashInvalid2);
            debug::error!("validate_parameters is false.");
            return Ok(false);
        }

        // If using the split fee method, order must have sufficient protocol fees.
        if order.fee_method == FeeMethod::SplitFee
            && (order.maker_protocol_fee < MinimumMakerProtocolFee::<T>::get()
                || order.taker_protocol_fee < MinimumTakerProtocolFee::<T>::get())
        {
            ensure!(false, Error::<T>::OrderHashInvalid3);
            debug::error!("fee_method is not split fee or maker_protocol_fee greater than setting or taker_protocol_fee greater than setting.");
            return Ok(false);
        }

        Ok(true)
    }

    //
    //#dev Validate a provided previously approved / signed order, hash, and signature.
    //#param hash OrderType hash (calculated:already, passed to recalculation:avoid)
    //#param order OrderType to validate
    //#param sig ECDSA signature
    //
    pub fn validate_order(
        hash: &[u8],
        order: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
        sig: &Signature,
    ) -> Result<bool, Error<T>> {
        // Not done in an if-conditional to prevent unnecessary ecrecover evaluation, which seems to happen even though it should short-circuit.
        frame_support::debug::RuntimeLogger::init();
        debug::error!("exchange is contract self.");
        print("================");
        // OrderType must have valid parameters.
        if !Self::validate_order_parameters(&order)? {
            debug::error!("exchange is contract self.");
            ensure!(false, Error::<T>::OrderHashInvalid4);
            return Ok(false);
        }

        // OrderType must have not been canceled or already filled.
        if CancelledOrFinalized::get(hash) {
            debug::error!("exchange is contract self.");
            ensure!(false, Error::<T>::OrderHashInvalid5);
            return Ok(false);
        }

        // OrderType authentication. OrderType must be either:
        // (a) previously approved
        if ApprovedOrders::get(hash) {
            debug::error!("exchange is contract self.");
            ensure!(false, Error::<T>::OrderHashInvalid6);
            return Ok(true);
        }

        // or (b) ECDSA-signed by maker.
        // if ecrecover(hash, sig.v, sig.r, sig.s) == order.maker {
        //     return true;
        // }
        if Self::check_signature(&sig, &hash, order.maker()).is_ok() {
            debug::error!("exchange is contract self.");

            return Ok(true);
        }
        ensure!(false, Error::<T>::OrderHashInvalid7);
        Ok(false)
    }

    // An alterantive way to validate a signature is:
    // Import the codec and traits:
    // Example function to verify the signature.

    pub fn check_signature(
    _signature: &Signature,
        _msg: &[u8],
        _signer: &T::AccountId,
    ) -> Result<(), Error<T>> {
// let mut bytes = [u8; 32];
// T::AccountId::decode(&mut &bytes[..]).unwrap_or_default();
        // if _signature.verify(_msg, _signer) {
        Ok(())
        // } else {
        //     Err(Error::<T>::OrderIdMissing.into())
        // }
    }

    //
    //#dev Approve an order and optionally mark it for orderbook inclusion. Must be called by the maker of the order
    //#param order OrderType to approve
    //#param orderbook_inclusion_desired Whether orderbook providers should include the order in their orderbooks
    //
    pub fn approve_order(
        origin: T::Origin,
        order: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
        orderbook_inclusion_desired: bool,
    ) -> DispatchResult {
        // CHECKS
        let _user = ensure_signed(origin)?;
        // Assert sender is authorized to approve order.
        ensure!(_user == order.maker, Error::<T>::OnlyMaker);

        // Calculate order hash.
        let hash: Vec<u8> = Self::hash_to_sign(&order)?;

        // Assert order has not already been approved.
        ensure!(
            !ApprovedOrders::get(hash.clone()),
            Error::<T>::OrderHashMissing
        );

        // EFFECTS

        // Mark order as approved.
        ApprovedOrders::insert(hash.clone(), true);

        // Log approval event. Must be split in two due to Solidity stack size limitations.
        Self::deposit_event(RawEvent::OrderApprovedPartOne(
            hash.clone(),
            order.exchange.clone(),
            order.maker.clone(),
            order.taker.clone(),
            order.maker_relayer_fee,
            order.taker_relayer_fee,
            order.maker_protocol_fee,
            order.taker_protocol_fee,
            order.fee_recipient.clone(),
            order.fee_method.clone(),
            order.side.clone(),
            order.sale_kind.clone(),
            order.target.clone(),
        ));

        Self::deposit_event(RawEvent::OrderApprovedPartTwo(
            hash.clone(),
            order.how_to_call.clone(),
            order.calldata.clone(),
            order.replacement_pattern.clone(),
            order.static_target.clone(),
            order.static_extradata.clone(),
            order.payment_token.clone(),
            order.base_price.clone(),
            order.extra.clone(),
            order.listing_time.clone(),
            order.expiration_time.clone(),
            order.salt.clone(),
            orderbook_inclusion_desired,
        ));

        Ok(())
    }

    //
    //#dev Cancel an order, preventing it from being matched. Must be called by the maker of the order
    //#param order OrderType to cancel
    //#param sig ECDSA signature
    //
    pub fn cancel_order(
        origin: T::Origin,
        order: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
        sig: &Signature,
    ) -> DispatchResult {
        // CHECKS
        let _user = ensure_signed(origin)?;

        // Assert sender is authorized to cancel order.
        ensure!(_user == order.maker, Error::<T>::OnlyMaker);

        // Calculate order hash.
        let hash = Self::require_valid_order(order, sig)?;
        // EFFECTS
        // Mark order as cancelled, preventing it from being matched.
        CancelledOrFinalized::insert(hash.clone(), true);

        // Log cancel event.
        Self::deposit_event(RawEvent::OrderCancelled(hash.clone()));

        Ok(())
    }

    //
    //#dev Calculate the current price of an order (fn:convenience)
    //#param order OrderType to calculate the price of
    //#return The current price of the order
    //
    pub fn calculate_current_price(
        order: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
    ) -> Result<BalanceOf<T>, Error<T>> {
        Self::calculate_final_price(
            &order.side,
            &order.sale_kind,
            order.base_price,
            order.extra,
            order.listing_time,
            order.expiration_time,
        )
    }

    //
    //#dev Calculate the price two orders would match at, if in fact they would match (fail:otherwise)
    //#param buy Buy-side order
    //#param sell Sell-side order
    //#return Match price
    //
    pub fn calculate_match_price(
        buy: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
        sell: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
    ) -> Result<BalanceOf<T>, Error<T>> {
        // Calculate sell price.
        let sell_price: BalanceOf<T> = Self::calculate_final_price(
            &sell.side,
            &sell.sale_kind,
            sell.base_price,
            sell.extra,
            sell.listing_time,
            sell.expiration_time,
        )?;

        // Calculate buy price.
        let buy_price: BalanceOf<T> = Self::calculate_final_price(
            &buy.side,
            &buy.sale_kind,
            buy.base_price,
            buy.extra,
            buy.listing_time,
            buy.expiration_time,
        )?;

        // Require price cross.
        ensure!(
            buy_price >= sell_price,
            Error::<T>::BuyPriceLessThanSellPrice
        );

        // Maker/taker priority.
        let price: BalanceOf<T> = if sell.fee_recipient != ContractSelf::<T>::get() {
            sell_price
        } else {
            buy_price
        };

        Ok(price)
    }

    //
    //#dev Execute all ERC20 token / Ether transfers associated with an order match (fees and buyer => transfer:seller)
    //#param buy Buy-side order
    //#param sell Sell-side order
    //
    pub fn execute_funds_transfer(
        msg_value: BalanceOf<T>,
        buy: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
        sell: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
    ) -> Result<BalanceOf<T>, Error<T>> {
        // let originprotocol_fee_recipient = ProtocolFeeRecipient::<T>::get();
        // Only payable in the special case of unwrapped Ether.
        if sell.payment_token != ContractSelf::<T>::get() {
            ensure!(msg_value == Zero::zero(), Error::<T>::ValueNotZero);
        }

        // Calculate match price.
        let price: BalanceOf<T> = Self::calculate_match_price(&buy, &sell)?;

        // If paying using a token (Ether:not), transfer tokens. This is done prior to fee payments to that a seller will have tokens before being charged fees.
        if price > Zero::zero() && sell.payment_token != ContractSelf::<T>::get() {
            Self::transfer_tokens(sell.payment_token(), &buy.maker(), sell.maker(), price)?;
        }

        // Amount that will be received by seller (Ether:for).
        let mut receive_amount: BalanceOf<T> = price;

        // Amount that must be sent by buyer (Ether:for).
        let mut required_amount: BalanceOf<T> = price;

        // Determine maker/taker and charge fees accordingly.
        if sell.fee_recipient != ContractSelf::<T>::get() {
            // Sell-side order is maker.
            Self::execute_funds_transfer_sell_side(
                buy,
                sell,
                &price,
                &mut receive_amount,
                &mut required_amount,
            )?;
        // // Assert taker fee is less than or equal to maximum fee specified by buyer.
        // ensure!(
        //     sell.taker_relayer_fee <= buy.taker_relayer_fee,
        //     Error::<T>::OrderIdMissing
        // );

        // if sell.fee_method == FeeMethod::SplitFee {
        //     // Assert taker fee is less than or equal to maximum fee specified by buyer.
        //     ensure!(
        //         sell.taker_protocol_fee <= buy.taker_protocol_fee,
        //         Error::<T>::OrderIdMissing
        //     );

        //     // Maker fees are deducted from the token amount that the maker receives. Taker fees are extra tokens that must be paid by the taker.

        //     if sell.maker_relayer_fee > Zero::zero() {
        //         let maker_relayer_fee: BalanceOf<T> = sell.maker_relayer_fee * price / INVERSE_BASIS_POINT.into();
        //         if sell.payment_token == ContractSelf::<T>::get() {
        //             receive_amount = receive_amount - maker_relayer_fee;
        //             // sell.fee_recipient.transfer(maker_relayer_fee);
        //           Self::transfer_tokens(
        //                 &ContractSelf::<T>::get(),
        //                 &ContractSelf::<T>::get(),
        //                 &sell.fee_recipient,
        //                 maker_relayer_fee,
        //             )?;
        //         } else {
        //           Self::transfer_tokens(
        //                 sell.payment_token(),
        //                 sell.maker(),
        //                 &sell.fee_recipient,
        //                 maker_relayer_fee,
        //             )?;
        //         }
        //     }

        //     if sell.taker_relayer_fee > Zero::zero() {
        //         let taker_relayer_fee: BalanceOf<T> = sell.taker_relayer_fee * price / INVERSE_BASIS_POINT.into();
        //         if sell.payment_token == ContractSelf::<T>::get() {
        //             required_amount = required_amount + taker_relayer_fee;
        //             // sell.fee_recipient.transfer(taker_relayer_fee);
        //           Self::transfer_tokens(
        //                 &ContractSelf::<T>::get(),
        //                 &ContractSelf::<T>::get(),
        //                 &sell.fee_recipient,
        //                 taker_relayer_fee,
        //             )?;
        //         } else {
        //           Self::transfer_tokens(
        //                 sell.payment_token(),
        //                 buy.maker(),
        //                 &sell.fee_recipient,
        //                 taker_relayer_fee,
        //             )?;
        //         }
        //     }

        //     if sell.maker_protocol_fee > Zero::zero() {
        //         let maker_protocol_fee: BalanceOf<T> = sell.maker_protocol_fee * price / INVERSE_BASIS_POINT.into();
        //         if sell.payment_token == ContractSelf::<T>::get() {
        //             receive_amount = receive_amount - maker_protocol_fee;
        //             // ProtocolFeeRecipient.transfer(maker_protocol_fee);
        //           Self::transfer_tokens(
        //                 &ContractSelf::<T>::get(),
        //                 &ContractSelf::<T>::get(),
        //                 &originprotocol_fee_recipient,
        //                 maker_protocol_fee,
        //             )?;
        //         } else {
        //           Self::transfer_tokens(
        //                 sell.payment_token(),
        //                 sell.maker(),
        //                 &originprotocol_fee_recipient,
        //                 maker_protocol_fee,
        //             )?;
        //         }
        //     }

        //     if sell.taker_protocol_fee > Zero::zero() {
        //         let taker_protocol_fee: BalanceOf<T> = sell.taker_protocol_fee * price / INVERSE_BASIS_POINT.into();
        //         if sell.payment_token == ContractSelf::<T>::get() {
        //             required_amount = required_amount + taker_protocol_fee;
        //             // ProtocolFeeRecipient.transfer(taker_protocol_fee);
        //           Self::transfer_tokens(
        //                 &ContractSelf::<T>::get(),
        //                 &ContractSelf::<T>::get(),
        //                 &originprotocol_fee_recipient,
        //                 taker_protocol_fee,
        //             )?;
        //         } else {
        //           Self::transfer_tokens(
        //                 sell.payment_token(),
        //                 buy.maker(),
        //                 &originprotocol_fee_recipient,
        //                 taker_protocol_fee,
        //             )?;
        //         }
        //     }
        // } else {
        //     // Charge maker fee to seller.
        //   Self::charge_protocol_fee(&sell.maker, &sell.fee_recipient, sell.maker_relayer_fee)?;

        //     // Charge taker fee to buyer.
        //   Self::charge_protocol_fee(&buy.maker, &sell.fee_recipient, sell.taker_relayer_fee)?;
        // }
        } else {
            // Buy-side order is maker.
            Self::execute_funds_transfer_buy_side(buy, sell, &price)?;

            // // Assert taker fee is less than or equal to maximum fee specified by seller.
            // ensure!(
            //     buy.taker_relayer_fee <= sell.taker_relayer_fee,
            //     Error::<T>::OrderIdMissing
            // );

            // if sell.fee_method == FeeMethod::SplitFee {
            //     // The Exchange does not escrow Ether, so direct Ether can only be used to with sell-side maker / buy-side taker orders.
            //     ensure!(sell.payment_token != ContractSelf::<T>::get(), Error::<T>::OrderIdMissing);

            //     // Assert taker fee is less than or equal to maximum fee specified by seller.
            //     ensure!(
            //         buy.taker_protocol_fee <= sell.taker_protocol_fee,
            //         Error::<T>::OrderIdMissing
            //     );

            //     if buy.maker_relayer_fee > Zero::zero() {
            //        let maker_relayer_fee =buy.maker_relayer_fee * price / INVERSE_BASIS_POINT.into();
            //       Self::transfer_tokens(
            //             sell.payment_token(),
            //             buy.maker(),
            //             &buy.fee_recipient,
            //             maker_relayer_fee,
            //         )?;
            //     }

            //     if buy.taker_relayer_fee > Zero::zero() {
            //        let taker_relayer_fee = buy.taker_relayer_fee * price / INVERSE_BASIS_POINT.into();
            //       Self::transfer_tokens(
            //             sell.payment_token(),
            //             sell.maker(),
            //             &buy.fee_recipient,
            //             taker_relayer_fee,
            //         )?;
            //     }

            //     if buy.maker_protocol_fee > Zero::zero() {
            //        let maker_protocol_fee = buy.maker_protocol_fee * price / INVERSE_BASIS_POINT.into();
            //       Self::transfer_tokens(
            //             sell.payment_token(),
            //             buy.maker(),
            //             &originprotocol_fee_recipient,
            //             maker_protocol_fee,
            //         )?;
            //     }

            //     if buy.taker_protocol_fee > Zero::zero() {
            //         let taker_protocol_fee = buy.taker_protocol_fee * price / INVERSE_BASIS_POINT.into();
            //       Self::transfer_tokens(
            //             &sell.payment_token,
            //             &sell.maker,
            //             &originprotocol_fee_recipient,
            //             taker_protocol_fee,
            //         )?;
            //     }

            // } else {
            //     // Charge maker fee to buyer.
            //   Self::charge_protocol_fee(&buy.maker, &buy.fee_recipient, buy.maker_relayer_fee)?;

            //     // Charge taker fee to seller.
            //   Self::charge_protocol_fee(&sell.maker, &buy.fee_recipient, buy.taker_relayer_fee)?;
            // }
        }

        if sell.payment_token == ContractSelf::<T>::get() {
            // Special-case Ether, order must be matched by buyer.
            ensure!(
                msg_value >= required_amount,
                Error::<T>::ValueLessThanRequiredAmount
            );
            // sell.maker.transfer(receive_amount);
            Self::transfer_tokens(
                &ContractSelf::<T>::get(),
                &ContractSelf::<T>::get(),
                &sell.maker,
                receive_amount,
            )?;
            // Allow overshoot for variable-price auctions, refund difference.
            let diff: BalanceOf<T> = msg_value - required_amount;
            if diff > Zero::zero() {
                // buy.maker.transfer(diff);
                Self::transfer_tokens(
                    &ContractSelf::<T>::get(),
                    &ContractSelf::<T>::get(),
                    buy.maker(),
                    diff,
                )?;
            }
        }

        // This contract should never hold Ether, however, we cannot assert this, since it is impossible to prevent anyone from sending Ether e.g. with selfdestruct.

        Ok(price)
    }

    //
    //#dev Execute all ERC20 token / Ether transfers associated with an order match (fees and buyer => transfer:seller)
    //#param buy Buy-side order
    //#param sell Sell-side order
    //
    pub fn execute_funds_transfer_sell_side(
        buy: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
        sell: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
        price: &BalanceOf<T>,
        receive_amount: &mut BalanceOf<T>,
        required_amount: &mut BalanceOf<T>,
    ) -> Result<BalanceOf<T>, Error<T>> {
        let originprotocol_fee_recipient = ProtocolFeeRecipient::<T>::get();

        // Determine maker/taker and charge fees accordingly.
        // Sell-side order is maker.

        // Assert taker fee is less than or equal to maximum fee specified by buyer.
        ensure!(
            sell.taker_relayer_fee <= buy.taker_relayer_fee,
            Error::<T>::SellTakerRelayerFeeGreaterThanBuyTakerRelayerFee
        );

        if sell.fee_method == FeeMethod::SplitFee {
            // Assert taker fee is less than or equal to maximum fee specified by buyer.
            ensure!(
                sell.taker_protocol_fee <= buy.taker_protocol_fee,
                Error::<T>::SellTakerProtocolFeeGreaterThanBuyTakerProtocolFee
            );

            // Maker fees are deducted from the token amount that the maker receives. Taker fees are extra tokens that must be paid by the taker.

            Self::transfer_tokens_fee_sell(
                sell.payment_token(),
                sell.maker(),
                &sell.fee_recipient,
                sell.maker_relayer_fee,
                price,
                receive_amount,
                true,
            )?;

            Self::transfer_tokens_fee_sell(
                sell.payment_token(),
                buy.maker(),
                &sell.fee_recipient,
                sell.taker_relayer_fee,
                price,
                required_amount,
                false,
            )?;

            Self::transfer_tokens_fee_sell(
                sell.payment_token(),
                sell.maker(),
                &originprotocol_fee_recipient,
                sell.maker_protocol_fee,
                price,
                receive_amount,
                true,
            )?;

            Self::transfer_tokens_fee_sell(
                sell.payment_token(),
                buy.maker(),
                &originprotocol_fee_recipient,
                sell.taker_protocol_fee,
                price,
                required_amount,
                false,
            )?;
        } else {
            // Charge maker fee to seller.
            Self::charge_protocol_fee(&sell.maker, &sell.fee_recipient, sell.maker_relayer_fee)?;

            // Charge taker fee to buyer.
            Self::charge_protocol_fee(&buy.maker, &sell.fee_recipient, sell.taker_relayer_fee)?;
        }

        // This contract should never hold Ether, however, we cannot assert this, since it is impossible to prevent anyone from sending Ether e.g. with selfdestruct.

        Ok(*price)
    }

    //
    //#dev Execute all ERC20 token / Ether transfers associated with an order match (fees and buyer => transfer:seller)
    //#param buy Buy-side order
    //#param sell Sell-side order
    //
    pub fn execute_funds_transfer_buy_side(
        buy: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
        sell: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
        price: &BalanceOf<T>,
    ) -> Result<BalanceOf<T>, Error<T>> {
        let originprotocol_fee_recipient = ProtocolFeeRecipient::<T>::get();

        // Determine maker/taker and charge fees accordingly.
        // Buy-side order is maker.

        // Assert taker fee is less than or equal to maximum fee specified by seller.
        ensure!(
            buy.taker_relayer_fee <= sell.taker_relayer_fee,
            Error::<T>::BuyTakerRelayerFeeGreaterThanSellTakerRelayerFee
        );

        if sell.fee_method == FeeMethod::SplitFee {
            // The Exchange does not escrow Ether, so direct Ether can only be used to with sell-side maker / buy-side taker orders.
            ensure!(
                sell.payment_token != ContractSelf::<T>::get(),
                Error::<T>::SellPaymentTokenEqualPaymentToken
            );

            // Assert taker fee is less than or equal to maximum fee specified by seller.
            ensure!(
                buy.taker_protocol_fee <= sell.taker_protocol_fee,
                Error::<T>::BuyTakerProtocolFeeGreaterThanSellTakerProtocolFee
            );

            Self::transfer_tokens_fee(
                sell.payment_token(),
                buy.maker(),
                &buy.fee_recipient,
                buy.maker_relayer_fee,
                price,
            )?;

            Self::transfer_tokens_fee(
                sell.payment_token(),
                sell.maker(),
                &buy.fee_recipient,
                buy.taker_relayer_fee,
                price,
            )?;

            Self::transfer_tokens_fee(
                sell.payment_token(),
                buy.maker(),
                &originprotocol_fee_recipient,
                buy.maker_protocol_fee,
                price,
            )?;

            Self::transfer_tokens_fee(
                &sell.payment_token,
                &sell.maker,
                &originprotocol_fee_recipient,
                buy.taker_protocol_fee,
                price,
            )?;
        } else {
            // Charge maker fee to buyer.
            Self::charge_protocol_fee(&buy.maker, &buy.fee_recipient, buy.maker_relayer_fee)?;

            // Charge taker fee to seller.
            Self::charge_protocol_fee(&sell.maker, &buy.fee_recipient, buy.taker_relayer_fee)?;
        }

        // This contract should never hold Ether, however, we cannot assert this, since it is impossible to prevent anyone from sending Ether e.g. with selfdestruct.

        Ok(*price)
    }

    //
    //#dev Return whether or not two orders can be matched with each other by basic parameters (does not check order signatures / calldata or perform calls:static)
    //#param buy Buy-side order
    //#param sell Sell-side order
    //#return Whether or not the two orders can be matched
    //
    pub fn orders_can_match(
        buy: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
        sell: &OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
    ) -> Result<bool, Error<T>> {
        //  Must be opposite-side.
        Ok((buy.side == Side::Buy && sell.side == Side::Sell) &&
            // Must use same fee method.
            (buy.fee_method == sell.fee_method) &&
            // Must use same payment token. 
            (buy.payment_token == sell.payment_token) &&
            // Must match maker/taker addresses. 
            (sell.taker == ContractSelf::<T>::get() || sell.taker == buy.maker) &&
            (buy.taker == ContractSelf::<T>::get() || buy.taker == sell.maker) &&
            // One must be maker and the other must be taker (no bool XOR Solidity:in). 
            ((sell.fee_recipient == ContractSelf::<T>::get() && buy.fee_recipient != ContractSelf::<T>::get()) || (sell.fee_recipient != ContractSelf::<T>::get() && buy.fee_recipient == ContractSelf::<T>::get())) &&
            // Must match target. 
            (buy.target == sell.target) &&
            // Must match how_to_call. 
            (buy.how_to_call == sell.how_to_call) &&
            // Buy-side order must be settleable. 
            Self::can_settle_order(buy.listing_time, buy.expiration_time)? &&
            // Sell-side order must be settleable. 
            Self::can_settle_order(sell.listing_time, sell.expiration_time)?)
    }

    //
    //#dev Atomically match two orders, ensuring validity of the match, and execute all associated state transitions. Protected against reentrancy by a contract-global lock.
    //#param buy Buy-side order
    //#param buy_sig Buy-side order signature
    //#param sell Sell-side order
    //#param sell_sig Sell-side order signature
    //
    pub fn atomic_match(
        msg_sender: T::AccountId,
        msg_value: BalanceOf<T>,
        buy: OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
        buy_sig: Signature,
        sell: OrderType<T::AccountId, T::Moment, BalanceOf<T>>,
        sell_sig: Signature,
        metadata: &[u8],
    ) -> Result<(), Error<T>> {
        //reentrancyGuard
        // CHECKS

        // Ensure buy order validity and calculate hash if necessary.
        let mut buy_hash: Vec<u8> = vec![];
        if buy.maker == msg_sender {
            ensure!(
                Self::validate_order_parameters(&buy)?,
                Error::<T>::OrderIdTooLong
            );
        } else {
            buy_hash = Self::require_valid_order(&buy, &buy_sig)?;
        }

        // Ensure sell order validity and calculate hash if necessary.
        let mut sell_hash: Vec<u8> = vec![];
        if sell.maker == msg_sender {
            ensure!(
                Self::validate_order_parameters(&sell)?,
                Error::<T>::OrderIdExists
            );
        } else {
            sell_hash = Self::require_valid_order(&sell, &sell_sig)?;
        }

        // Must be matchable.
        ensure!(
            Self::orders_can_match(&buy, &sell)?,
            Error::<T>::OrdersCannotMatch
        );

        // Target must exist (prevent malicious selfdestructs just prior to settlement:order).
        // BalanceOf<T> size;
        // AccountId target = sell.target;
        // assembly {
        //     size := extcodesize(target)
        // }
        // ensure!(size > 0, Error::<T>::OrderIdMissing);

        // Must match calldata after replacement, if specified.
        let mut buycalldata = buy.calldata.clone();
        let mut sellcalldata = sell.calldata.clone();
        if buy.replacement_pattern.len() > 0 {
            Self::guarded_array_replace(
                &mut buycalldata,
                &sell.calldata,
                &buy.replacement_pattern,
            )?;
        }
        if sell.replacement_pattern.len() > 0 {
            Self::guarded_array_replace(
                &mut sellcalldata,
                &buy.calldata,
                &sell.replacement_pattern,
            )?;
        }
        ensure!(
            Self::array_eq(&buycalldata, &sellcalldata)?,
            Error::<T>::OrderInvalidFieldName
        );

        // // Retrieve delegateProxy contract.
        // OwnableDelegateProxy delegateProxy = Registry.proxies(sell.maker);

        // // Proxy must exist.
        // ensure!(delegateProxy != ContractSelf::<T>::get(), Error::<T>::OrderIdMissing);

        // // Assert implementation.
        // ensure!(delegateProxy.implementation() == Registry.delegateProxyImplementation(), Error::<T>::OrderIdMissing);

        // // Access the passthrough AuthenticatedProxy.
        // AuthenticatedProxy proxy = AuthenticatedProxy(delegateProxy);

        // EFFECTS

        // Mark previously signed or approved orders as finalized.
        let buymaker: T::AccountId = buy.maker.clone();
        if msg_sender != buymaker {
            CancelledOrFinalized::insert(buy_hash.clone(), true);
        }
        let sellmaker: T::AccountId = sell.maker.clone();
        if msg_sender != sellmaker {
            CancelledOrFinalized::insert(sell_hash.clone(), true);
        }

        debug::info!(
            "[product_tracking_ocw] Error reading product_tracking_ocw::last_proccessed_block."
        );

        // INTERACTIONS

        // Execute funds transfer and pay fees.
        let price: BalanceOf<T> = Self::execute_funds_transfer(msg_value, &buy, &sell)?;

        // Execute specified call through proxy.
        //TODO
        // ensure!(
        //     proxy.proxy(sell.target, sell.how_to_call, sell.calldata),
        //     Error::<T>::OrderIdMissing
        // );

        // Static calls are intentionally done after the effectful call so they can check resulting state.

        // Handle buy-side static call if specified.
        // if buy.static_target != ContractSelf::<T>::get() {
        //     ensure!(Self::staticCall(buy.static_target, sell.calldata, buy.static_extradata), Error::<T>::OrderIdMissing);
        // }

        // // Handle sell-side static call if specified.
        // if sell.static_target != ContractSelf::<T>::get() {
        //     ensure!(Self::staticCall(sell.static_target, sell.calldata, sell.static_extradata), Error::<T>::OrderIdMissing);
        // }

        // Log match event.
        Self::deposit_event(RawEvent::OrdersMatched(
            buy_hash.clone(),
            sell_hash.clone(),
            if sell.fee_recipient != ContractSelf::<T>::get() {
                sell.maker.clone()
            } else {
                buy.maker.clone()
            },
            if sell.fee_recipient != ContractSelf::<T>::get() {
                buy.maker.clone()
            } else {
                sell.maker.clone()
            },
            price,
            metadata.to_vec(),
        ));

        Ok(())
    }

    // sale Kind interface
    //
    //#dev Check whether the parameters of a sale are valid
    //#param sale_kind Kind of sale
    //#param expiration_time OrderType expiration time
    //#return Whether the parameters were valid
    //
    pub fn validate_parameters(
        sale_kind: &SaleKind,
        expiration_time: T::Moment,
    ) -> Result<bool, Error<T>> {
        // Auctions must have a set expiration date.
        Ok(*sale_kind == SaleKind::FixedPrice || expiration_time > Zero::zero())
    }

    //
    //#dev Return whether or not an order can be settled
    //#dev Precondition: parameters have passed validate_parameters
    //#param listing_time OrderType listing time
    //#param expiration_time OrderType expiration time
    //
    pub fn can_settle_order(
        listing_time: T::Moment,
        expiration_time: T::Moment,
    ) -> Result<bool, Error<T>> {
        let now: T::Moment = T::Moment::from(3); //<timestamp::Module<T>>::now();//<system::Module<T>>::block_number() ;////<timestamp::Module<T>>::now();
        ensure!(
            (listing_time < now) && (expiration_time == Zero::zero() || now < expiration_time),
            Error::<T>::OrdersCannotMatch1
        );
        Ok((listing_time < now) && (expiration_time == Zero::zero() || now < expiration_time))
    }

    //
    //#dev Calculate the settlement price of an order
    //#dev Precondition: parameters have passed validate_parameters.
    //#param side OrderType side
    //#param sale_kind Method of sale
    //#param base_price OrderType base price
    //#param extra OrderType extra price data
    //#param listing_time OrderType listing time
    //#param expiration_time OrderType expiration time
    //
    pub fn calculate_final_price(
        side: &Side,
        sale_kind: &SaleKind,
        base_price: BalanceOf<T>,
        extra: T::Moment,
        listing_time: T::Moment,
        expiration_time: T::Moment,
    ) -> Result<BalanceOf<T>, Error<T>> {
        if *sale_kind == SaleKind::FixedPrice {
            Ok(base_price)
        } else if *sale_kind == SaleKind::DutchAuction {
            let now: T::Moment = Zero::zero(); // <system::Module<T>>::block_number();//<timestamp::Module<T>>::now() ;
            let diff: T::Moment = extra * (now - listing_time) / (expiration_time - listing_time);
            if *side == Side::Sell {
                // Sell-side - start price: base_price. End price: base_price - extra.
                Ok(base_price - Self::moment_to_balance(&diff))
            } else {
                // Buy-side - start price: base_price. End price: base_price + extra.
                Ok(base_price - Self::moment_to_balance(&diff))
            }
        } else {
            Ok(Zero::zero())
        }
    }

    //
    //Replace Vec<u8> in an array with Vec<u8> in another array, guarded by a bitmask
    //Efficiency of this fn is a bit unpredictable because of the EVM's word-specific model (arrays under 32 Vec<u8> will be slower)
    //#dev Mask must be the size of the byte array. A nonzero byte means the byte array can be changed.
    //#param array The original array
    //#param desired The target array
    //#param mask The mask specifying which bits can be changed
    //#return The updated byte array (the parameter will be modified inplace)
    //
    pub fn guarded_array_replace(
        array: &mut Vec<u8>,
        desired: &[u8],
        mask: &[u8],
    ) -> Result<bool, Error<T>> {
        ensure!(
            array.len() == desired.len(),
            Error::<T>::ArraySizeNotAsSameAsDesired
        );
        ensure!(
            array.len() == mask.len(),
            Error::<T>::ArraySizeNotAsSameAsMask
        );
        let arr = array.clone();
        for (i, &_item) in arr.iter().enumerate() {
            // Conceptually: array[i] = (!mask[i] && array[i]) || (mask[i] && desired[i]), bitwise in word chunks.
            array[i] = (!mask[i] & _item) | (mask[i] & desired[i]);
        }
        Ok(true)
    }

    //
    //Test if two arrays are equal
    //Source: https://github.com/GNSPS/solidity-Vec<u8>-utils/blob/master/contracts/BytesLib.sol
    //#dev Arrays must be of equal length, otherwise will return false
    //#param a First array
    //#param b Second array
    //#return Whether or not all Vec<u8> in the arrays are equal
    //
    pub fn array_eq(a: &[u8], b: &[u8]) -> Result<bool, Error<T>> {
        if a.len() != b.len() {
            return Ok(false);
        }

        Ok(a == b)
    }

    pub fn build_order_type_arr(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: &[u8],
        replacement_pattern: &[u8],
        static_extradata: &[u8],
    ) -> OrderType<T::AccountId, T::Moment, BalanceOf<T>> {
        Self::build_order_type(
            addrs[0].clone(),
            addrs[1].clone(),
            addrs[2].clone(),
            Self::u64_to_balance_saturated(uints[0]),
            Self::u64_to_balance_saturated(uints[1]),
            Self::u64_to_balance_saturated(uints[2]),
            Self::u64_to_balance_saturated(uints[3]),
            addrs[3].clone(),
            fee_method,
            side,
            sale_kind,
            addrs[4].clone(),
            how_to_call,
            calldata.to_vec(),
            replacement_pattern.to_vec(),
            addrs[5].clone(),
            static_extradata.to_vec(),
            addrs[6].clone(),
            Self::u64_to_balance_saturated(uints[4]),
            Self::u64_to_moment_saturated(uints[5]),
            Self::u64_to_moment_saturated(uints[6]),
            Self::u64_to_moment_saturated(uints[7]),
            uints[8],
        )
    }

    pub fn build_order_type_arr2(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        fee_methods_sides_kinds_how_to_calls: &[u8],
        calldata_buy: &[u8],
        calldata_sell: &[u8],
        replacement_pattern_buy: &[u8],
        replacement_pattern_sell: &[u8],
        static_extradata_buy: &[u8],
        static_extradata_sell: &[u8],
    ) -> Vec<OrderType<T::AccountId, T::Moment, BalanceOf<T>>> {
        let buy: OrderType<T::AccountId, T::Moment, BalanceOf<T>> = Self::build_order_type(
            addrs[0].clone(),
            addrs[1].clone(),
            addrs[2].clone(),
            Self::u64_to_balance_saturated(uints[0]),
            Self::u64_to_balance_saturated(uints[1]),
            Self::u64_to_balance_saturated(uints[2]),
            Self::u64_to_balance_saturated(uints[3]),
            addrs[3].clone(),
            FeeMethod::from(fee_methods_sides_kinds_how_to_calls[0]),
            Side::from(fee_methods_sides_kinds_how_to_calls[1]),
            SaleKind::from(fee_methods_sides_kinds_how_to_calls[2]),
            addrs[4].clone(),
            HowToCall::from(fee_methods_sides_kinds_how_to_calls[3]),
            calldata_buy.to_vec(),
            replacement_pattern_buy.to_vec(),
            addrs[5].clone(),
            static_extradata_buy.to_vec(),
            addrs[6].clone(),
            Self::u64_to_balance_saturated(uints[4]),
            Self::u64_to_moment_saturated(uints[5]),
            Self::u64_to_moment_saturated(uints[6]),
            Self::u64_to_moment_saturated(uints[7]),
            uints[8],
        );
        let sell: OrderType<T::AccountId, T::Moment, BalanceOf<T>> = Self::build_order_type(
            addrs[7].clone(),
            addrs[8].clone(),
            addrs[9].clone(),
            Self::u64_to_balance_saturated(uints[9]),
            Self::u64_to_balance_saturated(uints[10]),
            Self::u64_to_balance_saturated(uints[11]),
            Self::u64_to_balance_saturated(uints[12]),
            addrs[10].clone(),
            FeeMethod::from(fee_methods_sides_kinds_how_to_calls[4]),
            Side::from(fee_methods_sides_kinds_how_to_calls[5]),
            SaleKind::from(fee_methods_sides_kinds_how_to_calls[6]),
            addrs[11].clone(),
            HowToCall::from(fee_methods_sides_kinds_how_to_calls[7]),
            calldata_sell.to_vec(),
            replacement_pattern_sell.to_vec(),
            addrs[12].clone(),
            static_extradata_sell.to_vec(),
            addrs[13].clone(),
            Self::u64_to_balance_saturated(uints[13]),
            Self::u64_to_moment_saturated(uints[14]),
            Self::u64_to_moment_saturated(uints[15]),
            Self::u64_to_moment_saturated(uints[16]),
            uints[17].into(),
        );
        vec![buy, sell]
    }
    pub fn build_order_type(
        exchange: T::AccountId,
        // OrderType maker AccountId.
        maker: T::AccountId,
        // OrderType taker AccountId, if specified.
        taker: T::AccountId,
        // Maker relayer fee of the order, unused for taker order.
        maker_relayer_fee: BalanceOf<T>,
        // Taker relayer fee of the order, or maximum taker fee for a taker order.
        taker_relayer_fee: BalanceOf<T>,
        // Maker protocol fee of the order, unused for taker order.
        maker_protocol_fee: BalanceOf<T>,
        // Taker protocol fee of the order, or maximum taker fee for a taker order.
        taker_protocol_fee: BalanceOf<T>,
        // OrderType fee recipient or zero AccountId for taker order.
        fee_recipient: T::AccountId,
        // Fee method (protocol token or split fee).
        fee_method: FeeMethod,
        // Side (buy/sell).
        side: Side,
        // Kind of sale.
        sale_kind: SaleKind,
        // Target.
        target: T::AccountId,
        // Vec<u8>.
        how_to_call: HowToCall,
        // Calldata.
        calldata: Bytes,
        // Calldata replacement pattern, or an empty byte array for no replacement.
        replacement_pattern: Bytes,
        // Static call target, zero-AccountId for no static call.
        static_target: T::AccountId,
        // Static call extra data.
        static_extradata: Bytes,
        // Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether.
        payment_token: T::AccountId,
        // Base price of the order (in paymentTokens).
        base_price: BalanceOf<T>,
        // Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference.
        extra: T::Moment,
        // Listing timestamp.
        listing_time: T::Moment,
        // Expiration timestamp - 0 for no expiry.
        expiration_time: T::Moment,
        // OrderType salt, used to prevent duplicate hashes.
        salt: u64,
    ) -> OrderType<T::AccountId, T::Moment, BalanceOf<T>> {
        OrderType::<T::AccountId, T::Moment, BalanceOf<T>>::new(
            exchange,
            // OrderType maker AccountId.
            maker,
            // OrderType taker AccountId, if specified.
            taker,
            // Maker relayer fee of the order, unused for taker order.
            maker_relayer_fee,
            // Taker relayer fee of the order, or maximum taker fee for a taker order.
            taker_relayer_fee,
            // Maker protocol fee of the order, unused for taker order.
            maker_protocol_fee,
            // Taker protocol fee of the order, or maximum taker fee for a taker order.
            taker_protocol_fee,
            // OrderType fee recipient or zero AccountId for taker order.
            fee_recipient,
            // Fee method (protocol token or split fee).
            fee_method,
            // Side (buy/sell).
            side,
            // Kind of sale.
            sale_kind,
            // Target.
            target,
            // Vec<u8>.
            how_to_call,
            // Calldata.
            calldata,
            // Calldata replacement pattern, or an empty byte array for no replacement.
            replacement_pattern,
            // Static call target, zero-AccountId for no static call.
            static_target,
            // Static call extra data.
            static_extradata,
            // Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether.
            payment_token,
            // Base price of the order (in paymentTokens).
            base_price,
            // Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference.
            extra,
            // Listing timestamp.
            listing_time,
            // Expiration timestamp - 0 for no expiry.
            expiration_time,
            // OrderType salt, used to prevent duplicate hashes.
            salt,
        )
    }

    pub fn u32_to_balance(_input: BalanceOf<T>) {
        let my_u32: u32 = 0;
        let _my_balance: BalanceOf<T> = my_u32.into();
        let _my_balance1: BalanceOf<T> = my_u32.into();
        let _a = _my_balance - _my_balance1;
        let _s = _my_balance * _my_balance1;
        // let _my:u32 = _my_balance1.try_into<u32>();
        let _mm: T::Moment = T::Moment::from(3); //T::Moment::get();
        let _mm1: T::Moment = T::Moment::from(3); //T::Moment::get();
        let _mm: T::Moment = _mm + _mm1; //T::Moment::get();
        let _mm: T::Moment = Zero::zero(); //T::Moment::get();
        let _my_balance: BalanceOf<T> = Zero::zero();
        // let _m :u32 = _mm.try_into();
    }

    // pub fn u64_to_balance(_input: u64) -> Option<BalanceOf<T>> {
    // let my_u32:u32 = _input as u32;
    //  Some(my_u32.into())
    // //    BalanceOf::<T>::try_from(_input as u32)//.try_into().ok()
    // }

    pub fn u64_to_balance_saturated(_input: u64) -> BalanceOf<T> {
        // let my_u32:u32 = _input as u32;
        //  my_u32.into()
        BalanceOf::<T>::saturated_from(_input.into()) //.saturated_into()
    }

    pub fn u64_to_moment_saturated(_input: u64) -> T::Moment {
        // let my_u32:u32 = _input as u32;
        //  my_u32.into()
        T::Moment::saturated_from(_input.into()) //.saturated_into()
    }

    pub fn u64_to_balance_option(_input: u64) -> Option<BalanceOf<T>> {
        // use sp_std::convert::{TryFrom, TryInto};
        _input.try_into().ok()
        // Some(Zero::zero())
    }

    // Note the warning above about saturated conversions
    // pub fn Self::u64_to_balance_saturated(input: u64) -> BalanceOf<T> {
    //     input.saturated_into()
    // }

    pub fn balance_to_u64(input: BalanceOf<T>) -> Option<u64> {
        // use sp_std::convert::{TryFrom, TryInto};
        //         TryInto::<u64>::try_into(input).ok()

        Some(input.saturated_into::<u64>())
    }
    pub fn moment_to_u64(input: T::Moment) -> Option<u64> {
        // use sp_std::convert::{TryFrom, TryInto};
        //         TryInto::<u64>::try_into(input).ok()
        Some(input.saturated_into::<u64>())
    }

    // Note the warning above about saturated conversions
    // pub fn balance_to_u64_saturated(input: T::Balance) -> u64 {
    //     input.saturated_into::<u64>()
    // }
    //

    pub fn moment_to_balance(m: &T::Moment) -> BalanceOf<T> {
        let mut _b: BalanceOf<T> = Zero::zero();
        if let Some(m) = Self::moment_to_u64(*m) {
            if let Some(bo) = Self::u64_to_balance_option(m) {
                _b = bo;
            }
        }

        _b
    }
}

impl<AccountId, Moment, Balance> OrderType<AccountId, Moment, Balance>
where
    AccountId: Default,
    Moment: Default,
    Balance: Default,
{
    pub fn new(
        exchange: AccountId,
        // OrderType maker AccountId.
        maker: AccountId,
        // OrderType taker AccountId, if specified.
        taker: AccountId,
        // Maker relayer fee of the order, unused for taker order.
        maker_relayer_fee: Balance,
        // Taker relayer fee of the order, or maximum taker fee for a taker order.
        taker_relayer_fee: Balance,
        // Maker protocol fee of the order, unused for taker order.
        maker_protocol_fee: Balance,
        // Taker protocol fee of the order, or maximum taker fee for a taker order.
        taker_protocol_fee: Balance,
        // OrderType fee recipient or zero AccountId for taker order.
        fee_recipient: AccountId,
        // Fee method (protocol token or split fee).
        fee_method: FeeMethod,
        // Side (buy/sell).
        side: Side,
        // Kind of sale.
        sale_kind: SaleKind,
        // Target.
        target: AccountId,
        // Vec<u8>.
        how_to_call: HowToCall,
        // Calldata.
        calldata: Bytes,
        // Calldata replacement pattern, or an empty byte array for no replacement.
        replacement_pattern: Bytes,
        // Static call target, zero-AccountId for no static call.
        static_target: AccountId,
        // Static call extra data.
        static_extradata: Bytes,
        // Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether.
        payment_token: AccountId,
        // Base price of the order (in paymentTokens).
        base_price: Balance,
        // Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference.
        extra: Moment,
        // Listing timestamp.
        listing_time: Moment,
        // Expiration timestamp - 0 for no expiry.
        expiration_time: Moment,
        // OrderType salt, used to prevent duplicate hashes.
        salt: u64,
    ) -> Self {
        Self {
            index: 0,
            exchange: exchange,
            // OrderType maker AccountId.
            maker: maker,
            // OrderType taker AccountId, if specified.
            taker: taker,
            // Maker relayer fee of the order, unused for taker order.
            maker_relayer_fee: maker_relayer_fee,
            // Taker relayer fee of the order, or maximum taker fee for a taker order.
            taker_relayer_fee: taker_relayer_fee,
            // Maker protocol fee of the order, unused for taker order.
            maker_protocol_fee: maker_protocol_fee,
            // Taker protocol fee of the order, or maximum taker fee for a taker order.
            taker_protocol_fee: taker_protocol_fee,
            // OrderType fee recipient or zero AccountId for taker order.
            fee_recipient: fee_recipient,
            // Fee method (protocol token or split fee).
            fee_method: fee_method,
            // Side (buy/sell).
            side: side,
            // Kind of sale.
            sale_kind: sale_kind,
            // Target.
            target: target,
            // Vec<u8>.
            how_to_call: how_to_call,
            // Calldata.
            calldata: calldata,
            // Calldata replacement pattern, or an empty byte array for no replacement.
            replacement_pattern: replacement_pattern,
            // Static call target, zero-AccountId for no static call.
            static_target: static_target,
            // Static call extra data.
            static_extradata: static_extradata,
            // Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether.
            payment_token: payment_token,
            // Base price of the order (in paymentTokens).
            base_price: base_price,
            // Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference.
            extra: extra,
            // Listing timestamp.
            listing_time: listing_time,
            // Expiration timestamp - 0 for no expiry.
            expiration_time: expiration_time,
            // OrderType salt, used to prevent duplicate hashes.
            salt: salt,
            registered: Moment::default(),
        }
    }

    pub fn maker(&self) -> &AccountId {
        &self.maker
    }

    pub fn taker(&self) -> &AccountId {
        &self.taker
    }

    pub fn payment_token(&self) -> &AccountId {
        &self.payment_token
    }
}

#[derive(Default)]
pub struct OrderTypeBuilder<AccountId, Moment, Balance>
where
    AccountId: Default,
    Moment: Default,
    Balance: Default,
{
    pub index: u64,

    // // An order on the exchange.
    // Exchange AccountId, intended as a versioning mechanism.
    pub exchange: AccountId,
    // OrderType maker AccountId.
    pub maker: AccountId,
    // OrderType taker AccountId, if specified.
    pub taker: AccountId,
    // Maker relayer fee of the order, unused for taker order.
    pub maker_relayer_fee: Balance,
    // Taker relayer fee of the order, or maximum taker fee for a taker order.
    pub taker_relayer_fee: Balance,
    // Maker protocol fee of the order, unused for taker order.
    pub maker_protocol_fee: Balance,
    // Taker protocol fee of the order, or maximum taker fee for a taker order.
    pub taker_protocol_fee: Balance,
    // OrderType fee recipient or zero AccountId for taker order.
    pub fee_recipient: AccountId,
    // Fee method (protocol token or split fee).
    pub fee_method: FeeMethod,
    // Side (buy/sell).
    pub side: Side,
    // Kind of sale.
    pub sale_kind: SaleKind,
    // Target.
    pub target: AccountId,
    // Vec<u8>.
    pub how_to_call: HowToCall,
    // Calldata.
    pub calldata: Bytes,
    // Calldata replacement pattern, or an empty byte array for no replacement.
    pub replacement_pattern: Bytes,
    // Static call target, zero-AccountId for no static call.
    pub static_target: AccountId,
    // Static call extra data.
    pub static_extradata: Bytes,
    // Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether.
    pub payment_token: AccountId,
    // Base price of the order (in paymentTokens).
    pub base_price: Balance,
    // Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference.
    pub extra: Moment,
    // Listing timestamp.
    pub listing_time: Moment,
    // Expiration timestamp - 0 for no expiry.
    pub expiration_time: Moment,
    // OrderType salt, used to prevent duplicate hashes.
    pub salt: u64,
    pub registered: Moment,
}

impl<AccountId, Moment, Balance> OrderTypeBuilder<AccountId, Moment, Balance>
where
    AccountId: Default,
    Moment: Default,
    Balance: Default,
{
    pub fn new(
        exchange: AccountId,
        // OrderType maker AccountId.
        maker: AccountId,
        // OrderType taker AccountId, if specified.
        taker: AccountId,
        // Maker relayer fee of the order, unused for taker order.
        maker_relayer_fee: Balance,
        // Taker relayer fee of the order, or maximum taker fee for a taker order.
        taker_relayer_fee: Balance,
        // Maker protocol fee of the order, unused for taker order.
        maker_protocol_fee: Balance,
        // Taker protocol fee of the order, or maximum taker fee for a taker order.
        taker_protocol_fee: Balance,
        // OrderType fee recipient or zero AccountId for taker order.
        fee_recipient: AccountId,
        // Fee method (protocol token or split fee).
        fee_method: FeeMethod,
        // Side (buy/sell).
        side: Side,
        // Kind of sale.
        sale_kind: SaleKind,
        // Target.
        target: AccountId,
        // Vec<u8>.
        how_to_call: HowToCall,
        // Calldata.
        calldata: Bytes,
        // Calldata replacement pattern, or an empty byte array for no replacement.
        replacement_pattern: Bytes,
        // Static call target, zero-AccountId for no static call.
        static_target: AccountId,
        // Static call extra data.
        static_extradata: Bytes,
        // Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether.
        payment_token: AccountId,
        // Base price of the order (in paymentTokens).
        base_price: Balance,
        // Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference.
        extra: Moment,
        // Listing timestamp.
        listing_time: Moment,
        // Expiration timestamp - 0 for no expiry.
        expiration_time: Moment,
        // OrderType salt, used to prevent duplicate hashes.
        salt: u64,
    ) -> Self {
        Self {
            index: 0,
            exchange: exchange,
            // OrderType maker AccountId.
            maker: maker,
            // OrderType taker AccountId, if specified.
            taker: taker,
            // Maker relayer fee of the order, unused for taker order.
            maker_relayer_fee: maker_relayer_fee,
            // Taker relayer fee of the order, or maximum taker fee for a taker order.
            taker_relayer_fee: taker_relayer_fee,
            // Maker protocol fee of the order, unused for taker order.
            maker_protocol_fee: maker_protocol_fee,
            // Taker protocol fee of the order, or maximum taker fee for a taker order.
            taker_protocol_fee: taker_protocol_fee,
            // OrderType fee recipient or zero AccountId for taker order.
            fee_recipient: fee_recipient,
            // Fee method (protocol token or split fee).
            fee_method: fee_method,
            // Side (buy/sell).
            side: side,
            // Kind of sale.
            sale_kind: sale_kind,
            // Target.
            target: target,
            // Vec<u8>.
            how_to_call: how_to_call,
            // Calldata.
            calldata: calldata,
            // Calldata replacement pattern, or an empty byte array for no replacement.
            replacement_pattern: replacement_pattern,
            // Static call target, zero-AccountId for no static call.
            static_target: static_target,
            // Static call extra data.
            static_extradata: static_extradata,
            // Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether.
            payment_token: payment_token,
            // Base price of the order (in paymentTokens).
            base_price: base_price,
            // Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference.
            extra: extra,
            // Listing timestamp.
            listing_time: listing_time,
            // Expiration timestamp - 0 for no expiry.
            expiration_time: expiration_time,
            // OrderType salt, used to prevent duplicate hashes.
            salt: salt,
            registered: Moment::default(),
        }
    }

    // pub fn index_by(mut self, index: BalanceOf<T>) -> Self {
    //     self.index = index;
    //     self
    // }

    // pub fn identified_by(mut self, id: OrderId) -> Self {
    //     self.id = id;
    //     self
    // }

    // pub fn owned_by(mut self, owner: AccountId) -> Self {
    //     self.owner = owner;
    //     self
    // }

    // pub fn with_fields(mut self, fields: Option<Vec<OrderField>>) -> Self {
    //     self.fields = fields;
    //     self
    // }

    // pub fn registered_on(mut self, registered: Moment) -> Self {
    //     self.registered = registered;
    //     self
    // }

    pub fn build(self) -> OrderType<AccountId, Moment, Balance> {
        OrderType::<AccountId, Moment, Balance> {
            index: self.index,
            exchange: self.exchange,
            // OrderType maker AccountId.
            maker: self.maker,
            // OrderType taker AccountId, if specified.
            taker: self.taker,
            // Maker relayer fee of the order, unused for taker order.
            maker_relayer_fee: self.maker_relayer_fee,
            // Taker relayer fee of the order, or maximum taker fee for a taker order.
            taker_relayer_fee: self.taker_relayer_fee,
            // Maker protocol fee of the order, unused for taker order.
            maker_protocol_fee: self.maker_protocol_fee,
            // Taker protocol fee of the order, or maximum taker fee for a taker order.
            taker_protocol_fee: self.taker_protocol_fee,
            // OrderType fee recipient or zero AccountId for taker order.
            fee_recipient: self.fee_recipient,
            // Fee method (protocol token or split fee).
            fee_method: self.fee_method,
            // Side (buy/sell).
            side: self.side,
            // Kind of sale.
            sale_kind: self.sale_kind,
            // Target.
            target: self.target,
            // Vec<u8>.
            how_to_call: self.how_to_call,
            // Calldata.
            calldata: self.calldata,
            // Calldata replacement pattern, or an empty byte array for no replacement.
            replacement_pattern: self.replacement_pattern,
            // Static call target, zero-AccountId for no static call.
            static_target: self.static_target,
            // Static call extra data.
            static_extradata: self.static_extradata,
            // Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether.
            payment_token: self.payment_token,
            // Base price of the order (in paymentTokens).
            base_price: self.base_price,
            // Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference.
            extra: self.extra,
            // Listing timestamp.
            listing_time: self.listing_time,
            // Expiration timestamp - 0 for no expiry.
            expiration_time: self.expiration_time,
            // OrderType salt, used to prevent duplicate hashes.
            salt: self.salt,
            registered: self.registered,
        }
    }
}
