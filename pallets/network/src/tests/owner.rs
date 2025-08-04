use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use log::info;
use frame_support::traits::{OnInitialize, Currency};
use sp_std::collections::btree_set::BTreeSet;
use crate::{
  Error,
  SubnetName, 
  SubnetOwner,
  SubnetsData,
  SubnetState,
  RegisteredSubnetNodesData,
  HotkeySubnetNodeId,
  SubnetNodeClass,
  SubnetNode,
  SubnetNodeClassification,
  NetworkMinStakeBalance,
  SubnetRemovalReason,
  SubnetRepo,
  ChurnLimit,
  RegistrationQueueEpochs,
  ActivationGraceEpochs,
  IdleClassificationEpochs,
  IncludedClassificationEpochs,
  MaxSubnetNodePenalties,
  SubnetRegistrationInitialColdkeys,
  SubnetKeyTypes,
  NodeRemovalSystemV2,
  TotalActiveSubnetNodes,
  MaxSubnetNodes,
  HotkeyOwner,
  SubnetMinStakeBalance,
  SubnetMaxStakeBalance,
  LastSubnetDelegateStakeRewardsUpdate,
  SubnetDelegateStakeRewardsUpdatePeriod,
  MaxSubnetDelegateStakeRewardsPercentageChange,
  SubnetDelegateStakeRewardsPercentage,
  MaxRegisteredNodes,
  SubnetBootnodeAccess,
  SubnetData,
  KeyType,
  NodeRemovalPolicy,
  LogicExpr,
  NodeRemovalConditionType,
};
use sp_runtime::traits::TrailingZeroInput;
use codec::{Decode, Encode};
use sp_runtime::BoundedVec;

//
//
//
//
//
//
//
// Subnets Add/Remove
//
//
//
//
//
//
//
// owner_pause_subnet
// owner_unpause_subnet
// owner_deactivate_subnet
// owner_update_name
// owner_update_repo
// owner_update_description
// owner_update_misc
// owner_update_churn_limit
// owner_update_registration_queue_epochs
// owner_update_activation_grace_epochs
// owner_update_idle_classification_epochs
// owner_update_included_classification_epochs
// owner_update_max_node_penalties
// owner_add_initial_coldkeys
// owner_remove_initial_coldkeys
// owner_update_key_types
// owner_update_node_removal_policy
// owner_remove_subnet_node
// owner_update_min_stake
// owner_update_max_stake
// owner_update_delegate_stake_percentage
// owner_update_max_registered_nodes
// transfer_subnet_ownership
// accept_subnet_ownership
// owner_add_bootnode_access

#[test]
fn test_owner_pause_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    // Transfer to new owner
    assert_ok!(Network::owner_pause_subnet(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
    ));

    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetPaused {
        subnet_id: subnet_id, 
      }
    );

    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet_data.state, SubnetState::Paused);
    assert_eq!(subnet_data.start_epoch, epoch);
  });
}

#[test]
fn test_owner_unpause_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let coldkey = account(1000);
    let hotkey = account(1001);
    let start_epoch = epoch + 100;

    let hotkey_subnet_node_id = 1000;
    RegisteredSubnetNodesData::<Test>::insert(subnet_id, hotkey_subnet_node_id, SubnetNode {
      id: hotkey_subnet_node_id,
      hotkey: hotkey.clone(),
      peer_id: peer(0),
      bootnode_peer_id: peer(0),
      client_peer_id: peer(0),
      bootnode: None,
      delegate_reward_rate: 10,
      last_delegate_reward_rate_update: 0,
      classification: SubnetNodeClassification {
        node_class: SubnetNodeClass::Validator,
        start_epoch: start_epoch,
      },
      a: Some(BoundedVec::new()),
      b: Some(BoundedVec::new()),
      c: Some(BoundedVec::new()),
    });

    // Transfer to new owner
    assert_ok!(Network::owner_pause_subnet(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
    ));

    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet_data.state, SubnetState::Paused);
    assert_eq!(subnet_data.start_epoch, epoch);

    increase_epochs(10);

    let curr_epoch = Network::get_current_epoch_as_u32();
    let delta = curr_epoch - epoch;

    assert_ok!(Network::owner_unpause_subnet(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
    ));

    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetUnpaused {
        subnet_id: subnet_id, 
      }
    );

    // Ensure was activated
    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet_data.state, SubnetState::Active);
    assert_eq!(subnet_data.start_epoch, curr_epoch + 1);

    let node = RegisteredSubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
    // The start epoch update increases the epoch by 1
    assert_eq!(node.classification.start_epoch, start_epoch + delta + 1);
  });
}

