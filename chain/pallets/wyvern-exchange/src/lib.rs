//! # Substrate Enterprise Sample - OrderType Post example pallet

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::result::Result;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, sp_runtime::RuntimeDebug,sp_runtime::traits::{IdentifyAccount, Member, Verify},
    sp_std::collections::btree_set::BTreeSet, sp_io::*,sp_std::prelude::*
};
// traits::EnsureOrigin,
use frame_system::{self as system, ensure_signed};
use balances::Call as BalancesCall;

use sp_core::H256;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;
//////ETH  BEGIN

//////ETH END
// General constraints to limit data size
// Note: these could also be passed as trait config parameters
pub const ORDER_ID_MAX_LENGTH: usize = 36;
pub const ORDER_FIELD_NAME_MAX_LENGTH: usize = 10;
pub const ORDER_FIELD_VALUE_MAX_LENGTH: usize = 20;
pub const ORDER_MAX_FIELDS: usize = 3;

// /* Inverse basis point. */
pub const INVERSE_BASIS_POINT: usize = 10000;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

// Custom types

pub type OrderId = Vec<u8>;
pub type FieldName = Vec<u8>;
pub type FieldValue = Vec<u8>;

pub type Address = Vec<u8>;
pub type Bytes = Vec<u8>;

///sale kind interface
#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum SaleKind {
    FixedPrice,
    DutchAuction,
}

// /* Fee method: protocol fee or split fee. */
// enum FeeMethod { ProtocolFee, SplitFee }
#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum FeeMethod {
    ProtocolFee,
    SplitFee,
}

#[derive(Encode, Decode, Debug, Clone, PartialEq)]
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
    fn value(&self) -> u8 {
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
            0x1 => return HowToCall::DelegateCall,
        };
    }
}


impl FeeMethod {
    fn value(&self) -> u8 {
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
            0x1 => return FeeMethod::SplitFee,
        };
    }
}



impl SaleKind {
    fn value(&self) -> u8 {
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
            0x1 => return SaleKind::DutchAuction,
        };
    }
}


impl Side {
    fn value(&self) -> u8 {
        match *self {
            Side::Call => 0x0,
            Side::DelegateCall => 0x1,
        }
    }
}

impl From<u8> for Side {
    fn from(orig: u8) -> Self {
        match orig {
            0x0 => return Side::Call,
            0x1 => return Side::DelegateCall,
        };
    }
}


///exchange core begin

// OrderType contains master data (aka class-level) about a trade item.
// This data is typically registered once by the order's manufacturer / supplier,
// to be shared with other network participants, and remains largely static.
// It can also be used for instance-level (lot) master data.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct OrderType<AccountId, Moment> {
    index: u64,
    // // The order ID would typically be a GS1 GTIN (Global Trade Item Number),
    // // or ASIN (Amazon Standard Identification Number), or similar,
    // // a numeric or alpha-numeric code with a well-defined data structure.
    // id: OrderId,
    // // This is account that represents the owner of this order, as in
    // // the manufacturer or supplier providing this order within the value chain.
    // owner: AccountId,
    // // This a series of fields describing the order.
    // // Typically, there would at least be a textual description, and SKU(Stock-keeping unit).
    // // It could also contain instance / lot master data e.g. expiration, weight, harvest date.
    // fields: Option<Vec<OrderField>>,
    // // Timestamp (approximate) at which the product was registered on-chain.
    // registered: Moment,

    // /* An order on the exchange. */
    /* Exchange Address, intended as a versioning mechanism. */
    exchange: Address,
    /* OrderType maker Address. */
    maker: Address,
    /* OrderType taker Address, if specified. */
    taker: Address,
    /* Maker relayer fee of the order, unused for taker order. */
    makerRelayerFee: u64,
    /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
    takerRelayerFee: u64,
    /* Maker protocol fee of the order, unused for taker order. */
    makerProtocolFee: u64,
    /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
    takerProtocolFee: u64,
    /* OrderType fee recipient or zero Address for taker order. */
    feeRecipient: Address,
    /* Fee method (protocol token or split fee). */
    feeMethod: FeeMethod,
    /* Side (buy/sell). */
    side: Side,
    /* Kind of sale. */
    saleKind: SaleKind,
    /* Target. */
    target: Address,
    /* Vec<u8>. */
    howToCall: Vec<u8>,
    /* Calldata. */
    calldata: Bytes,
    /* Calldata replacement pattern, or an empty byte array for no replacement. */
    replacementPattern: Bytes,
    /* Static call target, zero-Address for no static call. */
    staticTarget: Address,
    /* Static call extra data. */
    staticExtradata: Bytes,
    /* Token used to pay for the order, or the zero-Address as a sentinel value for Ether. */
    paymentToken: Address,
    /* Base price of the order (in paymentTokens). */
    basePrice: u64,
    /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
    extra: u64,
    /* Listing timestamp. */
    listingTime: u64,
    /* Expiration timestamp - 0 for no expiry. */
    expirationTime: u64,
    /* OrderType salt, used to prevent duplicate hashes. */
    salt: u64,
}

//exchange core

// Add new types to the trait:

// pub trait Trait: system::Trait {
//     type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
//     type Public: IdentifyAccount<AccountId = Self::AccountId> + Clone;
//     type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
// }

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Public: IdentifyAccount<AccountId = Self::AccountId> + Clone;
    type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
    // type CreateRoleOrigin: EnsureOrigin<Self::Origin>;
}

