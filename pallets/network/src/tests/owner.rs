use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use crate::{
    ActivationGraceEpochs, ChurnLimit, DefaultMaxVectorLength, Error, HotkeyOwner,
    HotkeySubnetNodeId, IdleClassificationEpochs, IncludedClassificationEpochs, KeyType,
    LastSubnetDelegateStakeRewardsUpdate, LogicExpr, MaxActivationGraceEpochs,
    MaxDelegateStakePercentage, MaxIdleClassificationEpochs, MaxIncludedClassificationEpochs,
    MaxMaxRegisteredNodes, MaxMaxSubnetNodePenalties, MaxRegisteredNodes,
    MaxRegistrationQueueEpochs, MaxSubnetBootnodeAccess,
    MaxSubnetDelegateStakeRewardsPercentageChange, MaxSubnetMaxStake, MaxSubnetMinStake,
    MaxSubnetNodePenalties, MaxSubnetNodes, MaxSubnets, MinActivationGraceEpochs,
    MinDelegateStakePercentage, MinIdleClassificationEpochs, MinIncludedClassificationEpochs,
    MinMaxRegisteredNodes, MinMaxSubnetNodePenalties, MinRegistrationQueueEpochs,
    MinSubnetMaxStake, MinSubnetMinStake, NetworkMaxStakeBalance, NetworkMinStakeBalance,
    NodeRemovalConditionType, NodeRemovalPolicy, NodeRemovalSystemV2, RegisteredSubnetNodesData,
    RegistrationQueueEpochs, SubnetBootnodeAccess, SubnetData,
    SubnetDelegateStakeRewardsPercentage, SubnetDelegateStakeRewardsUpdatePeriod, SubnetKeyTypes,
    SubnetMaxStakeBalance, SubnetMinStakeBalance, SubnetName, SubnetNode, SubnetNodeClass,
    SubnetNodeClassification, SubnetOwner, SubnetRegistrationInitialColdkeys, SubnetRemovalReason,
    SubnetRepo, SubnetState, SubnetsData, TotalActiveSubnetNodes,
};
use codec::{Decode, Encode};
use frame_support::traits::Currency;
use frame_support::{assert_err, assert_ok};
use sp_runtime::traits::TrailingZeroInput;
use sp_runtime::BoundedVec;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;

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
fn test_owner_pause_subnet_must_be_active_error() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;
        let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

        build_registered_subnet_new(
            subnet_name.clone(),
            0,
            4,
            deposit_amount,
            stake_amount,
            true,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        let original_owner = account(1);

        // Set initial owner
        SubnetOwner::<Test>::insert(subnet_id, &original_owner);
        let epoch = Network::get_current_epoch_as_u32();

        // Transfer to new owner
        assert_err!(
            Network::owner_pause_subnet(RuntimeOrigin::signed(original_owner.clone()), subnet_id),
            Error::<Test>::SubnetMustBeActive
        );
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
        RegisteredSubnetNodesData::<Test>::insert(
            subnet_id,
            hotkey_subnet_node_id,
            SubnetNode {
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
                unique: Some(BoundedVec::new()),
                non_unique: Some(BoundedVec::new()),
                // c: Some(BoundedVec::new()),
            },
        );

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
fn test_owner_unpause_subnet_must_be_paused_error() {
    new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

    build_registered_subnet_new(
      subnet_name.clone(),
      0,
      4,
      deposit_amount,
      stake_amount,
      true,
    );

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

    let original_owner = account(1);

    // Set initial owner
    SubnetOwner::<Test>::insert(subnet_id, &original_owner);

    // Transfer to new owner
    assert_err!(
      Network::owner_unpause_subnet(
        RuntimeOrigin::signed(original_owner.clone()),
        subnet_id,
      ),
      Error::<Test>::SubnetMustBePaused
    );
  });
}

