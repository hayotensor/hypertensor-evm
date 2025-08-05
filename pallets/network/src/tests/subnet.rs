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
  MinSubnetNodes, 
  TotalSubnetNodes,
  SubnetsData,
  SubnetData,
  RegistrationSubnetData,
  SubnetRemovalReason,
  SubnetActivationEnactmentEpochs,
  SubnetRegistrationEpochs,
  SubnetState,
  TotalActiveSubnets,
  MaxSubnetNodes,
  SubnetSlot,
  SlotAssignment,
  AssignedSlots,
  MaxSubnets,
  LastRegistrationBlock,
  MinRegistrationCost,
  RegistrationCostDecayBlocks,
  LastRegistrationCost,
  MaxBootnodes,
  SubnetBootnodeAccess,
  DefaultMaxVectorLength,
  SubnetBootnodes,
};
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

#[test]
fn test_register_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    // let cost = Network::registration_cost(epoch);
    let cost = Network::get_current_registration_cost(block_number);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();
  
    let start = 0;
    let end = min_nodes + 1;

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
      subnets,
      max_subnet_nodes,
      subnet_name.clone().into(),
      start, 
      end
    );
    
    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch - epoch);

    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        account(1),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();

    // Check treasury pot
    let minimum_balance = Balances::minimum_balance();
    let pot = Treasury::pot();
    assert_eq!(cost, pot + minimum_balance);
  })
}

// #[test]
// fn test_register_subnet_subnet_registration_cooldown() {
//   new_test_ext().execute_with(|| {
//     let subnet_name: Vec<u8> = "subnet-name".into();

//     increase_epochs(1);

//     let epoch_length = EpochLength::get();
//     let block_number = System::block_number();
//     let epoch = System::block_number().saturating_div(epoch_length);
  
//     // let cost = Network::registration_cost(epoch);
//     let cost = Network::get_current_registration_cost(block_number);
  
//     let _ = Balances::deposit_creating(&account(0), cost+1000);
  
//     let min_nodes = MinSubnetNodes::<Test>::get();

//     let start = 0;
//     let end = min_nodes + 1;

//     let subnets = TotalActiveSubnets::<Test>::get() + 1;
//     let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//     let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
//       subnets,
//       max_subnet_nodes,
//       subnet_name.clone().into(),
//       start, 
//       end
//     );
  
  
//     let epoch_length = EpochLength::get();
//     let block_number = System::block_number();
//     let epoch = System::block_number().saturating_div(epoch_length);
//     let next_registration_epoch = Network::get_next_registration_epoch(epoch);
//     // increase_epochs(next_registration_epoch - epoch);

//     // --- Register subnet for activation
//     assert_ok!(
//       Network::register_subnet(
//         RuntimeOrigin::signed(account(0)),
//         account(1),
//         add_subnet_data,
//       )
//     );
  
//     let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
//     let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
//     let subnet_name: Vec<u8> = "subnet-name-2".into();

//     let subnets = TotalActiveSubnets::<Test>::get() + 1;
//     let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//     let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
//       subnets,
//       max_subnet_nodes,
//       subnet_name.clone().into(),
//       start, 
//       end
//     );
  

//     assert_err!(
//       Network::register_subnet(
//         RuntimeOrigin::signed(account(0)),
//         account(2),
//         add_subnet_data.clone(),
//       ),
//       Error::<Test>::SubnetRegistrationCooldown
//     );

//     let epoch_length = EpochLength::get();
//     let block_number = System::block_number();
//     let epoch = System::block_number().saturating_div(epoch_length);
//     let next_registration_epoch = Network::get_next_registration_epoch(epoch);
//     increase_epochs(next_registration_epoch);

//     let epoch_length = EpochLength::get();
//     let block_number = System::block_number();
//     let epoch = System::block_number().saturating_div(epoch_length);
  
//     // let cost = Network::registration_cost(epoch);
//     let cost = Network::get_current_registration_cost(block_number);
  
//     let _ = Balances::deposit_creating(&account(0), cost+1000);

//     // --- Register after cooldown
//     assert_ok!(
//       Network::register_subnet(
//         RuntimeOrigin::signed(account(0)),
//         account(3),
//         add_subnet_data.clone(),
//       )
//     );