decl_storage! {
    trait Store for Module<T: Trait> as OrderRegistry {
        NextOrderIndex: u64;
pub contract_self:AccountId;
 // /* The token used to pay exchange fees. */
    // ERC20 public exchangeToken;
pub exchangeToken:AccountId;
    // /* User registry. */
    // ProxyRegistry public registry;
pub registry:AccountId;
    // /* Token transfer proxy. */
    // TokenTransferProxy public tokenTransferProxy;
pub tokenTransferProxy:AccountId;
    // /* Cancelled / finalized orders, by hash. */
    // mapping(Vec<u8> => bool) public cancelledOrFinalized;
  pub cancelledOrFinalized get(fn cancelled_or_finalized): map hasher(blake2_128_concat) Vec<u8> => bool;
    // /* Orders verified by on-chain approval (alternative to ECDSA signatures so that smart contracts can place orders directly). */
    // mapping(Vec<u8> => bool) public approvedOrders;
  pub approvedOrders get(fn approved_orders): map hasher(blake2_128_concat) Vec<u8> => bool;
    // /* For split fee orders, minimum required protocol maker fee, in basis points. Paid to owner (who can change it). */
    // u64 public minimumMakerProtocolFee = 0;
pub minimumMakerProtocolFee:u64;
    // /* For split fee orders, minimum required protocol taker fee, in basis points. Paid to owner (who can change it). */
    // u64 public minimumTakerProtocolFee = 0;
pub minimumTakerProtocolFee:u64;
    // /* Recipient of protocol fees. */
    // Address public protocolFeeRecipient;
pub protocolFeeRecipient:Address;


 }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        // event OrderApprovedPartOne    (Vec<u8> indexed hash, Address exchange, Address indexed maker, Address taker,
        // u64 makerRelayerFee, u64 takerRelayerFee, u64 makerProtocolFee, u64 takerProtocolFee,
        // Address indexed feeRecipient, FeeMethod feeMethod, SaleKindInterface.Side side, SaleKindInterface.SaleKind saleKind, Address target);
        // event OrderApprovedPartTwo    (Vec<u8> indexed hash, AuthenticatedProxy.Vec<u8> howToCall, Vec<u8> calldata, Vec<u8> replacementPattern,
        // Address staticTarget, Vec<u8> staticExtradata, Address paymentToken, u64 basePrice,
        // u64 extra, u64 listingTime, u64 expirationTime, u64 salt, bool orderbookInclusionDesired);
        // event OrderCancelled          (Vec<u8> indexed hash);
        // event OrdersMatched           (Vec<u8> buyHash, Vec<u8> sellHash, Address indexed maker, Address indexed taker, u64 price, Vec<u8> indexed metadata);
      OrderApprovedPartOne(
            Vec<u8>,
            AccountId,
            AccountId,
            AccountId,
            u64,
            u64,
            u64,
            u64,
            AccountId,
            FeeMethod,
            Side,
            SaleKind,
            AccountId,
        ),
      OrderApprovedPartTwo(
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
            Vec<u8>,
            AccountId,
            Vec<u8>,
            AccountId,
            Vec<u8>,
            u64,
            u64,
            u64,
            u64,
            bool,
        ),
      OrderCancelled(Vec<u8>),
      OrdersMatched(Vec<u8>, Vec<u8>, AccountId, AccountId, u64, Vec<u8>),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        OrderIdMissing,
        OrderIdTooLong,
        OrderIdExists,
        OrderTooManyFields,
        OrderInvalidFieldName,
        OrderInvalidFieldValue
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

//         #[weight = 10_000]
//         pub fn post_order(origin, id: OrderId, owner: T::AccountId, fields: Option<Vec<OrderField>>) -> dispatch::DispatchResult {
//             // T::CreateRoleOrigin::ensure_origin(origin.clone())?;
//             let who = ensure_signed(origin)?;

//             // Validate order ID
//             Self::validate_order_id(&id)?;

//             // Validate order fields
//             Self::validate_order_fields(&fields)?;

//             // Check order doesn't exist yet (1 DB read)
//             Self::validate_new_order(&id)?;



//             // TODO: if organization has an attribute w/ GS1 Company prefix,
//             //       additional validation could be applied to the order ID
//             //       to ensure its validity (same company prefix as org).

//             // Generate next collection ID
//             let next_id = NextOrderIndex::get()
//                 .checked_add(1)
//                 .expect("order id error");

//             NextOrderIndex::put(next_id);

// if let Some(fields) = &fields {
//             for field in fields {
//             let mut index_arr: Vec<u64> = Vec::new();

//             if <OrdersOfOrganization>::contains_key(field.name(),field.value())
//             {
//                 index_arr = <OrdersOfOrganization>::get(field.name(),field.value());
//                 ensure!(!index_arr.contains(&next_id), "Account already has admin role");
//             }

//             index_arr.push(next_id);
//             <OrdersOfOrganization>::insert(field.name(),field.value(), index_arr);

//     //   <OrdersOfOrganization<T>>::append(&field, &next_id);
//             }
//    }


//             // Create a order instance
//             let order = Self::new_order()
//                 .identified_by(id.clone())
//                 .owned_by(owner.clone())
//                 .registered_on(<timestamp::Module<T>>::now())
//                 .with_fields(fields)
//                 .build();

//             // Add order & ownerOf (3 DB writes)
//             <Orders<T>>::insert(next_id, order);
//             <Orderi>::insert(&id, next_id);
//             // <OrdersOfOrganization<T>>::append(&owner, &id);


//                <OwnerOf<T>>::insert(&id, &owner);

//             Self::deposit_event(RawEvent::OrderPosted(who, id, owner));

//             Ok(())
//         }
    }
}

impl<T: Trait> Module<T> {
    /// exchange

    /**
     * @dev Call calculateFinalPrice - library fn exposed for testing.
     */
    fn calculateFinalPrice_(
        side: Side,
        saleKind: SaleKind,
        basePrice: u64,
        extra: u64,
        listingTime: u64,
        expirationTime: u64,
    ) -> u64 {
      Self::calculateFinalPrice(
            side,
            saleKind,
            basePrice,
            extra,
            listingTime,
            expirationTime,
        )
    }

