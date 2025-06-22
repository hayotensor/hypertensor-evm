use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use log::info;
use frame_support::traits::{OnInitialize, Currency};
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use frame_support::BoundedVec;
use sp_core::OpaquePeerId as PeerId;
use crate::{
  SubnetName, 
  MinStakeBalance,
  RegistrationQueue,
  ChurnLimit,
  RegistrationQueueEpochs,
};

///
///
///
///
///
///
///
/// Subnet Nodes Queue
///
///
///
///
///
///
///

#[test]
fn test_single_node_entry() {
  new_test_ext().execute_with(|| {

    let subnet_path: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let n_peers = 8;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, n_peers, deposit_amount, stake_amount);

    let subnet_id = SubnetName::<Test>::get(subnet_path.clone()).unwrap();

    ChurnLimit::<Test>::insert(subnet_id, 2);
    RegistrationQueueEpochs::<Test>::insert(subnet_id, 5);

    let start_epoch = Network::insert_subnet_node_to_queue(subnet_id, 42, 100);
    assert_eq!(start_epoch, 105); // 100 + 5 + (0 / 2)
    assert_eq!(RegistrationQueue::<Test>::get(1), vec![42]);
  });
}

#[test]
fn test_multiple_nodes_churn_2() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;

    ChurnLimit::<Test>::insert(subnet_id, 2);
    RegistrationQueueEpochs::<Test>::insert(subnet_id, 5);

    let e1 = Network::insert_subnet_node_to_queue(1, 1, 100);
    let e2 = Network::insert_subnet_node_to_queue(1, 2, 100);
    let e3 = Network::insert_subnet_node_to_queue(1, 3, 100);
    let e4 = Network::insert_subnet_node_to_queue(1, 4, 100);

    assert_eq!(e1, 105); // pos 0 → +0
    assert_eq!(e2, 105); // pos 1 → +0
    assert_eq!(e3, 106); // pos 2 → +1
    assert_eq!(e4, 106); // pos 3 → +1

    assert_eq!(RegistrationQueue::<Test>::get(1), vec![1, 2, 3, 4]);
  });
}

#[test]
fn test_duplicate_removal() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;

    ChurnLimit::<Test>::insert(subnet_id, 2);
    RegistrationQueueEpochs::<Test>::insert(subnet_id, 5);

    Network::insert_subnet_node_to_queue(1, 1, 100);
    Network::insert_subnet_node_to_queue(1, 2, 100);
    Network::insert_subnet_node_to_queue(1, 1, 100); // reinsert

    // 1 should now be at the end
    assert_eq!(RegistrationQueue::<Test>::get(1), vec![2, 1]);
  });
}

#[test]
fn test_churn_limit_1() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;

    ChurnLimit::<Test>::insert(subnet_id, 1);
    RegistrationQueueEpochs::<Test>::insert(subnet_id, 3);

    let e1 = Network::insert_subnet_node_to_queue(1, 10, 50);
    let e2 = Network::insert_subnet_node_to_queue(1, 11, 50);
    let e3 = Network::insert_subnet_node_to_queue(1, 12, 50);

    assert_eq!(e1, 53); // pos 0 → +0
    assert_eq!(e2, 54); // pos 1 → +1
    assert_eq!(e3, 55); // pos 2 → +2
  });
}

#[test]
fn test_queue_epochs_affect_start_epoch() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;

    ChurnLimit::<Test>::insert(subnet_id, 3);
    RegistrationQueueEpochs::<Test>::insert(subnet_id, 10);

    let e = Network::insert_subnet_node_to_queue(1, 99, 200);
    assert_eq!(e, 210); // 200 + 10 + (0 / 3)
  });
}

#[test]
fn test_large_queue_positions() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;

    ChurnLimit::<Test>::insert(subnet_id, 4);
    RegistrationQueueEpochs::<Test>::insert(subnet_id, 2);

      for i in 0..12 {
        let epoch = Network::insert_subnet_node_to_queue(1, i, 50);
        // Position i → floor(i / 4) → additional_epochs
        let expected_epoch = 52 + (i / 4);
        assert_eq!(epoch, expected_epoch);
      }

      assert_eq!(RegistrationQueue::<Test>::get(1).len(), 12);
  });
}
