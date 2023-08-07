use crate as pallet_template;
use crate::{ActorAuthenticator, Config, SideEffectType, SideEffectsHandlerTrait};
use frame_support::traits::{ConstU16, ConstU32, ConstU64, ConstU8};
use sp_core::H256;
use sp_runtime::{
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
};
type Block = frame_system::mocking::MockBlock<Test>;

const SIDE_EFFECTS_LENGTH_LIMIT: u32 = 10;

pub const PARAMETERS_SPLITTER: u8 = 0x7c;
pub const SIDE_EFFECTS_SPLITTER: u8 = 0x3b;

pub const FIRST_ACCOUNT_ID: &str =
    "5c55177d67b064bb5d189a3e1ddad9bc6646e02e64d6e308f5acbb1533ac430d";
pub const SECOND_ACCOUNT_ID: &str =
    "3c55177d67b064bb5d189a3e1ddad9bc6646e02e64d6e308f5acbb1533ac430d";
pub const THIRD_ACCOUNT_ID: &str =
    "9c55177d67b064bb5d189a3e1ddad9bc6646e02e64d6e308f5acbb1533ac430d";

pub const AMOUNT: u64 = 500_000;

pub const DEFAULT_ACTOR_ID: ActorId = 0;

// Type aliases
pub type AccountId = <Test as frame_system::Config>::AccountId;
pub type ActorId = <Test as Config>::ActorId;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
    pub enum Test
    {
        System: frame_system,
        TemplateModule: pallet_template,
    }
);

impl ActorAuthenticator<Test> for Test {
    /// Ensures specific actor under given account id can commit
    fn authorize_for_commit(
        _: &AccountId,
        actor_id: ActorId,
    ) -> Result<(), sp_runtime::DispatchError> {
        if actor_id == DEFAULT_ACTOR_ID {
            Ok(())
        } else {
            Err(sp_runtime::DispatchError::Other(
                "Actor is not authorized to commit",
            ))
        }
    }

    /// Ensures specific actor under given account id can revert
    fn authorize_for_revert(
        _: &AccountId,
        actor_id: ActorId,
    ) -> Result<(), sp_runtime::DispatchError> {
        if actor_id == DEFAULT_ACTOR_ID {
            Ok(())
        } else {
            Err(sp_runtime::DispatchError::Other(
                "Actor is not authorized to revert",
            ))
        }
    }
}

impl SideEffectsHandlerTrait for Test {
    /// Commit given side effects
    fn perform_commit(
        _side_effect_type: &[SideEffectType],
    ) -> Result<(), sp_runtime::DispatchError> {
        Ok(())
    }

    /// Revert given side effects
    fn perform_revert(
        _side_effect_type: &[SideEffectType],
    ) -> Result<(), sp_runtime::DispatchError> {
        Ok(())
    }
}

impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type DbWeight = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = ConstU64<250>;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ConstU16<42>;
    type OnSetCode = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_template::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type SideEffectsLengthLimit = ConstU32<SIDE_EFFECTS_LENGTH_LIMIT>;
    type ParametersSplitter = ConstU8<PARAMETERS_SPLITTER>;
    type SideEffectsSplitter = ConstU8<SIDE_EFFECTS_SPLITTER>;
    type ExecutionId = u64;
    type ActorId = u64;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    // Initialize the global logger with an env logger.
    env_logger::init();
    frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap()
        .into()
}