    /**
     * @dev Call hashOrder - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn hashOrder_(
        addrs: Vec<Address>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: HowToCall,
        calldata: Vec<u8>,
        replacementPattern: Vec<u8>,
        staticExtradata: Vec<u8>,
    ) -> Vec<u8> {
      Self::hashOrder(Self::buildOrderType(
            addrs[0],
            addrs[1],
            addrs[2],
            uints[0],
            uints[1],
            uints[2],
            uints[3],
            addrs[3],
            feeMethod,
            side,
            saleKind,
            addrs[4],
            howToCall,
            calldata,
            replacementPattern,
            addrs[5],
            staticExtradata,
            addrs[6],
            uints[4],
            uints[5],
            uints[6],
            uints[7],
            uints[8],
        ))
    }

    /**
     * @dev Call hashToSign - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn hashToSign_(
        addrs: Vec<Address>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: HowToCall,
        calldata: Vec<u8>,
        replacementPattern: Vec<u8>,
        staticExtradata: Vec<u8>,
    ) -> Vec<u8> {
      Self::hashToSign(Self::buildOrderType(
            addrs[0],
            addrs[1],
            addrs[2],
            uints[0],
            uints[1],
            uints[2],
            uints[3],
            addrs[3],
            feeMethod,
            side,
            saleKind,
            addrs[4],
            howToCall,
            calldata,
            replacementPattern,
            addrs[5],
            staticExtradata,
            addrs[6],
            uints[4],
            uints[5],
            uints[6],
            uints[7],
            uints[8],
        ))
    }

    /**
     * @dev Call validateOrderParameters - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn validateOrderParameters_(
        addrs: Vec<Address>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: Vec<u8>,
        calldata: Vec<u8>,
        replacementPattern: Vec<u8>,
        staticExtradata: Vec<u8>,
    ) -> bool {
        let order: OrderType = Self::buildOrderType(
            addrs[0],
            addrs[1],
            addrs[2],
            uints[0],
            uints[1],
            uints[2],
            uints[3],
            addrs[3],
            feeMethod,
            side,
            saleKind,
            addrs[4],
            howToCall,
            calldata,
            replacementPattern,
            addrs[5],
            staticExtradata,
            addrs[6],
            uints[4],
            uints[5],
            uints[6],
            uints[7],
            uints[8],
        );
      Self::validateOrderParameters(order)
    }

    /**
     * @dev Call validateOrder - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn validateOrder_(
        addrs: Vec<Address>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: Vec<u8>,
        calldata: Vec<u8>,
        replacementPattern: Vec<u8>,
        staticExtradata: Vec<u8>,
        sig: Signature,
    ) -> bool {
        let order: OrderType = Self::buildOrderType(
            addrs[0],
            addrs[1],
            addrs[2],
            uints[0],
            uints[1],
            uints[2],
            uints[3],
            addrs[3],
            feeMethod,
            side,
            saleKind,
            addrs[4],
            howToCall,
            calldata,
            replacementPattern,
            addrs[5],
            staticExtradata,
            addrs[6],
            uints[4],
            uints[5],
            uints[6],
            uints[7],
            uints[8],
        );
      Self::validateOrder(Self::hashToSign(order), order, sig)
    }

    /**
     * @dev Call approveOrder - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn approveOrder_(origin:AccountId,
        addrs: Vec<Address>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: Vec<u8>,
        calldata: Vec<u8>,
        replacementPattern: Vec<u8>,
        staticExtradata: Vec<u8>,
        orderbookInclusionDesired: bool,
    ) -> Result<(), Error<T>> {
       let  order: OrderType = Self::buildOrderType(
            addrs[0],
            addrs[1],
            addrs[2],
            uints[0],
            uints[1],
            uints[2],
            uints[3],
            addrs[3],
            feeMethod,
            side,
            saleKind,
            addrs[4],
            howToCall,
            calldata,
            replacementPattern,
            addrs[5],
            staticExtradata,
            addrs[6],
            uints[4],
            uints[5],
            uints[6],
            uints[7],
            uints[8],
        );
      Self::approveOrder(origin,order, orderbookInclusionDesired);
        Ok(())
    }

    /**
     * @dev Call cancelOrder - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn cancelOrder_(origin:AccountId,
        addrs: Vec<Address>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: Vec<u8>,
        calldata: Vec<u8>,
        replacementPattern: Vec<u8>,
        staticExtradata: Vec<u8>,
        sig: Signature,
    ) -> Result<(), Error<T>> {
      Self::cancelOrder(origin,
            Self::buildOrderType(
                addrs[0],
                addrs[1],
                addrs[2],
                uints[0],
                uints[1],
                uints[2],
                uints[3],
                addrs[3],
                feeMethod,
                side,
                saleKind,
                addrs[4],
                howToCall,
                calldata,
                replacementPattern,
                addrs[5],
                staticExtradata,
                addrs[6],
                uints[4],
                uints[5],
                uints[6],
                uints[7],
                uints[8],
            ),
            sig,
        )?;
        Ok(())
    }

    /**
     * @dev Call calculateCurrentPrice - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn calculateCurrentPrice_(
        addrs: Vec<Address>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: Vec<u8>,
        calldata: Vec<u8>,
        replacementPattern: Vec<u8>,
        staticExtradata: Vec<u8>,
    ) -> u64 {
      Self::calculateCurrentPrice(Self::buildOrderType(
            addrs[0],
            addrs[1],
            addrs[2],
            uints[0],
            uints[1],
            uints[2],
            uints[3],
            addrs[3],
            feeMethod,
            side,
            saleKind,
            addrs[4],
            howToCall,
            calldata,
            replacementPattern,
            addrs[5],
            staticExtradata,
            addrs[6],
            uints[4],
            uints[5],
            uints[6],
            uints[7],
            uints[8],
        ))
    }

    /**
     * @dev Call ordersCanMatch - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn ordersCanMatch_(
        addrs: Vec<Address>,
        uints: Vec<u64>,
        feeMethodsSidesKindsHowToCalls: Vec<u8>,
        calldataBuy: Vec<u8>,
        calldataSell: Vec<u8>,
        replacementPatternBuy: Vec<u8>,
        replacementPatternSell: Vec<u8>,
        staticExtradataBuy: Vec<u8>,
        staticExtradataSell: Vec<u8>,
    ) -> bool {
        let buy: OrderType<T::AccountId, T::Moment> = Self::buildOrderType(
            addrs[0],
            addrs[1],
            addrs[2],
            uints[0],
            uints[1],
            uints[2],
            uints[3],
            addrs[3],
            FeeMethod::from(feeMethodsSidesKindsHowToCalls[0]),
            Side::from(feeMethodsSidesKindsHowToCalls[1]),
            SaleKind::from(feeMethodsSidesKindsHowToCalls[2]),
            addrs[4],
            HowToCall::from(feeMethodsSidesKindsHowToCalls[3]),
            calldataBuy,
            replacementPatternBuy,
            addrs[5],
            staticExtradataBuy,
            (addrs[6]),
            uints[4],
            uints[5],
            uints[6],
            uints[7],
            uints[8],
        );
       let  sell: OrderType<T::AccountId, T::Moment> = Self::buildOrderType(
            addrs[7],
            addrs[8],
            addrs[9],
            uints[9],
            uints[10],
            uints[11],
            uints[12],
            addrs[10],
            FeeMethod::from(feeMethodsSidesKindsHowToCalls[4]),
            Side::from(feeMethodsSidesKindsHowToCalls[5]),
            SaleKind::from(feeMethodsSidesKindsHowToCalls[6]),
            addrs[11],
            HowToCall::from(feeMethodsSidesKindsHowToCalls[7]),
            calldataSell,
            replacementPatternSell,
            addrs[12],
            staticExtradataSell,
            (addrs[13]),
            uints[13],
            uints[14],
            uints[15],
            uints[16],
            uints[17],
        );
      Self::ordersCanMatch(buy, sell)
    }

    /**
     * @dev Return whether or not two orders' calldata specifications can match
     * @param buyCalldata Buy-side order calldata
     * @param buyReplacementPattern Buy-side order calldata replacement mask
     * @param sellCalldata Sell-side order calldata
     * @param sellReplacementPattern Sell-side order calldata replacement mask
     * @return Whether the orders' calldata can be matched
     */
    fn orderCalldataCanMatch(
        buyCalldata: Vec<u8>,
        buyReplacementPattern: Vec<u8>,
        sellCalldata: Vec<u8>,
        sellReplacementPattern: Vec<u8>,
    ) -> bool {
        if (buyReplacementPattern.length > 0) {
            Self::guardedArrayReplace(buyCalldata, sellCalldata, buyReplacementPattern);
        }
        if (sellReplacementPattern.length > 0) {
            Self::guardedArrayReplace(sellCalldata, buyCalldata, sellReplacementPattern);
        }

        Self::arrayEq(buyCalldata, sellCalldata)
    }

