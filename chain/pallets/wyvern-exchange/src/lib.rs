//! # Substrate Enterprise Sample - OrderType Post example pallet

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::result::Result;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,  ensure, 
sp_runtime::{RuntimeDebug,MultiSignature,traits::{IdentifyAccount, Member, Verify}},
traits::{Currency, Get, LockableCurrency, Randomness, ReservableCurrency},sp_io::hashing::keccak_256,
    dispatch::{DispatchError, DispatchResult},sp_std::collections::btree_set::BTreeSet, sp_std::prelude::*
};
// use sp_runtime::{generic, MultiSignature, traits::{Verify, BlakeTwo256, IdentifyAccount}};

// traits::EnsureOrigin,
use frame_system::{self as system, ensure_signed};
use balances::Call as BalancesCall;

// use sp_core::H256;
// use sp_io::hashing;

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
pub const TOKEN_OWNER: AccountId = [0 as u8; 32].into();
// /* Inverse basis point. */
pub const INVERSE_BASIS_POINT: u64 = 10000;

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
// pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
// pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

// Custom types
// pub type AccountId =Vec<u8>;
pub type OrderId = Vec<u8>;
pub type FieldName = Vec<u8>;
pub type FieldValue = Vec<u8>;

pub type Bytes =Vec<u8>;

///sale kind interface
#[derive(Encode, Decode, Debug, Clone,Eq,  PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Encode, Decode, Debug, Clone, Eq, PartialEq)]
pub enum SaleKind {
    FixedPrice,
    DutchAuction,
}

// /* Fee method: protocol fee or split fee. */
// enum FeeMethod { ProtocolFee, SplitFee }
#[derive(Encode, Decode, Debug, Clone,Eq,  PartialEq)]
pub enum FeeMethod {
    ProtocolFee,
    SplitFee,
}

#[derive(Encode, Decode, Debug, Clone,Eq,  PartialEq)]
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
            Side::Buy => 0x0,
            Side::Sell => 0x1,
        }
    }
}

impl From<u8> for Side {
    fn from(orig: u8) -> Self {
        match orig {
            0x0 => return Side::Buy,
            0x1 => return Side::Sell,
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
      // /* An order on the exchange. */
    pub index: u64,
    /* Exchange AccountId, intended as a versioning mechanism. */
    pub exchange: AccountId,
    /* OrderType maker AccountId. */
    pub maker: AccountId,
    /* OrderType taker AccountId, if specified. */
    pub taker: AccountId,
    /* Maker relayer fee of the order, unused for taker order. */
    pub makerRelayerFee: u64,
    /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
    pub takerRelayerFee: u64,
    /* Maker protocol fee of the order, unused for taker order. */
    pub makerProtocolFee: u64,
    /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
    pub takerProtocolFee: u64,
    /* OrderType fee recipient or zero AccountId for taker order. */
    pub feeRecipient: AccountId,
    /* Fee method (protocol token or split fee). */
    pub feeMethod: FeeMethod,
    /* Side (buy/sell). */
    pub side: Side,
    /* Kind of sale. */
    pub saleKind: SaleKind,
    /* Target. */
    pub target: AccountId,
    /* Vec<u8>. */
    pub howToCall: HowToCall,
    /* Calldata. */
    pub calldata: Bytes,
    /* Calldata replacement pattern, or an empty byte array for no replacement. */
    pub replacementPattern: Bytes,
    /* Static call target, zero-AccountId for no static call. */
    pub staticTarget: AccountId,
    /* Static call extra data. */
    pub staticExtradata: Bytes,
    /* Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether. */
    pub paymentToken: AccountId,
    /* Base price of the order (in paymentTokens). */
    pub basePrice: u64,
    /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
    pub extra: u64,
    /* Listing timestamp. */
    pub listingTime: u64,
    /* Expiration timestamp - 0 for no expiry. */
    pub expirationTime: u64,
    /* OrderType salt, used to prevent duplicate hashes. */
    pub salt: u64,
    pub registered: Moment,
}


//exchange core

// Add new types to the trait:

// pub trait Trait: system::Trait {
//     type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
//     type Public: IdentifyAccount<AccountId = TOKEN_OWNER> + Clone;
//     type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
// }

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Public: IdentifyAccount<AccountId = Self::AccountId> + Clone;
    type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
    /// Currency type for this module.
    type Currency: ReservableCurrency<Self::AccountId>
        + LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;
    // type CreateRoleOrigin: EnsureOrigin<Self::Origin>;
}

decl_storage! {
    trait Store for Module<T: Trait> as OrderRegistry {
        NextOrderIndex: u64;
pub ContractSelf:AccountId;
 // /* The token used to pay exchange fees. */
    // ERC20 public ExchangeToken;
pub ExchangeToken:AccountId;
    // /* User registry. */
    // ProxyRegistry public registry;
pub Registry:AccountId;
    // /* Token transfer proxy. */
    // TokenTransferProxy public TokenTransferProxy;
pub TokenTransferProxy:AccountId;
    // /* Cancelled / finalized orders, by hash. */
    // mapping(Vec<u8> => bool) public CancelledOrFinalized;
  pub CancelledOrFinalized get(fn cancelled_or_finalized): map hasher(blake2_128_concat) Vec<u8> => bool;
    // /* Orders verified by on-chain approval (alternative to ECDSA signatures so that smart contracts can place orders directly). */
    // mapping(Vec<u8> => bool) public ApprovedOrders;
  pub ApprovedOrders get(fn approved_orders): map hasher(blake2_128_concat) Vec<u8> => bool;
    // /* For split fee orders, minimum required protocol maker fee, in basis points. Paid to owner (who can change it). */
    // u64 public MinimumMakerProtocolFee = 0;
pub MinimumMakerProtocolFee:u64;
    // /* For split fee orders, minimum required protocol taker fee, in basis points. Paid to owner (who can change it). */
    // u64 public MinimumTakerProtocolFee = 0;
pub MinimumTakerProtocolFee:u64;
    // /* Recipient of protocol fees. */
    // AccountId public ProtocolFeeRecipient;
pub ProtocolFeeRecipient:AccountId;


 }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        // event OrderApprovedPartOne    (Vec<u8> indexed hash, AccountId exchange, AccountId indexed maker, AccountId taker,
        // u64 makerRelayerFee, u64 takerRelayerFee, u64 makerProtocolFee, u64 takerProtocolFee,
        // AccountId indexed feeRecipient, FeeMethod feeMethod, SaleKindInterface.Side side, SaleKindInterface.SaleKind saleKind, AccountId target);
        // event OrderApprovedPartTwo    (Vec<u8> indexed hash, AuthenticatedProxy.Vec<u8> howToCall, Vec<u8> calldata, Vec<u8> replacementPattern,
        // AccountId staticTarget, Vec<u8> staticExtradata, AccountId paymentToken, u64 basePrice,
        // u64 extra, u64 listingTime, u64 expirationTime, u64 salt, bool orderbookInclusionDesired);
        // event OrderCancelled          (Vec<u8> indexed hash);
        // event OrdersMatched           (Vec<u8> buyHash, Vec<u8> sellHash, AccountId indexed maker, AccountId indexed taker, u64 price, Vec<u8> indexed metadata);
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
            HowToCall,
            Vec<u8>,
            Vec<u8>,
            AccountId,
            Vec<u8>,
            AccountId,
            u64,
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
//             // T::CreateRoleOrigin::ensure_origin(&origin)?;
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
//                 .identified_by(&id)
//                 .owned_by(&owner)
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
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: HowToCall,
        calldata: &[u8],
        replacementPattern: &[u8],
        staticExtradata: &[u8],
    ) -> Vec<u8> {
      Self::hashOrder(&Self::buildOrderTypeArr(
          addrs,
        uints,
        feeMethod,
        side,
        saleKind,
        howToCall,
        calldata,
        replacementPattern,
        staticExtradata
        ))
    }

