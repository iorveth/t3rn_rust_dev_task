#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod types;
pub use sp_arithmetic::traits::{BaseArithmetic, CheckedAdd, One, Saturating, Zero};
use sp_runtime::AccountId32;
use sp_std::vec::Vec;
pub use types::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    pub use frame_support::{pallet_prelude::*, sp_runtime, BoundedVec};
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config:
        frame_system::Config + ActorAuthenticator<Self> + SideEffectsHandlerTrait
    {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Maximum number of side effects per execution
        #[pallet::constant]
        type SideEffectsLengthLimit: Get<u32>;

        /// A parameter splitter for a sequence of encoded bytes
        #[pallet::constant]
        type ParametersSplitter: Get<u8>;

        /// A side effects splitter for a sequence of encoded bytes
        #[pallet::constant]
        type SideEffectsSplitter: Get<u8>;

        /// Execution id
        type ExecutionId: NumericIdentifier;

        /// Actor identifier
        type ActorId: NumericIdentifier;
    }

    /// Next execution id value
    #[pallet::storage]
    #[pallet::getter(fn next_execution_id)]
    pub type NextExecutionId<T> = StorageValue<_, <T as Config>::ExecutionId>;

    /// Maps an execution id to the respective side effects
    #[pallet::storage]
    #[pallet::getter(fn side_effects)]
    pub type SideEffects<T> =
        StorageMap<_, Blake2_128Concat, <T as Config>::ExecutionId, SideEffectsVec<T>>;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/main-docs/build/events-errors/
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        Executed {
            who: T::AccountId,
            execution_id: T::ExecutionId,
            encoded_bytes: Vec<u8>,
        },

        Commited {
            who: T::AccountId,
            execution_id: T::ExecutionId,
            number_of_steps: u32,
        },

        Reverted {
            who: T::AccountId,
            execution_id: T::ExecutionId,
        },
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Storage overflow happened.
        StorageOverflow,

        /// Incorrect chain id byte provided
        IncorrectChainIdByte,

        /// Incorrect asset id byte provided
        IncorrectAssetIdByte,

        /// Incorrect execution type byte provided
        IncorrectExecutionTypeByte,

        /// Given execution type byte is not supported for this side effect
        ExecutionTypeByteIsNotSupported,

        /// Incorrect chain id or execution type byte
        IncorrectChainIdOrExecutionTypeByte,

        /// Incorrect length of side effect parameters
        IncorrectSideEffectParametersLength,

        /// Incorrect length of execution type parameters
        IncorrectExecutionTypeParametersLength,

        /// Given byte slice should represent a single byte
        NotASingleByte,

        /// An error during the AccountId32 parsing occured
        IncorrectAccountId,

        /// An error during the Amount parsing occured
        IncorrectAmount,

        /// A length limit for the number of side effects per execution reached
        SideEffectsLengthLimitExceeded,

        // Side effects under given execution id not found
        SideEffectsNotFound,

        /// There are no side side effects to commit during this operation
        NothingToCommit,

        /// There are no side effects to revert during this operation
        NothingToRevert,

        /// Number of steps to commit under given execution id is greater then number of side
        /// effects left to commit
        NumberOfStepsToCommitIsGreaterThenNumberOfSideEffectsLeftToCommit,
    }

    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Generate volatile side effects steps from encoded bytes provided
        #[pallet::call_index(0)]
        #[pallet::weight(0)]
        pub fn generate_side_effects_steps(
            origin: OriginFor<T>,
            encoded_bytes: Vec<u8>,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/main-docs/build/origins/
            let account_id = ensure_signed(origin)?;

            log::info!("\n Input bytes --> {:?}", encoded_bytes);

            // Split encoded bytes into side effects chunk bytes by provided delimeter
            let side_effects_bytes = Self::parse_side_effects(encoded_bytes.clone());

            let mut side_effects = SideEffectsVec::<T>::new();

            // Ensure side effects length limit bound satisfied
            ensure!(
                T::SideEffectsLengthLimit::get() as usize >= side_effects_bytes.len(),
                Error::<T>::SideEffectsLengthLimitExceeded
            );

            for side_effect in side_effects_bytes {
                // Split side effect bytes into parameters chunk bytes by provided delimeter
                let side_effect_parameters_bytes = Self::parse_parameters(side_effect);

                // Transform side effect parameters bytes into the volatile side effect
                let side_effect = Self::to_volatile_side_effect(side_effect_parameters_bytes)?;

                log::info!("\n Side effect --> {:#?}", side_effect);

                side_effects
                    .try_push(side_effect)
                    // Should not fail here
                    .map_err(|_| Error::<T>::SideEffectsLengthLimitExceeded)?;
            }

            //
            // == MUTATION SAFE ==
            //

            let execution_id = if let Some(execution_id) = Self::next_execution_id() {
                // Increment the value read from storage; will error in the event of overflow.
                execution_id
            } else {
                T::ExecutionId::zero()
            };

            // Compute next execution id
            let next_execution_id = execution_id
                .checked_add(&T::ExecutionId::one())
                .ok_or(Error::<T>::StorageOverflow)?;

            // Update side effects under given execution_id in storage
            <SideEffects<T>>::insert(execution_id, side_effects);

            // Update the value of next execution id in storage.
            <NextExecutionId<T>>::put(next_execution_id);

            // Emit an event.
            Self::deposit_event(Event::Executed {
                who: account_id,
                execution_id,
                encoded_bytes,
            });

            Ok(())
        }

        /// Commit a number of side effects steps under given execution id
        #[pallet::call_index(1)]
        #[pallet::weight(0)]
        pub fn commit(
            origin: OriginFor<T>,
            actor_id: T::ActorId,
            execution_id: T::ExecutionId,
            number_of_steps: u32,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/main-docs/build/origins/
            let account_id = ensure_signed(origin)?;

            // Ensures specific actor under given account id can commit
            <T as ActorAuthenticator<T>>::authorize_for_commit(&account_id, actor_id)?;

            // Ensure number of steps to commit is greater then zero
            ensure!(number_of_steps > 0, Error::<T>::NothingToCommit);

            // Ensure side effects under given execution id exists
            let side_effects = Self::ensure_side_effects_exist(execution_id)?;

            // Ensure given number of side effects steps can be successfully commited
            let number_of_commited_side_effects =
                Self::ensure_can_be_commited(&side_effects, number_of_steps)?;

            let commited_side_effects = Self::do_commit(
                side_effects,
                number_of_commited_side_effects,
                number_of_steps,
            )?;

            //
            // == MUTATION SAFE ==
            //

            // Update side effects under given execution_id in storage
            <SideEffects<T>>::insert(execution_id, commited_side_effects);

            // Emit an event.
            Self::deposit_event(Event::Commited {
                who: account_id,
                execution_id,
                number_of_steps,
            });

            Ok(())
        }

        /// Revert commited side effects under given execution id
        #[pallet::call_index(2)]
        #[pallet::weight(0)]
        pub fn revert(
            origin: OriginFor<T>,
            actor_id: T::ActorId,
            execution_id: T::ExecutionId,
        ) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/main-docs/build/origins/
            let account_id = ensure_signed(origin)?;

            // Ensure specific actor under given account id can revert
            <T as ActorAuthenticator<T>>::authorize_for_revert(&account_id, actor_id)?;

            // Ensure side effects under given execution id exists
            let side_effects = Self::ensure_side_effects_exist(execution_id)?;

            let number_of_commited_side_effects =
                Self::get_number_of_commited_side_effects(&side_effects);

            ensure!(
                number_of_commited_side_effects > 0,
                Error::<T>::NothingToRevert
            );

            let reverted_side_effects =
                Self::do_revert(side_effects, number_of_commited_side_effects)?;

            //
            // == MUTATION SAFE ==
            //

            // Update side effects under given execution_id in storage
            <SideEffects<T>>::insert(execution_id, reverted_side_effects);

            // Emit an event.
            Self::deposit_event(Event::Reverted {
                who: account_id,
                execution_id,
            });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Commit side effects in a given commit range
        fn do_commit(
            mut side_effects: SideEffectsVec<T>,
            number_of_commited_side_effects: usize,
            number_of_steps: u32,
        ) -> Result<SideEffectsVec<T>, sp_runtime::DispatchError> {
            let commit_range = number_of_commited_side_effects
                ..number_of_commited_side_effects + number_of_steps as usize;

            <T as SideEffectsHandlerTrait>::perform_commit(&side_effects[commit_range.clone()])?;

            for i in commit_range {
                side_effects[i].commit();
            }

            Ok(side_effects)
        }

        /// Revert commited side effects
        fn do_revert(
            mut side_effects: SideEffectsVec<T>,
            number_of_commited_side_effects: usize,
        ) -> Result<SideEffectsVec<T>, sp_runtime::DispatchError> {
            let revert_range = 0..number_of_commited_side_effects;

            <T as SideEffectsHandlerTrait>::perform_revert(&side_effects[revert_range.clone()])?;

            for i in revert_range {
                side_effects[i].revert();
            }
            Ok(side_effects)
        }

        /// Splits encoded bytes into side effects chunk bytes by provided delimeter
        fn parse_side_effects(encoded_bytes: Vec<u8>) -> Vec<Vec<u8>> {
            encoded_bytes
                .split(|byte| *byte == T::SideEffectsSplitter::get())
                // Filter empty side effects bytes chunks
                .filter_map(|subslice| {
                    if subslice.is_empty() {
                        None
                    } else {
                        Some(subslice.to_vec())
                    }
                })
                .collect()
        }

        /// Splits side effect bytes into parameters chunk bytes by provided delimeter
        fn parse_parameters(side_effect_bytes: Vec<u8>) -> Vec<Vec<u8>> {
            side_effect_bytes
                .split(|byte| *byte == T::ParametersSplitter::get())
                // Filter empty chunks of side effect parameters bytes
                .filter_map(|subslice| {
                    if subslice.is_empty() {
                        None
                    } else {
                        Some(subslice.to_vec())
                    }
                })
                .collect()
        }

        /// Transform side effect parameters bytes into the volatile side effect
        fn to_volatile_side_effect(
            side_effect_parameters_bytes: Vec<Vec<u8>>,
        ) -> Result<SideEffectType, sp_runtime::DispatchError> {
            log::info!(
                "\n Side effect parameters bytes --> {:#?}",
                side_effect_parameters_bytes
            );

            // Ensure numbers of parameters in side effect is valid
            Self::ensure_side_effect_parameters_length_is_correct(&side_effect_parameters_bytes)?;

            // Ensure given chain id byte can be successfully transformed into Chain
            let chain_id = Self::ensure_chain_id_is_correct(&side_effect_parameters_bytes[0])?;

            // Ensure given bytes sequence is either Chain or ExecutionType
            let chain_or_execution_id =
                Self::ensure_chain_or_execution_type(&side_effect_parameters_bytes[1])?;

            match chain_or_execution_id {
                ChainOrExecutionType::ExecutionType(execution_id) => match execution_id {
                    ExecutionTypes::Swap => {
                        // Transform swap execution type bytes into ExecutionType::Swap
                        let swap_execution_type =
                            Self::parse_swap_execution_type(&side_effect_parameters_bytes[2..])?;

                        // Construct new SideEffectTypeRecord instance with parameters provided
                        let side_effect = SideEffectTypeRecord::construct_side_effect(
                            chain_id,
                            None,
                            swap_execution_type,
                        );

                        Ok(side_effect)
                    }
                    ExecutionTypes::Tran => {
                        // Transform tran execution type bytes into ExecutionType::Tran
                        let tran_execution_type =
                            Self::parse_tran_execution_type(&side_effect_parameters_bytes[2..])?;

                        // Construct new SideEffectTypeRecord instance with parameters provided
                        let side_effect = SideEffectTypeRecord::construct_side_effect(
                            chain_id,
                            None,
                            tran_execution_type,
                        );

                        Ok(side_effect)
                    }
                    _ => Err(Error::<T>::ExecutionTypeByteIsNotSupported.into()),
                },
                ChainOrExecutionType::Chain(chain_to) => {
                    // Ensure given execution type id byte can be successfully transformed into
                    // ExecutionTypes
                    let execution_id = Self::ensure_execution_type_id_is_correct(
                        &side_effect_parameters_bytes[2],
                    )?;

                    match execution_id {
                        ExecutionTypes::MultiTran => {
                            // Transform multi tran execution type bytes into
                            // ExecutionType::MultiTran
                            let multi_tran_execution_type = Self::parse_multi_tran_execution_type(
                                &side_effect_parameters_bytes[3..],
                            )?;

                            // Construct new SideEffectTypeRecord instance with parameters provided
                            let side_effect = SideEffectTypeRecord::construct_side_effect(
                                chain_id,
                                Some(chain_to),
                                multi_tran_execution_type,
                            );

                            Ok(side_effect)
                        }
                        _ => Err(Error::<T>::ExecutionTypeByteIsNotSupported.into()),
                    }
                }
            }
        }

        /// Ensures numbers of parameters in side effect is valid
        fn ensure_side_effect_parameters_length_is_correct(
            side_effect_parameter_bytes: &[Vec<u8>],
        ) -> Result<(), sp_runtime::DispatchError> {
            match side_effect_parameter_bytes.len() {
                // Either 5 or 7 parameters for all present side effects
                5 | 7 => Ok(()),
                _ => Err(Error::<T>::IncorrectSideEffectParametersLength.into()),
            }
        }

        /// Ensures side effects under given execution id exists
        fn ensure_side_effects_exist(
            execution_id: T::ExecutionId,
        ) -> Result<SideEffectsVec<T>, sp_runtime::DispatchError> {
            Self::side_effects(execution_id).ok_or(Error::<T>::SideEffectsNotFound.into())
        }

        /// Ensures given number of side effects steps can be successfully commited
        fn ensure_can_be_commited(
            side_effects_vec: &SideEffectsVec<T>,
            number_of_steps: u32,
        ) -> Result<usize, sp_runtime::DispatchError> {
            let number_of_commited_side_effects =
                Self::get_number_of_commited_side_effects(side_effects_vec);

            let number_of_side_effects_left_to_commit =
                (side_effects_vec.len() - number_of_commited_side_effects) as u32;
            ensure!(
                number_of_steps <= number_of_side_effects_left_to_commit,
                Error::<T>::NumberOfStepsToCommitIsGreaterThenNumberOfSideEffectsLeftToCommit,
            );
            Ok(number_of_commited_side_effects)
        }

        /// Retrieves the number of commited side effects from side effects provided
        fn get_number_of_commited_side_effects(side_effects_vec: &SideEffectsVec<T>) -> usize {
            side_effects_vec
                .iter()
                .filter(|side_effect| side_effect.is_commited())
                .count()
        }

        /// Ensures given bytes sequence is either Chain or ExecutionType
        fn ensure_chain_or_execution_type(
            chain_or_execution_id: &[u8],
        ) -> Result<ChainOrExecutionType, sp_runtime::DispatchError> {
            let parameter = Self::ensure_single_byte_representation(chain_or_execution_id)?;
            if let Some(chain_id) = Chain::from_u8(parameter) {
                Ok(ChainOrExecutionType::Chain(chain_id))
            } else {
                ExecutionTypes::from_u8(parameter)
                    .map(|execution_type| ChainOrExecutionType::ExecutionType(execution_type))
                    .ok_or(Error::<T>::IncorrectChainIdOrExecutionTypeByte.into())
            }
        }

        /// Ensures given chain id byte can be successfully transformed into Chain
        fn ensure_chain_id_is_correct(chain_id: &[u8]) -> Result<Chain, sp_runtime::DispatchError> {
            let parameter = Self::ensure_single_byte_representation(chain_id)?;

            Chain::from_u8(parameter).ok_or(Error::<T>::IncorrectChainIdByte.into())
        }

        /// Ensures given asset id byte can be successfully transformed into Asset
        fn ensure_asset_id_is_correct(asset_id: &[u8]) -> Result<Asset, sp_runtime::DispatchError> {
            let parameter = Self::ensure_single_byte_representation(asset_id)?;

            Asset::from_u8(parameter).ok_or(Error::<T>::IncorrectAssetIdByte.into())
        }

        /// Ensures given execution type id byte can be successfully transformed into ExecutionTypes
        fn ensure_execution_type_id_is_correct(
            execution_type_id: &[u8],
        ) -> Result<ExecutionTypes, sp_runtime::DispatchError> {
            let parameter = Self::ensure_single_byte_representation(execution_type_id)?;

            ExecutionTypes::from_u8(parameter).ok_or(Error::<T>::IncorrectExecutionTypeByte.into())
        }

        /// Ensures given parameter slice is a single byte representation
        fn ensure_single_byte_representation(
            parameter: &[u8],
        ) -> Result<u8, sp_runtime::DispatchError> {
            ensure!(parameter.len() == 1, Error::<T>::NotASingleByte);

            Ok(parameter[0])
        }

        /// Ensures execution type parameters length is correct
        fn ensure_parameters_length_is_correct(
            parameters_length: usize,
            expected_length: usize,
        ) -> Result<(), sp_runtime::DispatchError> {
            ensure!(
                parameters_length == expected_length,
                Error::<T>::IncorrectExecutionTypeParametersLength
            );

            Ok(())
        }

        /// Transform swap execution type bytes into `ExecutionType::Swap`
        fn parse_swap_execution_type(
            swap_execution_type_bytes: &[Vec<u8>],
        ) -> Result<ExecutionType<AccountId32>, sp_runtime::DispatchError> {
            Self::ensure_parameters_length_is_correct(
                swap_execution_type_bytes.len(),
                SWAP_EXECUTION_TYPE_LENGTH,
            )?;

            let caller = Self::ensure_account_id_is_correct(&swap_execution_type_bytes[0])?;
            let to = Self::ensure_account_id_is_correct(&swap_execution_type_bytes[1])?;

            let amount_from = Self::ensure_amount_is_correct(&swap_execution_type_bytes[2])?;
            let amount_to = Self::ensure_amount_is_correct(&swap_execution_type_bytes[3])?;

            let asset = Self::ensure_asset_id_is_correct(&swap_execution_type_bytes[4])?;
            Ok(ExecutionType::Swap {
                caller,
                to,
                amount_from,
                amount_to,
                asset,
            })
        }

        /// Transform multi tran execution type bytes into `ExecutionType::MultiTran`
        fn parse_multi_tran_execution_type(
            multi_tran_execution_type_bytes: &[Vec<u8>],
        ) -> Result<ExecutionType<AccountId32>, sp_runtime::DispatchError> {
            Self::ensure_parameters_length_is_correct(
                multi_tran_execution_type_bytes.len(),
                MULTI_TRAN_EXECUTION_TYPE_LENGTH,
            )?;

            let caller = Self::ensure_account_id_is_correct(&multi_tran_execution_type_bytes[0])?;
            let to = Self::ensure_account_id_is_correct(&multi_tran_execution_type_bytes[1])?;

            let amount = Self::ensure_amount_is_correct(&multi_tran_execution_type_bytes[2])?;

            let asset = Self::ensure_asset_id_is_correct(&multi_tran_execution_type_bytes[3])?;
            Ok(ExecutionType::MultiTran {
                caller,
                to,
                amount,
                asset,
            })
        }

        /// Transform tran execution type bytes into `ExecutionType::Tran`
        fn parse_tran_execution_type(
            tran_execution_type_bytes: &[Vec<u8>],
        ) -> Result<ExecutionType<AccountId32>, sp_runtime::DispatchError> {
            Self::ensure_parameters_length_is_correct(
                tran_execution_type_bytes.len(),
                TRAN_EXECUTION_TYPE_LENGTH,
            )?;

            let caller = Self::ensure_account_id_is_correct(&tran_execution_type_bytes[0])?;
            let to = Self::ensure_account_id_is_correct(&tran_execution_type_bytes[1])?;

            let amount = Self::ensure_amount_is_correct(&tran_execution_type_bytes[2])?;

            Ok(ExecutionType::Tran { caller, to, amount })
        }

        /// Ensures given bytes sequence can be successfuly transformed into AccountId32
        fn ensure_account_id_is_correct(
            account_id: &[u8],
        ) -> Result<AccountId32, sp_runtime::DispatchError> {
            AccountId32::try_from(account_id).map_err(|_| Error::<T>::IncorrectAccountId.into())
        }

        /// Ensures given bytes sequence can be successfuly transformed into Amount
        fn ensure_amount_is_correct(amount: &[u8]) -> Result<Amount, sp_runtime::DispatchError> {
            let amount: [u8; 8] = amount.try_into().map_err(|_| Error::<T>::IncorrectAmount)?;
            Ok(Amount::from_be_bytes(amount))
        }
    }
}
