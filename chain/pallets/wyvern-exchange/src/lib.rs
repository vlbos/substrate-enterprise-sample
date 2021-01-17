//! # Substrate Enterprise Sample - Order Post example pallet

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::result::Result;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, sp_runtime::RuntimeDebug,
    sp_std::collections::btree_set::BTreeSet, sp_std::prelude::*,
};
// traits::EnsureOrigin,
use frame_system::{self as system, ensure_signed};
	use pallet_balances::Call as BalancesCall;

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
     pub const INVERSE_BASIS_POINT:usize = 10000;

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
pub enum Side {
  FixedPrice, 
DutchAuction,
}

    // /* Fee method: protocol fee or split fee. */
    // enum FeeMethod { ProtocolFee, SplitFee }
#[derive(Encode, Decode, Debug, Clone, PartialEq)]
pub enum FeeMethod {
ProtocolFee, 
SplitFee ,
}

impl Default for Side { fn default() -> Self { Self::Buy } }

impl Default for SaleKind { fn default() -> Self { Self::FixedPrice } }
impl Default for FeeMethod { fn default() -> Self { Self::ProtocolFee } }




///exchange core begin

// Order contains master data (aka class-level) about a trade item.
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
        exchange:Address,
        /* Order maker Address. */
        maker:Address,
        /* Order taker Address, if specified. */
        taker:Address,
        /* Maker relayer fee of the order, unused for taker order. */
        makerRelayerFee:u64,
        /* Taker relayer fee of the order, or maximum taker fee for a taker order. */
        takerRelayerFee:u64,
        /* Maker protocol fee of the order, unused for taker order. */
        makerProtocolFee:u64,
        /* Taker protocol fee of the order, or maximum taker fee for a taker order. */
        takerProtocolFee:u64,
        /* Order fee recipient or zero Address for taker order. */
        feeRecipient:Address,
        /* Fee method (protocol token or split fee). */
        feeMethod:FeeMethod,
        /* Side (buy/sell). */
         side:Side,
        /* Kind of sale. */
        saleKind:SaleKind;
        /* Target. */
        target:Address,
        /* HowToCall. */
        howToCall:Vec<u8>;
        /* Calldata. */
        calldata:Bytes,
        /* Calldata replacement pattern, or an empty byte array for no replacement. */
        replacementPattern:Bytes,
        /* Static call target, zero-Address for no static call. */
        staticTarget:Address,
        /* Static call extra data. */
        staticExtradata:Bytes,
        /* Token used to pay for the order, or the zero-Address as a sentinel value for Ether. */
        paymentToken:Address,
        /* Base price of the order (in paymentTokens). */
        basePrice:u64,
        /* Auction extra parameter - minimum bid increment for English auctions, starting/ending price difference. */
        extra:u64,
        /* Listing timestamp. */
        listingTime:u64,
        /* Expiration timestamp - 0 for no expiry. */
        expirationTime:u64,
        /* Order salt, used to prevent duplicate hashes. */
        salt:u64,

}

//exchange core

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    // type CreateRoleOrigin: EnsureOrigin<Self::Origin>;
}

