#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]
use codec::Codec;
use sp_std::vec::Vec;
use wyvern_exchange::{Side,SaleKind,FeeMethod,HowToCall};
// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait WyvernExchangeApi<AccountId,Balance, Moment,Signature> where
		AccountId: Codec,
Balance: Codec,
		Moment: Codec,
Signature:Codec,
{
		     fn calculate_final_price_ex(
        side: Side,
        sale_kind: SaleKind,
        base_price: Balance,
        extra: Moment,
        listing_time: Moment,
        expiration_time: Moment,
    ) -> Balance;
    fn hash_order_ex(
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    ) -> Vec<u8>;

    fn hash_to_sign_ex(
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    ) -> Vec<u8>;
   fn validate_order_parameters_ex(
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    ) -> bool;
   fn validate_order_ex(
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
        sig: Signature,
    ) -> bool ;
 fn calculate_current_price_ex(
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    ) -> Balance;
   fn orders_can_match_ex(
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_methods_sides_kinds_how_to_calls: Vec<u8>,
        calldata_buy: Vec<u8>,
        calldata_sell: Vec<u8>,
        replacement_pattern_buy: Vec<u8>,
        replacement_pattern_sell: Vec<u8>,
        static_extradata_buy: Vec<u8>,
        static_extradata_sell: Vec<u8>,
    ) -> bool ;
fn calculate_match_price_ex(
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_methods_sides_kinds_how_to_calls: Vec<u8>,
        calldata_buy: Vec<u8>,
        calldata_sell: Vec<u8>,
        replacement_pattern_buy: Vec<u8>,
        replacement_pattern_sell: Vec<u8>,
        static_extradata_buy: Vec<u8>,
        static_extradata_sell: Vec<u8>,
    ) -> Balance ;
	}
}