//     // --- Cooldown expected after registering again
//     let subnet_name: Vec<u8> = "subnet-name-3".into();

//     let subnets = TotalActiveSubnets::<Test>::get() + 1;
//     let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
//     let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
//       subnets,
//       max_subnet_nodes,
//       subnet_name.clone().into(),
//       start, 
//       end
//     );
  

//     assert_err!(
//       Network::register_subnet(
//         RuntimeOrigin::signed(account(1)),
//         account(4),
//         add_subnet_data.clone(),
//       ),
//       Error::<Test>::SubnetRegistrationCooldown
//     );
//   })
// }

#[test]
fn test_register_subnet_exists_error() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    // let cost = Network::registration_cost(epoch);
    let cost = Network::get_current_registration_cost(block_number);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let start = 0;
    let end = min_nodes + 1;

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
      subnets,
      max_subnet_nodes,
      subnet_name.clone().into(),
      start, 
      end
    );
  
  
    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch - epoch);

    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        account(1),
        add_subnet_data.clone(),
      )
    );
  
    assert_err!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        account(1),
        add_subnet_data.clone(),
      ),
      Error::<Test>::SubnetNameExist
    );

  })
}

#[test]
fn test_register_subnet_not_enough_balance_err() {
  new_test_ext().execute_with(|| {
    // let _ = Balances::deposit_creating(&account(0), cost+1000);  
    let subnet_name: Vec<u8> = "subnet-name".into();

    let min_nodes = MinSubnetNodes::<Test>::get();

    let start = 0;
    let end = min_nodes + 1;

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
      subnets,
      max_subnet_nodes,
      subnet_name.clone().into(),
      start, 
      end
    );
  

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch - epoch);

    assert_err!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        account(1),
        add_subnet_data,
      ),
      Error::<Test>::NotEnoughBalanceToRegisterSubnet
    );
  })
}

#[test]
fn test_activate_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    // let cost = Network::registration_cost(epoch);
    let cost = Network::get_current_registration_cost(block_number);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let start = 0;
    let end = min_nodes + 1;

    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
      subnets,
      max_subnet_nodes,
      subnet_name.clone().into(),
      start, 
      end
    );
    
    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch - epoch);
  
    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        account(1),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let name = subnet.name;
		let min_nodes = MinSubnetNodes::<Test>::get();

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 0..min_nodes {
      let _n = n + 1;
      let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      let peer_id = peer(subnets*max_subnet_nodes+_n);
      let bootnode_peer_id = peer(subnets*max_subnet_nodes+_n);
      let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
      assert_ok!(
        Network::add_subnet_node(
          RuntimeOrigin::signed(coldkey.clone()),
          subnet_id,
          hotkey.clone(),
          peer_id.clone(),
          bootnode_peer_id.clone(),
          None,
          0,
          amount,
          None,
          None,
          None,
        ) 
      );
    }
  
    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(subnet_id) + 100e+18 as u128;
    let _ = Balances::deposit_creating(&account(1), min_subnet_delegate_stake+500);
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        min_subnet_delegate_stake,
      ) 
    );

    // --- Increase blocks to max registration block
    let epochs = SubnetRegistrationEpochs::<Test>::get();
    increase_epochs(epochs + 1);
    let current_epoch = get_epoch();

    assert_ok!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      )
    );

    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
    assert_eq!(subnet.id, subnet_id);

    // ensure subnet exists and nothing changed but the activation block
    assert_eq!(subnet.id, id);
    assert_eq!(subnet.name, name);
    assert_eq!(subnet.state, SubnetState::Active);
  })
}

#[test]
fn test_activate_subnet_invalid_subnet_id_error() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    // let cost = Network::registration_cost(epoch);
    let cost = Network::get_current_registration_cost(block_number);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let start = 0;
    let end = min_nodes + 1;

    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
      subnets,
      max_subnet_nodes,
      subnet_name.clone().into(),
      start, 
      end
    );

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch - epoch);

    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        account(1),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let name = subnet.name;
		let min_nodes = MinSubnetNodes::<Test>::get();

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 0..min_nodes {
      let _n = n + 1;
      let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      let peer_id = peer(subnets*max_subnet_nodes+_n);
      let bootnode_peer_id = peer(subnets*max_subnet_nodes+_n);
      let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
      assert_ok!(
        Network::add_subnet_node(
          RuntimeOrigin::signed(coldkey.clone()),
          subnet_id,
          hotkey.clone(),
          peer_id.clone(),
          bootnode_peer_id.clone(),
          None,
          0,
          amount,
          None,
          None,
          None,
        ) 
      );
    }
  
    assert_err!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id+1,
      ),
      Error::<Test>::NotSubnetOwner
    );
  })
}