#[test]
fn test_owner_deactivate_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    // Transfer to new owner
    assert_ok!(Network::owner_deactivate_subnet(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
    ));

    assert_eq!(
			*network_events().last().unwrap(),
			Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::Owner
      }
		);

    assert_eq!(SubnetsData::<Test>::try_get(subnet_id), Err(()));

  });
}

#[test]
fn test_owner_update_name() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    let prev_name = subnet_data.name;

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let new_subnet_name: Vec<u8> = "new-subnet-name".into();
    assert_ok!(Network::owner_update_name(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_subnet_name.clone()
    ));

    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet_data.name, new_subnet_name.clone());

    assert_eq!(SubnetName::<Test>::get(&new_subnet_name.clone()).unwrap(), subnet_id);

    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetNameUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        prev_value: prev_name,
        value: new_subnet_name 
      }
    );
  });
}

#[test]
fn test_owner_update_repo() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    let prev_repo = subnet_data.repo;

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let new_subnet_repo: Vec<u8> = "new-subnet-repo".into();
    assert_ok!(Network::owner_update_repo(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_subnet_repo.clone()
    ));

    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet_data.repo, new_subnet_repo.clone());

    assert_eq!(SubnetRepo::<Test>::get(&new_subnet_repo.clone()).unwrap(), subnet_id);

    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetRepoUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        prev_value: prev_repo,
        value: new_subnet_repo 
      }
    );

  });
}

#[test]
fn test_owner_update_description() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    let prev_description = subnet_data.description;

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let new_subnet_description: Vec<u8> = "new-subnet-description".into();
    assert_ok!(Network::owner_update_description(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_subnet_description.clone()
    ));

    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet_data.description, new_subnet_description.clone());

    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetDescriptionUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        prev_value: prev_description,
        value: new_subnet_description.clone() 
      }
    );
  });
}

#[test]
fn test_owner_update_misc() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    let prev_misc = subnet_data.misc;

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let new_subnet_misc: Vec<u8> = "new-subnet-misc".into();
    assert_ok!(Network::owner_update_misc(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_subnet_misc.clone()
    ));

    let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet_data.misc, new_subnet_misc.clone());

    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetMiscUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        prev_value: prev_misc,
        value: new_subnet_misc.clone() 
      }
    );
  });
}

#[test]
fn test_owner_update_churn_limit() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let current_churn_limit = ChurnLimit::<Test>::get(subnet_id);

    let new_churn_limit = current_churn_limit + 1;
    assert_ok!(Network::owner_update_churn_limit(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_churn_limit
    ));

    let churn_limit = ChurnLimit::<Test>::get(subnet_id);
    assert_eq!(churn_limit, new_churn_limit);

    assert_eq!(
      *network_events().last().unwrap(),
      Event::ChurnLimitUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        value: new_churn_limit 
      }
    );
  });
}

#[test]
fn test_owner_update_registration_queue_epochs() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let reg_queue_epochs = RegistrationQueueEpochs::<Test>::get(subnet_id);

    let new_reg_queue_epochs = reg_queue_epochs + 1;
    assert_ok!(Network::owner_update_registration_queue_epochs(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_reg_queue_epochs
    ));

    let reg_queue_epochs = RegistrationQueueEpochs::<Test>::get(subnet_id);
    assert_eq!(reg_queue_epochs, new_reg_queue_epochs);

    assert_eq!(
      *network_events().last().unwrap(),
      Event::RegistrationQueueEpochsUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        value: reg_queue_epochs 
      }
    );
  });
}

