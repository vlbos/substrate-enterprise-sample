#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]


use codec::Codec;
use sp_std::vec::Vec;
use orderbook::{OrderQuery,OrderJSONType};

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait OrderbookApi<AccountId,  Moment> where
		AccountId: Codec,
		Moment: Codec,
 {
	  fn get_orders(
        order_query: &Option<OrderQuery<AccountId>>,
    ) -> Option<Vec<OrderJSONType<AccountId, Moment>>>;
 }
}