#[test]
fn test_activate_subnet_already_activated_err() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    // let cost = Network::registration_cost(epoch);
    let cost = Network::get_current_registration_cost(block_number);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let start = 0;
    let end = min_nodes + 1;

    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
      subnets,
      max_subnet_nodes,
      subnet_name.clone().into(),
      start, 
      end
    );
  
    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch - epoch);

    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        account(1),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let name = subnet.name;
		let min_nodes = MinSubnetNodes::<Test>::get();

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 0..min_nodes {
      let _n = n + 1;
      let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      let peer_id = peer(subnets*max_subnet_nodes+_n);
      let bootnode_peer_id = peer(subnets*max_subnet_nodes+_n);
      let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
      assert_ok!(
        Network::add_subnet_node(
          RuntimeOrigin::signed(coldkey.clone()),
          subnet_id,
          hotkey.clone(),
          peer_id.clone(),
          bootnode_peer_id.clone(),
          None,
          0,
          amount,
          None,
          None,
          None,
        ) 
      );
    }
  
    // let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance() + 100e+18 as u128;
    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(subnet_id) + 100e+18 as u128;
    let _ = Balances::deposit_creating(&account(1), min_subnet_delegate_stake+500);
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        min_subnet_delegate_stake,
      ) 
    );

    // --- Increase blocks to max registration block
    let epochs = SubnetRegistrationEpochs::<Test>::get();
    increase_epochs(epochs + 1);
    let current_epoch = get_epoch();

    assert_ok!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      )
    );

    assert_err!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      ),
      Error::<Test>::SubnetActivatedAlready
    );
  })
}

#[test]
fn test_activate_subnet_enactment_period_remove_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    // let cost = Network::registration_cost(epoch);
    let cost = Network::get_current_registration_cost(block_number);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let start = 0;
    let end = min_nodes + 1;

    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
      subnets,
      max_subnet_nodes,
      subnet_name.clone().into(),
      start, 
      end
    );
  
  
    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch - epoch);

    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        account(1),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();

    let id = subnet.id;
		let name = subnet.name;
		let min_nodes = MinSubnetNodes::<Test>::get();

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 0..min_nodes {
      let _n = n + 1;
      let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      let peer_id = peer(subnets*max_subnet_nodes+_n);
      let bootnode_peer_id = peer(subnets*max_subnet_nodes+_n);
      let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
      assert_ok!(
        Network::add_subnet_node(
          RuntimeOrigin::signed(coldkey.clone()),
          subnet_id,
          hotkey.clone(),
          peer_id.clone(),
          bootnode_peer_id.clone(),
          None,
          0,
          amount,
          None,
          None,
          None,
        ) 
      );
    }
  
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance() + 100e+18 as u128;
    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(subnet_id) + 100e+18 as u128;
    let _ = Balances::deposit_creating(&account(1), min_subnet_delegate_stake+500);
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        min_subnet_delegate_stake,
      ) 
    );

    // --- Increase blocks to max registration block
    let registration_epochs = SubnetRegistrationEpochs::<Test>::get();
    let enactment_epochs = SubnetActivationEnactmentEpochs::<Test>::get();
    increase_epochs(registration_epochs + enactment_epochs + 1);

    assert_ok!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      )
    );

    assert_eq!(
			*network_events().last().unwrap(),
			Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::EnactmentPeriod
      }
		);

    let removed_subnet_id = SubnetName::<Test>::try_get(subnet_name.clone());
    assert_eq!(removed_subnet_id, Err(()));
    let subnet = SubnetsData::<Test>::try_get(subnet_id);
    assert_eq!(subnet, Err(()));

    // --- Ensure nodes can be removed and unstake
    // post_subnet_removal_ensures(subnet_id, subnets, max_subnet_nodes, subnet_name, 0, total_subnet_nodes);
  })
}

