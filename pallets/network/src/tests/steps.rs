use super::mock::*;
use crate::tests::test_utils::*;
use frame_support::{
	assert_ok, assert_err
};
use frame_support::traits::Currency;
use crate::{
    Error,
    Event,
    SubnetName,
    HotkeySubnetNodeId,
    SubnetElectedValidator,
    SubnetNodeIdHotkey,
    SubnetConsensusSubmission,
    SubnetNodesData,
    SubnetNodeClass,
    TotalSubnetDelegateStakeBalance,
    TotalActiveSubnets,
    SubnetRegistrationEpoch,
    MaxSubnetNodes,
    MaxSubnets,
    NewRegistrationCostMultiplier,
    MaxSubnetRemovalInterval,
    SubnetsData,
    MinSubnetNodes,
    SubnetRemovalReason,
    SubnetPenaltyCount,
    MaxSubnetPenaltyCount,
    MaxSubnetPauseEpochs,
    SubnetState,
    SubnetRegistrationEpochs,
    SubnetActivationEnactmentEpochs,
    TotalActiveSubnetNodes,
};

#[test]
fn test_do_epoch_preliminaries_remove_expired_pause() {
  new_test_ext().execute_with(|| {
    NewRegistrationCostMultiplier::<Test>::put(1000000000000000000);
    MaxSubnetPenaltyCount::<Test>::set(3);

    let deposit_amount: u128 = 1000000000000000000000;
    let amount: u128 = 100000000000000000000;

    let max_subnets = MaxSubnets::<Test>::get();
    let end = MinSubnetNodes::<Test>::get();
    let last_subnet = 2;

    let mut remove_subnet_id = 0;
    let mut first_subnet_id = 0;
    for s in 0..last_subnet {
        let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
        build_activated_subnet_new(
            subnet_name.clone().into(), 
            0, 
            end, 
            deposit_amount, 
            amount,
        );
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        remove_subnet_id = subnet_id;

        if first_subnet_id == 0 {
            first_subnet_id = subnet_id;
        }
    }

    SubnetsData::<Test>::mutate(remove_subnet_id, |maybe_params| {
        let params = maybe_params.as_mut().unwrap();
        // Update state
        params.state = SubnetState::Paused;

        // Set to zero so its removed based on the `MaxSubnetPauseEpochs`
        params.start_epoch = 0;
    });

    increase_epochs(MaxSubnetPauseEpochs::<Test>::get() + 1);

    // iterate to accrue penalties while subnet is in a pause state
    for _ in 0..MaxSubnetPenaltyCount::<Test>::get().saturating_add(1) {
        let current_epoch = Network::get_current_epoch_as_u32();

        // always ensure subnet is at the minimum required delegate stake
        for s in 0..last_subnet {
            let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
            // This will revert if subnet doesn't exist so we know the removal subnet is still alive
            let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
            let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
            let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);
            if Balances::free_balance(&account(0)) <= min_subnet_delegate_stake + 500 {
                let _ = Balances::deposit_creating(&account(0), min_subnet_delegate_stake + 500);
            }
            assert_ok!(Network::add_to_delegate_stake(
                RuntimeOrigin::signed(account(0)),
                subnet_id,
                total_delegate_stake_balance - min_subnet_delegate_stake,
            ));
        }

        Network::do_epoch_preliminaries(System::block_number(), current_epoch);
        set_epoch(current_epoch + 1, 0);
    }

    assert_eq!(SubnetsData::<Test>::try_get(remove_subnet_id), Err(()));
    assert_eq!(
        *network_events().last().unwrap(),
        Event::SubnetDeactivated {
            subnet_id: remove_subnet_id,
            reason: SubnetRemovalReason::PauseExpired
        }
    );
  });
}

#[test]
fn test_do_epoch_preliminaries_remove_under_min_delegate_stake() {
  new_test_ext().execute_with(|| {
    NewRegistrationCostMultiplier::<Test>::put(1000000000000000000);

    let deposit_amount: u128 = 1000000000000000000000;
    let amount: u128 = 100000000000000000000;

    let max_subnets = MaxSubnets::<Test>::get();
    let end = MinSubnetNodes::<Test>::get();
    let last_subnet = 2;

    let mut remove_subnet_id = 0;
    let mut first_subnet_id = 0;
    for s in 0..last_subnet {
        let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
        build_activated_subnet_new(
            subnet_name.clone().into(), 
            0, 
            end, 
            deposit_amount, 
            amount,
        );
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        remove_subnet_id = subnet_id;

        if first_subnet_id == 0 {
            first_subnet_id = subnet_id;
        }
    }

    // increase past n+1 epoch so they are on their activation epochs
    increase_epochs(1);

    // ensure first subnet has enough dstake
    let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(first_subnet_id);
    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(first_subnet_id);
    assert_ok!(Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(0)),
        first_subnet_id,
        total_delegate_stake_balance - min_subnet_delegate_stake,
    ));

    // ensure removal subnet has less than minimum dstake
    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(remove_subnet_id);
    TotalSubnetDelegateStakeBalance::<Test>::insert(remove_subnet_id, min_subnet_delegate_stake-1);

    let current_epoch = Network::get_current_epoch_as_u32();
    Network::do_epoch_preliminaries(System::block_number(), current_epoch);

    assert_eq!(SubnetsData::<Test>::try_get(remove_subnet_id), Err(()));
    assert_eq!(
        *network_events().last().unwrap(),
        Event::SubnetDeactivated {
            subnet_id: remove_subnet_id,
            reason: SubnetRemovalReason::MinSubnetDelegateStake
        }
    );
  });
}

