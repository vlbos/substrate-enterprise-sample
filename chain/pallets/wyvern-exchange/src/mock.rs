// Creating mock runtime here

use crate as wyvern_exchange;
use crate::{Module, Trait};
use core::marker::PhantomData;
use frame_support::{
    impl_outer_dispatch, impl_outer_event, impl_outer_origin, parameter_types, traits::Currency,
    traits::EnsureOrigin, traits::StorageMapShim, weights::Weight,
};
use frame_system as system;
use frame_system::RawOrigin;
use sp_core::{sr25519, Pair, H256};
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};

impl_outer_origin! {
    pub enum Origin for Test {}
}

// impl_outer_dispatch! {
//     pub enum Call for Test where origin: Origin {
//         balances::Balances,
//         // WyvernExchange,
//     }
// }

impl_outer_event! {
    pub enum TestEvent for Test {
balances<T>,
        system<T>,
        wyvern_exchange<T>,
    }
}

impl balances::Trait for Test {
    type Balance = u64;
    type DustRemoval = ();
    // type TransferPayment = ();
    type ExistentialDeposit = ExistentialDeposit;

    type MaxLocks = ();
    type Event = TestEvent;
    type AccountStore = System;
    type WeightInfo = ();
}

// For testing the pallet, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of pallets we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
pub const ExistentialDeposit: u64 = 500;
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = sr25519::Public;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = MaximumBlockWeight;
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type PalletInfo = ();
    type AccountData = balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

impl timestamp::Trait for Test {
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = ();
    type WeightInfo = ();
}

impl Trait for Test {
    type Event = TestEvent;
    type Currency = Balances;

    // type CreateRoleOrigin = MockOrigin<Test>;
}

pub type WyvernExchange = Module<Test>;
pub type System = system::Module<Test>;
pub type Timestamp = timestamp::Module<Test>;
pub type Balances = balances::Module<Test>;

pub struct MockOrigin<T>(PhantomData<T>);

impl<T: Trait> EnsureOrigin<T::Origin> for MockOrigin<T> {
    type Success = T::AccountId;
    fn try_origin(o: T::Origin) -> Result<Self::Success, T::Origin> {
        o.into().and_then(|o| match o {
            RawOrigin::Signed(ref who) => Ok(who.clone()),
            r => Err(T::Origin::from(r)),
        })
    }
}

pub fn print_all_events() {
    println!("------------------- Print Events Started -------------------");
    for event in <system::Module<Test>>::events() {
        println!("{:?}", event);
    }
    println!("------------------- Print Events Ended -------------------");
}

pub fn create_account_test(account_id: sr25519::Public) {
    let _ = Balances::deposit_creating(&account_id, 100_000_000_000_000_000);
}
// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap();

    let mut ext = sp_io::TestExternalities::from(storage);
    // Events are not emitted on block 0 -> advance to block 1.
    // Any dispatchable calls made during genesis block will have no events emitted.
    ext.execute_with(|| System::set_block_number(1));
    ext
}

pub fn account_key(s: &str) -> sr25519::Public {
    sr25519::Pair::from_string(&format!("//{}", s), None)
        .expect("static values are valid; qed")
        .public()
}