    /**
     * @dev Call calculateMatchPrice - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn calculateMatchPrice_(
        addrs: Vec<Address>,
        uints: Vec<u64>,
        feeMethodsSidesKindsHowToCalls: Vec<u8>,
        calldataBuy: Vec<u8>,
        calldataSell: Vec<u8>,
        replacementPatternBuy: Vec<u8>,
        replacementPatternSell: Vec<u8>,
        staticExtradataBuy: Vec<u8>,
        staticExtradataSell: Vec<u8>,
    ) -> u64 {
       let  buy: OrderType<T::AccountId, T::Moment> = Self::buildOrderType(
            addrs[0],
            addrs[1],
            addrs[2],
            uints[0],
            uints[1],
            uints[2],
            uints[3],
            addrs[3],
            FeeMethod::from(feeMethodsSidesKindsHowToCalls[0]),
            Side::from(feeMethodsSidesKindsHowToCalls[1]),
            SaleKind::from(feeMethodsSidesKindsHowToCalls[2]),
            addrs[4],
            HowToCall::from(feeMethodsSidesKindsHowToCalls[3]),
            calldataBuy,
            replacementPatternBuy,
            addrs[5],
            staticExtradataBuy,
            (addrs[6]),
            uints[4],
            uints[5],
            uints[6],
            uints[7],
            uints[8],
        );
        let sell: OrderType<T::AccountId, T::Moment> = Self::buildOrderType(
            addrs[7],
            addrs[8],
            addrs[9],
            uints[9],
            uints[10],
            uints[11],
            uints[12],
            addrs[10],
            FeeMethod::from(feeMethodsSidesKindsHowToCalls[4]),
            Side::from(feeMethodsSidesKindsHowToCalls[5]),
            SaleKind::from(feeMethodsSidesKindsHowToCalls[6]),
            addrs[11],
            HowToCall::from(feeMethodsSidesKindsHowToCalls[7]),
            calldataSell,
            replacementPatternSell,
            addrs[12],
            staticExtradataSell,
            (addrs[13]),
            uints[13],
            uints[14],
            uints[15],
            uints[16],
            uints[17],
        );
      Self::calculateMatchPrice(buy, sell)
    }

    /**
     * @dev Call atomicMatch - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn atomicMatch_(
        addrs: Vec<Address>,
        uints: Vec<u64>,
        feeMethodsSidesKindsHowToCalls: Vec<u8>,
        calldataBuy: Vec<u8>,
        calldataSell: Vec<u8>,
        replacementPatternBuy: Vec<u8>,
        replacementPatternSell: Vec<u8>,
        staticExtradataBuy: Vec<u8>,
        staticExtradataSell: Vec<u8>,
        sig: Vec<Signature>,
        rssMetadata: Vec<u8>,
    ) -> Result<(), Error<T>> {
      Self::atomicMatch(
            Self::buildOrderType(
                addrs[0],
                addrs[1],
                addrs[2],
                uints[0],
                uints[1],
                uints[2],
                uints[3],
                addrs[3],
                FeeMethod::from(feeMethodsSidesKindsHowToCalls[0]),
                Side::from(feeMethodsSidesKindsHowToCalls[1]),
                SaleKind::from(feeMethodsSidesKindsHowToCalls[2]),
                addrs[4],
                HowToCall::from(feeMethodsSidesKindsHowToCalls[3]),
                calldataBuy,
                replacementPatternBuy,
                addrs[5],
                staticExtradataBuy,
                (addrs[6]),
                uints[4],
                uints[5],
                uints[6],
                uints[7],
                uints[8],
            ),
            sig[0],
            Self::buildOrderType(
                addrs[7],
                addrs[8],
                addrs[9],
                uints[9],
                uints[10],
                uints[11],
                uints[12],
                addrs[10],
                FeeMethod::from(feeMethodsSidesKindsHowToCalls[4]),
                Side::from(feeMethodsSidesKindsHowToCalls[5]),
                SaleKind::from(feeMethodsSidesKindsHowToCalls[6]),
                addrs[11],
                HowToCall::from(feeMethodsSidesKindsHowToCalls[7]),
                calldataSell,
                replacementPatternSell,
                addrs[12],
                staticExtradataSell,
                (addrs[13]),
                uints[13],
                uints[14],
                uints[15],
                uints[16],
                uints[17],
            ),
            sig[1],
            rssMetadata,
        )?;
        Ok(())
    }

    ///exchange core
    /**
     * @dev Change the minimum maker fee paid to the protocol (only:owner)
     * @param newMinimumMakerProtocolFee New fee to set in basis points
     */
    fn changeMinimumMakerProtocolFee(newMinimumMakerProtocolFee: u64) -> Result<(), Error<T>>
// onlyOwner
    {
        minimumMakerProtocolFee.put(newMinimumMakerProtocolFee);
        Ok(())
    }

    /**
     * @dev Change the minimum taker fee paid to the protocol (only:owner)
     * @param newMinimumTakerProtocolFee New fee to set in basis points
     */
    fn changeMinimumTakerProtocolFee(newMinimumTakerProtocolFee: u64) -> Result<(), Error<T>> {
        // onlyOwner
        minimumTakerProtocolFee.put(newMinimumTakerProtocolFee);
        Ok(())
    }

    /**
     * @dev Change the protocol fee recipient (only:owner)
     * @param newProtocolFeeRecipient New protocol fee recipient Address
     */
    fn changeProtocolFeeRecipient(newProtocolFeeRecipient: Address) -> Result<(), Error<T>> {
        // onlyOwner
        protocolFeeRecipient.put(newProtocolFeeRecipient);
        Ok(())
    }

    /**
     * @dev Transfer tokens
     * @param token Token to transfer
     * @param from Address to charge fees
     * @param to Address to receive fees
     * @param amount Amount of protocol tokens to charge
     */
    fn transferTokens(
        token: Address,
        from: Address,
        to: Address,
        amount: u64,
    ) -> Result<(), Error<T>> {
        if amount > 0 {
            // ensure!(tokenTransferProxy.transferFrom(token, from, to, amount), Error::<T>::OrderIdMissing);
            // let call = Box::new(Call::Balances(BalancesCall::transfer(6, 1)));
            // ensure!(Proxy::proxy(Origin::signed(2), 1, None, call.clone()), Error::<T>::OrderIdMissing);
        }
        Ok(())
    }

    /**
     * @dev Charge a fee in protocol tokens
     * @param from Address to charge fees
     * @param to Address to receive fees
     * @param amount Amount of protocol tokens to charge
     */
    fn chargeProtocolFee(from: Address, to: Address, amount: u64) -> Result<(), Error<T>> {
      Self::transferTokens(exchangeToken::get(), from, to, amount);
        Ok(())
    }

    /**
     * @dev Hash an order, returning the canonical order hash, without the message prefix
     * @param order OrderType to hash
     * @return Hash of order
     */
    fn hashOrder(order: &OrderType<T::AccountId, T::Moment>) -> Vec<u8> {
        // hash := keccak256(add(array, 0x20), size)
        //    sp_io::hashing::blake2_256(&h).into()
        sp_io::hashing::keccak_256(&order.encode()).into()
        // }
        // }
        // return hash;
    }

    /**
     * @dev Hash an order, returning the hash that a client must sign, including the standard message prefix
     * @param order OrderType to hash
     * @return Hash of message prefix and order hash per Ethereum format
     */
    fn hashToSign(order: &OrderType<T::AccountId, T::Moment>) -> Vec<u8> {
        sp_io::hashing::keccak256("\x19Ethereum Signed Message:\n32", hashOrder(order))
    }

    /**
     * @dev Assert an order is valid and return its hash
     * @param order OrderType to validate
     * @param sig ECDSA signature
     */
    fn requireValidOrder(order: &OrderType<T::AccountId, T::Moment>, sig: &Signature) -> Vec<u8> {
        let hash: Vec<u8> = Self::hashToSign(order);
        ensure!(Self::validateOrder(hash, order, sig), Error::<T>::OrderIdMissing);
        hash
    }

    /**
     * @dev Validate order parameters (does *not* check validity:signature)
     * @param order OrderType to validate
     */
    fn validateOrderParameters(order: &OrderType<T::AccountId, T::Moment>) -> bool {
        /* OrderType must be targeted at this protocol version (this contract:Exchange). */
        //TODO
        // if order.exchange != 0 {
        //     return false;
        // }

        /* OrderType must possess valid sale kind parameter combination. */
        if !Self::validateParameters(order.saleKind, order.expirationTime) {
            return false;
        }

        /* If using the split fee method, order must have sufficient protocol fees. */
        if order.feeMethod == FeeMethod::SplitFee
            && (order.makerProtocolFee < minimumMakerProtocolFee::get()
                || order.takerProtocolFee < minimumTakerProtocolFee::get())
        {
            return false;
        }

        true
    }