#[test]
fn test_activate_subnet_initializing_error() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    // let cost = Network::registration_cost(epoch);
    let cost = Network::get_current_registration_cost(block_number);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let start = 0;
    let end = min_nodes + 1;

    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
      subnets,
      max_subnet_nodes,
      subnet_name.clone().into(),
      start, 
      end
    );
  
  
    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch - epoch);

    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        account(1),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let name = subnet.name;
		let min_nodes = MinSubnetNodes::<Test>::get();

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 0..min_nodes {
      let _n = n + 1;
      let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      let peer_id = peer(subnets*max_subnet_nodes+_n);
      let bootnode_peer_id = peer(subnets*max_subnet_nodes+_n);
      let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
      assert_ok!(
        Network::add_subnet_node(
          RuntimeOrigin::signed(coldkey.clone()),
          subnet_id,
          hotkey.clone(),
          peer_id.clone(),
          bootnode_peer_id.clone(),
          None,
          0,
          amount,
          None,
          None,
          None,
        ) 
      );
    }
  
    // let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance() + 100e+18 as u128;
    let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(subnet_id) + 100e+18 as u128;
    let _ = Balances::deposit_creating(&account(1), min_subnet_delegate_stake+500);
    // --- Add the minimum required delegate stake balance to activate the subnet
    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(1)),
        subnet_id,
        min_subnet_delegate_stake,
      ) 
    );

    assert_err!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      ),
      Error::<Test>::SubnetInitializing
    );
  })
}

#[test]
fn test_activate_subnet_min_subnet_nodes_remove_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    // let cost = Network::registration_cost(epoch);
    let cost = Network::get_current_registration_cost(block_number);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let start = 0;
    let end = min_nodes + 1;

    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
      subnets,
      max_subnet_nodes,
      subnet_name.clone().into(),
      start, 
      end
    );
  
    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch - epoch);

    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        account(1),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let name = subnet.name;
		let min_nodes = MinSubnetNodes::<Test>::get();

    // --- Increase epochs to max registration epoch
    let epochs = SubnetRegistrationEpochs::<Test>::get();
    increase_epochs(epochs + 1);

    assert_ok!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      )
    );

    assert_eq!(
			*network_events().last().unwrap(),
			Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::MinSubnetNodes
      }
		);

    let removed_subnet_id = SubnetName::<Test>::try_get(subnet_name.clone());
    assert_eq!(removed_subnet_id, Err(()));
    let subnet = SubnetsData::<Test>::try_get(subnet_id);
    assert_eq!(subnet, Err(()));
  })
}

#[test]
fn test_activate_subnet_min_delegate_balance_remove_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_name: Vec<u8> = "subnet-name".into();

    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
  
    // let cost = Network::registration_cost(epoch);
    let cost = Network::get_current_registration_cost(block_number);
  
    let _ = Balances::deposit_creating(&account(0), cost+1000);
  
    let min_nodes = MinSubnetNodes::<Test>::get();

    let start = 0;
    let end = min_nodes + 1;

    let max_subnets = MaxSubnets::<Test>::get();
    let subnets = TotalActiveSubnets::<Test>::get() + 1;
    let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
    let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
      subnets,
      max_subnet_nodes,
      subnet_name.clone().into(),
      start, 
      end
    );
  
    let epoch_length = EpochLength::get();
    let block_number = System::block_number();
    let epoch = System::block_number().saturating_div(epoch_length);
    let next_registration_epoch = Network::get_next_registration_epoch(epoch);
    increase_epochs(next_registration_epoch - epoch);

    // --- Register subnet for activation
    assert_ok!(
      Network::register_subnet(
        RuntimeOrigin::signed(account(0)),
        account(1),
        add_subnet_data,
      )
    );
  
    let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
    let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  
    let id = subnet.id;
		let name = subnet.name;
		let min_nodes = MinSubnetNodes::<Test>::get();

    // --- Add subnet nodes
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    for n in 0..min_nodes {
      let _n = n + 1;
      let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
      let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
      let peer_id = peer(subnets*max_subnet_nodes+_n);
      let bootnode_peer_id = peer(subnets*max_subnet_nodes+_n);
      let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
      assert_ok!(
        Network::add_subnet_node(
          RuntimeOrigin::signed(coldkey.clone()),
          subnet_id,
          hotkey.clone(),
          peer_id.clone(),
          bootnode_peer_id.clone(),
          None,
          0,
          amount,
          None,
          None,
          None,
        ) 
      );
    }
  
    // --- Increase epochs to max registration epoch
    let epochs = SubnetRegistrationEpochs::<Test>::get();
    increase_epochs(epochs + 1);

    assert_ok!(
      Network::activate_subnet(
        RuntimeOrigin::signed(account(0)),
        subnet_id,
      )
    );

    assert_eq!(
			*network_events().last().unwrap(),
			Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::MinSubnetDelegateStake
      }
		);

    let removed_subnet_id = SubnetName::<Test>::try_get(subnet_name.clone());
    assert_eq!(removed_subnet_id, Err(()));
    let subnet = SubnetsData::<Test>::try_get(subnet_id);
    assert_eq!(subnet, Err(()));
  })
}

