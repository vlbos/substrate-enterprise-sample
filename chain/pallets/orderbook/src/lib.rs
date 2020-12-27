//! # Substrate Enterprise Sample - Order Post example pallet

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use core::result::Result;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure, sp_runtime::RuntimeDebug,
    sp_std::prelude::*, 
};
// traits::EnsureOrigin,
use frame_system::{self as system, ensure_signed};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// General constraints to limit data size
// Note: these could also be passed as trait config parameters
pub const ORDER_ID_MAX_LENGTH: usize = 36;
pub const ORDER_FIELD_NAME_MAX_LENGTH: usize = 10;
pub const ORDER_FIELD_VALUE_MAX_LENGTH: usize = 20;
pub const ORDER_MAX_FIELDS: usize = 3;

// Custom types
pub type OrderId = Vec<u8>;
pub type FieldName = Vec<u8>;
pub type FieldValue = Vec<u8>;

// Order contains master data (aka class-level) about a trade item.
// This data is typically registered once by the order's manufacturer / supplier,
// to be shared with other network participants, and remains largely static.
// It can also be used for instance-level (lot) master data.
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct OrderJSONType<AccountId, Moment> {
    // The order ID would typically be a GS1 GTIN (Global Trade Item Number),
    // or ASIN (Amazon Standard Identification Number), or similar,
    // a numeric or alpha-numeric code with a well-defined data structure.
    id: OrderId,
    // This is account that represents the owner of this order, as in
    // the manufacturer or supplier providing this order within the value chain.
    owner: AccountId,
    // This a series of fields describing the order.
    // Typically, there would at least be a textual description, and SKU(Stock-keeping unit).
    // It could also contain instance / lot master data e.g. expiration, weight, harvest date.
    fields: Option<Vec<OrderField>>,
    // Timestamp (approximate) at which the prodct was registered on-chain.
    registered: Moment,
}

// Contains a name-value pair for a order fielderty e.g. description: Ingredient ABC
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct OrderField {
    // Name of the order fielderty e.g. desc or description
    name: FieldName,
    // Value of the order fielderty e.g. Ingredient ABC
    value: FieldValue,
}

impl OrderField {
    pub fn new(name: &[u8], value: &[u8]) -> Self {
        Self {
            name: name.to_vec(),
            value: value.to_vec(),
        }
    }

    pub fn name(&self) -> &[u8] {
        self.name.as_ref()
    }

    pub fn value(&self) -> &[u8] {
        self.value.as_ref()
    }
}

pub trait Trait: system::Trait + timestamp::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    // type CreateRoleOrigin: EnsureOrigin<Self::Origin>;
}

decl_storage! {
    trait Store for Module<T: Trait> as OrderRegistry {
        pub Orders get(fn order_by_id): map hasher(blake2_128_concat) OrderId => Option<OrderJSONType<T::AccountId, T::Moment>>;
        pub OrdersOfOrganization get(fn orders_of_org): map hasher(blake2_128_concat) T::AccountId => Vec<OrderId>;
        pub OwnerOf get(fn owner_of): map hasher(blake2_128_concat) OrderId => Option<T::AccountId>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        OrderPosted(AccountId, OrderId, AccountId),
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

            // Create a order instance
            let order = Self::new_order()
                .identified_by(id.clone())
                .owned_by(owner.clone())
                .registered_on(<timestamp::Module<T>>::now())
                .with_fields(fields)
                .build();

            // Add order & ownerOf (3 DB writes)
            <Orders<T>>::insert(&id, order);
            <OrdersOfOrganization<T>>::append(&owner, &id);
            <OwnerOf<T>>::insert(&id, &owner);

            Self::deposit_event(RawEvent::OrderPosted(who, id, owner));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    // Helper methods
    fn new_order() -> OrderBuilder<T::AccountId, T::Moment> {
        OrderBuilder::<T::AccountId, T::Moment>::default()
    }

    pub fn validate_order_id(id: &[u8]) -> Result<(), Error<T>> {
        // Basic order ID validation
        ensure!(!id.is_empty(), Error::<T>::OrderIdMissing);
        ensure!(
            id.len() <= ORDER_ID_MAX_LENGTH,
            Error::<T>::OrderIdTooLong
        );
        Ok(())
    }

    pub fn validate_new_order(id: &[u8]) -> Result<(), Error<T>> {
        // Order existence check
        ensure!(
            !<Orders<T>>::contains_key(id),
            Error::<T>::OrderIdExists
        );
        Ok(())
    }

    pub fn validate_order_fields(fields: &Option<Vec<OrderField>>) -> Result<(), Error<T>> {
        if let Some(fields) = fields {
            ensure!(
                fields.len() <= ORDER_MAX_FIELDS,
                Error::<T>::OrderTooManyFields,
            );
            for field in fields {
                ensure!(
                    field.name().len() <= ORDER_FIELD_NAME_MAX_LENGTH,
                    Error::<T>::OrderInvalidFieldName
                );
                ensure!(
                    field.value().len() <= ORDER_FIELD_VALUE_MAX_LENGTH,
                    Error::<T>::OrderInvalidFieldValue
                );
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct OrderBuilder<AccountId, Moment>
where
    AccountId: Default,
    Moment: Default,
{
    id: OrderId,
    owner: AccountId,
    fields: Option<Vec<OrderField>>,
    registered: Moment,
}

impl<AccountId, Moment> OrderBuilder<AccountId, Moment>
where
    AccountId: Default,
    Moment: Default,
{
    pub fn identified_by(mut self, id: OrderId) -> Self {
        self.id = id;
        self
    }

    pub fn owned_by(mut self, owner: AccountId) -> Self {
        self.owner = owner;
        self
    }

    pub fn with_fields(mut self, fields: Option<Vec<OrderField>>) -> Self {
        self.fields = fields;
        self
    }

    pub fn registered_on(mut self, registered: Moment) -> Self {
        self.registered = registered;
        self
    }

    pub fn build(self) -> OrderJSONType<AccountId, Moment> {
        OrderJSONType::<AccountId, Moment> {
            id: self.id,
            owner: self.owner,
            fields: self.fields,
            registered: self.registered,
        }
    }
}