    /**
     * @dev Validate a provided previously approved / signed order, hash, and signature.
     * @param hash OrderType hash (calculated:already, passed to recalculation:avoid)
     * @param order OrderType to validate
     * @param sig ECDSA signature
     */
    fn validateOrder(hash: Vec<u8>, order: &OrderType<T::AccountId, T::Moment>, sig: &Signature) -> bool {
        /* Not done in an if-conditional to prevent unnecessary ecrecover evaluation, which seems to happen even though it should short-circuit. */

        /* OrderType must have valid parameters. */
        if !validateOrderParameters(order) {
            return false;
        }

        /* OrderType must have not been canceled or already filled. */
        if cancelledOrFinalized.get(hash) {
            return false;
        }

        /* OrderType authentication. OrderType must be either:
        (a) previously approved */
        if approvedOrders.get(hash) {
            return true;
        }

        /* or (b) ECDSA-signed by maker. */
        // if ecrecover(hash, sig.v, sig.r, sig.s) == order.maker {
        //     return true;
        // }
        if check_signature(sig, hash, order.maker) == OK(()) {
            return true;
        }

        false
    }

    // An alterantive way to validate a signature is:

    // Import the codec and traits:

    // Example function to verify the signature.

    pub fn check_signature(
        signature: &T::Signature,
        msg: &[u8],
        signer: &T::AccountId,
    ) -> Result<(),Error<T>> {
        if signature.verify(msg, signer) {
            Ok(())
        } else {
            Err(Error::<T>::OrderIdMissing.into())
        }
    }

    /**
     * @dev Approve an order and optionally mark it for orderbook inclusion. Must be called by the maker of the order
     * @param order OrderType to approve
     * @param orderbookInclusionDesired Whether orderbook providers should include the order in their orderbooks
     */
    fn approveOrder(origin:AccountId,order: &OrderType<T::AccountId, T::Moment>, orderbookInclusionDesired: bool) {
        /* CHECKS */

        /* Assert sender is authorized to approve order. */
        ensure!(origin == order.maker, Error::<T>::OrderIdMissing);

        /* Calculate order hash. */
        let hash: Vec<u8> = Self::hashToSign(order);

        /* Assert order has not already been approved. */
        ensure!(!approvedOrders.get(hash), Error::<T>::OrderIdMissing);

        /* EFFECTS */

        /* Mark order as approved. */
        approvedOrders.insert(hash, true);

        /* Log approval event. Must be split in two due to Solidity stack size limitations. */
        {
            Self::deposit_event(RawEvent::OrderApprovedPartOne(
                hash,
                order.exchange,
                order.maker,
                order.taker,
                order.makerRelayerFee,
                order.takerRelayerFee,
                order.makerProtocolFee,
                order.takerProtocolFee,
                order.feeRecipient,
                order.feeMethod,
                order.side,
                order.saleKind,
                order.target,
            ));
        }
        {
            Self::deposit_event(RawEvent::OrderApprovedPartTwo(
                hash,
                order.howToCall,
                order.calldata,
                order.replacementPattern,
                order.staticTarget,
                order.staticExtradata,
                order.paymentToken,
                order.basePrice,
                order.extra,
                order.listingTime,
                order.expirationTime,
                order.salt,
                orderbookInclusionDesired,
            ));
        }
    }

    /**
     * @dev Cancel an order, preventing it from being matched. Must be called by the maker of the order
     * @param order OrderType to cancel
     * @param sig ECDSA signature
     */
    fn cancelOrder(origin:AccountId,order: &OrderType<T::AccountId, T::Moment>, sig: &Signature) -> Result<(), Error<T>> {
        /* CHECKS */

        /* Calculate order hash. */
        let hash: Vec<u8> = Self::requireValidOrder(order, sig);

        /* Assert sender is authorized to cancel order. */
        ensure!(origin == order.maker, Error::<T>::OrderIdMissing);

        /* EFFECTS */

        /* Mark order as cancelled, preventing it from being matched. */
        cancelledOrFinalized.insert(hash, true);

        /* Log cancel event. */
        Self::deposit_event(RawEvent::OrderCancelled(hash));
        Ok(())
    }

    /**
     * @dev Calculate the current price of an order (fn:convenience)
     * @param order OrderType to calculate the price of
     * @return The current price of the order
     */
    fn calculateCurrentPrice(order: &OrderType<T::AccountId, T::Moment>) -> u64 {
        Self::calculateFinalPrice(
            order.side,
            order.saleKind,
            order.basePrice,
            order.extra,
            order.listingTime,
            order.expirationTime,
        )
    }

    /**
     * @dev Calculate the price two orders would match at, if in fact they would match (fail:otherwise)
     * @param buy Buy-side order
     * @param sell Sell-side order
     * @return Match price
     */
    fn calculateMatchPrice(buy: OrderType, sell: OrderType) -> u64 {
        /* Calculate sell price. */
        let sellPrice: u64 = Self::calculateFinalPrice(
            sell.side,
            sell.saleKind,
            sell.basePrice,
            sell.extra,
            sell.listingTime,
            sell.expirationTime,
        );

        /* Calculate buy price. */
       let buyPrice: u64 = Self::calculateFinalPrice(
            buy.side,
            buy.saleKind,
            buy.basePrice,
            buy.extra,
            buy.listingTime,
            buy.expirationTime,
        );

        /* Require price cross. */
        ensure!(buyPrice >= sellPrice, Error::<T>::OrderIdMissing);

        /* Maker/taker priority. */
        if sell.feeRecipient != Address{} {
            sellPrice
        } else {
            buyPrice
        }
    }