#[test]
fn test_do_epoch_preliminaries_remove_greater_than_max_penalties() {
  new_test_ext().execute_with(|| {
    NewRegistrationCostMultiplier::<Test>::put(1000000000000000000);

    let deposit_amount: u128 = 1000000000000000000000;
    let amount: u128 = 100000000000000000000;

    let max_subnets = MaxSubnets::<Test>::get();
    let end = MinSubnetNodes::<Test>::get();
    let last_subnet = 2;

    let mut remove_subnet_id = 0;
    let mut first_subnet_id = 0;
    for s in 0..last_subnet {
        let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
        build_activated_subnet_new(
            subnet_name.clone().into(), 
            0, 
            end, 
            deposit_amount, 
            amount,
        );
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        remove_subnet_id = subnet_id;

        if first_subnet_id == 0 {
            first_subnet_id = subnet_id;
        }
    }

    // increase past n+1 epoch so they are on their activation epochs
    increase_epochs(1);

    // ensure subnets have enough dstake
    for s in 0..last_subnet {
        let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
        let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);
        assert_ok!(Network::add_to_delegate_stake(
            RuntimeOrigin::signed(account(0)),
            subnet_id,
            total_delegate_stake_balance - min_subnet_delegate_stake,
        ));
    }

    // ensure removal subnet has n+1 max penalties
    SubnetPenaltyCount::<Test>::insert(remove_subnet_id, MaxSubnetPenaltyCount::<Test>::get() + 1);

    let current_epoch = Network::get_current_epoch_as_u32();
    Network::do_epoch_preliminaries(System::block_number(), current_epoch);

    assert_eq!(SubnetsData::<Test>::try_get(remove_subnet_id), Err(()));
    assert_eq!(
        *network_events().last().unwrap(),
        Event::SubnetDeactivated {
            subnet_id: remove_subnet_id,
            reason: SubnetRemovalReason::MaxPenalties
        }
    );
  });
}

#[test]
fn test_do_epoch_preliminaries_remove_max_subnets() {
  new_test_ext().execute_with(|| {
    NewRegistrationCostMultiplier::<Test>::put(1000000000000000000);

    let deposit_amount: u128 = 1000000000000000000000;
    let amount: u128 = 100000000000000000000;

    let max_subnets = MaxSubnets::<Test>::get();
    let end = MinSubnetNodes::<Test>::get();

    let mut remove_subnet_id = 0;
    for s in 0..max_subnets.saturating_add(1) {
        let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 
        build_activated_subnet_new_excess_subnets(
            subnet_name.clone().into(), 
            0, 
            end, 
            deposit_amount, 
            amount,
            1
        );
        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        remove_subnet_id = subnet_id;

        if s + 1 != max_subnets.saturating_add(1) {
            // Force all non-last subnets to be > last subnet 
            TotalSubnetDelegateStakeBalance::<Test>::mutate(subnet_id, |mut n| {
                *n += 1000
            });
        }
    }

    // increase past n+1 epoch so they are on their activation epochs
    increase_epochs(1);

    let removal_interval = MaxSubnetRemovalInterval::<Test>::get();
    let current_epoch = Network::get_current_epoch_as_u32();

    // Set on an epoch where a subnet should not be removed
    let offset_epoch = removal_interval - current_epoch % removal_interval;

    set_epoch(current_epoch + offset_epoch + 1, 0);
    let mut non_removal_epoch_touched = false;

    let current_epoch = Network::get_current_epoch_as_u32();
    
    for e in 0..removal_interval {
        let current_epoch = Network::get_current_epoch_as_u32();
        let is_removal_epoch: bool = current_epoch % removal_interval == 0;

        if !is_removal_epoch {
            // always ensure subnet is at the minimum required delegate stake
            for s in 0..max_subnets.saturating_add(1) {
                let subnet_name: Vec<u8> = format!("subnet-name-{s}").into(); 

                // This will revert if subnet doesn't exist so we know the removal subnet is still alive
                let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
                let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
                let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);
                let add = if subnet_id == remove_subnet_id {
                    total_delegate_stake_balance - min_subnet_delegate_stake
                } else {
                    total_delegate_stake_balance - min_subnet_delegate_stake + 100
                };
                assert_ok!(Network::add_to_delegate_stake(
                    RuntimeOrigin::signed(account(0)),
                    subnet_id,
                    add,
                ));
            }
        }
        Network::do_epoch_preliminaries(System::block_number(), current_epoch);
        if !is_removal_epoch {
            non_removal_epoch_touched = true;
            assert_eq!(SubnetsData::<Test>::get(remove_subnet_id).unwrap().id, remove_subnet_id);
        }
        set_epoch(current_epoch + 1, 0);
    }
    assert!(non_removal_epoch_touched);
    assert_eq!(SubnetsData::<Test>::try_get(remove_subnet_id), Err(()));
    assert_eq!(
        *network_events().last().unwrap(),
        Event::SubnetDeactivated {
            subnet_id: remove_subnet_id,
            reason: SubnetRemovalReason::MaxSubnets
        }
    );
  });
}

