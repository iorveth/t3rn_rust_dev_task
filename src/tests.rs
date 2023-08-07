use crate::{mock::*, Event, *};
use frame_support::assert_ok;
use std::str::FromStr;

#[test]
fn test_bytes_to_side_effects_extrinsics() {
    new_test_ext().execute_with(|| {
        // Go past genesis block so events get deposited
        System::set_block_number(1);

        // Runtime state before call
        assert_eq!(TemplateModule::next_execution_id(), None);
        assert_eq!(SideEffects::<Test>::iter().count(), 0);

        let first_account_id: [u8; 32] = *AccountId32::from_str(FIRST_ACCOUNT_ID).unwrap().as_ref();

        let second_account_id: [u8; 32] =
            *AccountId32::from_str(SECOND_ACCOUNT_ID).unwrap().as_ref();
        let amount = AMOUNT.to_be_bytes();

        // Add multi tran execution side effect
        let mut encoded_bytes: Vec<u8> = vec![
            KUSAMA_CHAIN_OR_ASSET,
            PARAMETERS_SPLITTER,
            POLKADOT_CHAIN_OR_ASSET,
            PARAMETERS_SPLITTER,
            MULTI_TRAN_EXECUTION_TYPE,
            PARAMETERS_SPLITTER,
        ];

        encoded_bytes.append(&mut first_account_id.to_vec());
        encoded_bytes.push(PARAMETERS_SPLITTER);
        encoded_bytes.append(&mut second_account_id.to_vec());
        encoded_bytes.push(PARAMETERS_SPLITTER);
        encoded_bytes.append(&mut amount.to_vec());
        encoded_bytes.push(PARAMETERS_SPLITTER);
        encoded_bytes.push(POLKADOT_CHAIN_OR_ASSET);

        encoded_bytes.push(SIDE_EFFECTS_SPLITTER);

        let third_account_id: [u8; 32] = *AccountId32::from_str(THIRD_ACCOUNT_ID).unwrap().as_ref();

        // Add tran execution side effect
        encoded_bytes.push(POLKADOT_CHAIN_OR_ASSET);
        encoded_bytes.push(PARAMETERS_SPLITTER);
        encoded_bytes.push(TRAN_EXECUTION_TYPE);
        encoded_bytes.push(PARAMETERS_SPLITTER);
        encoded_bytes.append(&mut second_account_id.to_vec());
        encoded_bytes.push(PARAMETERS_SPLITTER);
        encoded_bytes.append(&mut third_account_id.to_vec());
        encoded_bytes.push(PARAMETERS_SPLITTER);
        encoded_bytes.append(&mut amount.to_vec());

        // Execute generate_side_effects_steps extrinsic
        assert_ok!(TemplateModule::generate_side_effects_steps(
            RuntimeOrigin::signed(1),
            encoded_bytes.clone()
        ));

        let execution_id = 0;

        // Runtime tested state after call
        assert_eq!(TemplateModule::next_execution_id(), Some(1));

        // Ensure side effects added succesfully
        assert_eq!(SideEffects::<Test>::iter().count(), 1);

        let first_execution_type = ExecutionType::<AccountId32>::MultiTran {
            caller: AccountId32::from_str(FIRST_ACCOUNT_ID).unwrap(),
            to: AccountId32::from_str(SECOND_ACCOUNT_ID).unwrap(),
            amount: AMOUNT,
            asset: Asset::from_u8(POLKADOT_CHAIN_OR_ASSET).unwrap(),
        };

        let first_side_effect = SideEffectTypeRecord::<AccountId32>::MultiChain {
            chain_from: Chain::from_u8(KUSAMA_CHAIN_OR_ASSET).unwrap(),
            chain_to: Chain::from_u8(POLKADOT_CHAIN_OR_ASSET).unwrap(),
            side_effect: SideEffect::new(first_execution_type),
        };

        let second_execution_type = ExecutionType::<AccountId32>::Tran {
            caller: AccountId32::from_str(SECOND_ACCOUNT_ID).unwrap(),
            to: AccountId32::from_str(THIRD_ACCOUNT_ID).unwrap(),
            amount: AMOUNT,
        };

        let second_side_effect = SideEffectTypeRecord::<AccountId32>::SingleChain {
            chain: Chain::from_u8(POLKADOT_CHAIN_OR_ASSET).unwrap(),
            side_effect: SideEffect::new(second_execution_type),
        };

        let mut side_effects = vec![first_side_effect, second_side_effect];

        assert_eq!(
            TemplateModule::side_effects(execution_id)
                .unwrap()
                .into_inner(),
            side_effects
        );

        // Assert that the correct event was deposited
        System::assert_last_event(
            Event::Executed {
                who: 1,
                execution_id,
                encoded_bytes,
            }
            .into(),
        );

        // Execute commit extrinsic
        let number_of_steps = 1;
        assert_ok!(TemplateModule::commit(
            RuntimeOrigin::signed(1),
            DEFAULT_ACTOR_ID,
            execution_id,
            number_of_steps
        ));

        // Ensure side effects commited successfully
        side_effects[0].commit();
        assert_eq!(
            TemplateModule::side_effects(execution_id)
                .unwrap()
                .into_inner(),
            side_effects
        );

        // Assert that the correct event was deposited
        System::assert_last_event(
            Event::Commited {
                who: 1,
                execution_id,
                number_of_steps,
            }
            .into(),
        );

        // Execute revert extrinsic
        assert_ok!(TemplateModule::revert(
            RuntimeOrigin::signed(1),
            DEFAULT_ACTOR_ID,
            execution_id
        ));

        // Ensure side effects reverted successfully
        side_effects[0].revert();
        assert_eq!(
            TemplateModule::side_effects(execution_id)
                .unwrap()
                .into_inner(),
            side_effects
        );

        // Assert that the correct event was deposited
        System::assert_last_event(
            Event::Reverted {
                who: 1,
                execution_id,
            }
            .into(),
        );
    });
}