#[test]
fn test_owner_update_activation_grace_epochs() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let act_grace_epochs = ActivationGraceEpochs::<Test>::get(subnet_id);

    let new_act_grace_epochs = act_grace_epochs + 1;
    assert_ok!(Network::owner_update_activation_grace_epochs(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_act_grace_epochs
    ));

    let act_grace_epochs = ActivationGraceEpochs::<Test>::get(subnet_id);
    assert_eq!(act_grace_epochs, new_act_grace_epochs);

    assert_eq!(
      *network_events().last().unwrap(),
      Event::ActivationGraceEpochsUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        value: act_grace_epochs 
      }
    );
  });
}

#[test]
fn test_owner_update_idle_classification_epochs() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let idle_epochs = IdleClassificationEpochs::<Test>::get(subnet_id);

    let new_idle_epochs = idle_epochs + 1;
    assert_ok!(Network::owner_update_idle_classification_epochs(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_idle_epochs
    ));

    let idle_epochs = IdleClassificationEpochs::<Test>::get(subnet_id);
    assert_eq!(idle_epochs, new_idle_epochs);

    assert_eq!(
      *network_events().last().unwrap(),
      Event::IdleClassificationEpochsUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        value: idle_epochs 
      }
    );
  });
}

#[test]
fn test_owner_update_included_classification_epochs() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let included_epochs = IncludedClassificationEpochs::<Test>::get(subnet_id);

    let new_included_epochs = included_epochs + 1;
    assert_ok!(Network::owner_update_included_classification_epochs(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_included_epochs
    ));

    let included_epochs = IncludedClassificationEpochs::<Test>::get(subnet_id);
    assert_eq!(included_epochs, new_included_epochs);

    assert_eq!(
      *network_events().last().unwrap(),
      Event::IncludedClassificationEpochsUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        value: included_epochs 
      }
    );
  });
}

#[test]
fn test_owner_update_max_node_penalties() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let max_penalties = MaxSubnetNodePenalties::<Test>::get(subnet_id);

    let new_max_penalties = max_penalties + 1;
    assert_ok!(Network::owner_update_max_node_penalties(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_max_penalties
    ));

    let max_penalties = MaxSubnetNodePenalties::<Test>::get(subnet_id);
    assert_eq!(max_penalties, new_max_penalties);

    assert_eq!(
      *network_events().last().unwrap(),
      Event::MaxSubnetNodePenaltiesUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        value: max_penalties 
      }
    );
  });
}

#[test]
fn test_owner_add_initial_coldkeys() {
  new_test_ext().execute_with(|| {
    increase_epochs(1);
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnet_id = 1;
    let subnet_data = SubnetData {
      id: subnet_id,
      name: subnet_name.clone(),
      repo: subnet_name.clone(),
      description: subnet_name.clone(),
      misc: subnet_name.clone(),
      state: SubnetState::Registered,
      start_epoch: u32::MAX,
    };

    // Store subnet data
    SubnetsData::<Test>::insert(subnet_id, &subnet_data);

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);

    let new_coldkeys = BTreeSet::from([account(0)]);
    assert_ok!(Network::owner_add_initial_coldkeys(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_coldkeys.clone()
    ));

    let coldkeys = SubnetRegistrationInitialColdkeys::<Test>::get(subnet_id).unwrap();
    assert_eq!(coldkeys.clone(), new_coldkeys.clone());

    assert_eq!(
      *network_events().last().unwrap(),
      Event::AddSubnetRegistrationInitialColdkeys { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        coldkeys: coldkeys.clone()
      }
    );
  });
}

#[test]
fn test_owner_remove_initial_coldkeys() {
  new_test_ext().execute_with(|| {
    increase_epochs(1);

    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnet_id = 1;
    let subnet_data = SubnetData {
      id: subnet_id,
      name: subnet_name.clone(),
      repo: subnet_name.clone(),
      description: subnet_name.clone(),
      misc: subnet_name.clone(),
      state: SubnetState::Registered,
      start_epoch: u32::MAX,
    };

    // Store subnet data
    SubnetsData::<Test>::insert(subnet_id, &subnet_data);

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);

    let new_coldkeys = BTreeSet::from([account(0), account(1)]);
    assert_ok!(Network::owner_add_initial_coldkeys(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_coldkeys.clone()
    ));

    let coldkeys = SubnetRegistrationInitialColdkeys::<Test>::get(subnet_id).unwrap();
    assert_eq!(coldkeys, new_coldkeys.clone());

    let remove_coldkeys = BTreeSet::from([account(1)]);
    assert_ok!(Network::owner_remove_initial_coldkeys(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      remove_coldkeys.clone()
    ));


    let mut test_vec: Vec<<Test as frame_system::Config>::AccountId> = Vec::new();

    assert_eq!(
      *network_events().last().unwrap(),
      Event::RemoveSubnetRegistrationInitialColdkeys { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        coldkeys: remove_coldkeys.clone()
      }
    );

    let expected_coldkeys = BTreeSet::from([account(0)]);
    let coldkeys = SubnetRegistrationInitialColdkeys::<Test>::get(subnet_id).unwrap();
    assert_eq!(coldkeys, expected_coldkeys.clone());   
  });
}