#[test]
fn test_owner_unpause_subnet_verify_queue_updated() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;
        let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

        let start = 0;
        let end = 4;

        build_activated_subnet_new(
            subnet_name.clone(),
            start,
            end,
            deposit_amount,
            stake_amount,
        );

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();

        // Set up registered nodes in the queue
        // These are to be tested against to ensure their start epochs update
        let churn_limit = ChurnLimit::<Test>::get(subnet_id);
        let start = end + 1;
        let end = start + churn_limit;
        build_registered_nodes_in_queue(subnet_id, start, end, deposit_amount, stake_amount);

        let max_subnets = MaxSubnets::<Test>::get();
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

        // Store data
        let mut registered_nodes_data: BTreeMap<u32, u32> = BTreeMap::new(); // node ID => start_epoch
        for n in start..end {
            let _n = n + 1;
            let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, _n);
            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
            let subnet_node_data =
                RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id)
                    .unwrap();
            registered_nodes_data.insert(
                hotkey_subnet_node_id,
                subnet_node_data.classification.start_epoch,
            );
        }

        let original_owner = account(1);

        // Set initial owner
        SubnetOwner::<Test>::insert(subnet_id, &original_owner);

        let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        // Pause subnet
        assert_ok!(Network::owner_pause_subnet(
            RuntimeOrigin::signed(original_owner.clone()),
            subnet_id,
        ));

        // increase epoch
        let epoch_increase = 3;
        increase_epochs(3);

        let unpause_subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);
        let epoch_delta = unpause_subnet_epoch - subnet_epoch;

        // Transfer to new owner
        assert_ok!(Network::owner_unpause_subnet(
            RuntimeOrigin::signed(original_owner.clone()),
            subnet_id,
        ));

        for n in start..end {
            let _n = n + 1;
            let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, _n);
            let hotkey_subnet_node_id =
                HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
            let subnet_node_data =
                RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id)
                    .unwrap();

            if let Some(prev_start_epoch) = registered_nodes_data.get(&hotkey_subnet_node_id) {
                assert_eq!(
                    *prev_start_epoch + epoch_increase + 1,
                    subnet_node_data.classification.start_epoch
                );
            } else {
                assert!(false);
            }
        }
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

        // Subnet 2
        let subnet_name_2: Vec<u8> = "subnet-name-2".into();
        build_activated_subnet_new(subnet_name_2.clone(), 0, 4, deposit_amount, stake_amount);
        let subnet_id_2 = SubnetName::<Test>::get(subnet_name_2.clone()).unwrap();
        let owner_2 = account(2);
        SubnetOwner::<Test>::insert(subnet_id_2, &owner_2);


        let new_subnet_name: Vec<u8> = "new-subnet-name".into();
        assert_ok!(Network::owner_update_name(
            RuntimeOrigin::signed(original_owner.clone()),
            subnet_id,
            new_subnet_name.clone()
        ));

        let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
        assert_eq!(subnet_data.name, new_subnet_name.clone());

        assert_eq!(
            SubnetName::<Test>::get(&new_subnet_name.clone()).unwrap(),
            subnet_id
        );

        assert_eq!(
            *network_events().last().unwrap(),
            Event::SubnetNameUpdate {
                subnet_id: subnet_id,
                owner: original_owner.clone(),
                prev_value: prev_name,
                value: new_subnet_name.clone()
            }
        );

        // Update to a new name and check old one was removed
        let new_subnet_name_2: Vec<u8> = "new-subnet-name-2".into();
        assert_ok!(Network::owner_update_name(
            RuntimeOrigin::signed(original_owner.clone()),
            subnet_id,
            new_subnet_name_2.clone()
        ));
        let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
        assert_eq!(subnet_data.name, new_subnet_name_2.clone());
        assert_eq!(
            SubnetName::<Test>::try_get(&new_subnet_name.clone()),
            Err(())
        );
        assert_eq!(
            SubnetName::<Test>::get(&new_subnet_name_2.clone()).unwrap(),
            subnet_id
        );

        // Update subnet 2 to the original name
        assert_ok!(Network::owner_update_name(
            RuntimeOrigin::signed(owner_2.clone()),
            subnet_id_2,
            new_subnet_name.clone()
        ));
        let subnet_data = SubnetsData::<Test>::get(subnet_id_2).unwrap();
        assert_eq!(subnet_data.name, new_subnet_name.clone());
        assert_eq!(
            SubnetName::<Test>::get(&new_subnet_name.clone()).unwrap(),
            subnet_id_2
        );

    });
}