#[test]
fn test_assign_subnet_slot_success() {
	new_test_ext().execute_with(|| {
		let subnet_id = 1;

		let slot = Network::assign_subnet_slot(subnet_id).unwrap();
		assert_eq!(slot, 2); // Should assign slot 2, since 0-1 is skipped

		assert_eq!(SubnetSlot::<Test>::get(subnet_id), Some(2));
		assert_eq!(SlotAssignment::<Test>::get(2), Some(subnet_id));
		assert!(AssignedSlots::<Test>::get().contains(&2));
	});
}

#[test]
fn test_assign_all_slots_and_fail() {
	new_test_ext().execute_with(|| {
		let max_slots = EpochLength::get();

		// Fill all slots from 1..max_slots
		for i in 2..max_slots {
			let subnet_id = i;
			assert_ok!(Network::assign_subnet_slot(subnet_id));
		}

		// Now this call should fail with NoAvailableSlots
		let result = Network::assign_subnet_slot(999);
		assert_noop!(result, Error::<Test>::NoAvailableSlots);
	});
}

#[test]
fn test_free_slot_removes_assignment() {
	new_test_ext().execute_with(|| {
		let subnet_id = 42;
		let _ = Network::assign_subnet_slot(subnet_id);

		assert!(SubnetSlot::<Test>::contains_key(subnet_id));
		assert!(AssignedSlots::<Test>::get().len() > 0);

		Network::free_slot_of_subnet(subnet_id);

		assert!(!SubnetSlot::<Test>::contains_key(subnet_id));
		assert_eq!(SlotAssignment::<Test>::iter().count(), 0);
		assert_eq!(AssignedSlots::<Test>::get().len(), 0);
	});
}

#[test]
fn test_free_slot_does_nothing_if_slot_not_found() {
	new_test_ext().execute_with(|| {
		// Should be a no-op, no panic
		Network::free_slot_of_subnet(123);

		// Make sure storage still empty
		assert_eq!(SubnetSlot::<Test>::iter().count(), 0);
		assert_eq!(SlotAssignment::<Test>::iter().count(), 0);
		assert_eq!(AssignedSlots::<Test>::get().len(), 0);
	});
}

#[test]
fn test_assign_and_free_reassigns_correctly() {
	new_test_ext().execute_with(|| {
		let subnet1 = 1;
		let subnet2 = 2;

		let slot1 = Network::assign_subnet_slot(subnet1).unwrap();
		assert_eq!(slot1, 2);

		Network::free_slot_of_subnet(subnet1);

		// Should now reuse slot 2
		let slot2 = Network::assign_subnet_slot(subnet2).unwrap();
		assert_eq!(slot2, 2);
	});
}