#[test]
fn test_owner_update_key_types() {
  new_test_ext().execute_with(|| {
    increase_epochs(1);

    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnet_id = 1;
    let subnet_data = SubnetData {
      id: subnet_id,
      name: subnet_name.clone(),
      repo: subnet_name.clone(),
      description: subnet_name.clone(),
      misc: subnet_name.clone(),
      state: SubnetState::Registered,
      start_epoch: u32::MAX,
    };

    // Store subnet data
    SubnetsData::<Test>::insert(subnet_id, &subnet_data);

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);

    let new_keytypes = BTreeSet::from([KeyType::Ed25519]);
    assert_ok!(Network::owner_update_key_types(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_keytypes.clone()
    ));

    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetKeyTypesUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        value: new_keytypes.clone(),
      }
    );

    let key_types = SubnetKeyTypes::<Test>::get(subnet_id);
    assert_eq!(key_types, new_keytypes.clone());
  });
}

#[test]
fn test_owner_update_node_removal_policy() {
  new_test_ext().execute_with(|| {
    increase_epochs(1);

    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    let subnet_id = 1;
    let subnet_data = SubnetData {
      id: subnet_id,
      name: subnet_name.clone(),
      repo: subnet_name.clone(),
      description: subnet_name.clone(),
      misc: subnet_name.clone(),
      state: SubnetState::Registered,
      start_epoch: u32::MAX,
    };

    // Store subnet data
    SubnetsData::<Test>::insert(subnet_id, &subnet_data);

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);

    let removal_policy = NodeRemovalPolicy {
      logic: LogicExpr::And(
        Box::new(LogicExpr::Condition(NodeRemovalConditionType::DeltaBelowScore(200))),
        Box::new(LogicExpr::Condition(NodeRemovalConditionType::DeltaBelowNodeDelegateStakeBalance(100))),
      )
    };

    assert_ok!(Network::owner_update_node_removal_policy(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      removal_policy.clone()
    ));

    assert_eq!(
      *network_events().last().unwrap(),
      Event::NodeRemovalSystemV2Update { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        value: removal_policy.clone()
      }
    );

    let policy = NodeRemovalSystemV2::<Test>::get(subnet_id).unwrap();
    assert_eq!(removal_policy.clone(), policy);
  });
}

#[test]
fn test_owner_update_min_stake() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let min_stake = SubnetMinStakeBalance::<Test>::get(subnet_id);

    let new_min_stake = min_stake + 1;
    assert_ok!(Network::owner_update_min_stake(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_min_stake
    ));

    let min_stake = SubnetMinStakeBalance::<Test>::get(subnet_id);
    assert_eq!(min_stake, new_min_stake);

    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetMinStakeBalanceUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        value: min_stake
      }
    );
  });
}

#[test]
fn test_owner_update_max_stake() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let max_stake = SubnetMaxStakeBalance::<Test>::get(subnet_id);
    log::error!("max_stake {:?}", max_stake);

    let new_max_stake = max_stake - 1;
    assert_ok!(Network::owner_update_max_stake(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_max_stake
    ));

    let max_stake = SubnetMaxStakeBalance::<Test>::get(subnet_id);
    assert_eq!(max_stake, new_max_stake);

    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetMaxStakeBalanceUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        value: max_stake
      }
    );
  });
}

