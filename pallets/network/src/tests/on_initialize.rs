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
  OverwatchEpochLengthMultiplier,
  NewRegistrationCostMultiplier,
  SubnetConsensusSubmission,
  AccountSubnetStake,
  FinalSubnetEmissionWeights,
  MaxOverwatchNodes,
  HotkeyOverwatchNodeId,
  OverwatchCommit,
  OverwatchReveal,
  OverwatchCommits,
  OverwatchReveals,
  AccountOverwatchStake,
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

    let max_onodes = MaxOverwatchNodes::<Test>::get();
    let max_subnets = MaxSubnets::<Test>::get();
    let min_subnet_nodes = MinSubnetNodes::<Test>::get();
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let deposit_amount: u128 = get_min_stake_balance() + 500;
    let amount: u128 =         get_min_stake_balance();

    let node_count = min_subnet_nodes;
    let end = node_count;
    let overwatch_count = 16;

    // Register max subnets and subnet nodes
    build_activated_subnet_with_overwatch_nodes_v2(
      0, 
      end, 
      overwatch_count,
      deposit_amount, 
      amount
    );

    // default onode weights
    let weight: u128 = 1000000000000000000;
    let salt: Vec<u8> = b"secret-salt".to_vec();
    let commit_hash = make_commit(weight, salt.clone());

    let epoch_length = EpochLength::get();
    let multiplier = OverwatchEpochLengthMultiplier::<Test>::get();
    let overwatch_epoch_length = epoch_length.saturating_mul(multiplier);

    let epochs = 16;
    let mut epochs_complete = 0;

    let mut last_overwatch_commit_epoch = 0;
    let mut last_overwatch_reveal_epoch = 0;

    let mut snodes_rewarded = false;
    let mut overwatch_rewarded = false;


    let mut calculate_overwatch_rewards_v3_ran = false;
    let mut do_epoch_preliminaries_ran = false;
    let mut handle_subnet_emission_weights_ran = false;
    let mut emission_step_ran = false;

    for _ in 0..(epochs*epoch_length*multiplier) {
      let block = System::block_number();
      // log::error!("block: {:?}", block);

      let current_epoch = block.saturating_div(epoch_length);
      let current_overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();
      let epoch_slot = block % epoch_length;

			if (block - 1) >= overwatch_epoch_length && (block - 1) % overwatch_epoch_length == 0 {
        let mut ostake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> = BTreeMap::new();
        if last_overwatch_reveal_epoch > 0 && current_overwatch_epoch > last_overwatch_reveal_epoch {
          for n in end-1..end+overwatch_count {
            let _n = n + 1;
            let o_n = _n - end + 1;
            let coldkey = get_overwatch_coldkey(max_subnet_nodes, max_subnets, max_onodes, o_n);
            let hotkey = get_overwatch_hotkey(max_subnet_nodes, max_subnets, max_onodes, _n);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());

            assert_ne!(overwatch_stake, 0);
            ostake_snapshot.insert(hotkey.clone(), overwatch_stake);
          }
        }

        Network::on_initialize(block);
        calculate_overwatch_rewards_v3_ran = true;
        
        if last_overwatch_reveal_epoch > 0 && current_overwatch_epoch > last_overwatch_reveal_epoch {
          for n in end-1..end+overwatch_count {
            let _n = n + 1;
            let o_n = _n - end + 1;
            let coldkey = get_overwatch_coldkey(max_subnet_nodes, max_subnets, max_onodes, o_n);
            let hotkey = get_overwatch_hotkey(max_subnet_nodes, max_subnets, max_onodes, _n);
            let overwatch_stake = AccountOverwatchStake::<Test>::get(hotkey.clone());

            if let Some(old_stake) = ostake_snapshot.get(&hotkey) {
              assert!(overwatch_stake > *old_stake);
              if !overwatch_rewarded {
                overwatch_rewarded = true;
              }
            } else {
              assert!(false); // auto-fail
            }
          }
        }
      }

      // Overwatch
      let in_commit_period = Network::in_overwatch_commit_period();

      if in_commit_period && last_overwatch_commit_epoch != current_overwatch_epoch {
        let mut commits = Vec::new();
        for s in 0..max_subnets {
          let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
          let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
          let commit = OverwatchCommit {
            subnet_id: subnet_id,
            weight: commit_hash
          };
          commits.push(commit);
        }

        for n in end-1..end+overwatch_count {
          let _n = n + 1;
          let o_n = _n - end + 1;
          let coldkey = get_overwatch_coldkey(max_subnet_nodes, max_subnets, max_onodes, o_n);
          let hotkey = get_overwatch_hotkey(max_subnet_nodes, max_subnets, max_onodes, _n);
          let overwatch_node_id = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();  
          assert_ok!(
            Network::commit_overwatch_subnet_weights(
              RuntimeOrigin::signed(hotkey.clone()),
              overwatch_node_id,
              commits.clone()
            )
          );

          for s in 0..max_subnets {
            let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
            let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

            let stored = OverwatchCommits::<Test>::get((current_overwatch_epoch, overwatch_node_id, subnet_id)).unwrap();
            assert_eq!(stored, commit_hash);
          }
        }
        
        last_overwatch_commit_epoch = current_overwatch_epoch;
      } else if !in_commit_period && last_overwatch_reveal_epoch != current_overwatch_epoch {
        let mut reveals = Vec::new();
        for s in 0..max_subnets {
          let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
          let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
          let reveal = OverwatchReveal {
            subnet_id: subnet_id,
            weight: weight,
            salt: salt.clone()
          };
          reveals.push(reveal);
        }

        for n in end-1..end+overwatch_count {
          let _n = n + 1;
          let o_n = _n - end + 1;
          let coldkey = get_overwatch_coldkey(max_subnet_nodes, max_subnets, max_onodes, o_n);
          let hotkey = get_overwatch_hotkey(max_subnet_nodes, max_subnets, max_onodes, _n);
          let overwatch_node_id = HotkeyOverwatchNodeId::<Test>::get(hotkey.clone()).unwrap();
          assert_ok!(
            Network::reveal_overwatch_subnet_weights(
              RuntimeOrigin::signed(hotkey.clone()),
              overwatch_node_id,
              reveals.clone()
            )
          );
          for s in 0..max_subnets {
            let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
            let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

            let revealed = OverwatchReveals::<Test>::get((current_overwatch_epoch, subnet_id, overwatch_node_id)).unwrap();
            assert_eq!(revealed, weight);
          }
        }

        last_overwatch_reveal_epoch = current_overwatch_epoch;
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
          for n in 0..end {
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
          for n in 0..end {
            let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, n+1);

            let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

            if let Some(old_stake) = stake_snapshot.get(&hotkey) {
              assert!(stake > *old_stake);
              if !snodes_rewarded {
                snodes_rewarded = true;
              }
            } else {
              assert!(false); // auto-fail
            }
          }

        }
			}

      System::set_block_number(block + 1);
    }

    // assert_eq!(epochs, epochs_complete);
    assert!(last_overwatch_commit_epoch > 0);
    assert!(last_overwatch_reveal_epoch > 0);
    assert_eq!(calculate_overwatch_rewards_v3_ran, true);
    assert_eq!(snodes_rewarded, true);
    assert_eq!(overwatch_rewarded, true);

    for s in 0..max_subnets {
      let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
      // - Ensure subnet is present. `unwrap` will panic is doesn't exist
      let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    }
  });
}