#[test]
fn test_owner_update_name_name_exists_error() {
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

        assert_err!(
            Network::owner_update_name(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                prev_name.clone()
            ),
            Error::<Test>::SubnetNameExist
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


        let subnet_name_2: Vec<u8> = "subnet-name-2".into();
        build_activated_subnet_new(subnet_name_2.clone(), 0, 4, deposit_amount, stake_amount);
        let subnet_id_2 = SubnetName::<Test>::get(subnet_name_2.clone()).unwrap();
        let owner_2 = account(2);
        SubnetOwner::<Test>::insert(subnet_id_2, &owner_2);

        let new_subnet_repo: Vec<u8> = "new-subnet-repo".into();
        assert_ok!(Network::owner_update_repo(
            RuntimeOrigin::signed(original_owner.clone()),
            subnet_id,
            new_subnet_repo.clone()
        ));

        let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
        assert_eq!(subnet_data.repo, new_subnet_repo.clone());

        assert_eq!(
            SubnetRepo::<Test>::get(&new_subnet_repo.clone()).unwrap(),
            subnet_id
        );

        assert_eq!(
            *network_events().last().unwrap(),
            Event::SubnetRepoUpdate {
                subnet_id: subnet_id,
                owner: original_owner.clone(),
                prev_value: prev_repo,
                value: new_subnet_repo.clone()
            }
        );

        // Update to a new repo and check old one was removed
        let new_subnet_repo_2: Vec<u8> = "new-subnet-repo_2".into();
        assert_ok!(Network::owner_update_repo(
            RuntimeOrigin::signed(original_owner.clone()),
            subnet_id,
            new_subnet_repo_2.clone()
        ));
        let subnet_data = SubnetsData::<Test>::get(subnet_id).unwrap();
        assert_eq!(subnet_data.repo, new_subnet_repo_2.clone());
        assert_eq!(
            SubnetRepo::<Test>::try_get(&new_subnet_repo.clone()),
            Err(())
        );
        assert_eq!(
            SubnetRepo::<Test>::get(&new_subnet_repo_2.clone()).unwrap(),
            subnet_id
        );

        // Update subnet 2 to the original repo
        assert_ok!(Network::owner_update_repo(
            RuntimeOrigin::signed(owner_2.clone()),
            subnet_id_2,
            new_subnet_repo.clone()
        ));
        let subnet_data = SubnetsData::<Test>::get(subnet_id_2).unwrap();
        assert_eq!(subnet_data.repo, new_subnet_repo.clone());
        assert_eq!(
            SubnetRepo::<Test>::get(&new_subnet_repo.clone()).unwrap(),
            subnet_id_2
        );
    });
}