    /**
     * @dev Call hashToSign - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn hashToSign_(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: HowToCall,
        calldata: &[u8],
        replacementPattern: &[u8],
        staticExtradata: &[u8],
    ) -> Vec<u8> {
      Self::hashToSign(&Self::buildOrderTypeArr(
          addrs,
        uints,
        feeMethod,
        side,
        saleKind,
        howToCall,
        calldata,
        replacementPattern,
        staticExtradata
        ))
    }

    /**
     * @dev Call validateOrderParameters - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn validateOrderParameters_(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: HowToCall,
        calldata: &[u8],
        replacementPattern: &[u8],
        staticExtradata: &[u8],
    ) -> bool {
        let order: OrderType<T::AccountId,T::Moment> = Self::buildOrderTypeArr(
          addrs,
        uints,
        feeMethod,
        side,
        saleKind,
        howToCall,
        calldata,
        replacementPattern,
        staticExtradata
        );
      Self::validateOrderParameters(&order)
    }

    /**
     * @dev Call validateOrder - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn validateOrder_(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: HowToCall,
        calldata: &[u8],
        replacementPattern: &[u8],
        staticExtradata: &[u8],
        sig: T::Signature,
    ) -> bool {
        let order: OrderType<T::AccountId,T::Moment> = Self::buildOrderTypeArr(
          addrs,
        uints,
        feeMethod,
        side,
        saleKind,
        howToCall,
        calldata,
        replacementPattern,
        staticExtradata
        );
      Self::validateOrder(Self::hashToSign(&order), &order, &sig)
    }

    /**
     * @dev Call approveOrder - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    pub fn approveOrder_(origin:T::Origin,
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: HowToCall,
        calldata: &[u8],
        replacementPattern: &[u8],
        staticExtradata: &[u8],
        orderbookInclusionDesired: bool,
    ) -> Result<(), Error<T>> {
       let  order: OrderType<T::AccountId,T::Moment> = Self::buildOrderTypeArr(
          addrs,
        uints,
        feeMethod,
        side,
        saleKind,
        howToCall,
        calldata,
        replacementPattern,
        staticExtradata
        );
      Self::approveOrder(origin,&order, orderbookInclusionDesired);
        Ok(())
    }

    /**
     * @dev Call cancelOrder - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn cancelOrder_(origin:T::Origin,
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: HowToCall,
        calldata: &[u8],
        replacementPattern: &[u8],
        staticExtradata: &[u8],
        sig: T::Signature,
    ) -> Result<(), Error<T>> {
      Self::cancelOrder(origin,
            &Self::buildOrderTypeArr(
          addrs,
        uints,
        feeMethod,
        side,
        saleKind,
        howToCall,
        calldata,
        replacementPattern,
        staticExtradata
            ),
            &sig,
        );
        Ok(())
    }

    /**
     * @dev Call calculateCurrentPrice - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn calculateCurrentPrice_(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: HowToCall,
        calldata: &[u8],
        replacementPattern: &[u8],
        staticExtradata: &[u8],
    ) -> u64 {
      Self::calculateCurrentPrice(&Self::buildOrderTypeArr(
          addrs,
        uints,
        feeMethod,
        side,
        saleKind,
        howToCall,
        calldata,
        replacementPattern,
        staticExtradata
        ))
    }

    /**
     * @dev Call ordersCanMatch - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn ordersCanMatch_(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethodsSidesKindsHowToCalls: &[u8],
        calldataBuy: &[u8],
        calldataSell: &[u8],
        replacementPatternBuy: &[u8],
        replacementPatternSell: &[u8],
        staticExtradataBuy: &[u8],
        staticExtradataSell: &[u8],
    ) -> bool {
           let bs  = buildOrderTypeArr2( addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethodsSidesKindsHowToCalls: &[u8],
        calldataBuy: &[u8],
        calldataSell: &[u8],
        replacementPatternBuy: &[u8],
        replacementPatternSell: &[u8],
        staticExtradataBuy: &[u8],
        staticExtradataSell: &[u8]);
      Self::ordersCanMatch(bs[0], bs[1])
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
        buyCalldata: &[u8],
        buyReplacementPattern: &[u8],
        sellCalldata: &[u8],
        sellReplacementPattern: &[u8],
    ) -> bool {
        if buyReplacementPattern.len() > 0 {
            Self::guardedArrayReplace(&mut buyCalldata, sellCalldata, buyReplacementPattern);
        }
        if sellReplacementPattern.len() > 0 {
            Self::guardedArrayReplace(&mut sellCalldata, buyCalldata, sellReplacementPattern);
        }

        Self::arrayEq(buyCalldata, sellCalldata)
    }

    /**
     * @dev Call calculateMatchPrice - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn calculateMatchPrice_(
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethodsSidesKindsHowToCalls: &[u8],
        calldataBuy: &[u8],
        calldataSell: &[u8],
        replacementPatternBuy: &[u8],
        replacementPatternSell: &[u8],
        staticExtradataBuy: &[u8],
        staticExtradataSell: &[u8],
    ) -> Result<u64,Error<T>> {
    let bs  = buildOrderTypeArr2( addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethodsSidesKindsHowToCalls: &[u8],
        calldataBuy: &[u8],
        calldataSell: &[u8],
        replacementPatternBuy: &[u8],
        replacementPatternSell: &[u8],
        staticExtradataBuy: &[u8],
        staticExtradataSell: &[u8]);
      Self::calculateMatchPrice(bs[0], bs[1])
    }

    /**
     * @dev Call atomicMatch - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn atomicMatch_(origin:T::Origin,
        addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethodsSidesKindsHowToCalls: &[u8],
        calldataBuy: &[u8],
        calldataSell: &[u8],
        replacementPatternBuy: &[u8],
        replacementPatternSell: &[u8],
        staticExtradataBuy: &[u8],
        staticExtradataSell: &[u8],
        sig: Vec<T::Signature>,
        rssMetadata: &[u8],
    ) -> Result<(), Error<T>> {
let user = ensure_signed(origin) as T::AccountId;
     
let bs = buildOrderTypeArr2(addrs,
        uints,
        feeMethodsSidesKindsHowToCalls,
        calldataBuy,
        calldataSell,
        replacementPatternBuy,
        replacementPatternSell,
        staticExtradataBuy,
        staticExtradataSell);
 Self::atomicMatch(
            user,
            0,
            bs[0],
            sig[0],
            bs[1],
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
        MinimumMakerProtocolFee::put(newMinimumMakerProtocolFee);
        Ok(())
    }

    /**
     * @dev Change the minimum taker fee paid to the protocol (only:owner)
     * @param newMinimumTakerProtocolFee New fee to set in basis points
     */
    fn changeMinimumTakerProtocolFee(newMinimumTakerProtocolFee: u64) -> Result<(), Error<T>> {
        // onlyOwner
        MinimumTakerProtocolFee::put(newMinimumTakerProtocolFee);
        Ok(())
    }