#[test]
fn test_owner_update_delegate_stake_percentage() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let dstake_perc = SubnetDelegateStakeRewardsPercentage::<Test>::get(subnet_id);

    let new_dstake_perc = dstake_perc + 1;
    assert_ok!(Network::owner_update_delegate_stake_percentage(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_dstake_perc
    ));

    let dstake_perc = SubnetDelegateStakeRewardsPercentage::<Test>::get(subnet_id);
    assert_eq!(dstake_perc, new_dstake_perc);

    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetDelegateStakeRewardsPercentageUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        value: dstake_perc
      }
    );
  });
}

#[test]
fn test_owner_update_max_registered_nodes() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();


    build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);
    let epoch = Network::get_current_epoch_as_u32();

    let max_reg_nodes = MaxRegisteredNodes::<Test>::get(subnet_id);

    let new_max_reg_nodes = max_reg_nodes + 1;
    assert_ok!(Network::owner_update_max_registered_nodes(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_max_reg_nodes
    ));

    let max_reg_nodes = MaxRegisteredNodes::<Test>::get(subnet_id);
    assert_eq!(max_reg_nodes, new_max_reg_nodes);

    assert_eq!(
      *network_events().last().unwrap(),
      Event::MaxRegisteredNodesUpdate { 
        subnet_id: subnet_id,
        owner: original_owner.clone(), 
        value: max_reg_nodes
      }
    );
  });
}

#[test]
fn test_transfer_and_accept_ownership_works() {
  new_test_ext().execute_with(|| {
    increase_epochs(1);

    let subnet_id = 0;
    let original_owner = account(1);
    let new_owner = account(2);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);

    // Transfer to new owner
    assert_ok!(Network::do_transfer_subnet_ownership(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_owner.clone()
    ));

    // Accept by new owner
    assert_ok!(Network::do_accept_subnet_ownership(
      RuntimeOrigin::signed(new_owner.clone()),
      subnet_id
    ));

    // Check ownership
    assert_eq!(SubnetOwner::<Test>::get(subnet_id), Some(new_owner.clone()));

    assert_eq!(
      *network_events().last().unwrap(),
      Event::AcceptPendingSubnetOwner { 
        subnet_id: subnet_id,
        new_owner: new_owner.clone() 
      }
    );
  });
}

#[test]
fn test_transfer_cannot_be_accepted_by_wrong_account() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let original_owner = account(3);
    let new_owner = account(4);
    let wrong_account = account(5);

    SubnetOwner::<Test>::insert(subnet_id, &original_owner);

    assert_ok!(Network::do_transfer_subnet_ownership(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_owner
    ));

    assert_err!(
      Network::do_accept_subnet_ownership(
        RuntimeOrigin::signed(wrong_account),
        subnet_id
      ),
      Error::<Test>::NotPendingSubnetOwner
    );
  });
}

#[test]
fn test_owner_can_cancel_transfer_by_resetting_owner() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let original_owner = account(6);
    let new_owner = account(7);
    let zero_address = <Test as frame_system::Config>::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap();

    SubnetOwner::<Test>::insert(subnet_id, &original_owner);

    assert_ok!(Network::do_transfer_subnet_ownership(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      new_owner.clone()
    ));

    assert_ok!(Network::do_transfer_subnet_ownership(
      RuntimeOrigin::signed(original_owner.clone()),
      subnet_id,
      zero_address
    ));

    assert_err!(
      Network::do_accept_subnet_ownership(
        RuntimeOrigin::signed(new_owner.clone()),
        subnet_id
      ),
      Error::<Test>::NotPendingSubnetOwner
    );
  });
}

#[test]
fn test_accept_without_pending_transfer_should_fail() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let user = account(8);

    assert_err!(
      Network::do_accept_subnet_ownership(
        RuntimeOrigin::signed(user),
        subnet_id
      ),
      Error::<Test>::NoPendingSubnetOwner
    );
  });
}

#[test]
fn test_non_owner_cannot_transfer() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let actual_owner = account(9);
    let fake_owner = account(10);
    let target = account(11);

    SubnetOwner::<Test>::insert(subnet_id, &actual_owner);

    assert_err!(
      Network::do_transfer_subnet_ownership(
        RuntimeOrigin::signed(fake_owner),
        subnet_id,
        target
      ),
      Error::<Test>::NotSubnetOwner
    );
  });
}
