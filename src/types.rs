use super::*;

use codec::{Codec, Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::TypeInfo;
pub type Amount = u64;

// Length values for all available execution types
pub const SWAP_EXECUTION_TYPE_LENGTH: usize = 5;
pub const TRAN_EXECUTION_TYPE_LENGTH: usize = 3;
pub const MULTI_TRAN_EXECUTION_TYPE_LENGTH: usize = 4;

// Chain ids representations for all supported chains
pub const POLKADOT_CHAIN_OR_ASSET: u8 = 1;
pub const KUSAMA_CHAIN_OR_ASSET: u8 = 2;
pub const ROCOCO_CHAIN_OR_ASSET: u8 = 3;
pub const T3RN_CHAIN_OR_ASSET: u8 = 4;

// Execution type ids representations for all supported execution types
pub const SWAP_EXEC_TYPE: u8 = 5;
pub const TRAN_EXECUTION_TYPE: u8 = 6;
pub const MULTI_TRAN_EXECUTION_TYPE: u8 = 7;

/// An enum representing all supported chains and their corresponding ids
#[repr(u8)]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum Chain {
    Polkadot = POLKADOT_CHAIN_OR_ASSET,
    Kusama = KUSAMA_CHAIN_OR_ASSET,
    Rococo = ROCOCO_CHAIN_OR_ASSET,
    T3rn = T3RN_CHAIN_OR_ASSET,
}

impl Chain {
    /// Parse `Chain` from byte, returns `None` if not supported
    pub fn from_u8(chain_id: u8) -> Option<Self> {
        let chain_id = match chain_id {
            POLKADOT_CHAIN_OR_ASSET => Chain::Polkadot,
            KUSAMA_CHAIN_OR_ASSET => Chain::Kusama,
            ROCOCO_CHAIN_OR_ASSET => Chain::Rococo,
            T3RN_CHAIN_OR_ASSET => Chain::T3rn,
            _ => return None,
        };

        Some(chain_id)
    }
}

/// An enum representing all supported execution types and their corresponding ids
#[repr(u8)]
pub enum ExecutionTypes {
    Swap = SWAP_EXEC_TYPE,
    Tran = TRAN_EXECUTION_TYPE,
    MultiTran = MULTI_TRAN_EXECUTION_TYPE,
}

impl ExecutionTypes {
    /// Parse `ExecutionTypes` from byte, returns `None` if not supported
    pub fn from_u8(execution_type: u8) -> Option<Self> {
        let execution_type = match execution_type {
            SWAP_EXEC_TYPE => ExecutionTypes::Swap,
            TRAN_EXECUTION_TYPE => ExecutionTypes::Tran,
            MULTI_TRAN_EXECUTION_TYPE => ExecutionTypes::MultiTran,
            _ => return None,
        };

        Some(execution_type)
    }
}

/// Represents either `Chain` or `ExecutionTypes`
pub enum ChainOrExecutionType {
    Chain(Chain),
    ExecutionType(ExecutionTypes),
}

/// Represents `Asset` related to corresponding `Chain`
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct Asset(Chain);

impl Asset {
    /// Parse `Asset` from byte, returns `None` if not supported
    pub fn from_u8(asset_id: u8) -> Option<Self> {
        Chain::from_u8(asset_id).map(|chain_id| Self(chain_id))
    }
}

/// Represents all existing execution types and their corresponding values
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum ExecutionType<AccountId> {
    Swap {
        caller: AccountId,
        to: AccountId,
        amount_from: Amount,
        amount_to: Amount,
        asset: Asset,
    },
    Tran {
        caller: AccountId,
        to: AccountId,
        amount: Amount,
    },
    MultiTran {
        caller: AccountId,
        to: AccountId,
        amount: Amount,
        asset: Asset,
    },
}

/// An enum representation for side effect state
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum SideEffectState {
    Submitted,
    Committed,
    Reverted,
}

/// Side effect representation for `SingleChain` and `MultiChain` operations
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub enum SideEffectTypeRecord<AccountId> {
    SingleChain {
        chain: Chain,
        side_effect: SideEffect<AccountId>,
    },
    MultiChain {
        chain_from: Chain,
        chain_to: Chain,
        side_effect: SideEffect<AccountId>,
    },
}