    /**
     * @dev Execute all ERC20 token / Ether transfers associated with an order match (fees and buyer => transfer:seller)
     * @param buy Buy-side order
     * @param sell Sell-side order
     */
    fn executeFundsTransfer(msg_value: u64, buy: OrderType, sell: OrderType) -> u64 {
        let originprotocolFeeRecipient = protocolFeeRecipient::get();
        /* Only payable in the special case of unwrapped Ether. */
        if sell.paymentToken != Address{} {
            ensure!(msg_value == 0, Error::<T>::OrderIdMissing);
        }

        /* Calculate match price. */
       let  price: u64 = Self::calculateMatchPrice(buy, sell);

        /* If paying using a token (Ether:not), transfer tokens. This is done prior to fee payments to that a seller will have tokens before being charged fees. */
        if price > 0 && sell.paymentToken != Address{} {
          Self::transferTokens(sell.paymentToken, buy.maker, sell.maker, price);
        }

        /* Amount that will be received by seller (Ether:for). */
        let receiveAmount: u64 = price;

        /* Amount that must be sent by buyer (Ether:for). */
        let requiredAmount: u64 = price;

        /* Determine maker/taker and charge fees accordingly. */
        if sell.feeRecipient != Address{} {
            /* Sell-side order is maker. */

            /* Assert taker fee is less than or equal to maximum fee specified by buyer. */
            ensure!(
                sell.takerRelayerFee <= buy.takerRelayerFee,
                Error::<T>::OrderIdMissing
            );

            if sell.feeMethod == FeeMethod::SplitFee {
                /* Assert taker fee is less than or equal to maximum fee specified by buyer. */
                ensure!(
                    sell.takerProtocolFee <= buy.takerProtocolFee,
                    Error::<T>::OrderIdMissing
                );

                /* Maker fees are deducted from the token amount that the maker receives. Taker fees are extra tokens that must be paid by the taker. */

                if sell.makerRelayerFee > 0 {
                    let makerRelayerFee: u64 = ((sell.makerRelayerFee * price) / INVERSE_BASIS_POINT);
                    if sell.paymentToken == Address{} {
                        receiveAmount = (receiveAmount - makerRelayerFee);
                        // sell.feeRecipient.transfer(makerRelayerFee);
                      Self::transferTokens(
                            Self::AccountId,
                            Self::AccountId,
                            sell.feeRecipient,
                            makerRelayerFee,
                        );
                    } else {
                      Self::transferTokens(
                            sell.paymentToken,
                            sell.maker,
                            sell.feeRecipient,
                            makerRelayerFee,
                        );
                    }
                }

                if sell.takerRelayerFee > 0 {
                    let takerRelayerFee: u64 = ((sell.takerRelayerFee * price) / INVERSE_BASIS_POINT);
                    if sell.paymentToken == Address{} {
                        requiredAmount = (requiredAmount + takerRelayerFee);
                        // sell.feeRecipient.transfer(takerRelayerFee);
                      Self::transferTokens(
                            Self::AccountId,
                            Self::AccountId,
                            sell.feeRecipient,
                            takerRelayerFee,
                        );
                    } else {
                      Self::transferTokens(
                            sell.paymentToken,
                            buy.maker,
                            sell.feeRecipient,
                            takerRelayerFee,
                        );
                    }
                }

                if sell.makerProtocolFee > 0 {
                    let makerProtocolFee: u64 = ((sell.makerProtocolFee * price) / INVERSE_BASIS_POINT);
                    if sell.paymentToken == Address{} {
                        receiveAmount = (receiveAmount - makerProtocolFee);
                        // protocolFeeRecipient.transfer(makerProtocolFee);
                      Self::transferTokens(
                            Self::AccountId,
                            Self::AccountId,
                            originprotocolFeeRecipient,
                            makerProtocolFee,
                        );
                    } else {
                      Self::transferTokens(
                            sell.paymentToken,
                            sell.maker,
                            originprotocolFeeRecipient,
                            makerProtocolFee,
                        );
                    }
                }

                if sell.takerProtocolFee > 0 {
                    let takerProtocolFee: u64 = ((sell.takerProtocolFee * price) / INVERSE_BASIS_POINT);
                    if sell.paymentToken == Address{} {
                        requiredAmount = (requiredAmount + takerProtocolFee);
                        // protocolFeeRecipient.transfer(takerProtocolFee);
                      Self::transferTokens(
                            Self::AccountId,
                            Self::AccountId,
                            originprotocolFeeRecipient,
                            takerProtocolFee,
                        );
                    } else {
                      Self::transferTokens(
                            sell.paymentToken,
                            buy.maker,
                            originprotocolFeeRecipient,
                            takerProtocolFee,
                        );
                    }
                }
            } else {
                /* Charge maker fee to seller. */
              Self::chargeProtocolFee(sell.maker, sell.feeRecipient, sell.makerRelayerFee);

                /* Charge taker fee to buyer. */
              Self::chargeProtocolFee(buy.maker, sell.feeRecipient, sell.takerRelayerFee);
            }
        } else {
            /* Buy-side order is maker. */

            /* Assert taker fee is less than or equal to maximum fee specified by seller. */
            ensure!(
                buy.takerRelayerFee <= sell.takerRelayerFee,
                Error::<T>::OrderIdMissing
            );

            if sell.feeMethod == FeeMethod::SplitFee {
                /* The Exchange does not escrow Ether, so direct Ether can only be used to with sell-side maker / buy-side taker orders. */
                ensure!(sell.paymentToken != Address{}, Error::<T>::OrderIdMissing);

                /* Assert taker fee is less than or equal to maximum fee specified by seller. */
                ensure!(
                    buy.takerProtocolFee <= sell.takerProtocolFee,
                    Error::<T>::OrderIdMissing
                );

                if buy.makerRelayerFee > 0 {
                    makerRelayerFee = ((buy.makerRelayerFee * price) / INVERSE_BASIS_POINT);
                  Self::transferTokens(
                        sell.paymentToken,
                        buy.maker,
                        buy.feeRecipient,
                        makerRelayerFee,
                    );
                }

                if buy.takerRelayerFee > 0 {
                    takerRelayerFee = ((buy.takerRelayerFee * price) / INVERSE_BASIS_POINT);
                  Self::transferTokens(
                        sell.paymentToken,
                        sell.maker,
                        buy.feeRecipient,
                        takerRelayerFee,
                    );
                }

                if buy.makerProtocolFee > 0 {
                    makerProtocolFee = ((buy.makerProtocolFee * price) / INVERSE_BASIS_POINT);
                  Self::transferTokens(
                        sell.paymentToken,
                        buy.maker,
                        originprotocolFeeRecipient,
                        makerProtocolFee,
                    );
                }

                if buy.takerProtocolFee > 0 {
                    takerProtocolFee = ((buy.takerProtocolFee * price) / INVERSE_BASIS_POINT);
                  Self::transferTokens(
                        sell.paymentToken,
                        sell.maker,
                        originprotocolFeeRecipient,
                        takerProtocolFee,
                    );
                }
            } else {
                /* Charge maker fee to buyer. */
              Self::chargeProtocolFee(buy.maker, buy.feeRecipient, buy.makerRelayerFee);

                /* Charge taker fee to seller. */
              Self::chargeProtocolFee(sell.maker, buy.feeRecipient, buy.takerRelayerFee);
            }
        }

        if sell.paymentToken == Address{} {
            /* Special-case Ether, order must be matched by buyer. */
            ensure!(msg_value >= requiredAmount, Error::<T>::OrderIdMissing);
            // sell.maker.transfer(receiveAmount);
          Self::transferTokens(Self::AccountId, Self::AccountId, sell.maker, receiveAmount);
            /* Allow overshoot for variable-price auctions, refund difference. */
            let diff: u64 = (msg_value - requiredAmount);
            if diff > 0 {
                // buy.maker.transfer(diff);
              Self::transferTokens(Self::AccountId, Self::AccountId, buy.maker, diff);
            }
        }

        /* This contract should never hold Ether, however, we cannot assert this, since it is impossible to prevent anyone from sending Ether e.g. with selfdestruct. */

        return price;
    }

    /**
     * @dev Return whether or not two orders can be matched with each other by basic parameters (does not check order signatures / calldata or perform calls:static)
     * @param buy Buy-side order
     * @param sell Sell-side order
     * @return Whether or not the two orders can be matched
     */
    fn ordersCanMatch(buy: OrderType, sell: OrderType) -> bool {
        (
            //  Must be opposite-side.
            (buy.side == SaleKindInterface.Side.Buy && sell.side == SaleKindInterface.Side.Sell) &&
            // Must use same fee method.
            (buy.feeMethod == sell.feeMethod) &&
            // Must use same payment token. 
            (buy.paymentToken == sell.paymentToken) &&
            // Must match maker/taker addresses. 
            (sell.taker == Address{} || sell.taker == buy.maker) &&
            (buy.taker == Address{} || buy.taker == sell.maker) &&
            // One must be maker and the other must be taker (no bool XOR Solidity:in). 
            ((sell.feeRecipient == Address{} && buy.feeRecipient != Address{}) || (sell.feeRecipient != Address{} && buy.feeRecipient == Address{})) &&
            // Must match target. 
            (buy.target == sell.target) &&
            // Must match howToCall. 
            (buy.howToCall == sell.howToCall) &&
            // Buy-side order must be settleable. 
            SaleKindInterface.canSettleOrder(buy.listingTime, buy.expirationTime) &&
            // Sell-side order must be settleable. 
            SaleKindInterface.canSettleOrder(sell.listingTime, sell.expirationTime)
        )
    }

