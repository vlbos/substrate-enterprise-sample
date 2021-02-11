//! RPC interface for the transaction payment module.

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
use wyvern_exchange_runtime_api::WyvernExchangeApi as WyvernExchangeRuntimeApi;
use wyvern_exchange::{Side,SaleKind,FeeMethod,HowToCall};

#[rpc]
pub trait WyvernExchangeApi<BlockHash,AccountId,Balance,Moment,Signature> {
	#[rpc(name = "WyvernExchange_calculateFinalPriceEx")]
  fn calculate_final_price_ex(&self,
        side: Side,
        sale_kind: SaleKind,
        base_price: Balance,
        extra: Moment,
        listing_time: Moment,
        expiration_time: Moment,
    at: Option<BlockHash>) -> Result<Balance>;

	#[rpc(name = "WyvernExchange_hashOrderEx")]
    fn hash_order_ex(&self,
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    at: Option<BlockHash>) -> Result<Vec<u8>>;

#[rpc(name = "WyvernExchange_hashToSignEx")]
    fn hash_to_sign_ex(&self,
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    at: Option<BlockHash>) -> Result<Vec<u8>>;

#[rpc(name = "WyvernExchange_validateOrderParametersEx")]
   fn validate_order_parameters_ex(&self,
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    at: Option<BlockHash>) -> Result<bool>;

#[rpc(name = "WyvernExchange_validateOrderEx")]
   fn validate_order_ex(&self,
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
    at: Option<BlockHash>) -> Result<bool> ;

#[rpc(name = "WyvernExchange_calculateCurrentPriceEx")]
 fn calculate_current_price_ex(&self,
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    at: Option<BlockHash>) -> Result<Balance>;

#[rpc(name = "WyvernExchange_ordersCanMatchEx")]
   fn orders_can_match_ex(&self,
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_methods_sides_kinds_how_to_calls: Vec<u8>,
        calldata_buy: Vec<u8>,
        calldata_sell: Vec<u8>,
        replacement_pattern_buy: Vec<u8>,
        replacement_pattern_sell: Vec<u8>,
        static_extradata_buy: Vec<u8>,
        static_extradata_sell: Vec<u8>,
    at: Option<BlockHash>) -> Result<bool> ;

#[rpc(name = "WyvernExchange_calculateMatchPriceEx")]
fn calculate_match_price_ex(&self,
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_methods_sides_kinds_how_to_calls: Vec<u8>,
        calldata_buy: Vec<u8>,
        calldata_sell: Vec<u8>,
        replacement_pattern_buy: Vec<u8>,
        replacement_pattern_sell: Vec<u8>,
        static_extradata_buy: Vec<u8>,
        static_extradata_sell: Vec<u8>,
    at: Option<BlockHash>) -> Result<Balance> ;
}

/// A struct that implements the `WyvernExchangeApi`.
pub struct WyvernExchange<C, M> {
	// If you have more generics, no need to WyvernExchange<C, M, N, P, ...>
	// just use a tuple like WyvernExchange<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> WyvernExchange<C, M> {
	/// Create new `WyvernExchange` instance with the given reference to the client.
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

impl<C, Block> WyvernExchangeApi<<Block as BlockT>::Hash,AccountId,Balance,Moment,Signature> for WyvernExchange<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static,
	C: ProvideRuntimeApi<Block>,
	C: HeaderBackend<Block>,
	C::Api: WyvernExchangeRuntimeApi<Block,AccountId,Balance,Moment,Signature>,
{
 fn calculate_final_price_ex(&self,
        side: Side,
        sale_kind: SaleKind,
        base_price: Balance,
        extra: Moment,
        listing_time: Moment,
        expiration_time: Moment,
    at: Option<BlockHash>) -> Result<Balance>{
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.calculate_final_price_ex(  side,
        sale_kind,
        base_price,
        extra,
        listing_time,
        expiration_time,&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
    fn hash_order_ex(&self,
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    at: Option<BlockHash>) -> Result<Vec<u8>>{
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.hash_order_ex(
        addrs,
        uints,
        fee_method,
        side,
        sale_kind,
        how_to_call,
        calldata,
        replacement_pattern,
        static_extradata,&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

    fn hash_to_sign_ex(&self,
        addrs,
        uints,
        fee_method,
        side,
        sale_kind,
        how_to_call,
        calldata,
        replacement_pattern,
        static_extradata,
    at: Option<BlockHash>) -> Result<Vec<u8>>{
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.hash_to_sign_ex(
        addrs,
        uints,
        fee_method,
        side,
        sale_kind,
        how_to_call,
        calldata,
        replacement_pattern,
        static_extradata,&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
   fn validate_order_parameters_ex(&self,
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    at: Option<BlockHash>) -> Result<bool>{
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.validate_order_parameters_ex(
        addrs,
        uints,
        fee_method,
        side,
        sale_kind,
        how_to_call,
        calldata,
        replacement_pattern,
        static_extradata,&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
   fn validate_order_ex(&self,
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
    at: Option<BlockHash>) -> Result<bool> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.validate_order_ex(
        addrs,
        uints,
        fee_method,
        side,
        sale_kind,
        how_to_call,
        calldata,
        replacement_pattern,
        static_extradata,
        sig,&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
 fn calculate_current_price_ex(&self,
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_method: FeeMethod,
        side: Side,
        sale_kind: SaleKind,
        how_to_call: HowToCall,
        calldata: Vec<u8>,
        replacement_pattern: Vec<u8>,
        static_extradata: Vec<u8>,
    at: Option<BlockHash>) -> Result<Balance>{
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.calculate_current_price_ex(
        addrs,
        uints,
        fee_method,
        side,
        sale_kind,
        how_to_call,
        calldata,
        replacement_pattern,
        static_extradata,&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
   fn orders_can_match_ex(&self,
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_methods_sides_kinds_how_to_calls: Vec<u8>,
        calldata_buy: Vec<u8>,
        calldata_sell: Vec<u8>,
        replacement_pattern_buy: Vec<u8>,
        replacement_pattern_sell: Vec<u8>,
        static_extradata_buy: Vec<u8>,
        static_extradata_sell: Vec<u8>,
    at: Option<BlockHash>) -> Result<bool> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.orders_can_match_ex(
        addrs,
        uints,
        fee_methods_sides_kinds_how_to_calls,
        calldata_buy,
        calldata_sell,
        replacement_pattern_buy,
        replacement_pattern_sell,
        static_extradata_buy,
        static_extradata_sell,&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
fn calculate_match_price_ex(&self,
        addrs: Vec<AccountId>,
        uints: Vec<u64>,
        fee_methods_sides_kinds_how_to_calls: Vec<u8>,
        calldata_buy: Vec<u8>,
        calldata_sell: Vec<u8>,
        replacement_pattern_buy: Vec<u8>,
        replacement_pattern_sell: Vec<u8>,
        static_extradata_buy: Vec<u8>,
        static_extradata_sell: Vec<u8>,
    at: Option<BlockHash>) -> Result<Balance> 
{
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let runtime_api_result = api.calculate_match_price_ex(
        addrs,
        uints,
        fee_methods_sides_kinds_how_to_calls,
        calldata_buy,
        calldata_sell,
        replacement_pattern_buy,
        replacement_pattern_sell,
        static_extradata_buy,
        static_extradata_sell,&at);
		runtime_api_result.map_err(|e| RpcError {
			code: ErrorCode::ServerError(9876), // No real reason for this value
			message: "Something wrong".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