impl<AccountId> SideEffectTypeRecord<AccountId> {
    /// Construct new `SideEffectTypeRecord` instance with parameters provided
    pub fn construct_side_effect(
        chain: Chain,
        chain_to: Option<Chain>,
        execution_type: ExecutionType<AccountId>,
    ) -> Self {
        if let Some(chain_to) = chain_to {
            SideEffectTypeRecord::MultiChain {
                chain_from: chain,
                chain_to,
                side_effect: SideEffect::new(execution_type),
            }
        } else {
            SideEffectTypeRecord::SingleChain {
                chain,
                side_effect: SideEffect::new(execution_type),
            }
        }
    }

    /// Checks whether `SideEffect` `state` is set to `SideEffectState::Committed`
    pub fn is_commited(&self) -> bool {
        match self {
            Self::SingleChain { side_effect, .. } => side_effect.is_commited(),
            Self::MultiChain { side_effect, .. } => side_effect.is_commited(),
        }
    }

    /// Sets `SideEffect` `state` to `SideEffectState::Committed`
    pub fn commit(&mut self) {
        match self {
            Self::SingleChain { side_effect, .. } => side_effect.commit(),
            Self::MultiChain { side_effect, .. } => side_effect.commit(),
        }
    }

    /// Sets `SideEffect` `state` to `SideEffectState::Reverted`
    pub fn revert(&mut self) {
        match self {
            Self::SingleChain { side_effect, .. } => side_effect.revert(),
            Self::MultiChain { side_effect, .. } => side_effect.revert(),
        }
    }
}

/// Volatile side effect representation
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct SideEffect<AccountId> {
    execution_type: ExecutionType<AccountId>,
    state: SideEffectState,
}

impl<AccountId> SideEffect<AccountId> {
    /// Creates new instance of `SideEffect`
    pub fn new(execution_type: ExecutionType<AccountId>) -> Self {
        Self {
            execution_type,
            // for newly submitted side effects, the state is equal to `Submitted`
            state: SideEffectState::Submitted,
        }
    }

    /// Sets `SideEffect` `state` to `SideEffectState::Committed`
    pub fn commit(&mut self) {
        self.state = SideEffectState::Committed
    }

    /// Sets `SideEffect` `state` to `SideEffectState::Reverted`
    pub fn revert(&mut self) {
        self.state = SideEffectState::Reverted
    }

    /// Checks whether `SideEffect` `state` is set to `SideEffectState::Committed`
    pub fn is_commited(&self) -> bool {
        if let SideEffectState::Committed = self.state {
            true
        } else {
            false
        }
    }
}

// Aliases
pub type SideEffectType = SideEffectTypeRecord<AccountId32>;
pub type SideEffectsVec<T> = BoundedVec<SideEffectType, <T as Config>::SideEffectsLengthLimit>;

/// A numeric identifier trait
pub trait NumericIdentifier:
    Parameter
    + Member
    + BaseArithmetic
    + Codec
    + Default
    + Copy
    + Clone
    + MaybeSerializeDeserialize
    + Eq
    + PartialEq
    + Ord
    + Zero
    + From<u64>
    + Into<u64>
    + MaxEncodedLen
{
}

/// A trait authenticator for side effects related operations
pub trait ActorAuthenticator<T: Config> {
    /// Ensures specific actor under given account id can commit
    fn authorize_for_commit(
        account_id: &T::AccountId,
        actor_id: T::ActorId,
    ) -> Result<(), sp_runtime::DispatchError>;

    /// Ensures specific actor under given account id can revert
    fn authorize_for_revert(
        account_id: &T::AccountId,
        actor_id: T::ActorId,
    ) -> Result<(), sp_runtime::DispatchError>;
}

/// A handler trait for side effects related operations
pub trait SideEffectsHandlerTrait {
    /// Commit given side effects
    fn perform_commit(side_effect_type: &[SideEffectType])
        -> Result<(), sp_runtime::DispatchError>;

    /// Revert given side effects
    fn perform_revert(side_effect_type: &[SideEffectType])
        -> Result<(), sp_runtime::DispatchError>;
}

impl NumericIdentifier for u64 {}
