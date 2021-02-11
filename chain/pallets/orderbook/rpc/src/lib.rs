//! RPC interface for the transaction payment module.

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
use orderbook_runtime_api::OrderbookApi as OrderbookRuntimeApi;
use orderbook::{OrderQuery,OrderJSONType};

type OrderQuery<T> = <T as orderbook::Trait>::OrderQuery;

#[rpc]
pub trait OrderbookApi<BlockHash,AccountId,Moment> {
	#[rpc(name = "orderbook_getOrders")]
    fn get_orders(&self,
        order_query: &Option<OrderQuery<AccountId>>, at: Option<BlockHash>
    ) -> Result<Option<Vec<OrderJSONType<AccountId, Moment>>>>;
}

/// A struct that implements the `OrderbookApi`.
pub struct Orderbook<C, M> {
	// If you have more generics, no need to Orderbook<C, M, N, P, ...>
	// just use a tuple like Orderbook<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> Orderbook<C, M> {
	/// Create new `Orderbook` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

/// Error type of this RPC api.
// pub enum Error {
// 	/// The transaction was not decodable.
// 	DecodeError,
// 	/// The call to runtime failed.
// 	RuntimeError,
// }
//
// impl From<Error> for i64 {
// 	fn from(e: Error) -> i64 {
// 		match e {
// 			Error::RuntimeError => 1,
// 			Error::DecodeError => 2,
// 		}
// 	}
// }

impl<C, Block> OrderbookApi<<Block as BlockT>::Hash> for Orderbook<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: OrderbookRuntimeApi<Block,AccountId, Moment>,
{
	fn get_orders(&self,
        order_query: &Option<OrderQuery<AccountId>>, at:Option<<Block as BlockT>::Hash>
    ) -> Result<Option<Vec<OrderJSONType<AccountId, Moment>>>>{
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.get_orders(order_query,&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
}

}



