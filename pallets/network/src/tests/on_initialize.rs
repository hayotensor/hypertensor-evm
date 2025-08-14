use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use log::info;
use crate::{
  MaxSubnets,
  MinSubnetNodes,
  MaxSubnetNodes,
  SubnetName,
  SlotAssignment,
  TotalSubnetDelegateStakeBalance,
  SubnetElectedValidator,
  NewRegistrationCostMultiplier,
  SubnetConsensusSubmission,
  AccountSubnetStake,
  FinalSubnetEmissionWeights,
};
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use frame_support::traits::{OnInitialize, Currency};
use sp_std::collections::btree_map::BTreeMap;

//
//
//
//
//
//
//
// On Initialize Hook
//
//
//
//
//
//
//

#[test]
fn test_on_initialize() {
  new_test_ext().execute_with(|| {
    NewRegistrationCostMultiplier::<Test>::put(1200000000000000000);
    let alice = 0;
    // let _ = Balances::deposit_creating(&account(alice), 1000000000000000000000000);

    let max_subnets = MaxSubnets::<Test>::get();
    let min_subnet_nodes = MinSubnetNodes::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let deposit_amount: u128 = get_min_stake_balance() + 500;
    let amount: u128 =         get_min_stake_balance();

    let node_count = min_subnet_nodes;

    // Register max subnets and subnet nodes
    for s in 0..max_subnets {
      let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
			build_activated_subnet_new(
				subnet_name.clone().into(), 
				0, 
				node_count, 
				deposit_amount, 
				amount
			);
    }

    let epoch_length = EpochLength::get();
    let epochs = 16;
    let mut epochs_complete = 0;
    for _ in 0..(epochs*epoch_length) {
      let block = System::block_number();
      log::error!("block: {:?}", block);
      let current_epoch = block.saturating_div(epoch_length);
      let epoch_slot = block % epoch_length;


      if (block - 2) >= epoch_length && (block - 2) % epoch_length == 0 {

      }

      if block >= epoch_length && block % epoch_length == 0 {
        epochs_complete += 1;
        for s in 0..max_subnets {
          // - Subnets must have min dstake at this time in the block steps
          let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
          let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
          let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
          let mut min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);
          if total_delegate_stake_balance < min_subnet_delegate_stake {
            let mut delta = min_subnet_delegate_stake - total_delegate_stake_balance;
            assert_ok!(
              Network::add_to_delegate_stake(
                RuntimeOrigin::signed(account(alice)),
                subnet_id,
                delta,
              ) 
            );
          }

          // Get subnet epoch, is previous epoch because we're on block step 0
          let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
          let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
          if validator_id != None {
            run_subnet_consensus_step(subnet_id);
          }
        }
				// Remove unqualified subnets
				// Network::do_epoch_preliminaries(block, current_epoch);
        Network::on_initialize(block);
			} else if (block - 2) >= epoch_length && (block - 2) % epoch_length == 0 {
				// Network::handle_subnet_emission_weights(current_epoch);
        Network::on_initialize(block);

        // - Ensure `handle_subnet_emission_weights` ran
        let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(current_epoch);
        for s in 0..max_subnets {
          let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
          let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
          let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
          assert!(subnet_weight.is_some());
        }
			} else if let Some(subnet_id) = SlotAssignment::<Test>::get(epoch_slot) {
        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, subnet_epoch - 1);
        if epochs_complete > 1 {
          assert!(submission != None);
        }
        let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> = BTreeMap::new();
        if submission != None {
          for n in 0..node_count {
            let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, n+1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            assert_ne!(stake, 0);
            stake_snapshot.insert(hotkey.clone(), stake);
          }
        }

				// Network::emission_step(block, current_epoch, subnet_epoch, subnet_id);
        Network::on_initialize(block);

        // - Ensure rewards were distributed
        if submission != None {
          for n in 0..node_count {
            let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, n+1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
              assert!(stake > *old_stake);
            } else {
              assert!(false); // auto-fail
            }
          }

        }
			}

      System::set_block_number(block + 1);
    }

    assert_eq!(epochs, epochs_complete);

    for s in 0..max_subnets {
      let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
      // - Ensure subnet is present. `unwrap` will panic is doesn't exist
      let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    }
  });
}