    /**
     * @dev Atomically match two orders, ensuring validity of the match, and execute all associated state transitions. Protected against reentrancy by a contract-global lock.
     * @param buy Buy-side order
     * @param buySig Buy-side order signature
     * @param sell Sell-side order
     * @param sellSig Sell-side order signature
     */
    fn atomicMatch(
        msg_sender: AccountId,
        msg_value: u64,
        buy: OrderType,
        buySig: Sig,
        sell: OrderType,
        sellSig: Signature,
        metadata: Vec<u8>,
    ) {
        //reentrancyGuard
        /* CHECKS */

        /* Ensure buy order validity and calculate hash if necessary. */
        let buyHash: Vec<u8> = vec![];
        if buy.maker == msg_sender {
            ensure!(Self::validateOrderParameters(buy), Error::<T>::OrderIdMissing);
        } else {
            buyHash = requireValidOrder(buy, buySig);
        }

        /* Ensure sell order validity and calculate hash if necessary. */
        let sellHash: Vec<u8> = vec![];
        if sell.maker == msg_sender {
            ensure!(Self::validateOrderParameters(sell), Error::<T>::OrderIdMissing);
        } else {
            sellHash = requireValidOrder(sell, sellSig);
        }

        /* Must be matchable. */
        ensure!(Self::ordersCanMatch(buy, sell), Error::<T>::OrderIdMissing);

        /* Target must exist (prevent malicious selfdestructs just prior to settlement:order). */
        // u64 size;
        // Address target = sell.target;
        // assembly {
        //     size := extcodesize(target)
        // }
        // ensure!(size > 0, Error::<T>::OrderIdMissing);

        /* Must match calldata after replacement, if specified. */
        if buy.replacementPattern.length > 0 {
            Self::guardedArrayReplace(buy.calldata, sell.calldata, buy.replacementPattern);
        }
        if sell.replacementPattern.length > 0 {
            Self::guardedArrayReplace(sell.calldata, buy.calldata, sell.replacementPattern);
        }
        ensure!(
            Self::arrayEq(buy.calldata, sell.calldata),
            Error::<T>::OrderIdMissing
        );

        // /* Retrieve delegateProxy contract. */
        // OwnableDelegateProxy delegateProxy = registry.proxies(sell.maker);

        // /* Proxy must exist. */
        // ensure!(delegateProxy != Address{}, Error::<T>::OrderIdMissing);

        // /* Assert implementation. */
        // ensure!(delegateProxy.implementation() == registry.delegateProxyImplementation(), Error::<T>::OrderIdMissing);

        // /* Access the passthrough AuthenticatedProxy. */
        // AuthenticatedProxy proxy = AuthenticatedProxy(delegateProxy);

        /* EFFECTS */

        /* Mark previously signed or approved orders as finalized. */
        if msg_sender != buy.maker {
            cancelledOrFinalized.insert(buyHash, true);
        }
        if msg_sender != sell.maker {
            cancelledOrFinalized.insert(sellHash, true);
        }

        /* INTERACTIONS */

        /* Execute funds transfer and pay fees. */
        let price: u64 = executeFundsTransfer(msg_value, buy, sell);

        /* Execute specified call through proxy. */
        ensure!(
            proxy.proxy(sell.target, sell.howToCall, sell.calldata),
            Error::<T>::OrderIdMissing
        );

        /* Static calls are intentionally done after the effectful call so they can check resulting state. */

        /* Handle buy-side static call if specified. */
        // if buy.staticTarget != Address{} {
        //     ensure!(Self::staticCall(buy.staticTarget, sell.calldata, buy.staticExtradata), Error::<T>::OrderIdMissing);
        // }

        // /* Handle sell-side static call if specified. */
        // if sell.staticTarget != Address{} {
        //     ensure!(Self::staticCall(sell.staticTarget, sell.calldata, sell.staticExtradata), Error::<T>::OrderIdMissing);
        // }

        /* Log match event. */
        Self::deposit_event(RawEvent::OrdersMatched(
            buyHash,
            sellHash,
            if sell.feeRecipient != Address{} {
                sell.maker
            } else {
                buy.maker
            },
            if sell.feeRecipient != Address{} {
                buy.maker
            } else {
                sell.maker
            },
            price,
            metadata,
        ));
    }

    /// sale Kind interface
    /**
     * @dev Check whether the parameters of a sale are valid
     * @param saleKind Kind of sale
     * @param expirationTime OrderType expiration time
     * @return Whether the parameters were valid
     */
    fn validateParameters(saleKind: SaleKind, expirationTime: u64) -> bool {
        /* Auctions must have a set expiration date. */
        (saleKind == SaleKind::FixedPrice || expirationTime > 0)
    }

    /**
     * @dev Return whether or not an order can be settled
     * @dev Precondition: parameters have passed validateParameters
     * @param listingTime OrderType listing time
     * @param expirationTime OrderType expiration time
     */
    fn canSettleOrder(listingTime: u64, expirationTime: u64) -> bool {
        (listingTime < now) && (expirationTime == 0 || now < expirationTime)
    }

    /**
     * @dev Calculate the settlement price of an order
     * @dev Precondition: parameters have passed validateParameters.
     * @param side OrderType side
     * @param saleKind Method of sale
     * @param basePrice OrderType base price
     * @param extra OrderType extra price data
     * @param listingTime OrderType listing time
     * @param expirationTime OrderType expiration time
     */
    fn calculateFinalPrice(
        side: Side,
        saleKind: SaleKind,
        basePrice: u64,
        extra: u64,
        listingTime: u64,
        expirationTime: u64,
    ) -> u64 {
        if saleKind == SaleKind::FixedPrice {
            return basePrice;
        } else if saleKind == SaleKind::DutchAuction {
            let diff: u64 = extra * (now - listingTime) / (expirationTime - listingTime);
            if side == Side::Sell {
                /* Sell-side - start price: basePrice. End price: basePrice - extra. */
                return basePrice - diff;
            } else {
                /* Buy-side - start price: basePrice. End price: basePrice + extra. */
                return basePrice + diff;
            }
        }

        0
    }