// #[test]
// fn test_on_initialize() {
//   new_test_ext().execute_with(|| {
//     NewRegistrationCostMultiplier::<Test>::put(1200000000000000000);
//     let alice = 0;
//     // let _ = Balances::deposit_creating(&account(alice), 1000000000000000000000000);

//     let max_subnets = MaxSubnets::<Test>::get();
//     let min_subnet_nodes = MinSubnetNodes::<Test>::get();
//     let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//     let deposit_amount: u128 = get_min_stake_balance() + 500;
//     let amount: u128 =         get_min_stake_balance();

//     let node_count = min_subnet_nodes;

//     // Register max subnets and subnet nodes
//     // for s in 0..max_subnets {
//     //   let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
// 		// 	build_activated_subnet_new(
// 		// 		subnet_name.clone().into(), 
// 		// 		0, 
// 		// 		node_count, 
// 		// 		deposit_amount, 
// 		// 		amount
// 		// 	);
//     // }
//     build_activated_subnet_with_overwatch_nodes_v2(
//       0, 
//       node_count, 
//       16,
//       deposit_amount, 
//       amount
//     );
//     // Register overwatch nodes
//     // build_overwatch_nodes(0, 12, get_min_overwatch_stake_balance());

//     let epoch_length = EpochLength::get();
//     let epochs = 16;
//     let mut epochs_complete = 0;