    /**
     * @dev Change the protocol fee recipient (only:owner)
     * @param newProtocolFeeRecipient New protocol fee recipient AccountId
     */
    fn changeProtocolFeeRecipient(newProtocolFeeRecipient: &T::AccountId) -> Result<(), Error<T>> {
        // onlyOwner
        ProtocolFeeRecipient::put(newProtocolFeeRecipient.clone());
        Ok(())
    }

    /**
     * @dev Transfer tokens
     * @param token Token to transfer
     * @param from AccountId to charge fees
     * @param to AccountId to receive fees
     * @param amount Amount of protocol tokens to charge
     */
    fn transferTokens(
        token: &AccountId,
        from: &AccountId,
        to: &AccountId,
        amount: u64,
    ) -> Result<(), Error<T>> {
        if amount > 0 {
            // ensure!(TokenTransferProxy.transferFrom(token, from, to, amount), Error::<T>::OrderIdMissing);
            // let call = Box::new(Call::Balances(BalancesCall::transfer(6, 1)));
            // ensure!(Proxy::proxy(Origin::signed(2), 1, None, &call), Error::<T>::OrderIdMissing);
        }
        Ok(())
    }

    /**
     * @dev Charge a fee in protocol tokens
     * @param from AccountId to charge fees
     * @param to AccountId to receive fees
     * @param amount Amount of protocol tokens to charge
     */
    fn chargeProtocolFee(from: AccountId, to: AccountId, amount: u64) -> Result<(), Error<T>> {
      Self::transferTokens(ExchangeToken::get(), from, to, amount);
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
        keccak_256(&order.encode()).into()
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
        keccak_256(Self::hashOrder(&order)).to_vec()
    }