#[test]
fn test_get_current_registration_cost() {
  new_test_ext().execute_with(|| {
    // ---- Initial state ----
    // Default cost should be 1000e18 (LastRegistrationCost default)
    let initial_cost = LastRegistrationCost::<Test>::get();
    // let initial_cost = Network::get_current_registration_cost();
    // assert_eq!(initial_cost, 1000000000000000000000);

    // ---- Simulate elapsed blocks with no updates ----
    // Move forward half the decay period
    let half_decay = RegistrationCostDecayBlocks::<Test>::get() / 2;
    let last_block = LastRegistrationBlock::<Test>::get();
    System::set_block_number(last_block + half_decay);

    let cost_after_half_decay = Network::get_current_registration_cost(System::block_number());
    // Cost should be between min_price and initial_cost
    let min_price = MinRegistrationCost::<Test>::get();
    assert!(cost_after_half_decay < initial_cost);
    assert!(cost_after_half_decay > min_price);

    // ---- Move to full decay period ----
    System::set_block_number(last_block + RegistrationCostDecayBlocks::<Test>::get());
    let cost_after_full_decay = Network::get_current_registration_cost(System::block_number());
    // Cost should be at min price
    assert_eq!(cost_after_full_decay, min_price);

    // // ---- Move beyond full decay ----
    System::set_block_number(last_block + RegistrationCostDecayBlocks::<Test>::get() * 2);
    let cost_after_double_decay = Network::get_current_registration_cost(System::block_number());
    // Still at min price
    assert_eq!(cost_after_double_decay, min_price);
  });
}

#[test]
fn test_update_bootnodes() {
  new_test_ext().execute_with(|| {
    increase_epochs(1);
    // --- Setup ---
    let caller = account(0);
    let unauth_caller = account(1);
    let max_bootnodes = MaxBootnodes::<Test>::get();
    let subnet_id = 1u32;

    assert_err!(
      Network::update_bootnodes(
        RuntimeOrigin::signed(caller.clone()),
        subnet_id,
        BTreeSet::new(),
        BTreeSet::new(),
      ),
      Error::<Test>::InvalidSubnetId
    );

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


    // Give caller access to manage bootnodes
    SubnetBootnodeAccess::<Test>::insert(subnet_id, BTreeSet::from([caller.clone()]));

    // Helper to build a bounded vec from bytes
    let bv = |b: u8| BoundedVec::<u8, DefaultMaxVectorLength>::try_from(vec![b]).unwrap();

    // --- Case 1: Add bootnodes ---
    let add_set = BTreeSet::from([bv(1), bv(2)]);
    assert_ok!(Network::update_bootnodes(
      RuntimeOrigin::signed(caller.clone()),
      subnet_id,
      add_set.clone(),
      BTreeSet::new(),
    ));

    // Verify bootnodes added
    let stored = SubnetBootnodes::<Test>::get(subnet_id);
    assert!(stored.contains(&bv(1)));
    assert!(stored.contains(&bv(2)));

    // --- Case 2: Remove a bootnode ---
    let remove_set = BTreeSet::from([bv(1)]);
    assert_ok!(
      Network::update_bootnodes(
        RuntimeOrigin::signed(caller.clone()),
        subnet_id,
        BTreeSet::new(),
        remove_set.clone(),
      )
    );

    // Verify bootnode removed
    let stored = SubnetBootnodes::<Test>::get(subnet_id);
    assert!(!stored.contains(&bv(1)));
    assert!(stored.contains(&bv(2))); // bv(2) still present

    // --- Case 3: Too many bootnodes ---
    // Fill to max
    let mut add_set = BTreeSet::new();
    for i in 3..=max_bootnodes as u8 {
      add_set.insert(bv(i));
    }
    assert_ok!(Network::update_bootnodes(
      RuntimeOrigin::signed(caller.clone()),
      subnet_id,
      add_set.clone(),
      BTreeSet::new(),
    ));


    // Try to add one more (should fail)
    let too_many = BTreeSet::from([bv(99), bv(100)]);
    assert_err!(
      Network::update_bootnodes(
        RuntimeOrigin::signed(caller.clone()),
        subnet_id,
        too_many.clone(),
        BTreeSet::new(),
      ),
      Error::<Test>::TooManyBootnodes
    );

    // --- Case 4: Unauthorized caller ---
    assert_err!(
      Network::update_bootnodes(
        RuntimeOrigin::signed(unauth_caller),
        subnet_id,
        BTreeSet::new(),
        BTreeSet::new(),
      ),
      Error::<Test>::InvalidAccess
    );

    // --- Case 5: Check event ---
    assert_eq!(
      *network_events().last().unwrap(),
      Event::BootnodesUpdated {
        subnet_id,
        added: add_set.clone(),
        removed: BTreeSet::new(),
      }
    );
  });
}