#[test]
fn test_owner_update_name_repo_exists_error() {
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

        assert_err!(
            Network::owner_update_repo(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                prev_repo.clone()
            ),
            Error::<Test>::SubnetRepoExist
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
fn test_owner_update_registration_queue_epochs_invalid_registration_queue_epochs_error() {
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

        let epochs = MinRegistrationQueueEpochs::<Test>::get() - 1;

        assert_err!(
            Network::owner_update_registration_queue_epochs(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                epochs
            ),
            Error::<Test>::InvalidRegistrationQueueEpochs
        );

        let epochs = MaxRegistrationQueueEpochs::<Test>::get() + 1;

        assert_err!(
            Network::owner_update_registration_queue_epochs(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                epochs
            ),
            Error::<Test>::InvalidRegistrationQueueEpochs
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
fn test_owner_update_activation_grace_epochs_invalid_activation_grace_epochs() {
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

        let epochs = MinActivationGraceEpochs::<Test>::get() - 1;

        assert_err!(
            Network::owner_update_activation_grace_epochs(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                epochs
            ),
            Error::<Test>::InvalidActivationGraceEpochs
        );

        let epochs = MaxActivationGraceEpochs::<Test>::get() + 1;

        assert_err!(
            Network::owner_update_activation_grace_epochs(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                epochs
            ),
            Error::<Test>::InvalidActivationGraceEpochs
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
fn test_owner_update_idle_classification_epochs_invalid_idle_classification_epochs() {
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

        let epochs = MinIdleClassificationEpochs::<Test>::get() - 1;

        assert_err!(
            Network::owner_update_idle_classification_epochs(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                epochs
            ),
            Error::<Test>::InvalidIdleClassificationEpochs
        );

        let epochs = MaxIdleClassificationEpochs::<Test>::get() + 1;

        assert_err!(
            Network::owner_update_idle_classification_epochs(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                epochs
            ),
            Error::<Test>::InvalidIdleClassificationEpochs
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
fn test_owner_update_included_classification_epochs_invalid_included_classification_epochs() {
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

        let epochs = MinIncludedClassificationEpochs::<Test>::get() - 1;

        assert_err!(
            Network::owner_update_included_classification_epochs(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                epochs
            ),
            Error::<Test>::InvalidIncludedClassificationEpochs
        );

        let epochs = MaxIncludedClassificationEpochs::<Test>::get() + 1;

        assert_err!(
            Network::owner_update_included_classification_epochs(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                epochs
            ),
            Error::<Test>::InvalidIncludedClassificationEpochs
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
fn test_owner_update_max_node_penalties_invalid_max_node_penalties() {
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

        let value = MinMaxSubnetNodePenalties::<Test>::get() - 1;

        assert_err!(
            Network::owner_update_max_node_penalties(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                value
            ),
            Error::<Test>::InvalidMaxSubnetNodePenalties
        );

        let value = MaxMaxSubnetNodePenalties::<Test>::get() + 1;

        assert_err!(
            Network::owner_update_max_node_penalties(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                value
            ),
            Error::<Test>::InvalidMaxSubnetNodePenalties
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
fn test_owner_add_initial_coldkeys_must_be_registering() {
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

        let new_coldkeys = BTreeSet::from([account(0)]);
        assert_err!(
            Network::owner_add_initial_coldkeys(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                new_coldkeys.clone()
            ),
            Error::<Test>::SubnetMustBeRegistering
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
fn test_owner_remove_initial_coldkeys_must_be_registering() {
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

        let new_coldkeys = BTreeSet::from([account(0)]);
        assert_err!(
            Network::owner_remove_initial_coldkeys(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                new_coldkeys.clone()
            ),
            Error::<Test>::SubnetMustBeRegistering
        );
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
                Box::new(LogicExpr::Condition(
                    NodeRemovalConditionType::DeltaBelowScore(200),
                )),
                Box::new(LogicExpr::Condition(
                    NodeRemovalConditionType::DeltaBelowNodeDelegateStakeBalance(100),
                )),
            ),
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
fn test_owner_remove_subnet_node() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;
        let stake_amount: u128 = NetworkMinStakeBalance::<Test>::get();

        build_activated_subnet_new(subnet_name.clone(), 0, 4, deposit_amount, stake_amount);
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
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
fn test_owner_update_min_stake_invalid_min_Stake() {
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

        // let value = MinSubnetMinStake::<Test>::get() - 1;
        let value = NetworkMinStakeBalance::<Test>::get() - 1;

        assert_err!(
            Network::owner_update_min_stake(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                value
            ),
            Error::<Test>::InvalidSubnetMinStake
        );

        // let value = MaxSubnetMinStake::<Test>::get() + 1;
        let value = NetworkMinStakeBalance::<Test>::get() - 1;

        assert_err!(
            Network::owner_update_min_stake(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                value
            ),
            Error::<Test>::InvalidSubnetMinStake
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
fn test_owner_update_max_stake_invalid_max_stake() {
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

        // let value = MinSubnetMaxStake::<Test>::get() - 1;
        let value = NetworkMaxStakeBalance::<Test>::get() + 1;

        assert_err!(
            Network::owner_update_max_stake(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                value
            ),
            Error::<Test>::InvalidSubnetMaxStake
        );

        // let value = MaxSubnetMaxStake::<Test>::get() + 1;
        let value = NetworkMaxStakeBalance::<Test>::get() + 1;

        assert_err!(
            Network::owner_update_max_stake(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                value
            ),
            Error::<Test>::InvalidSubnetMaxStake
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
        let block = System::block_number();

        let last_update = LastSubnetDelegateStakeRewardsUpdate::<Test>::get(subnet_id);
        let update_period = SubnetDelegateStakeRewardsUpdatePeriod::<Test>::get();

        let update_to_block = if block - last_update < update_period {
            last_update + update_period
        } else {
            System::block_number()
        };

        System::set_block_number(update_to_block + 1);

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
fn test_owner_update_delegate_stake_percentage_update_too_soon() {
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
        let block = System::block_number();

        let last_update = LastSubnetDelegateStakeRewardsUpdate::<Test>::get(subnet_id);
        let update_period = SubnetDelegateStakeRewardsUpdatePeriod::<Test>::get();

        let update_to_block = if block - last_update < update_period {
            last_update + update_period
        } else {
            System::block_number()
        };

        System::set_block_number(update_to_block + 1);

        let dstake_perc = SubnetDelegateStakeRewardsPercentage::<Test>::get(subnet_id);

        let new_dstake_perc = dstake_perc + 1;
        assert_ok!(Network::owner_update_delegate_stake_percentage(
            RuntimeOrigin::signed(original_owner.clone()),
            subnet_id,
            new_dstake_perc
        ));

        assert_err!(
            Network::owner_update_delegate_stake_percentage(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                new_dstake_perc
            ),
            Error::<Test>::DelegateStakePercentageUpdateTooSoon
        );
    });
}

#[test]
fn test_owner_update_delegate_stake_percentage_update_too_large() {
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
        let block = System::block_number();

        let last_update = LastSubnetDelegateStakeRewardsUpdate::<Test>::get(subnet_id);
        let update_period = SubnetDelegateStakeRewardsUpdatePeriod::<Test>::get();

        let update_to_block = if last_update + update_period > block {
            last_update + update_period
        } else {
            System::block_number()
        };

        System::set_block_number(update_to_block + 1);

        let dstake_perc = SubnetDelegateStakeRewardsPercentage::<Test>::get(subnet_id);

        let new_dstake_perc = dstake_perc + 1;
        assert_ok!(Network::owner_update_delegate_stake_percentage(
            RuntimeOrigin::signed(original_owner.clone()),
            subnet_id,
            new_dstake_perc
        ));

        let block = System::block_number();
        let last_update = LastSubnetDelegateStakeRewardsUpdate::<Test>::get(subnet_id);
        let update_to_block = if last_update + update_period > block {
            last_update + update_period
        } else {
            System::block_number()
        };

        System::set_block_number(update_to_block + 1);

        assert_err!(
            Network::owner_update_delegate_stake_percentage(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                950000000000000000
            ),
            Error::<Test>::DelegateStakePercentageAbsDiffTooLarge
        );

        assert_err!(
            Network::owner_update_delegate_stake_percentage(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                0
            ),
            Error::<Test>::DelegateStakePercentageAbsDiffTooLarge
        );

        // insert to max
        SubnetDelegateStakeRewardsPercentage::<Test>::insert(
            subnet_id,
            MaxDelegateStakePercentage::<Test>::get(),
        );
        assert_err!(
            Network::owner_update_delegate_stake_percentage(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                MaxDelegateStakePercentage::<Test>::get() + 1
            ),
            Error::<Test>::InvalidDelegateStakePercentage
        );

        SubnetDelegateStakeRewardsPercentage::<Test>::insert(
            subnet_id,
            MinDelegateStakePercentage::<Test>::get(),
        );
        assert_err!(
            Network::owner_update_delegate_stake_percentage(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                MinDelegateStakePercentage::<Test>::get() - 1
            ),
            Error::<Test>::InvalidDelegateStakePercentage
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
fn test_owner_update_max_registered_nodes_invalid_max_registered_nodes() {
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

        let value = MinMaxRegisteredNodes::<Test>::get() - 1;

        assert_err!(
            Network::owner_update_max_registered_nodes(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                value
            ),
            Error::<Test>::InvalidMaxRegisteredNodes
        );

        let value = MaxMaxRegisteredNodes::<Test>::get() + 1;

        assert_err!(
            Network::owner_update_max_registered_nodes(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                value
            ),
            Error::<Test>::InvalidMaxRegisteredNodes
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

        assert_eq!(
            *network_events().last().unwrap(),
            Event::TransferPendingSubnetOwner {
                subnet_id: subnet_id,
                owner: original_owner.clone(),
                new_owner: new_owner.clone()
            }
        );

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
            Network::do_accept_subnet_ownership(RuntimeOrigin::signed(wrong_account), subnet_id),
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
        let zero_address =
            <Test as frame_system::Config>::AccountId::decode(&mut TrailingZeroInput::zeroes())
                .unwrap();

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
            Network::do_accept_subnet_ownership(RuntimeOrigin::signed(user), subnet_id),
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

#[test]
fn test_owner_add_bootnode_access() {
    new_test_ext().execute_with(|| {
        increase_epochs(1);
        let subnet_id = 1;

        let subnet_name: Vec<u8> = "subnet-name".into();
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

        let original_owner = account(60);

        SubnetOwner::<Test>::insert(subnet_id, &original_owner);

        let new_access = account(70);

        let access_set = SubnetBootnodeAccess::<Test>::get(subnet_id);

        assert_ok!(Network::owner_add_bootnode_access(
            RuntimeOrigin::signed(original_owner.clone()),
            subnet_id,
            new_access.clone()
        ));

        let new_access_set = SubnetBootnodeAccess::<Test>::get(subnet_id);

        assert!(new_access_set.get(&new_access.clone()).is_some());

        assert_eq!(
            *network_events().last().unwrap(),
            Event::AddSubnetBootnodeAccess {
                subnet_id: subnet_id,
                owner: original_owner.clone(),
                new_account: new_access.clone()
            }
        );

        let bv = |b: u8| BoundedVec::<u8, DefaultMaxVectorLength>::try_from(vec![b]).unwrap();

        // Add a bootnode using the added account
        let add_set = BTreeSet::from([bv(1), bv(2)]);
        assert_ok!(Network::update_bootnodes(
            RuntimeOrigin::signed(new_access.clone()),
            subnet_id,
            add_set.clone(),
            BTreeSet::new(),
        ));

        assert_eq!(
            *network_events().last().unwrap(),
            Event::BootnodesUpdated {
                subnet_id: subnet_id,
                added: add_set.clone(),
                removed: BTreeSet::new(),
            }
        );

        // Fail if access already granted
        assert_err!(
            Network::owner_add_bootnode_access(
                RuntimeOrigin::signed(original_owner.clone()),
                subnet_id,
                new_access.clone()
            ),
            Error::<Test>::InAccessList
        );

        SubnetBootnodeAccess::<Test>::remove(subnet_id);

        let max_access_nodes = MaxSubnetBootnodeAccess::<Test>::get();

        let mut touched = false; // make sure logic is touched

        for n in 0..max_access_nodes + 2 {
            let _n = n + 1;
            let account = account(n);
            if _n > max_access_nodes {
                touched = true;
                assert_err!(
                    Network::owner_add_bootnode_access(
                        RuntimeOrigin::signed(original_owner.clone()),
                        subnet_id,
                        account
                    ),
                    Error::<Test>::MaxSubnetBootnodeAccess
                );
            } else {
                assert_ok!(Network::owner_add_bootnode_access(
                    RuntimeOrigin::signed(original_owner.clone()),
                    subnet_id,
                    account
                ));
            }
        }

        assert!(touched);
    });
}

#[test]
fn test_not_subnet_owner_and_invalid_subnet_id() {
    new_test_ext().execute_with(|| {
        let subnet_id = 1;
        let actual_owner = account(9);
        let fake_owner = account(10);
        let target = account(11);

        SubnetOwner::<Test>::insert(subnet_id, &actual_owner);

        assert_err!(
            Network::do_owner_pause_subnet(RuntimeOrigin::signed(fake_owner), subnet_id),
            Error::<Test>::NotSubnetOwner
        );

        assert_err!(
            Network::do_owner_unpause_subnet(RuntimeOrigin::signed(fake_owner), subnet_id),
            Error::<Test>::NotSubnetOwner
        );

        assert_err!(
            Network::do_owner_deactivate_subnet(RuntimeOrigin::signed(fake_owner), subnet_id),
            Error::<Test>::NotSubnetOwner
        );

        let new_subnet_name: Vec<u8> = "new-subnet-name".into();

        assert_err!(
            Network::do_owner_update_name(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                new_subnet_name.clone()
            ),
            Error::<Test>::NotSubnetOwner
        );

        let new_subnet_repo: Vec<u8> = "new-subnet-repo".into();

        assert_err!(
            Network::do_owner_update_repo(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                new_subnet_name.clone()
            ),
            Error::<Test>::NotSubnetOwner
        );

        let new_subnet_description: Vec<u8> = "new-subnet-description".into();

        assert_err!(
            Network::do_owner_update_description(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                new_subnet_description
            ),
            Error::<Test>::NotSubnetOwner
        );

        let new_subnet_misc: Vec<u8> = "new-subnet-misc".into();

        assert_err!(
            Network::do_owner_update_misc(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                new_subnet_misc
            ),
            Error::<Test>::NotSubnetOwner
        );

        assert_err!(
            Network::do_owner_update_churn_limit(RuntimeOrigin::signed(fake_owner), subnet_id, 1),
            Error::<Test>::NotSubnetOwner
        );

        assert_err!(
            Network::do_owner_update_registration_queue_epochs(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                1
            ),
            Error::<Test>::NotSubnetOwner
        );

        assert_err!(
            Network::do_owner_update_activation_grace_epochs(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                1
            ),
            Error::<Test>::NotSubnetOwner
        );

        assert_err!(
            Network::do_owner_update_idle_classification_epochs(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                1
            ),
            Error::<Test>::NotSubnetOwner
        );

        assert_err!(
            Network::do_owner_update_included_classification_epochs(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                1
            ),
            Error::<Test>::NotSubnetOwner
        );

        assert_err!(
            Network::do_owner_update_max_node_penalties(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                1
            ),
            Error::<Test>::NotSubnetOwner
        );

        let new_coldkeys = BTreeSet::from([account(0)]);
        assert_err!(
            Network::do_owner_add_initial_coldkeys(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                new_coldkeys.clone()
            ),
            Error::<Test>::NotSubnetOwner
        );

        assert_err!(
            Network::do_owner_remove_initial_coldkeys(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                new_coldkeys.clone()
            ),
            Error::<Test>::NotSubnetOwner
        );

        let new_keytypes = BTreeSet::from([KeyType::Ed25519]);
        assert_err!(
            Network::do_owner_update_key_types(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                new_keytypes
            ),
            Error::<Test>::NotSubnetOwner
        );

        let removal_policy = NodeRemovalPolicy {
            logic: LogicExpr::And(
                Box::new(LogicExpr::Condition(
                    NodeRemovalConditionType::DeltaBelowScore(200),
                )),
                Box::new(LogicExpr::Condition(
                    NodeRemovalConditionType::DeltaBelowNodeDelegateStakeBalance(100),
                )),
            ),
        };

        assert_err!(
            Network::do_owner_update_node_removal_policy(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                removal_policy.clone()
            ),
            Error::<Test>::NotSubnetOwner
        );

        assert_err!(
            Network::do_owner_activate_subnet_node(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                0,
                0
            ),
            Error::<Test>::NotSubnetOwner
        );

        assert_err!(
            Network::do_owner_update_min_stake(RuntimeOrigin::signed(fake_owner), subnet_id, 1),
            Error::<Test>::NotSubnetOwner
        );
        assert_err!(
            Network::do_owner_update_max_stake(RuntimeOrigin::signed(fake_owner), subnet_id, 1),
            Error::<Test>::NotSubnetOwner
        );
        assert_err!(
            Network::do_owner_update_delegate_stake_percentage(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                1
            ),
            Error::<Test>::NotSubnetOwner
        );
        assert_err!(
            Network::do_owner_update_max_registered_nodes(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                1
            ),
            Error::<Test>::NotSubnetOwner
        );

        assert_err!(
            Network::do_owner_add_bootnode_access(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                account(1)
            ),
            Error::<Test>::NotSubnetOwner
        );
        assert_err!(
            Network::do_owner_remove_bootnode_access(
                RuntimeOrigin::signed(fake_owner),
                subnet_id,
                account(1)
            ),
            Error::<Test>::NotSubnetOwner
        );
    });
}
