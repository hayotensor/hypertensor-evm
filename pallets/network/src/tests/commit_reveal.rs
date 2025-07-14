use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use log::info;
use crate::{
  SubnetsData,
  OverwatchCommits,
  OverwatchReveals,
  SubnetState,
  SubnetData,
  OverwatchNode,
  OverwatchNodes,
  OverwatchNodeIdHotkey,
  OverwatchCommit,
  OverwatchReveal,
  TotalOverwatchNodeUids,
  HotkeyOwner,
};
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use sp_runtime::traits::Hash;

//
//
//
//
//
//
//
// Overwatch Commit-Reveal
//
//
//
//
//
//
//

fn make_commit(weight: u128, salt: Vec<u8>) -> sp_core::H256 {
  Hashing::hash_of(&(weight, salt))
}

#[test]
fn test_commit_and_reveal_weights_success() {
  new_test_ext().execute_with(|| {
    let coldkey: AccountId = account(1);
    let hotkey: AccountId = account(2);
    let overwatch_node_id = 1;
    let subnet_id = 99;
    let epoch = 0;

    // Setup: assign ownership and create subnet
    let subnet_data = SubnetData {
      id: 1,
      name: "subnet_name".into(),
      repo: "github".into(),
      description: "description".into(),
      misc: "misc".into(),
      state: SubnetState::Active,
      start_epoch: 0,
    };

    SubnetsData::<Test>::insert(subnet_id, subnet_data);

    TotalOverwatchNodeUids::<Test>::mutate(|n: &mut u32| *n += 1);
    let current_uid = TotalOverwatchNodeUids::<Test>::get();

    let overwatch_node = OverwatchNode {
      id: current_uid,
      hotkey: hotkey.clone(),
    };

    OverwatchNodes::<Test>::insert(overwatch_node_id, overwatch_node);
    HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
    OverwatchNodeIdHotkey::<Test>::insert(current_uid, hotkey.clone());

    // Weight + salt
    let weight: u128 = 123456;
    let salt: Vec<u8> = b"secret-salt".to_vec();
    let commit_hash = make_commit(weight, salt.clone());

    // Commit
    assert_ok!(Network::do_commit_ow_weights(
        RuntimeOrigin::signed(hotkey.clone()),
        overwatch_node_id,
        vec![OverwatchCommit {
            subnet_id,
            weight: commit_hash
        }]
    ));

    // Ensure it's stored
    let stored = OverwatchCommits::<Test>::get((epoch, overwatch_node_id, subnet_id)).unwrap();
    assert_eq!(stored, commit_hash);

    // Reveal
    assert_ok!(Network::do_reveal_ow_weights(
        RuntimeOrigin::signed(hotkey.clone()),
        overwatch_node_id,
        vec![OverwatchReveal {
            subnet_id,
            weight,
            salt
        }]
    ));

    // Ensure revealed weight is correct
    let revealed = OverwatchReveals::<Test>::get((epoch, subnet_id, overwatch_node_id)).unwrap();
    assert_eq!(revealed, weight);
  });
}