decl_storage! {
    trait Store for Module<T: Trait> as OrderRegistry {
        NextOrderIndex: u64;
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
    // mapping(bytes32 => bool) public cancelledOrFinalized;
  pub cancelledOrFinalized get(fn cancelled_or_finalized): map hasher(blake2_128_concat) Vec<u8> => bool;
    // /* Orders verified by on-chain approval (alternative to ECDSA signatures so that smart contracts can place orders directly). */
    // mapping(bytes32 => bool) public approvedOrders;
  pub approvedOrders get(fn approved_orders): map hasher(blake2_128_concat) Vec<u8> => bool;
    // /* For split fee orders, minimum required protocol maker fee, in basis points. Paid to owner (who can change it). */
    // uint public minimumMakerProtocolFee = 0;
pub minimumMakerProtocolFee:u64;
    // /* For split fee orders, minimum required protocol taker fee, in basis points. Paid to owner (who can change it). */
    // uint public minimumTakerProtocolFee = 0;
pub minimumTakerProtocolFee:u64;
    // /* Recipient of protocol fees. */
    // address public protocolFeeRecipient;
pub protocolFeeRecipient:Address;


 }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
    // event OrderApprovedPartOne    (bytes32 indexed hash, address exchange, address indexed maker, address taker, 
// uint makerRelayerFee, uint takerRelayerFee, uint makerProtocolFee, uint takerProtocolFee, 
// address indexed feeRecipient, FeeMethod feeMethod, SaleKindInterface.Side side, SaleKindInterface.SaleKind saleKind, address target);
    // event OrderApprovedPartTwo    (bytes32 indexed hash, AuthenticatedProxy.HowToCall howToCall, bytes calldata, bytes replacementPattern, 
// address staticTarget, bytes staticExtradata, address paymentToken, uint basePrice, 
// uint extra, uint listingTime, uint expirationTime, uint salt, bool orderbookInclusionDesired);
    // event OrderCancelled          (bytes32 indexed hash);
    // event OrdersMatched           (bytes32 buyHash, bytes32 sellHash, address indexed maker, address indexed taker, uint price, bytes32 indexed metadata);
OrderApprovedPartOne(Vec<u8>, AccountId, AccountId,AccountId,
u64,u64, u64,u64,
AccountId,FeeMethod,Side,SaleKind, AccountId),
OrderApprovedPartTwo(Vec<u8>, Vec<u8>,Vec<u8>, Vec<u8>, 
AccountId,Vec<u8>,AccountId,Vec<u8>, 
u64, u64,u64,u64,bool),
OrderCancelled(Vec<u8>),       
 OrdersMatched(Vec<u8>, Vec<u8>, AccountId,AccountId,u64,Vec<u8>),
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

        #[weight = 10_000]
        pub fn post_order(origin, id: OrderId, owner: T::AccountId, fields: Option<Vec<OrderField>>) -> dispatch::DispatchResult {
            // T::CreateRoleOrigin::ensure_origin(origin.clone())?;
            let who = ensure_signed(origin)?;

            // Validate order ID
            Self::validate_order_id(&id)?;

            // Validate order fields
            Self::validate_order_fields(&fields)?;

            // Check order doesn't exist yet (1 DB read)
            Self::validate_new_order(&id)?;



            // TODO: if organization has an attribute w/ GS1 Company prefix,
            //       additional validation could be applied to the order ID
            //       to ensure its validity (same company prefix as org).

            // Generate next collection ID
            let next_id = NextOrderIndex::get()
                .checked_add(1)
                .expect("order id error");

            NextOrderIndex::put(next_id);

if let Some(fields) = &fields {
            for field in fields {
            let mut index_arr: Vec<u64> = Vec::new();

            if <OrdersOfOrganization>::contains_key(field.name(),field.value())
            {
                index_arr = <OrdersOfOrganization>::get(field.name(),field.value());
                ensure!(!index_arr.contains(&next_id), "Account already has admin role");
            }

            index_arr.push(next_id);
            <OrdersOfOrganization>::insert(field.name(),field.value(), index_arr);

    //   <OrdersOfOrganization<T>>::append(&field, &next_id);
            }
   }


            // Create a order instance
            let order = Self::new_order()
                .identified_by(id.clone())
                .owned_by(owner.clone())
                .registered_on(<timestamp::Module<T>>::now())
                .with_fields(fields)
                .build();

            // Add order & ownerOf (3 DB writes)
            <Orders<T>>::insert(next_id, order);
            <Orderi>::insert(&id, next_id);
            // <OrdersOfOrganization<T>>::append(&owner, &id);


               <OwnerOf<T>>::insert(&id, &owner);

            Self::deposit_event(RawEvent::OrderPosted(who, id, owner));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
   

/// exchange 

    /**
     * @dev Call calculateFinalPrice - library fn exposed for testing.
     */
    fn calculateFinalPrice(side:Side, saleKind:SaleKind, basePrice:uint, extra:uint, listingTime:uint, expirationTime:uint)
        
        
        -> uint
    {
        return calculateFinalPrice(side, saleKind, basePrice, extra, listingTime, expirationTime);
    }

    /**
     * @dev Call hashOrder - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn hashOrder_(
        addrs:address[7],
        uints:uint[9],
        feeMethod:FeeMethod,
        side:Side,
        saleKind:SaleKind,
        howToCall:HowToCall,
        calldata:bytes,
        replacementPattern:bytes,
        staticExtradata:bytes)
        
        
        -> bytes32
    {
        return hashOrder(
          Order(addrs[0], addrs[1], addrs[2], uints[0], uints[1], uints[2], uints[3], addrs[3], feeMethod, side, saleKind, addrs[4], howToCall, calldata, replacementPattern, addrs[5], staticExtradata, ERC20(addrs[6]), uints[4], uints[5], uints[6], uints[7], uints[8])
        );
    }

    /**
     * @dev Call hashToSign - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn hashToSign_(
        addrs:address[7],
        uints:uint[9],
        feeMethod:FeeMethod,
        side:Side,
        saleKind:SaleKind,
        howToCall:HowToCall,
        calldata:bytes,
        replacementPattern:bytes,
        staticExtradata:bytes)
        
        
        -> bytes32
    { 
        return hashToSign(
          Order(addrs[0], addrs[1], addrs[2], uints[0], uints[1], uints[2], uints[3], addrs[3], feeMethod, side, saleKind, addrs[4], howToCall, calldata, replacementPattern, addrs[5], staticExtradata, ERC20(addrs[6]), uints[4], uints[5], uints[6], uints[7], uints[8])
        );
    }

    /**
     * @dev Call validateOrderParameters - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn validateOrderParameters_ (
        addrs:address[7],
        uints:uint[9],
        feeMethod:FeeMethod,
        side:Side,
        saleKind:SaleKind,
        howToCall:Vec<u8>,
        calldata:bytes,
        replacementPattern:bytes,
        staticExtradata:bytes)
        
        
        -> bool
    {
        Order  order = Order(addrs[0], addrs[1], addrs[2], uints[0], uints[1], uints[2], uints[3], addrs[3], feeMethod, side, saleKind, addrs[4], howToCall, calldata, replacementPattern, addrs[5], staticExtradata, ERC20(addrs[6]), uints[4], uints[5], uints[6], uints[7], uints[8]);
        return validateOrderParameters(
          order
        );
    }

    /**
     * @dev Call validateOrder - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn validateOrder_ (
        addrs:address[7],
        uints:uint[9],
        feeMethod:FeeMethod,
        side:Side,
        saleKind:SaleKind,
        howToCall:HowToCall,
        calldata:bytes,
        replacementPattern:bytes,
        staticExtradata:bytes,
        v:uint8,
        r:bytes32,
        s:bytes32)
        
        
        -> bool
    {
        Order  order = Order(addrs[0], addrs[1], addrs[2], uints[0], uints[1], uints[2], uints[3], addrs[3], feeMethod, side, saleKind, addrs[4], howToCall, calldata, replacementPattern, addrs[5], staticExtradata, ERC20(addrs[6]), uints[4], uints[5], uints[6], uints[7], uints[8]);
        return validateOrder(
          hashToSign(order),
          order,
          Sig(v, r, s)
        );
    }

    /**
     * @dev Call approveOrder - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn approveOrder_ (
        addrs:address[7],
        uints:uint[9],
        feeMethod:FeeMethod,
        side:Side,
        saleKind:SaleKind,
        howToCall:HowToCall,
        calldata:bytes,
        replacementPattern:bytes,
        staticExtradata:bytes,
        orderbookInclusionDesired:bool) 
        
    {
        Order  order = Order(addrs[0], addrs[1], addrs[2], uints[0], uints[1], uints[2], uints[3], addrs[3], feeMethod, side, saleKind, addrs[4], howToCall, calldata, replacementPattern, addrs[5], staticExtradata, ERC20(addrs[6]), uints[4], uints[5], uints[6], uints[7], uints[8]);
        return approveOrder(order, orderbookInclusionDesired);
    }

    /**
     * @dev Call cancelOrder - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn cancelOrder_(
        addrs:address[7],
        uints:uint[9],
        feeMethod:FeeMethod,
        side:Side,
        saleKind:SaleKind,
        howToCall:HowToCall,
        calldata:bytes,
        replacementPattern:bytes,
        staticExtradata:bytes,
        v:uint8,
        r:bytes32,
        s:bytes32)
        
    {

        return cancelOrder(
          Order(addrs[0], addrs[1], addrs[2], uints[0], uints[1], uints[2], uints[3], addrs[3], feeMethod, side, saleKind, addrs[4], howToCall, calldata, replacementPattern, addrs[5], staticExtradata, ERC20(addrs[6]), uints[4], uints[5], uints[6], uints[7], uints[8]),
          Sig(v, r, s)
        );
    }

    /**
     * @dev Call calculateCurrentPrice - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn calculateCurrentPrice_(
        addrs:address[7],
        uints:uint[9],
        feeMethod:FeeMethod,
        side:Side,
        saleKind:SaleKind,
        howToCall:HowToCall,
        calldata:bytes,
        replacementPattern:bytes,
        staticExtradata:bytes)
        
        
        -> uint
    {
        return calculateCurrentPrice(
          Order(addrs[0], addrs[1], addrs[2], uints[0], uints[1], uints[2], uints[3], addrs[3], feeMethod, side, saleKind, addrs[4], howToCall, calldata, replacementPattern, addrs[5], staticExtradata, ERC20(addrs[6]), uints[4], uints[5], uints[6], uints[7], uints[8])
        );
    }

    /**
     * @dev Call ordersCanMatch - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn ordersCanMatch_(
        addrs:address[14],
        uints:uint[18],
        feeMethodsSidesKindsHowToCalls:uint8[8],
        calldataBuy:bytes,
        calldataSell:bytes,
        replacementPatternBuy:bytes,
        replacementPatternSell:bytes,
        staticExtradataBuy:bytes,
        staticExtradataSell:bytes)
        
        
        -> bool
    {
        Order  buy = Order(addrs[0], addrs[1], addrs[2], uints[0], uints[1], uints[2], uints[3], addrs[3], FeeMethod(feeMethodsSidesKindsHowToCalls[0]), Side(feeMethodsSidesKindsHowToCalls[1]), SaleKind(feeMethodsSidesKindsHowToCalls[2]), addrs[4], HowToCall(feeMethodsSidesKindsHowToCalls[3]), calldataBuy, replacementPatternBuy, addrs[5], staticExtradataBuy, ERC20(addrs[6]), uints[4], uints[5], uints[6], uints[7], uints[8]);
        Order  sell = Order(addrs[7], addrs[8], addrs[9], uints[9], uints[10], uints[11], uints[12], addrs[10], FeeMethod(feeMethodsSidesKindsHowToCalls[4]), Side(feeMethodsSidesKindsHowToCalls[5]), SaleKind(feeMethodsSidesKindsHowToCalls[6]), addrs[11], HowToCall(feeMethodsSidesKindsHowToCalls[7]), calldataSell, replacementPatternSell, addrs[12], staticExtradataSell, ERC20(addrs[13]), uints[13], uints[14], uints[15], uints[16], uints[17]);
        return ordersCanMatch(
          buy,
          sell
        );
    }

    /**
     * @dev Return whether or not two orders' calldata specifications can match
     * @param buyCalldata Buy-side order calldata
     * @param buyReplacementPattern Buy-side order calldata replacement mask
     * @param sellCalldata Sell-side order calldata
     * @param sellReplacementPattern Sell-side order calldata replacement mask
     * @return Whether the orders' calldata can be matched
     */
    fn orderCalldataCanMatch(buyCalldata:bytes, buyReplacementPattern:bytes, sellCalldata:bytes, sellReplacementPattern:bytes)
        
        
        -> bool
    {
        if (buyReplacementPattern.length > 0) {
          ArrayUtils.guardedArrayReplace(buyCalldata, sellCalldata, buyReplacementPattern);
        }
        if (sellReplacementPattern.length > 0) {
          ArrayUtils.guardedArrayReplace(sellCalldata, buyCalldata, sellReplacementPattern);
        }
        return ArrayUtils.arrayEq(buyCalldata, sellCalldata);
    }

    /**
     * @dev Call calculateMatchPrice - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn calculateMatchPrice_(
        addrs:address[14],
        uints:uint[18],
        feeMethodsSidesKindsHowToCalls:uint8[8],
        calldataBuy:bytes,
        calldataSell:bytes,
        replacementPatternBuy:bytes,
        replacementPatternSell:bytes,
        staticExtradataBuy:bytes,
        staticExtradataSell:bytes)
        
        
        -> uint
    {
        Order  buy = Order(addrs[0], addrs[1], addrs[2], uints[0], uints[1], uints[2], uints[3], addrs[3], FeeMethod(feeMethodsSidesKindsHowToCalls[0]), Side(feeMethodsSidesKindsHowToCalls[1]), SaleKind(feeMethodsSidesKindsHowToCalls[2]), addrs[4], HowToCall(feeMethodsSidesKindsHowToCalls[3]), calldataBuy, replacementPatternBuy, addrs[5], staticExtradataBuy, ERC20(addrs[6]), uints[4], uints[5], uints[6], uints[7], uints[8]);
        Order  sell = Order(addrs[7], addrs[8], addrs[9], uints[9], uints[10], uints[11], uints[12], addrs[10], FeeMethod(feeMethodsSidesKindsHowToCalls[4]), Side(feeMethodsSidesKindsHowToCalls[5]), SaleKind(feeMethodsSidesKindsHowToCalls[6]), addrs[11], HowToCall(feeMethodsSidesKindsHowToCalls[7]), calldataSell, replacementPatternSell, addrs[12], staticExtradataSell, ERC20(addrs[13]), uints[13], uints[14], uints[15], uints[16], uints[17]);
        return calculateMatchPrice(
          buy,
          sell
        );
    }

    /**
     * @dev Call atomicMatch - Solidity ABI encoding workaround:limitation, hopefully temporary.
     */
    fn atomicMatch_(
        addrs:address[14],
        uints:uint[18],
        feeMethodsSidesKindsHowToCalls:uint8[8],
        calldataBuy:bytes,
        calldataSell:bytes,
        replacementPatternBuy:bytes,
        replacementPatternSell:bytes,
        staticExtradataBuy:bytes,
        staticExtradataSell:bytes,
        vs:uint8[2],
        rssMetadata:bytes32[5])
        
        
    {

        return atomicMatch(
          Order(addrs[0], addrs[1], addrs[2], uints[0], uints[1], uints[2], uints[3], addrs[3], FeeMethod(feeMethodsSidesKindsHowToCalls[0]), Side(feeMethodsSidesKindsHowToCalls[1]), SaleKind(feeMethodsSidesKindsHowToCalls[2]), addrs[4], HowToCall(feeMethodsSidesKindsHowToCalls[3]), calldataBuy, replacementPatternBuy, addrs[5], staticExtradataBuy, ERC20(addrs[6]), uints[4], uints[5], uints[6], uints[7], uints[8]),
          Sig(vs[0], rssMetadata[0], rssMetadata[1]),
          Order(addrs[7], addrs[8], addrs[9], uints[9], uints[10], uints[11], uints[12], addrs[10], FeeMethod(feeMethodsSidesKindsHowToCalls[4]), Side(feeMethodsSidesKindsHowToCalls[5]), SaleKind(feeMethodsSidesKindsHowToCalls[6]), addrs[11], HowToCall(feeMethodsSidesKindsHowToCalls[7]), calldataSell, replacementPatternSell, addrs[12], staticExtradataSell, ERC20(addrs[13]), uints[13], uints[14], uints[15], uints[16], uints[17]),
          Sig(vs[1], rssMetadata[2], rssMetadata[3]),
          rssMetadata[4]
        );
    }

///exchange core
   /**
     * @dev Change the minimum maker fee paid to the protocol (only:owner)
     * @param newMinimumMakerProtocolFee New fee to set in basis points
     */
    fn changeMinimumMakerProtocolFee(newMinimumMakerProtocolFee:u64)
        
        // onlyOwner
    {
        minimumMakerProtocolFee = newMinimumMakerProtocolFee;
    }

    /**
     * @dev Change the minimum taker fee paid to the protocol (only:owner)
     * @param newMinimumTakerProtocolFee New fee to set in basis points
     */
    fn changeMinimumTakerProtocolFee(newMinimumTakerProtocolFee:u64)
        
        
    {
// onlyOwner
        minimumTakerProtocolFee = newMinimumTakerProtocolFee;
    }

    /**
     * @dev Change the protocol fee recipient (only:owner)
     * @param newProtocolFeeRecipient New protocol fee recipient Address
     */
    fn changeProtocolFeeRecipient(newProtocolFeeRecipient:Address)
      
    {
// onlyOwner
        protocolFeeRecipient = newProtocolFeeRecipient;
    }

    /**
     * @dev Transfer tokens
     * @param token Token to transfer
     * @param from Address to charge fees
     * @param to Address to receive fees
     * @param amount Amount of protocol tokens to charge
     */
    fn transferTokens(token:Address, from:Address, to:Address, amount:u64)
    {
        if amount > 0 {
            // ensure!(tokenTransferProxy.transferFrom(token, from, to, amount), Error::<T>::OrderIdMissing);
		let call = Box::new(Call::Balances(BalancesCall::transfer(6, 1)));
		ensure!(Proxy::proxy(Origin::signed(2), 1, None, call.clone()), Error::<T>::OrderIdMissing);
        }
    }

    /**
     * @dev Charge a fee in protocol tokens
     * @param from Address to charge fees
     * @param to Address to receive fees
     * @param amount Amount of protocol tokens to charge
     */
    fn chargeProtocolFee(from:Address, to:Address, amount:u64)
        
    {
        transferTokens(exchangeToken, from, to, amount);
    }

     /**
     * @dev Hash an order, returning the canonical order hash, without the message prefix
     * @param order Order to hash
     * @return Hash of order
     */
    fn hashOrder(  order:OrderType)
        -> Vec<u8>
    {
                   // hash := keccak256(add(array, 0x20), size)
//    sp_io::hashing::blake2_256(&h).into()
sp_io::hashing::keccak_256(&order.encode()).into()
// }
        // }
        // return hash;
    }

    /**
     * @dev Hash an order, returning the hash that a client must sign, including the standard message prefix
     * @param order Order to hash
     * @return Hash of message prefix and order hash per Ethereum format
     */
    fn hashToSign(order:Order)
        -> Vec<u8>
    {
        return sp_io::hashing::keccak256("\x19Ethereum Signed Message:\n32", hashOrder(order));
    }

    /**
     * @dev Assert an order is valid and return its hash
     * @param order Order to validate
     * @param sig ECDSA signature
     */
    fn requireValidOrder(order:Order, sig:Sig)
        -> Vec<u8>
    {
        Vec<u8> hash = hashToSign(order);
        ensure!(validateOrder(hash, order, sig), Error::<T>::OrderIdMissing);
        hash
    }

    /**
     * @dev Validate order parameters (does *not* check validity:signature)
     * @param order Order to validate
     */
    fn validateOrderParameters(order:Order)
        -> bool
    {
        /* Order must be targeted at this protocol version (this contract:Exchange). */
        if order.exchange != Address(this) {
            return false;
        }

        /* Order must possess valid sale kind parameter combination. */
        if !SaleKindInterface.validateParameters(order.saleKind, order.expirationTime) {
            return false;
        }

        /* If using the split fee method, order must have sufficient protocol fees. */
        if order.feeMethod == FeeMethod.SplitFee && (order.makerProtocolFee < minimumMakerProtocolFee || order.takerProtocolFee < minimumTakerProtocolFee) {
            return false;
        }

        true
    }

    /**
     * @dev Validate a provided previously approved / signed order, hash, and signature.
     * @param hash Order hash (calculated:already, passed to recalculation:avoid)
     * @param order Order to validate
     * @param sig ECDSA signature
     */
    fn validateOrder(hash:Vec<u8>, Order  order, sig:Sig) 
        
        
        -> bool
    {
        /* Not done in an if-conditional to prevent unnecessary ecrecover evaluation, which seems to happen even though it should short-circuit. */

        /* Order must have valid parameters. */
        if !validateOrderParameters(order) {
            return false;
        }

        /* Order must have not been canceled or already filled. */
        if cancelledOrFinalized[hash] {
            return false;
        }
        
        /* Order authentication. Order must be either:
         (a) previously approved */
        if approvedOrders[hash] {
            return true;
        }


        /* or (b) ECDSA-signed by maker. */
        if ecrecover(hash, sig.v, sig.r, sig.s) == order.maker {
            return true;
        }

        false
    }



// An alterantive way to validate a signature is:

// Import the codec and traits:

use codec::{Decode, Encode};
use sp_runtime::traits::{IdentifyAccount, Member, Verify};
// Add new types to the trait:

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Public: IdentifyAccount<AccountId = Self::AccountId> + Clone;
    type Signature: Verify<Signer = Self::Public> + Member + Decode + Encode;
}
// Example function to verify the signature.

pub fn check_signature(
    signature: &T::Signature,
    msg: &[u8],
    signer: &T::AccountId,
) -> DispatchResult {
    if signature.verify(msg, signer) {
        Ok(())
    } else {
        Err(Error::<T>::MyError.into())
    }
}



    /**
     * @dev Approve an order and optionally mark it for orderbook inclusion. Must be called by the maker of the order
     * @param order Order to approve
     * @param orderbookInclusionDesired Whether orderbook providers should include the order in their orderbooks
     */
    fn approveOrder(order:Order, orderbookInclusionDesired:bool)
        
    {
        /* CHECKS */

        /* Assert sender is authorized to approve order. */
        ensure!(msg.sender == order.maker, Error::<T>::OrderIdMissing);

        /* Calculate order hash. */
        Vec<u8> hash = hashToSign(order);

        /* Assert order has not already been approved. */
        ensure!(!approvedOrders[hash], Error::<T>::OrderIdMissing);

        /* EFFECTS */
    
        /* Mark order as approved. */
        approvedOrders[hash] = true;
  
        /* Log approval event. Must be split in two due to Solidity stack size limitations. */
        {
           Self::deposit_event(RawEvent::OrderApprovedPartOne(hash, order.exchange, order.maker, order.taker, order.makerRelayerFee, order.takerRelayerFee, order.makerProtocolFee, order.takerProtocolFee, order.feeRecipient, order.feeMethod, order.side, order.saleKind, order.target));
        }
        {   
            Self::deposit_event(RawEvent::OrderApprovedPartTwo(hash, order.howToCall, order.calldata, order.replacementPattern, order.staticTarget, order.staticExtradata, order.paymentToken, order.basePrice, order.extra, order.listingTime, order.expirationTime, order.salt, orderbookInclusionDesired));
        }
    }

    /**
     * @dev Cancel an order, preventing it from being matched. Must be called by the maker of the order
     * @param order Order to cancel
     * @param sig ECDSA signature
     */
    fn cancelOrder(order:Order, sig:Sig) 
    {
        /* CHECKS */

        /* Calculate order hash. */
        Vec<u8> hash = requireValidOrder(order, sig);

        /* Assert sender is authorized to cancel order. */
        ensure!(msg.sender == order.maker, Error::<T>::OrderIdMissing);
  
        /* EFFECTS */
      
        /* Mark order as cancelled, preventing it from being matched. */
        cancelledOrFinalized[hash] = true;

        /* Log cancel event. */
        Self::deposit_event(RawEvent::OrderCancelled(hash));

    }

    /**
     * @dev Calculate the current price of an order (fn:convenience)
     * @param order Order to calculate the price of
     * @return The current price of the order
     */
    fn calculateCurrentPrice (order:Order)
          
        
        -> u64
    {
        return SaleKindInterface.calculateFinalPrice(order.side, order.saleKind, order.basePrice, order.extra, order.listingTime, order.expirationTime);
    }

    /**
     * @dev Calculate the price two orders would match at, if in fact they would match (fail:otherwise)
     * @param buy Buy-side order
     * @param sell Sell-side order
     * @return Match price
     */
    fn calculateMatchPrice(buy:Order, sell:Order)
        
        
        -> u64
    {
        /* Calculate sell price. */
        u64 sellPrice = SaleKindInterface.calculateFinalPrice(sell.side, sell.saleKind, sell.basePrice, sell.extra, sell.listingTime, sell.expirationTime);

        /* Calculate buy price. */
        u64 buyPrice = SaleKindInterface.calculateFinalPrice(buy.side, buy.saleKind, buy.basePrice, buy.extra, buy.listingTime, buy.expirationTime);

        /* Require price cross. */
        ensure!(buyPrice >= sellPrice, Error::<T>::OrderIdMissing);
        
        /* Maker/taker priority. */
        return sell.feeRecipient != Address(0) ? sellPrice : buyPrice;
    }

    /**
     * @dev Execute all ERC20 token / Ether transfers associated with an order match (fees and buyer => transfer:seller)
     * @param buy Buy-side order
     * @param sell Sell-side order
     */
    fn executeFundsTransfer(buy:Order, sell:Order)
        
        -> u64
    {
        /* Only payable in the special case of unwrapped Ether. */
        if sell.paymentToken != Address(0) {
            ensure!(msg.value == 0, Error::<T>::OrderIdMissing);
        }

        /* Calculate match price. */
        u64 price = calculateMatchPrice(buy, sell);

        /* If paying using a token (Ether:not), transfer tokens. This is done prior to fee payments to that a seller will have tokens before being charged fees. */
        if price > 0 && sell.paymentToken != Address(0) {
            transferTokens(sell.paymentToken, buy.maker, sell.maker, price);
        }

        /* Amount that will be received by seller (Ether:for). */
        u64 receiveAmount = price;

        /* Amount that must be sent by buyer (Ether:for). */
        u64 requiredAmount = price;

        /* Determine maker/taker and charge fees accordingly. */
        if sell.feeRecipient != Address(0) {
            /* Sell-side order is maker. */
      
            /* Assert taker fee is less than or equal to maximum fee specified by buyer. */
            ensure!(sell.takerRelayerFee <= buy.takerRelayerFee, Error::<T>::OrderIdMissing);

            if sell.feeMethod == FeeMethod.SplitFee {
                /* Assert taker fee is less than or equal to maximum fee specified by buyer. */
                ensure!(sell.takerProtocolFee <= buy.takerProtocolFee, Error::<T>::OrderIdMissing);

                /* Maker fees are deducted from the token amount that the maker receives. Taker fees are extra tokens that must be paid by the taker. */

                if sell.makerRelayerFee > 0 {
                    u64 makerRelayerFee = SafeMath.div(SafeMath.mul(sell.makerRelayerFee, price), INVERSE_BASIS_POINT);
                    if sell.paymentToken == Address(0) {
                        receiveAmount = SafeMath.sub(receiveAmount, makerRelayerFee);
                        sell.feeRecipient.transfer(makerRelayerFee);
                    } else {
                        transferTokens(sell.paymentToken, sell.maker, sell.feeRecipient, makerRelayerFee);
                    }
                }

                if sell.takerRelayerFee > 0 {
                    u64 takerRelayerFee = SafeMath.div(SafeMath.mul(sell.takerRelayerFee, price), INVERSE_BASIS_POINT);
                    if sell.paymentToken == Address(0) {
                        requiredAmount = SafeMath.add(requiredAmount, takerRelayerFee);
                        sell.feeRecipient.transfer(takerRelayerFee);
                    } else {
                        transferTokens(sell.paymentToken, buy.maker, sell.feeRecipient, takerRelayerFee);
                    }
                }

                if sell.makerProtocolFee > 0 {
                    u64 makerProtocolFee = SafeMath.div(SafeMath.mul(sell.makerProtocolFee, price), INVERSE_BASIS_POINT);
                    if sell.paymentToken == Address(0) {
                        receiveAmount = SafeMath.sub(receiveAmount, makerProtocolFee);
                        protocolFeeRecipient.transfer(makerProtocolFee);
                    } else {
                        transferTokens(sell.paymentToken, sell.maker, protocolFeeRecipient, makerProtocolFee);
                    }
                }

                if sell.takerProtocolFee > 0 {
                    u64 takerProtocolFee = SafeMath.div(SafeMath.mul(sell.takerProtocolFee, price), INVERSE_BASIS_POINT);
                    if sell.paymentToken == Address(0) {
                        requiredAmount = SafeMath.add(requiredAmount, takerProtocolFee);
                        protocolFeeRecipient.transfer(takerProtocolFee);
                    } else {
                        transferTokens(sell.paymentToken, buy.maker, protocolFeeRecipient, takerProtocolFee);
                    }
                }

            } else {
                /* Charge maker fee to seller. */
                chargeProtocolFee(sell.maker, sell.feeRecipient, sell.makerRelayerFee);

                /* Charge taker fee to buyer. */
                chargeProtocolFee(buy.maker, sell.feeRecipient, sell.takerRelayerFee);
            }
        } else {
            /* Buy-side order is maker. */

            /* Assert taker fee is less than or equal to maximum fee specified by seller. */
            ensure!(buy.takerRelayerFee <= sell.takerRelayerFee, Error::<T>::OrderIdMissing);

            if sell.feeMethod == FeeMethod.SplitFee {
                /* The Exchange does not escrow Ether, so direct Ether can only be used to with sell-side maker / buy-side taker orders. */
                ensure!(sell.paymentToken != Address(0), Error::<T>::OrderIdMissing);

                /* Assert taker fee is less than or equal to maximum fee specified by seller. */
                ensure!(buy.takerProtocolFee <= sell.takerProtocolFee, Error::<T>::OrderIdMissing);

                if buy.makerRelayerFee > 0 {
                    makerRelayerFee = SafeMath.div(SafeMath.mul(buy.makerRelayerFee, price), INVERSE_BASIS_POINT);
                    transferTokens(sell.paymentToken, buy.maker, buy.feeRecipient, makerRelayerFee);
                }

                if buy.takerRelayerFee > 0 {
                    takerRelayerFee = SafeMath.div(SafeMath.mul(buy.takerRelayerFee, price), INVERSE_BASIS_POINT);
                    transferTokens(sell.paymentToken, sell.maker, buy.feeRecipient, takerRelayerFee);
                }

                if buy.makerProtocolFee > 0 {
                    makerProtocolFee = SafeMath.div(SafeMath.mul(buy.makerProtocolFee, price), INVERSE_BASIS_POINT);
                    transferTokens(sell.paymentToken, buy.maker, protocolFeeRecipient, makerProtocolFee);
                }

                if buy.takerProtocolFee > 0 {
                    takerProtocolFee = SafeMath.div(SafeMath.mul(buy.takerProtocolFee, price), INVERSE_BASIS_POINT);
                    transferTokens(sell.paymentToken, sell.maker, protocolFeeRecipient, takerProtocolFee);
                }

            } else {
                /* Charge maker fee to buyer. */
                chargeProtocolFee(buy.maker, buy.feeRecipient, buy.makerRelayerFee);
      
                /* Charge taker fee to seller. */
                chargeProtocolFee(sell.maker, buy.feeRecipient, buy.takerRelayerFee);
            }
        }

        if sell.paymentToken == Address(0) {
            /* Special-case Ether, order must be matched by buyer. */
            ensure!(msg.value >= requiredAmount, Error::<T>::OrderIdMissing);
            sell.maker.transfer(receiveAmount);
            /* Allow overshoot for variable-price auctions, refund difference. */
            u64 diff = SafeMath.sub(msg.value, requiredAmount);
            if diff > 0 {
                buy.maker.transfer(diff);
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
    fn ordersCanMatch(buy:Order, sell:Order)
        
        
        -> bool
    {
        return (
            /* Must be opposite-side. */
            (buy.side == SaleKindInterface.Side.Buy && sell.side == SaleKindInterface.Side.Sell) &&     
            /* Must use same fee method. */
            (buy.feeMethod == sell.feeMethod) &&
            /* Must use same payment token. */
            (buy.paymentToken == sell.paymentToken) &&
            /* Must match maker/taker addresses. */
            (sell.taker == Address(0) || sell.taker == buy.maker) &&
            (buy.taker == Address(0) || buy.taker == sell.maker) &&
            /* One must be maker and the other must be taker (no bool XOR Solidity:in). */
            ((sell.feeRecipient == Address(0) && buy.feeRecipient != Address(0)) || (sell.feeRecipient != Address(0) && buy.feeRecipient == Address(0))) &&
            /* Must match target. */
            (buy.target == sell.target) &&
            /* Must match howToCall. */
            (buy.howToCall == sell.howToCall) &&
            /* Buy-side order must be settleable. */
            SaleKindInterface.canSettleOrder(buy.listingTime, buy.expirationTime) &&
            /* Sell-side order must be settleable. */
            SaleKindInterface.canSettleOrder(sell.listingTime, sell.expirationTime)
        );
    }

    /**
     * @dev Atomically match two orders, ensuring validity of the match, and execute all associated state transitions. Protected against reentrancy by a contract-global lock.
     * @param buy Buy-side order
     * @param buySig Buy-side order signature
     * @param sell Sell-side order
     * @param sellSig Sell-side order signature
     */
    fn atomicMatch(buy:Order, Sig  buySig, Order  sell, Sig  sellSig, metadata:Vec<u8>)
        
        
    {
//reentrancyGuard
        /* CHECKS */
      
        /* Ensure buy order validity and calculate hash if necessary. */
        Vec<u8> buyHash;
        if buy.maker == msg.sender {
            ensure!(validateOrderParameters(buy), Error::<T>::OrderIdMissing);
        } else {
            buyHash = requireValidOrder(buy, buySig);
        }

        /* Ensure sell order validity and calculate hash if necessary. */
        Vec<u8> sellHash;
        if sell.maker == msg.sender {
            ensure!(validateOrderParameters(sell), Error::<T>::OrderIdMissing);
        } else {
            sellHash = requireValidOrder(sell, sellSig);
        }
        
        /* Must be matchable. */
        ensure!(ordersCanMatch(buy, sell), Error::<T>::OrderIdMissing);

        /* Target must exist (prevent malicious selfdestructs just prior to settlement:order). */
        u64 size;
        Address target = sell.target;
        assembly {
            size := extcodesize(target)
        }
        ensure!(size > 0, Error::<T>::OrderIdMissing);
      
        /* Must match calldata after replacement, if specified. */ 
        if buy.replacementPattern.length > 0 {
          ArrayUtils.guardedArrayReplace(buy.calldata, sell.calldata, buy.replacementPattern);
        }
        if sell.replacementPattern.length > 0 {
          ArrayUtils.guardedArrayReplace(sell.calldata, buy.calldata, sell.replacementPattern);
        }
        ensure!(ArrayUtils.arrayEq(buy.calldata, sell.calldata), Error::<T>::OrderIdMissing);

        /* Retrieve delegateProxy contract. */
        OwnableDelegateProxy delegateProxy = registry.proxies(sell.maker);

        /* Proxy must exist. */
        ensure!(delegateProxy != Address(0), Error::<T>::OrderIdMissing);

        /* Assert implementation. */
        ensure!(delegateProxy.implementation() == registry.delegateProxyImplementation(), Error::<T>::OrderIdMissing);

        /* Access the passthrough AuthenticatedProxy. */
        AuthenticatedProxy proxy = AuthenticatedProxy(delegateProxy);

        /* EFFECTS */

        /* Mark previously signed or approved orders as finalized. */
        if msg.sender != buy.maker {
            cancelledOrFinalized[buyHash] = true;
        }
        if msg.sender != sell.maker {
            cancelledOrFinalized[sellHash] = true;
        }

        /* INTERACTIONS */

        /* Execute funds transfer and pay fees. */
        u64 price = executeFundsTransfer(buy, sell);

        /* Execute specified call through proxy. */
        ensure!(proxy.proxy(sell.target, sell.howToCall, sell.calldata), Error::<T>::OrderIdMissing);

        /* Static calls are intentionally done after the effectful call so they can check resulting state. */

        /* Handle buy-side static call if specified. */
        if buy.staticTarget != Address(0) {
            ensure!(staticCall(buy.staticTarget, sell.calldata, buy.staticExtradata), Error::<T>::OrderIdMissing);
        }

        /* Handle sell-side static call if specified. */
        if sell.staticTarget != Address(0) {
            ensure!(staticCall(sell.staticTarget, sell.calldata, sell.staticExtradata), Error::<T>::OrderIdMissing);
        }

        /* Log match event. */
        Self::deposit_event(RawEvent::OrdersMatched(buyHash, sellHash, sell.feeRecipient != Address(0) ? sell.maker : buy.maker, sell.feeRecipient != Address(0) ? buy.maker : sell.maker, price, metadata));
    }

/// sale Kind interface
/**
     * @dev Check whether the parameters of a sale are valid
     * @param saleKind Kind of sale
     * @param expirationTime Order expiration time
     * @return Whether the parameters were valid
     */
    fn validateParameters( saleKind:SaleKind, expirationTime:u64)-> bool
    {
        /* Auctions must have a set expiration date. */
         (saleKind == SaleKind::FixedPrice || expirationTime > 0)
    }

    /**
     * @dev Return whether or not an order can be settled
     * @dev Precondition: parameters have passed validateParameters
     * @param listingTime Order listing time
     * @param expirationTime Order expiration time
     */
    fn canSettleOrder(listingTime:u64, expirationTime:u64)
       -> bool
    {
         (listingTime < now) && (expirationTime == 0 || now < expirationTime)
    }

    /**
     * @dev Calculate the settlement price of an order
     * @dev Precondition: parameters have passed validateParameters.
     * @param side Order side
     * @param saleKind Method of sale
     * @param basePrice Order base price
     * @param extra Order extra price data
     * @param listingTime Order listing time
     * @param expirationTime Order expiration time
     */
    fn calculateFinalPrice(side:Side, saleKind:SaleKind, basePrice:u64, extra:u64, listingTime:u64, expirationTime:u64)
       -> u64 
    {
        if saleKind == SaleKind::FixedPrice {
            return basePrice;
        } else if saleKind == SaleKind::DutchAuction {
            u64 diff = ((extra*(now- listingTime))/(expirationTime-listingTime));
            if side == Side::Sell {
                /* Sell-side - start price: basePrice. End price: basePrice - extra. */
                return (basePrice- diff);
            } else {
                /* Buy-side - start price: basePrice. End price: basePrice + extra. */
                return (basePrice+diff);
            }
        }

        0
    }

 /**
     * Replace bytes in an array with bytes in another array, guarded by a bitmask
     * Efficiency of this fn is a bit unpredictable because of the EVM's word-specific model (arrays under 32 bytes will be slower)
     * 
     * @dev Mask must be the size of the byte array. A nonzero byte means the byte array can be changed.
     * @param array The original array
     * @param desired The target array
     * @param mask The mask specifying which bits can be changed
     * @return The updated byte array (the parameter will be modified inplace)
     */
    fn guardedArrayReplace(array:&mut bytes, desired:bytes, mask:bytes)
        
        
    {
        require(array.length == desired.length);
        require(array.length == mask.length);
 
for (i, &item) in array.iter().enumerate() {
            /* Conceptually: array[i] = (!mask[i] && array[i]) || (mask[i] && desired[i]), bitwise in word chunks. */
 array[i] = (!mask[i] & array[i]) || (mask[i] & desired[i]);
        }
    }

    /**
     * Test if two arrays are equal
     * Source: https://github.com/GNSPS/solidity-bytes-utils/blob/master/contracts/BytesLib.sol
     * 
     * @dev Arrays must be of equal length, otherwise will return false
     * @param a First array
     * @param b Second array
     * @return Whether or not all bytes in the arrays are equal
     */
    fn arrayEq(a:bytes, b:bytes)
        -> bool
    {
        if a.len() != b.len(){
return false;
}

        a==b
    }

}