//     let mut last_overwatch_commit_epoch = 0;
//     let mut last_overwatch_reveal_epoch = 0;
//     for _ in 0..(epochs*epoch_length) {
//       let block = System::block_number();
//       log::error!("block: {:?}", block);
//       let current_epoch = block.saturating_div(epoch_length);
//       let epoch_slot = block % epoch_length;

//       // Overwatch
//       if (block - 2) >= epoch_length && (block - 2) % epoch_length == 0 {
//         let current_overwatch_epoch = Network::get_current_overwatch_epoch_as_u32();
//         let in_commit_period = Network::in_overwatch_commit_period();

//         if in_commit_period && last_overwatch_commit_epoch != current_overwatch_epoch {
//           for s in 0..max_subnets {

//           }
//         } else if !in_commit_period && last_overwatch_reveal_epoch != current_overwatch_epoch {
//           for s in 0..max_subnets {

//           }
//         }

//         last_overwatch_commit_epoch = current_overwatch_epoch;
//       }

//       if block >= epoch_length && block % epoch_length == 0 {
//         epochs_complete += 1;
//         for s in 0..max_subnets {
//           // - Subnets must have min dstake at this time in the block steps
//           let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
//           let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//           let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
//           let mut min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);
//           if total_delegate_stake_balance < min_subnet_delegate_stake {
//             let mut delta = min_subnet_delegate_stake - total_delegate_stake_balance;
//             assert_ok!(
//               Network::add_to_delegate_stake(
//                 RuntimeOrigin::signed(account(alice)),
//                 subnet_id,
//                 delta,
//               ) 
//             );
//           }

//           // Get subnet epoch, is previous epoch because we're on block step 0
//           let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
//           let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
//           if validator_id != None {
//             run_subnet_consensus_step(subnet_id);
//           }
//         }
// 				// Remove unqualified subnets
// 				// Network::do_epoch_preliminaries(block, current_epoch);
//         Network::on_initialize(block);
// 			} else if (block - 2) >= epoch_length && (block - 2) % epoch_length == 0 {
// 				// Network::handle_subnet_emission_weights(current_epoch);
//         Network::on_initialize(block);

//         // - Ensure `handle_subnet_emission_weights` ran
//         let subnet_emission_weights = FinalSubnetEmissionWeights::<Test>::get(current_epoch);
//         for s in 0..max_subnets {
//           let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
//           let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//           let subnet_weight = subnet_emission_weights.weights.get(&subnet_id);
//           assert!(subnet_weight.is_some());
//         }
// 			} else if let Some(subnet_id) = SlotAssignment::<Test>::get(epoch_slot) {
//         let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

//         let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, subnet_epoch - 1);
//         if epochs_complete > 1 {
//           assert!(submission != None);
//         }
//         let mut stake_snapshot: BTreeMap<<Test as frame_system::Config>::AccountId, u128> = BTreeMap::new();
//         if submission != None {
//           for n in 0..node_count {
//             let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, n+1);

//             let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

//             assert_ne!(stake, 0);
//             stake_snapshot.insert(hotkey.clone(), stake);
//           }
//         }

// 				// Network::emission_step(block, current_epoch, subnet_epoch, subnet_id);
//         Network::on_initialize(block);

//         // - Ensure rewards were distributed
//         if submission != None {
//           for n in 0..node_count {
//             let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, n+1);

//             let stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

//             if let Some(old_stake) = stake_snapshot.get(&hotkey) {
//               assert!(stake > *old_stake);
//             } else {
//               assert!(false); // auto-fail
//             }
//           }

//         }
// 			}

//       System::set_block_number(block + 1);
//     }

//     assert_eq!(epochs, epochs_complete);

//     for s in 0..max_subnets {
//       let subnet_name: Vec<u8> = format!("subnet-name-{s}").into();
//       // - Ensure subnet is present. `unwrap` will panic is doesn't exist
//       let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//     }
//   });
// }