    /**
     * Replace Vec<u8> in an array with Vec<u8> in another array, guarded by a bitmask
     * Efficiency of this fn is a bit unpredictable because of the EVM's word-specific model (arrays under 32 Vec<u8> will be slower)
     *
     * @dev Mask must be the size of the byte array. A nonzero byte means the byte array can be changed.
     * @param array The original array
     * @param desired The target array
     * @param mask The mask specifying which bits can be changed
     * @return The updated byte array (the parameter will be modified inplace)
     */
    fn guardedArrayReplace(
        array: &mut Vec<u8>,
        desired: Vec<u8>,
        mask: Vec<u8>,
    ) -> Result<(), Error<T>> {
        ensure!(array.length == desired.length, Error::<T>::OrderIdMissing);
        ensure!(array.length == mask.length, Error::<T>::OrderIdMissing);

        for (i, &item) in array.iter().enumerate() {
            /* Conceptually: array[i] = (!mask[i] && array[i]) || (mask[i] && desired[i]), bitwise in word chunks. */
            array[i] = (!mask[i] & array[i]) || (mask[i] & desired[i]);
        }
        Ok(())
    }

    /**
     * Test if two arrays are equal
     * Source: https://github.com/GNSPS/solidity-Vec<u8>-utils/blob/master/contracts/BytesLib.sol
     *
     * @dev Arrays must be of equal length, otherwise will return false
     * @param a First array
     * @param b Second array
     * @return Whether or not all Vec<u8> in the arrays are equal
     */
    fn arrayEq(a: Vec<u8>, b: Vec<u8>) -> bool {
        if a.len() != b.len() {
            return false;
        }

        a == b
    }

pub fn buildOrderType(index: u64,

    exchange: Address,
    /* OrderType maker Address. */
    maker: Address,
    /* OrderType taker Address, if specified. */
    taker: Address,
    /* Maker relayer fee of the order, unused for taker order. */
    makerRelayerFee: u64,
    /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
    takerRelayerFee: u64,
    /* Maker protocol fee of the order, unused for taker order. */
    makerProtocolFee: u64,
    /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
    takerProtocolFee: u64,
    /* OrderType fee recipient or zero Address for taker order. */
    feeRecipient: Address,
    /* Fee method (protocol token or split fee). */
    feeMethod: FeeMethod,
    /* Side (buy/sell). */
    side: Side,
    /* Kind of sale. */
    saleKind: SaleKind,
    /* Target. */
    target: Address,
    /* Vec<u8>. */
    howToCall: Vec<u8>,
    /* Calldata. */
    calldata: Bytes,
    /* Calldata replacement pattern, or an empty byte array for no replacement. */
    replacementPattern: Bytes,
    /* Static call target, zero-Address for no static call. */
    staticTarget: Address,
    /* Static call extra data. */
    staticExtradata: Bytes,
    /* Token used to pay for the order, or the zero-Address as a sentinel value for Ether. */
    paymentToken: Address,
    /* Base price of the order (in paymentTokens). */
    basePrice: u64,
    /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
    extra: u64,
    /* Listing timestamp. */
    listingTime: u64,
    /* Expiration timestamp - 0 for no expiry. */
    expirationTime: u64,
    /* OrderType salt, used to prevent duplicate hashes. */
    salt: u64) -> OrderType<AccountId, Moment> {
        OrderType::<AccountId,Moment> {
    index,
    exchange,
    /* OrderType maker Address. */
    maker,
    /* OrderType taker Address, if specified. */
    taker,
    /* Maker relayer fee of the order, unused for taker order. */
    makerRelayerFee,
    /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
    takerRelayerFee,
    /* Maker protocol fee of the order, unused for taker order. */
    makerProtocolFee,
    /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
    takerProtocolFee,
    /* OrderType fee recipient or zero Address for taker order. */
    feeRecipient,
    /* Fee method (protocol token or split fee). */
    feeMethod,
    /* Side (buy/sell). */
    side,
    /* Kind of sale. */
    saleKind,
    /* Target. */
    target,
    /* Vec<u8>. */
    howToCall,
    /* Calldata. */
    calldata,
    /* Calldata replacement pattern, or an empty byte array for no replacement. */
    replacementPattern,
    /* Static call target, zero-Address for no static call. */
    staticTarget,
    /* Static call extra data. */
    staticExtradata,
    /* Token used to pay for the order, or the zero-Address as a sentinel value for Ether. */
    paymentToken,
    /* Base price of the order (in paymentTokens). */
    basePrice,
    /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
    extra,
    /* Listing timestamp. */
    listingTime,
    /* Expiration timestamp - 0 for no expiry. */
    expirationTime,
    /* OrderType salt, used to prevent duplicate hashes. */
    salt
}
}

}




// #[derive(Default)]
// pub struct OrderTypeBuilder<AccountId, Moment>
// where
//     AccountId: Default,
//     Moment: Default,
// {
//     index: u64,
//     id: OrderId,
//     owner: AccountId,
//     fields: Option<Vec<OrderField>>,
//     registered: Moment,
// }

// impl<AccountId, Moment> OrderTypeBuilder<AccountId, Moment>
// where
//     AccountId: Default,
//     Moment: Default,
// {
//     pub fn index_by(mut self, index: u64) -> Self {
//         self.index = index;
//         self
//     }

//     pub fn identified_by(mut self, id: OrderId) -> Self {
//         self.id = id;
//         self
//     }

//     pub fn owned_by(mut self, owner: AccountId) -> Self {
//         self.owner = owner;
//         self
//     }

//     pub fn with_fields(mut self, fields: Option<Vec<OrderField>>) -> Self {
//         self.fields = fields;
//         self
//     }

//     pub fn registered_on(mut self, registered: Moment) -> Self {
//         self.registered = registered;
//         self
//     }

//     pub fn build(self) -> OrderType<AccountId, Moment> {
//         OrderType::<AccountId, Moment> {
//             index: self.index,
//             id: self.id,
//             owner: self.owner,
//             fields: self.fields,
//             registered: self.registered,
//         }
//     }

//     pub fn build() -> OrderType<AccountId, Moment> {
//         OrderType<AccountId, Moment> {
//     index: u64,

//     exchange: Address,
//     /* OrderType maker Address. */
//     maker: Address,
//     /* OrderType taker Address, if specified. */
//     taker: Address,
//     /* Maker relayer fee of the order, unused for taker order. */
//     makerRelayerFee: u64,
//     /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
//     takerRelayerFee: u64,
//     /* Maker protocol fee of the order, unused for taker order. */
//     makerProtocolFee: u64,
//     /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
//     takerProtocolFee: u64,
//     /* OrderType fee recipient or zero Address for taker order. */
//     feeRecipient: Address,
//     /* Fee method (protocol token or split fee). */
//     feeMethod: FeeMethod,
//     /* Side (buy/sell). */
//     side: Side,
//     /* Kind of sale. */
//     saleKind: SaleKind,
//     /* Target. */
//     target: Address,
//     /* Vec<u8>. */
//     howToCall: Vec<u8>,
//     /* Calldata. */
//     calldata: Bytes,
//     /* Calldata replacement pattern, or an empty byte array for no replacement. */
//     replacementPattern: Bytes,
//     /* Static call target, zero-Address for no static call. */
//     staticTarget: Address,
//     /* Static call extra data. */
//     staticExtradata: Bytes,
//     /* Token used to pay for the order, or the zero-Address as a sentinel value for Ether. */
//     paymentToken: Address,
//     /* Base price of the order (in paymentTokens). */
//     basePrice: u64,
//     /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
//     extra: u64,
//     /* Listing timestamp. */
//     listingTime: u64,
//     /* Expiration timestamp - 0 for no expiry. */
//     expirationTime: u64,
//     /* OrderType salt, used to prevent duplicate hashes. */
//     salt: u64,
// }
//     }
// }