    /**
     * @dev Assert an order is valid and return its hash
     * @param order OrderType to validate
     * @param sig ECDSA signature
     */
    fn requireValidOrder(order: &OrderType<T::AccountId, T::Moment>, sig: &T::Signature) -> Result<Vec<u8>,Error<T>> {
        let hash: Vec<u8> = Self::hashToSign(&order);
        ensure!(Self::validateOrder(&hash, order, sig), Error::<T>::OrderIdMissing);
        Ok(hash)
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
            && (order.makerProtocolFee < MinimumMakerProtocolFee::get()
                || order.takerProtocolFee < MinimumTakerProtocolFee::get())
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
    fn validateOrder(hash: &[u8], order: &OrderType<T::AccountId, T::Moment>, sig: &T::Signature) -> bool {
        /* Not done in an if-conditional to prevent unnecessary ecrecover evaluation, which seems to happen even though it should short-circuit. */

        /* OrderType must have valid parameters. */
        if !Self::validateOrderParameters(&order) {
            return false;
        }

        /* OrderType must have not been canceled or already filled. */
        if CancelledOrFinalized::get(hash) {
            return false;
        }

        /* OrderType authentication. OrderType must be either:
        (a) previously approved */
        if ApprovedOrders::get(hash) {
            return true;
        }

        /* or (b) ECDSA-signed by maker. */
        // if ecrecover(hash, sig.v, sig.r, sig.s) == order.maker {
        //     return true;
        // }
        if Self::check_signature(&sig, &hash, order.maker()).is_ok() {
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
    fn approveOrder(origin:T::Origin,order: &OrderType<T::AccountId, T::Moment>, orderbookInclusionDesired: bool)->DispatchResult {
        /* CHECKS */
        let user = ensure_signed(origin)?;
        /* Assert sender is authorized to approve order. */
        ensure!(  user == order.maker, Error::<T>::OrderIdMissing);

        /* Calculate order hash. */
        let hash: Vec<u8> = Self::hashToSign(&order);

        /* Assert order has not already been approved. */
        ensure!(!ApprovedOrders::get(hash), Error::<T>::OrderIdMissing);

        /* EFFECTS */

        /* Mark order as approved. */
        ApprovedOrders::insert(hash, true);

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
Ok(())
    }

    /**
     * @dev Cancel an order, preventing it from being matched. Must be called by the maker of the order
     * @param order OrderType to cancel
     * @param sig ECDSA signature
     */
    fn cancelOrder(origin:T::Origin,order: &OrderType<T::AccountId, T::Moment>, sig: &T::Signature) -> DispatchResult {
        /* CHECKS */
let user = ensure_signed(origin)?;
       

        /* Assert sender is authorized to cancel order. */
        ensure!(user == order.maker, Error::<T>::OrderIdMissing);

 /* Calculate order hash. */
        let hash = Self::requireValidOrder(order, sig)?;
        /* EFFECTS */
        /* Mark order as cancelled, preventing it from being matched. */
        CancelledOrFinalized::insert(hash, true);

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
    fn calculateMatchPrice(buy: OrderType<T::AccountId,T::Moment>, sell: OrderType<T::AccountId,T::Moment>) -> Result<u64,Error<T>> {
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
       let price:u64 =  if sell.feeRecipient != TOKEN_OWNER {
            sellPrice
        } else {
            buyPrice
        };

        Ok(price)
    }

    /**
     * @dev Execute all ERC20 token / Ether transfers associated with an order match (fees and buyer => transfer:seller)
     * @param buy Buy-side order
     * @param sell Sell-side order
     */
    fn executeFundsTransfer(msg_value: u64, buy: OrderType<T::AccountId,T::Moment>, sell: OrderType<T::AccountId,T::Moment>) -> Result<u64,Error<T>> {
        let originprotocolFeeRecipient = ProtocolFeeRecipient::get();
        /* Only payable in the special case of unwrapped Ether. */
        if sell.paymentToken != TOKEN_OWNER {
            ensure!(msg_value == 0, Error::<T>::OrderIdMissing);
        }

        /* Calculate match price. */
       let  price: u64 = Self::calculateMatchPrice(buy, sell)?;

        /* If paying using a token (Ether:not), transfer tokens. This is done prior to fee payments to that a seller will have tokens before being charged fees. */
        if price > 0 && sell.paymentToken != TOKEN_OWNER {
          Self::transferTokens(sell.paymentToken(), &buy.maker(), sell.maker(), price);
        }

        /* Amount that will be received by seller (Ether:for). */
        let receiveAmount: u64 = price;

        /* Amount that must be sent by buyer (Ether:for). */
        let requiredAmount: u64 = price;

        /* Determine maker/taker and charge fees accordingly. */
        if sell.feeRecipient != TOKEN_OWNER {
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
                    let makerRelayerFee: u64 = sell.makerRelayerFee * price / INVERSE_BASIS_POINT;
                    if sell.paymentToken == TOKEN_OWNER {
                        receiveAmount = receiveAmount - makerRelayerFee;
                        // sell.feeRecipient.transfer(makerRelayerFee);
                      Self::transferTokens(
                            &TOKEN_OWNER,
                            &TOKEN_OWNER,
                            &sell.feeRecipient,
                            makerRelayerFee,
                        );
                    } else {
                      Self::transferTokens(
                            sell.paymentToken(),
                            sell.maker(),
                            &sell.feeRecipient,
                            makerRelayerFee,
                        );
                    }
                }

                if sell.takerRelayerFee > 0 {
                    let takerRelayerFee: u64 = sell.takerRelayerFee * price / INVERSE_BASIS_POINT;
                    if sell.paymentToken == TOKEN_OWNER {
                        requiredAmount = requiredAmount + takerRelayerFee;
                        // sell.feeRecipient.transfer(takerRelayerFee);
                      Self::transferTokens(
                            &TOKEN_OWNER,
                            &TOKEN_OWNER,
                            &sell.feeRecipient,
                            takerRelayerFee,
                        );
                    } else {
                      Self::transferTokens(
                            sell.paymentToken(),
                            buy.maker(),
                            &sell.feeRecipient,
                            takerRelayerFee,
                        );
                    }
                }

                if sell.makerProtocolFee > 0 {
                    let makerProtocolFee: u64 = sell.makerProtocolFee * price / INVERSE_BASIS_POINT;
                    if sell.paymentToken == TOKEN_OWNER {
                        receiveAmount = receiveAmount - makerProtocolFee;
                        // ProtocolFeeRecipient.transfer(makerProtocolFee);
                      Self::transferTokens(
                            &TOKEN_OWNER,
                            &TOKEN_OWNER,
                            &originprotocolFeeRecipient,
                            makerProtocolFee,
                        );
                    } else {
                      Self::transferTokens(
                            sell.paymentToken(),
                            sell.maker(),
                            &originprotocolFeeRecipient,
                            makerProtocolFee,
                        );
                    }
                }

                if sell.takerProtocolFee > 0 {
                    let takerProtocolFee: u64 = sell.takerProtocolFee * price / INVERSE_BASIS_POINT;
                    if sell.paymentToken == TOKEN_OWNER {
                        requiredAmount = requiredAmount + takerProtocolFee;
                        // ProtocolFeeRecipient.transfer(takerProtocolFee);
                      Self::transferTokens(
                            &TOKEN_OWNER,
                            &TOKEN_OWNER,
                            &originprotocolFeeRecipient,
                            takerProtocolFee,
                        );
                    } else {
                      Self::transferTokens(
                            sell.paymentToken(),
                            buy.maker(),
                            &originprotocolFeeRecipient,
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
                ensure!(sell.paymentToken != TOKEN_OWNER, Error::<T>::OrderIdMissing);

                /* Assert taker fee is less than or equal to maximum fee specified by seller. */
                ensure!(
                    buy.takerProtocolFee <= sell.takerProtocolFee,
                    Error::<T>::OrderIdMissing
                );

                if buy.makerRelayerFee > 0 {
                   let makerRelayerFee =buy.makerRelayerFee * price / INVERSE_BASIS_POINT;
                  Self::transferTokens(
                        sell.paymentToken(),
                        buy.maker(),
                        &buy.feeRecipient,
                        makerRelayerFee,
                    );
                }

                if buy.takerRelayerFee > 0 {
                   let takerRelayerFee = buy.takerRelayerFee * price / INVERSE_BASIS_POINT;
                  Self::transferTokens(
                        sell.paymentToken(),
                        sell.maker(),
                        &buy.feeRecipient,
                        takerRelayerFee,
                    );
                }

                if buy.makerProtocolFee > 0 {
                   let makerProtocolFee = buy.makerProtocolFee * price / INVERSE_BASIS_POINT;
                  Self::transferTokens(
                        sell.paymentToken(),
                        buy.maker(),
                        &originprotocolFeeRecipient,
                        makerProtocolFee,
                    );
                }

                if buy.takerProtocolFee > 0 {
                    let takerProtocolFee = buy.takerProtocolFee * price / INVERSE_BASIS_POINT;
                  Self::transferTokens(
                        sell.paymentToken(),
                        sell.maker,
                        &originprotocolFeeRecipient,
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

        if sell.paymentToken == TOKEN_OWNER {
            /* Special-case Ether, order must be matched by buyer. */
            ensure!(msg_value >= requiredAmount, Error::<T>::OrderIdMissing);
            // sell.maker.transfer(receiveAmount);
          Self::transferTokens(TOKEN_OWNER, TOKEN_OWNER, sell.maker, receiveAmount);
            /* Allow overshoot for variable-price auctions, refund difference. */
            let diff: u64 = msg_value - requiredAmount;
            if diff > 0 {
                // buy.maker.transfer(diff);
              Self::transferTokens(&TOKEN_OWNER, &TOKEN_OWNER, buy.maker(), diff);
            }
        }

        /* This contract should never hold Ether, however, we cannot assert this, since it is impossible to prevent anyone from sending Ether e.g. with selfdestruct. */

         Ok(price)
    }

    /**
     * @dev Return whether or not two orders can be matched with each other by basic parameters (does not check order signatures / calldata or perform calls:static)
     * @param buy Buy-side order
     * @param sell Sell-side order
     * @return Whether or not the two orders can be matched
     */
    fn ordersCanMatch(buy: OrderType<T::AccountId,T::Moment>, sell: OrderType<T::AccountId,T::Moment>) -> bool {
            //  Must be opposite-side.
            (buy.side == Side::Buy && sell.side == Side::Sell) &&
            // Must use same fee method.
            (buy.feeMethod == sell.feeMethod) &&
            // Must use same payment token. 
            (buy.paymentToken == sell.paymentToken) &&
            // Must match maker/taker addresses. 
            (sell.taker == TOKEN_OWNER || sell.taker == buy.maker) &&
            (buy.taker == TOKEN_OWNER || buy.taker == sell.maker) &&
            // One must be maker and the other must be taker (no bool XOR Solidity:in). 
            ((sell.feeRecipient == TOKEN_OWNER && buy.feeRecipient != TOKEN_OWNER) || (sell.feeRecipient != TOKEN_OWNER && buy.feeRecipient == TOKEN_OWNER)) &&
            // Must match target. 
            (buy.target == sell.target) &&
            // Must match howToCall. 
            (buy.howToCall == sell.howToCall) &&
            // Buy-side order must be settleable. 
            Self::canSettleOrder(buy.listingTime, buy.expirationTime) &&
            // Sell-side order must be settleable. 
            Self::canSettleOrder(sell.listingTime, sell.expirationTime)
    }

    /**
     * @dev Atomically match two orders, ensuring validity of the match, and execute all associated state transitions. Protected against reentrancy by a contract-global lock.
     * @param buy Buy-side order
     * @param buySig Buy-side order signature
     * @param sell Sell-side order
     * @param sellSig Sell-side order signature
     */
    fn atomicMatch(
        msg_sender: T::AccountId,
        msg_value: u64,
        buy: OrderType<T::AccountId,T::Moment>,
        buySig: T::Signature,
        sell: OrderType<T::AccountId,T::Moment>,
        sellSig: T::Signature,
        metadata: &[u8],
    ) -> Result<(),Error<T>>
{
        //reentrancyGuard
        /* CHECKS */

        /* Ensure buy order validity and calculate hash if necessary. */
        let buyHash: Vec<u8> = vec![];
        if buy.maker == msg_sender {
            ensure!(Self::validateOrderParameters(&buy), Error::<T>::OrderIdMissing);
        } else {
            buyHash = Self::requireValidOrder(&buy, &buySig)?;
        }

        /* Ensure sell order validity and calculate hash if necessary. */
        let sellHash: Vec<u8> = vec![];
        if sell.maker == msg_sender {
            ensure!(Self::validateOrderParameters(&sell), Error::<T>::OrderIdMissing);
        } else {
            sellHash = Self::requireValidOrder(&sell, &sellSig)?;
        }

        /* Must be matchable. */
        ensure!(Self::ordersCanMatch(buy, sell), Error::<T>::OrderIdMissing);

        /* Target must exist (prevent malicious selfdestructs just prior to settlement:order). */
        // u64 size;
        // AccountId target = sell.target;
        // assembly {
        //     size := extcodesize(target)
        // }
        // ensure!(size > 0, Error::<T>::OrderIdMissing);

        /* Must match calldata after replacement, if specified. */
        if buy.replacementPattern.len() > 0 {
            Self::guardedArrayReplace(&mut buy.calldata, &sell.calldata, &buy.replacementPattern);
        }
        if sell.replacementPattern.len() > 0 {
            Self::guardedArrayReplace(&mut sell.calldata, &buy.calldata, &sell.replacementPattern);
        }
        ensure!(
            Self::arrayEq(buy.calldata, sell.calldata),
            Error::<T>::OrderIdMissing
        );

        // /* Retrieve delegateProxy contract. */
        // OwnableDelegateProxy delegateProxy = Registry.proxies(sell.maker);

        // /* Proxy must exist. */
        // ensure!(delegateProxy != TOKEN_OWNER, Error::<T>::OrderIdMissing);

        // /* Assert implementation. */
        // ensure!(delegateProxy.implementation() == Registry.delegateProxyImplementation(), Error::<T>::OrderIdMissing);

        // /* Access the passthrough AuthenticatedProxy. */
        // AuthenticatedProxy proxy = AuthenticatedProxy(delegateProxy);

        /* EFFECTS */

        /* Mark previously signed or approved orders as finalized. */
        let buymaker:T::AccountId = buy.maker;
        if msg_sender != buymaker {
            CancelledOrFinalized::insert(buyHash, true);
        }
        let sellmaker:T::AccountId = sell.maker;
        if msg_sender != sellmaker {
            CancelledOrFinalized::insert(sellHash, true);
        }

        /* INTERACTIONS */

        /* Execute funds transfer and pay fees. */
        let price: u64 = Self::executeFundsTransfer(msg_value, buy, sell)?;

        /* Execute specified call through proxy. */
        //TODO
        // ensure!(
        //     proxy.proxy(sell.target, sell.howToCall, sell.calldata),
        //     Error::<T>::OrderIdMissing
        // );

        /* Static calls are intentionally done after the effectful call so they can check resulting state. */

        /* Handle buy-side static call if specified. */
        // if buy.staticTarget != TOKEN_OWNER {
        //     ensure!(Self::staticCall(buy.staticTarget, sell.calldata, buy.staticExtradata), Error::<T>::OrderIdMissing);
        // }

        // /* Handle sell-side static call if specified. */
        // if sell.staticTarget != TOKEN_OWNER {
        //     ensure!(Self::staticCall(sell.staticTarget, sell.calldata, sell.staticExtradata), Error::<T>::OrderIdMissing);
        // }

        /* Log match event. */
        Self::deposit_event(RawEvent::OrdersMatched(
            buyHash,
            sellHash,
            if sell.feeRecipient != TOKEN_OWNER {
                sell.maker
            } else {
                buy.maker
            },
            if sell.feeRecipient != TOKEN_OWNER {
                buy.maker
            } else {
                sell.maker
            },
            price,
            metadata.to_vec(),
        ));

        Ok(())
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
        saleKind == SaleKind::FixedPrice || expirationTime > 0
    }

    /**
     * @dev Return whether or not an order can be settled
     * @dev Precondition: parameters have passed validateParameters
     * @param listingTime OrderType listing time
     * @param expirationTime OrderType expiration time
     */
    fn canSettleOrder(listingTime: u64, expirationTime: u64) -> bool {
        let now:u64 =  0;//<system::Module<T>>::block_number() ;//<timestamp::Module<T>>::now();
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
            let now :i64 = 0;// <system::Module<T>>::block_number();//<timestamp::Module<T>>::now() ;
            let diff: i64 = extra as i64 * (now - listingTime as i64) / (expirationTime as i64 - listingTime as i64);
            if side == Side::Sell {
                /* Sell-side - start price: basePrice. End price: basePrice - extra. */
                return (basePrice as i64  - diff) as u64;
            } else {
                /* Buy-side - start price: basePrice. End price: basePrice + extra. */
                return basePrice  + diff as u64;
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
        desired: &[u8],
        mask: &[u8],
    ) -> Result<(), Error<T>> {
        ensure!(array.len() == desired.len(), Error::<T>::OrderIdMissing);
        ensure!(array.len() == mask.len(), Error::<T>::OrderIdMissing);

        for (i, &item) in array.iter().enumerate() {
            /* Conceptually: array[i] = (!mask[i] && array[i]) || (mask[i] && desired[i]), bitwise in word chunks. */
            array[i] = (!mask[i] & array[i]) | (mask[i] & desired[i]) ;
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
    fn arrayEq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }

        a == b
    }


pub fn buildOrderTypeArr(
 addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethod: FeeMethod,
        side: Side,
        saleKind: SaleKind,
        howToCall: HowToCall,
        calldata: &[u8],
        replacementPattern: &[u8],
        staticExtradata: &[u8],) -> OrderType<T::AccountId, T::Moment>
{
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
            uints[8])
}


pub fn buildOrderTypeArr2(
   addrs: Vec<T::AccountId>,
        uints: Vec<u64>,
        feeMethodsSidesKindsHowToCalls: &[u8],
        calldataBuy: &[u8],
        calldataSell: &[u8],
        replacementPatternBuy: &[u8],
        replacementPatternSell: &[u8],
        staticExtradataBuy: &[u8],
        staticExtradataSell: &[u8],) -> Vec<OrderType<T::AccountId, T::Moment>>
{
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
            calldataBuy.to_vec(),
            replacementPatternBuy.to_vec(),
            addrs[5].to_vec(),
            staticExtradataBuy.to_vec(),
            addrs[6],
            uints[4],
            uints[5],
            uints[6],
            uints[7],
            uints[8]);
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
            calldataSell.to_vec(),
            replacementPatternSell.to_vec(),
            addrs[12],
            staticExtradataSell.to_vec(),
            addrs[13],
            uints[13],
            uints[14],
            uints[15],
            uints[16],
            uints[17],
        );
vec![buy,sell]
}
pub fn buildOrderType(
    exchange:T::AccountId,
    /* OrderType maker AccountId. */
    maker:T::AccountId,
    /* OrderType taker AccountId, if specified. */
    taker:T::AccountId,
    /* Maker relayer fee of the order, unused for taker order. */
    makerRelayerFee: u64,
    /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
    takerRelayerFee: u64,
    /* Maker protocol fee of the order, unused for taker order. */
    makerProtocolFee: u64,
    /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
    takerProtocolFee: u64,
    /* OrderType fee recipient or zero AccountId for taker order. */
    feeRecipient:T::AccountId,
    /* Fee method (protocol token or split fee). */
    feeMethod: FeeMethod,
    /* Side (buy/sell). */
    side: Side,
    /* Kind of sale. */
    saleKind: SaleKind,
    /* Target. */
    target:T::AccountId,
    /* Vec<u8>. */
    howToCall: HowToCall,
    /* Calldata. */
    calldata: Bytes,
    /* Calldata replacement pattern, or an empty byte array for no replacement. */
    replacementPattern: Bytes,
    /* Static call target, zero-AccountId for no static call. */
    staticTarget:T::AccountId,
    /* Static call extra data. */
    staticExtradata: Bytes,
    /* Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether. */
    paymentToken:T::AccountId,
    /* Base price of the order (in paymentTokens). */
    basePrice: u64,
    /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
    extra: u64,
    /* Listing timestamp. */
    listingTime: u64,
    /* Expiration timestamp - 0 for no expiry. */
    expirationTime: u64,
    /* OrderType salt, used to prevent duplicate hashes. */
    salt: u64) -> OrderType<T::AccountId, T::Moment> {
        OrderType::<T::AccountId,T::Moment>::new(
  exchange,
    /* OrderType maker AccountId. */
  maker,
    /* OrderType taker AccountId, if specified. */
  taker,
    /* Maker relayer fee of the order, unused for taker order. */
  makerRelayerFee,
    /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
takerRelayerFee,
    /* Maker protocol fee of the order, unused for taker order. */
makerProtocolFee,
    /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
takerProtocolFee,
    /* OrderType fee recipient or zero AccountId for taker order. */
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
    /* Static call target, zero-AccountId for no static call. */
staticTarget,
    /* Static call extra data. */
staticExtradata,
    /* Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether. */
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
        )
}
   
  

}


impl<AccountId, Moment> OrderType<AccountId, Moment>
where
    AccountId: Default,
    Moment: Default,
{


pub fn new(

    exchange: AccountId,
    /* OrderType maker AccountId. */
    maker: AccountId,
    /* OrderType taker AccountId, if specified. */
    taker: AccountId,
    /* Maker relayer fee of the order, unused for taker order. */
    makerRelayerFee: u64,
    /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
    takerRelayerFee: u64,
    /* Maker protocol fee of the order, unused for taker order. */
    makerProtocolFee: u64,
    /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
    takerProtocolFee: u64,
    /* OrderType fee recipient or zero AccountId for taker order. */
    feeRecipient: AccountId,
    /* Fee method (protocol token or split fee). */
    feeMethod: FeeMethod,
    /* Side (buy/sell). */
    side: Side,
    /* Kind of sale. */
    saleKind: SaleKind,
    /* Target. */
    target: AccountId,
    /* Vec<u8>. */
    howToCall: HowToCall,
    /* Calldata. */
    calldata: Bytes,
    /* Calldata replacement pattern, or an empty byte array for no replacement. */
    replacementPattern: Bytes,
    /* Static call target, zero-AccountId for no static call. */
    staticTarget: AccountId,
    /* Static call extra data. */
    staticExtradata: Bytes,
    /* Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether. */
    paymentToken: AccountId,
    /* Base price of the order (in paymentTokens). */
    basePrice: u64,
    /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
    extra: u64,
    /* Listing timestamp. */
    listingTime: u64,
    /* Expiration timestamp - 0 for no expiry. */
    expirationTime: u64,
    /* OrderType salt, used to prevent duplicate hashes. */
    salt: u64) -> Self {
        Self{
    index:0,
    exchange:exchange,
    /* OrderType maker AccountId. */
    maker:maker,
    /* OrderType taker AccountId, if specified. */
    taker:taker,
    /* Maker relayer fee of the order, unused for taker order. */
    makerRelayerFee:makerRelayerFee,
    /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
  takerRelayerFee:takerRelayerFee,
    /* Maker protocol fee of the order, unused for taker order. */
  makerProtocolFee:makerProtocolFee,
    /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
  takerProtocolFee:takerProtocolFee,
    /* OrderType fee recipient or zero AccountId for taker order. */
  feeRecipient:feeRecipient,
    /* Fee method (protocol token or split fee). */
  feeMethod:feeMethod,
    /* Side (buy/sell). */
  side:side,
    /* Kind of sale. */
  saleKind:saleKind,
    /* Target. */
  target:target,
    /* Vec<u8>. */
  howToCall:howToCall,
    /* Calldata. */
  calldata:calldata,
    /* Calldata replacement pattern, or an empty byte array for no replacement. */
  replacementPattern:replacementPattern,
    /* Static call target, zero-AccountId for no static call. */
  staticTarget:staticTarget,
    /* Static call extra data. */
  staticExtradata:staticExtradata,
    /* Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether. */
  paymentToken:paymentToken,
    /* Base price of the order (in paymentTokens). */
  basePrice:basePrice,
    /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
  extra:extra,
    /* Listing timestamp. */
  listingTime:listingTime,
    /* Expiration timestamp - 0 for no expiry. */
  expirationTime:expirationTime,
    /* OrderType salt, used to prevent duplicate hashes. */
  salt:salt,
registered: timestamp::now(),
}
    }

    pub fn maker(&self) -> &[u8] {
        &self.maker
    }

   pub fn taker(&self) -> &[u8] {
        &self.taker
    }

 pub fn paymentToken(&self) -> &[u8] {
        &self.paymentToken
    }
}

#[derive(Default)]
pub struct OrderTypeBuilder<AccountId, Moment>
where
    AccountId: Default,
    Moment: Default,
{
pub index: u64,
  
    // /* An order on the exchange. */
    /* Exchange AccountId, intended as a versioning mechanism. */
    pub exchange: AccountId,
    /* OrderType maker AccountId. */
    pub maker: AccountId,
    /* OrderType taker AccountId, if specified. */
    pub taker: AccountId,
    /* Maker relayer fee of the order, unused for taker order. */
    pub makerRelayerFee: u64,
    /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
    pub takerRelayerFee: u64,
    /* Maker protocol fee of the order, unused for taker order. */
    pub makerProtocolFee: u64,
    /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
    pub takerProtocolFee: u64,
    /* OrderType fee recipient or zero AccountId for taker order. */
    pub feeRecipient: AccountId,
    /* Fee method (protocol token or split fee). */
    pub feeMethod: FeeMethod,
    /* Side (buy/sell). */
    pub side: Side,
    /* Kind of sale. */
    pub saleKind: SaleKind,
    /* Target. */
    pub target: AccountId,
    /* Vec<u8>. */
    pub howToCall: HowToCall,
    /* Calldata. */
    pub calldata: Bytes,
    /* Calldata replacement pattern, or an empty byte array for no replacement. */
    pub replacementPattern: Bytes,
    /* Static call target, zero-AccountId for no static call. */
    pub staticTarget: AccountId,
    /* Static call extra data. */
    pub staticExtradata: Bytes,
    /* Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether. */
    pub paymentToken: AccountId,
    /* Base price of the order (in paymentTokens). */
    pub basePrice: u64,
    /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
    pub extra: u64,
    /* Listing timestamp. */
    pub listingTime: u64,
    /* Expiration timestamp - 0 for no expiry. */
    pub expirationTime: u64,
    /* OrderType salt, used to prevent duplicate hashes. */
    pub salt: u64,
    pub registered: Moment,
}

impl<AccountId, Moment> OrderTypeBuilder<AccountId, Moment>
where
    AccountId: Default,
    Moment: Default,
{


pub fn new(

    exchange: AccountId,
    /* OrderType maker AccountId. */
    maker: AccountId,
    /* OrderType taker AccountId, if specified. */
    taker: AccountId,
    /* Maker relayer fee of the order, unused for taker order. */
    makerRelayerFee: u64,
    /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
    takerRelayerFee: u64,
    /* Maker protocol fee of the order, unused for taker order. */
    makerProtocolFee: u64,
    /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
    takerProtocolFee: u64,
    /* OrderType fee recipient or zero AccountId for taker order. */
    feeRecipient: AccountId,
    /* Fee method (protocol token or split fee). */
    feeMethod: FeeMethod,
    /* Side (buy/sell). */
    side: Side,
    /* Kind of sale. */
    saleKind: SaleKind,
    /* Target. */
    target: AccountId,
    /* Vec<u8>. */
    howToCall: HowToCall,
    /* Calldata. */
    calldata: Bytes,
    /* Calldata replacement pattern, or an empty byte array for no replacement. */
    replacementPattern: Bytes,
    /* Static call target, zero-AccountId for no static call. */
    staticTarget: AccountId,
    /* Static call extra data. */
    staticExtradata: Bytes,
    /* Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether. */
    paymentToken: AccountId,
    /* Base price of the order (in paymentTokens). */
    basePrice: u64,
    /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
    extra: u64,
    /* Listing timestamp. */
    listingTime: u64,
    /* Expiration timestamp - 0 for no expiry. */
    expirationTime: u64,
    /* OrderType salt, used to prevent duplicate hashes. */
    salt: u64) -> Self{
Self {
    index:0,
    exchange:exchange,
    /* OrderType maker AccountId. */
    maker:maker,
    /* OrderType taker AccountId, if specified. */
    taker:taker,
    /* Maker relayer fee of the order, unused for taker order. */
    makerRelayerFee:makerRelayerFee,
    /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
  takerRelayerFee:takerRelayerFee,
    /* Maker protocol fee of the order, unused for taker order. */
  makerProtocolFee:makerProtocolFee,
    /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
  takerProtocolFee:takerProtocolFee,
    /* OrderType fee recipient or zero AccountId for taker order. */
  feeRecipient:feeRecipient,
    /* Fee method (protocol token or split fee). */
  feeMethod:feeMethod,
    /* Side (buy/sell). */
  side:side,
    /* Kind of sale. */
  saleKind:saleKind,
    /* Target. */
  target:target,
    /* Vec<u8>. */
  howToCall:howToCall,
    /* Calldata. */
  calldata:calldata,
    /* Calldata replacement pattern, or an empty byte array for no replacement. */
  replacementPattern:replacementPattern,
    /* Static call target, zero-AccountId for no static call. */
  staticTarget:staticTarget,
    /* Static call extra data. */
  staticExtradata:staticExtradata,
    /* Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether. */
  paymentToken:paymentToken,
    /* Base price of the order (in paymentTokens). */
  basePrice:basePrice,
    /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
  extra:extra,
    /* Listing timestamp. */
  listingTime:listingTime,
    /* Expiration timestamp - 0 for no expiry. */
  expirationTime:expirationTime,
    /* OrderType salt, used to prevent duplicate hashes. */
  salt:salt,
registered: timestamp::now(),
}
}

    // pub fn index_by(mut self, index: u64) -> Self {
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

    pub fn build(self) -> OrderType<AccountId, Moment> {
           OrderType::<AccountId,Moment> {
    index:self.index,
    exchange:self.exchange,
    /* OrderType maker AccountId. */
    maker:self.maker,
    /* OrderType taker AccountId, if specified. */
    taker:self.taker,
    /* Maker relayer fee of the order, unused for taker order. */
    makerRelayerFee:self.makerRelayerFee,
    /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
  takerRelayerFee:self.takerRelayerFee,
    /* Maker protocol fee of the order, unused for taker order. */
  makerProtocolFee:self.makerProtocolFee,
    /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
  takerProtocolFee:self.takerProtocolFee,
    /* OrderType fee recipient or zero AccountId for taker order. */
  feeRecipient:self.feeRecipient,
    /* Fee method (protocol token or split fee). */
  feeMethod:self.feeMethod,
    /* Side (buy/sell). */
  side:self.side,
    /* Kind of sale. */
  saleKind:self.saleKind,
    /* Target. */
  target:self.target,
    /* Vec<u8>. */
  howToCall:self.howToCall,
    /* Calldata. */
  calldata:self.calldata,
    /* Calldata replacement pattern, or an empty byte array for no replacement. */
  replacementPattern:self.replacementPattern,
    /* Static call target, zero-AccountId for no static call. */
  staticTarget:self.staticTarget,
    /* Static call extra data. */
  staticExtradata:self.staticExtradata,
    /* Token used to pay for the order, or the zero-AccountId as a sentinel value for Ether. */
  paymentToken:self.paymentToken,
    /* Base price of the order (in paymentTokens). */
  basePrice:self.basePrice,
    /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
  extra:self.extra,
    /* Listing timestamp. */
  listingTime:self.listingTime,
    /* Expiration timestamp - 0 for no expiry. */
  expirationTime:self.expirationTime,
    /* OrderType salt, used to prevent duplicate hashes. */
  salt:self.salt,
registered:self.registered,
}
    }


    }