#[test]
fn test_do_epoch_preliminaries_remove_registered_min_nodes() {
  new_test_ext().execute_with(|| {
    NewRegistrationCostMultiplier::<Test>::put(1000000000000000000);

    let subnet_registration_epochs = SubnetRegistrationEpochs::<Test>::get();
    let subnet_enactment_epochs = SubnetActivationEnactmentEpochs::<Test>::get();

    let deposit_amount: u128 = 1000000000000000000000;
    let amount: u128 = 100000000000000000000;

    let max_subnets = MaxSubnets::<Test>::get();
    let end = MinSubnetNodes::<Test>::get();

    let subnet_name: Vec<u8> = format!("subnet-name-remove").into(); 
    build_registered_subnet_new(
        subnet_name.clone(),
        0,
        end,
        deposit_amount,
        amount,
        false
    );

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let max_enactment_epoch = SubnetsData::<Test>::get(subnet_id);

    let max_registration_epoch = SubnetRegistrationEpoch::<Test>::get(subnet_id).unwrap().saturating_add(subnet_registration_epochs);
    let max_enactment_epoch = max_registration_epoch.saturating_add(subnet_enactment_epochs);

    // Force set node count below minimum
    TotalActiveSubnetNodes::<Test>::insert(subnet_id, MinSubnetNodes::<Test>::get() - 1);

    // push into enactment period
    set_epoch(max_enactment_epoch, 0);

    let current_epoch = Network::get_current_epoch_as_u32();
    
    Network::do_epoch_preliminaries(System::block_number(), current_epoch);

    assert_eq!(SubnetsData::<Test>::try_get(subnet_id), Err(()));
    assert_eq!(
        *network_events().last().unwrap(),
        Event::SubnetDeactivated {
            subnet_id,
            reason: SubnetRemovalReason::MinSubnetNodes
        }
    );
  });
}

#[test]
fn test_do_epoch_preliminaries_remove_past_enactment_phase() {
  new_test_ext().execute_with(|| {
    NewRegistrationCostMultiplier::<Test>::put(1000000000000000000);

    let subnet_registration_epochs = SubnetRegistrationEpochs::<Test>::get();
    let subnet_enactment_epochs = SubnetActivationEnactmentEpochs::<Test>::get();

    let deposit_amount: u128 = 1000000000000000000000;
    let amount: u128 = 100000000000000000000;

    let max_subnets = MaxSubnets::<Test>::get();
    let end = MinSubnetNodes::<Test>::get();

    let subnet_name: Vec<u8> = format!("subnet-name-remove").into(); 
    build_registered_subnet_new(
        subnet_name.clone(),
        0,
        end,
        deposit_amount,
        amount,
        false
    );

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let max_enactment_epoch = SubnetsData::<Test>::get(subnet_id);

    let max_registration_epoch = SubnetRegistrationEpoch::<Test>::get(subnet_id).unwrap().saturating_add(subnet_registration_epochs);
    let max_enactment_epoch = max_registration_epoch.saturating_add(subnet_enactment_epochs);

    // push into enactment period
    set_epoch(max_enactment_epoch + 1, 0);

    let current_epoch = Network::get_current_epoch_as_u32();
    
    Network::do_epoch_preliminaries(System::block_number(), current_epoch);

    assert_eq!(SubnetsData::<Test>::try_get(subnet_id), Err(()));
    assert_eq!(
        *network_events().last().unwrap(),
        Event::SubnetDeactivated {
            subnet_id,
            reason: SubnetRemovalReason::EnactmentPeriod
        }
    );
  });
}
