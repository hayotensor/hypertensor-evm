use super::mock::*;
use crate::Event;
use sp_core::OpaquePeerId as PeerId;
use frame_support::assert_ok;
use log::info;
use crate::{
  SubnetNodeConsensusData, 
  TotalStake, 
  SubnetElectedValidator,
  SubnetName, 
  SubnetNodeClass,
  SubnetsData,
  AccountSubnetStake, 
  NetworkMinStakeBalance,
  TotalSubnetDelegateStakeBalance,
  AccountSubnetDelegateStakeShares, 
  RegistrationSubnetData,
  KeyType,
  StakeUnbondingLedger, 
  TotalSubnetStake, 
  HotkeySubnetNodeId, 
  SubnetNodeIdHotkey, 
  SubnetNodesData, 
  PeerIdSubnetNodeId,
  HotkeyOwner,
  MaxSubnetNodes,
  MinSubnetNodes,
  TotalSubnetNodes,
  TotalSubnetNodeUids,
  BootstrapPeerIdSubnetNodeId,
  SubnetNodeUniqueParam,
  SubnetPenaltyCount,
  SubnetConsensusSubmission,
  Proposals,
  SubnetRegistrationInitialColdkeys,
  SubnetNodeNonUniqueParamLastSet,
  SubnetNodePenalties,
  SubnetRegistrationEpochs,
  SubnetOwner,
  SubnetRegistrationEpoch,
  TotalActiveSubnetNodes,
  TotalActiveSubnets,
  SubnetSlot,
  NodeRemovalSystem,
  TotalOverwatchNodeUids,
  OverwatchNode,
  OverwatchNodes,
  OverwatchNodeIdHotkey,
  ColdkeyHotkeys,
  AccountOverwatchStake,
  TotalOverwatchStake,
  OverwatchReveals,
  HotkeyOverwatchNodeId,
  MaxSubnets,
  ConsensusData,
  SubnetNode,
  SubnetNodeElectionSlots,
  RegisteredSubnetNodesData,
  SubnetState,
  SubnetData,
};
use sp_std::collections::btree_map::BTreeMap;
use frame_support::traits::{OnInitialize, Currency, ExistenceRequirement};
use sp_std::collections::btree_set::BTreeSet;
use sp_runtime::SaturatedConversion;
use sp_io::hashing::blake2_128;
use frame_support::{
	storage::bounded_vec::BoundedVec,
};

pub type AccountIdOf<Test> = <Test as frame_system::Config>::AccountId;

// pub fn account(id: u32) -> AccountIdOf<Test> {
// 	[id as u8; 32].into()
// }
pub fn account(id: u32) -> AccountIdOf<Test> {
    let mut bytes = [0u8; 32];
    bytes[0..4].copy_from_slice(&id.to_le_bytes());
    bytes.into()
}

pub fn get_coldkey(subnets: u32, max_subnet_nodes: u32, n: u32) -> AccountIdOf<Test> {
  account(subnets*max_subnet_nodes+n)
}

pub fn get_hotkey(subnets: u32, max_subnet_nodes: u32, max_subnets: u32, n: u32) -> AccountIdOf<Test> {
  account(subnets*max_subnets*max_subnet_nodes+n)
}

// it is possible to use `use libp2p::PeerId;` with `PeerId::random()`
// https://github.com/paritytech/substrate/blob/033d4e86cc7eff0066cd376b9375f815761d653c/frame/node-authorization/src/mock.rs#L90
pub fn peer(id: u32) -> PeerId {
	// let peer_id = format!("12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA{id}");
  let peer_id = format!("QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N{id}"); 
	PeerId(peer_id.into())
}
// bafzbeie5745rpv2m6tjyuugywy4d5ewrqgqqhfnf445he3omzpjbx5xqxe
// QmYyQSo1c1Ym7orWxLYvCrM2EmxFTANf8wXmmE7DWjhx5N
// 12D3KooWD3eckifWpRn9wQpMG9R9hX3sD158z7EqHWmweQAJU5SA

pub fn get_min_stake_balance() -> u128 {
	NetworkMinStakeBalance::<Test>::get()
}

pub const PERCENTAGE_FACTOR: u128 = 1000000000000000000_u128;
pub const DEFAULT_SCORE: u128 = 500000000000000000;
pub const MAX_SUBNET_NODES: u32 = 254;
pub const DEFAULT_REGISTRATION_BLOCKS: u32 = 130_000;
pub const DEFAULT_DELEGATE_REWARD_RATE: u128 = 100000000000000000; // 10%
pub const ALICE_EXPECTED_BALANCE: u128 = 1000000000000000000000000; // 1,000,000

pub fn build_activated_subnet_new(subnet_name: Vec<u8>, start: u32, mut end: u32, deposit_amount: u128, amount: u128) {
  let alice = 0;
  let alice_balance = Balances::free_balance(&account(alice));
  if alice_balance == 0 {
    let _ = Balances::deposit_creating(&account(alice), ALICE_EXPECTED_BALANCE);
  }

  let epoch_length = EpochLength::get();
  let block_number = System::block_number();
  let epoch = System::block_number().saturating_div(epoch_length);
  let next_registration_epoch = Network::get_next_registration_epoch(epoch);
  increase_epochs(next_registration_epoch.saturating_sub(epoch));

  let subnets = TotalActiveSubnets::<Test>::get() + 1;
  let max_subnets = MaxSubnets::<Test>::get();
  let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

  let owner_coldkey = account(subnets*max_subnets*max_subnet_nodes);
  let owner_hotkey = account(subnets*max_subnets*max_subnet_nodes+1);

  // let cost = Network::registration_cost(epoch);
  let cost = Network::get_current_registration_cost(block_number);
  // let _ = Balances::deposit_creating(&owner_coldkey.clone(), cost+500);
  log::error!("cost {:?}", cost);
  assert_ok!(
    Balances::transfer(
      &account(0), // alice
      &owner_coldkey.clone(),
      cost+500,
      ExistenceRequirement::KeepAlive,
    )
  );

  let min_nodes = MinSubnetNodes::<Test>::get();

  if end == 0 {
    end = min_nodes;
  }

  let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
    subnets,
    max_subnet_nodes,
    subnet_name.clone().into(),
    start,
    end
  );

  // --- Register subnet for activation
  assert_ok!(
    Network::register_subnet(
      RuntimeOrigin::signed(owner_coldkey.clone()),
      owner_hotkey.clone(),
      add_subnet_data,
    )
  );

  let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
  let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  assert_eq!(subnet.state, SubnetState::Registered);
  let owner = SubnetOwner::<Test>::get(subnet_id).unwrap();
  assert_eq!(owner, owner_coldkey.clone());

  let epoch_length = EpochLength::get();
  let epoch = System::block_number() / epoch_length;

  let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

  // --- Add subnet nodes
  let block_number = System::block_number();
  let mut amount_staked = 0;
  for n in start..end {
    let _n = n + 1;
    let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
    let peer_id = peer(subnets*max_subnet_nodes+_n);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+_n);
    assert_ok!(
      Balances::transfer(
        &account(0), // alice
        &coldkey.clone(),
        amount+500,
        ExistenceRequirement::KeepAlive,
      )
    );
    // let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
    amount_staked += amount;
    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id,
        None,
        0,
        amount,
        None,
        None,
      ) 
    );

    // assert!(false);
    // assert_eq!(
    //   *network_events().last().unwrap(),
    //   Event::SubnetNodeActivated {
    //     subnet_id: subnet_id, 
    //     subnet_node_id: n
    //   }
    // ); 

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
    // assert_eq!(hotkey_subnet_node_id, coldkey_n);

    let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_id_hotkey, hotkey.clone());

    // Is activated, registered element is removed
    assert_eq!(RegisteredSubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id), Err(()));

    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_data.hotkey, hotkey.clone());

    let key_owner = HotkeyOwner::<Test>::get(subnet_node_data.hotkey.clone());
    assert_eq!(key_owner, coldkey.clone());

    assert_eq!(subnet_node_data.peer_id, peer_id.clone());

    // --- Is ``Validator`` if registered before subnet activation
    assert_eq!(subnet_node_data.classification.node_class, SubnetNodeClass::Validator);
    assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, subnet_epoch));

    let subnet_node_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer_id.clone());
    assert_eq!(subnet_node_account, hotkey_subnet_node_id);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
    assert_eq!(account_subnet_stake, amount);

    let mut is_electable = false;
    for node_id in SubnetNodeElectionSlots::<Test>::get(subnet_id).iter() {
      if *node_id == hotkey_subnet_node_id {
        is_electable = true;
      }
    }
    assert!(is_electable);
  }

  let total_nodes = TotalActiveSubnetNodes::<Test>::get(subnet_id);
  assert_eq!(total_nodes, end);

  let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
  assert_eq!(total_subnet_stake, amount_staked);

  let total_stake = TotalStake::<Test>::get();
  assert_eq!(total_subnet_stake, amount_staked);

  // --- Increase epochs to max registration epoch
  let epochs = SubnetRegistrationEpochs::<Test>::get();
  increase_epochs(epochs + 1);

  let delegate_staker_account = 1;
  // Add 100e18 to account for block increase on activation
  let mut min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);
  min_subnet_delegate_stake = min_subnet_delegate_stake + Network::percent_mul(min_subnet_delegate_stake, 10000000000000000);
  // let _ = Balances::deposit_creating(&account(delegate_staker_account), min_subnet_delegate_stake+500);
  assert_ok!(
    Balances::transfer(
      &account(0), // alice
      &account(delegate_staker_account),
      min_subnet_delegate_stake+500,
      ExistenceRequirement::KeepAlive,
    )
  );

  // log::error!(" ");
  // log::error!("Subnet ID {:?}", subnet_id);
  // log::error!("tes.rs min_subnet_delegate_stake        {:?}", min_subnet_delegate_stake);

  assert_ne!(min_subnet_delegate_stake, u128::MAX);
  // --- Add the minimum required delegate stake balance to activate the subnet
  assert_ok!(
    Network::add_to_delegate_stake(
      RuntimeOrigin::signed(account(delegate_staker_account)),
      subnet_id,
      min_subnet_delegate_stake,
    ) 
  );

  let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
  assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);
  // log::error!("tes.rs total_delegate_stake_balance     {:?}", total_delegate_stake_balance);

  let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(subnet_id);
  // log::error!("tes.rs min_subnet_delegate_stake        {:?}", min_subnet_delegate_stake);

  assert_ok!(
    Network::activate_subnet(
      RuntimeOrigin::signed(owner_coldkey.clone()),
      subnet_id,
    )
  );

  assert_eq!(
    *network_events().last().unwrap(),
    Event::SubnetActivated {
      subnet_id: subnet_id, 
    }
  );

  let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  assert_eq!(subnet.state, SubnetState::Active);

  increase_epochs(2);

  // --- Check validator chosen on activation
  // let next_epoch = System::block_number() / epoch_length + 1;
  // let validator = SubnetElectedValidator::<Test>::get(subnet_id, next_epoch as u32);
  // assert!(validator != None, "Validator is None");
}

pub fn build_activated_subnet_with_delegator_rewards(
  subnet_name: Vec<u8>, 
  start: u32, 
  mut end: u32, 
  deposit_amount: u128, 
  amount: u128,
  delegate_reward_rate: u128,
) {
  let epoch_length = EpochLength::get();
  let block_number = System::block_number();
  let epoch = System::block_number().saturating_div(epoch_length);
  let next_registration_epoch = Network::get_next_registration_epoch(epoch);
  increase_epochs(next_registration_epoch.saturating_sub(epoch));

  let subnets = TotalActiveSubnets::<Test>::get() + 1;
  let max_subnets = MaxSubnets::<Test>::get();
  let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

  let owner_coldkey = account(subnets*max_subnets*max_subnet_nodes);
  let owner_hotkey = account(subnets*max_subnets*max_subnet_nodes+1);

  let cost = Network::get_current_registration_cost(block_number);
  let _ = Balances::deposit_creating(&owner_coldkey.clone(), cost+1000);

  let min_nodes = MinSubnetNodes::<Test>::get();

  if end == 0 {
    end = min_nodes;
  }


  let add_subnet_data: RegistrationSubnetData<AccountId> = default_registration_subnet_data(
    subnets,
    max_subnet_nodes,
    subnet_name.clone().into(),
    start, 
    end
  );

  // --- Register subnet for activation
  assert_ok!(
    Network::register_subnet(
      RuntimeOrigin::signed(owner_coldkey.clone()),
      owner_hotkey.clone(),
      add_subnet_data,
    )
  );

  let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
  let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();
  let owner = SubnetOwner::<Test>::get(subnet_id).unwrap();
  assert_eq!(owner, owner_coldkey.clone());

  let epoch_length = EpochLength::get();
  let epoch = System::block_number() / epoch_length;
  let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

  // --- Add subnet nodes
  let block_number = System::block_number();
  let mut amount_staked = 0;
  for n in start..end {
    let _n = n + 1;
    let coldkey = get_coldkey(subnets, max_subnet_nodes, _n);
    let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, _n);
    let peer_id = peer(subnets*max_subnet_nodes+_n);
    let bootnode_peer_id = peer(subnets*max_subnet_nodes+_n);

    let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);
    amount_staked += amount;
    assert_ok!(
      Network::add_subnet_node(
        RuntimeOrigin::signed(coldkey.clone()),
        subnet_id,
        hotkey.clone(),
        peer_id.clone(),
        bootnode_peer_id,
        None,
        0,
        amount,
        None,
        None,
      ) 
    );

    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
    // assert_eq!(hotkey_subnet_node_id, coldkey_n);

    let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_id_hotkey, hotkey.clone());

    let subnet_node_data = SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
    assert_eq!(subnet_node_data.hotkey, hotkey.clone());

    let key_owner = HotkeyOwner::<Test>::get(subnet_node_data.hotkey.clone());
    assert_eq!(key_owner, coldkey.clone());

    assert_eq!(subnet_node_data.peer_id, peer_id.clone());

    // --- Is ``Validator`` if registered before subnet activation
    assert_eq!(subnet_node_data.classification.node_class, SubnetNodeClass::Validator);
    assert!(subnet_node_data.has_classification(&SubnetNodeClass::Validator, subnet_epoch));

    let subnet_node_account = PeerIdSubnetNodeId::<Test>::get(subnet_id, peer_id.clone());
    assert_eq!(subnet_node_account, hotkey_subnet_node_id);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
    assert_eq!(account_subnet_stake, amount);
  }

  let total_subnet_stake = TotalSubnetStake::<Test>::get(subnet_id);
  assert_eq!(total_subnet_stake, amount_staked);

  let total_stake = TotalStake::<Test>::get();
  assert_eq!(total_subnet_stake, amount_staked);

  let delegate_staker_account = 1000;
  // Add 100e18 to account for block increase on activation
  let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance_v2(subnet_id) + 100e+18 as u128;
  
  let _ = Balances::deposit_creating(&account(delegate_staker_account), min_subnet_delegate_stake+500);
  // --- Add the minimum required delegate stake balance to activate the subnet
  assert_ok!(
    Network::add_to_delegate_stake(
      RuntimeOrigin::signed(account(delegate_staker_account)),
      subnet_id,
      min_subnet_delegate_stake,
    ) 
  );

  let total_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
  assert_eq!(total_delegate_stake_balance, min_subnet_delegate_stake);

  let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(delegate_staker_account), subnet_id);
  // 1000 is for inflation attack mitigation
  // assert_eq!(min_subnet_delegate_stake - 1000, delegate_shares);

  // --- Increase epochs to max registration epoch
  let epochs = SubnetRegistrationEpochs::<Test>::get();
  increase_epochs(epochs + 1);
  
  assert_ok!(
    Network::activate_subnet(
      RuntimeOrigin::signed(owner_coldkey.clone()),
      subnet_id,
    )
  );

  increase_epochs(2);

  assert_eq!(
    *network_events().last().unwrap(),
    Event::SubnetActivated {
      subnet_id: subnet_id, 
    }
  );
}

pub fn get_initial_coldkeys(subnets: u32, max_subnet_nodes: u32, start: u32, end: u32) -> BTreeSet<AccountId> {
  let mut whitelist = BTreeSet::new();
  for n in start..end {
    let _n = n + 1;
    whitelist.insert(account(subnets*max_subnet_nodes+_n));
  }
  whitelist
}

pub fn default_registration_subnet_data(
  subnets: u32,
  max_subnet_nodes: u32,
  name: Vec<u8>,
  start: u32, 
  end: u32
) -> RegistrationSubnetData<AccountId> {
  let seed_bytes: &[u8] = &name;
  let add_subnet_data = RegistrationSubnetData {
    name: name.clone(),
    repo: blake2_128(seed_bytes).to_vec(), // must be unique
    description: Vec::new(),
    misc: Vec::new(),
    churn_limit: 4,
    min_stake: 100e+18 as u128,
    max_stake: 10000e+18 as u128,
    delegate_stake_percentage: 100000000000000000, // 10%
    registration_queue_epochs: 4,
    activation_grace_epochs: 4,
    queue_classification_epochs: 4,
    included_classification_epochs: 4,
    max_node_penalties: 3,
    initial_coldkeys: get_initial_coldkeys(subnets, max_subnet_nodes, start, end),
    max_registered_nodes: 100,
    node_removal_system: NodeRemovalSystem::Consensus,
    key_types: BTreeSet::from([KeyType::Rsa]),
  };
  add_subnet_data
}

// Returns total staked on subnet
// pub fn build_subnet_nodes(subnet_id: u32, start: u32, end: u32, deposit_amount: u128, amount: u128) -> u128 {
//   let mut amount_staked = 0;
//   for n in start+1..end+1 {
//     let _ = Balances::deposit_creating(&account(n), deposit_amount);
//     amount_staked += amount;
//     assert_ok!(
//       Network::add_subnet_node(
//         RuntimeOrigin::signed(account(n)),
//         subnet_id,
//         account(n),
//         peer(n),
//         peer(n),
//         0,
//         amount,
//         None,
//         None,
//       ) 
//     );
//     post_successful_add_subnet_node_asserts(n, subnet_id, amount);
//   }
//   amount_staked
// }

pub fn post_subnet_removal_ensures(
  subnet_id: u32, 
  subnets: u32, // count
  max_subnet_nodes: u32,
  name: Vec<u8>, 
  start: u32, 
  end: u32
) {
  assert_eq!(SubnetsData::<Test>::try_get(subnet_id), Err(()));
  assert_eq!(SubnetName::<Test>::try_get(name), Err(()));
  // assert_eq!(LastSubnetRegistration::<Test>::try_get(subnet_id), Err(()));
  // assert_eq!(SubnetRegistrationEpoch::<Test>::try_get(subnet_id), Err(()));
  assert_eq!(SubnetRegistrationInitialColdkeys::<Test>::try_get(subnet_id), Err(()));
  assert_eq!(SubnetNodesData::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(TotalSubnetNodes::<Test>::contains_key(subnet_id), false);
  assert_eq!(TotalSubnetNodeUids::<Test>::contains_key(subnet_id), false);
  assert_eq!(PeerIdSubnetNodeId::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(BootstrapPeerIdSubnetNodeId::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(SubnetNodeUniqueParam::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(HotkeySubnetNodeId::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(SubnetNodeIdHotkey::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(SubnetPenaltyCount::<Test>::contains_key(subnet_id), false);
  assert_eq!(SubnetElectedValidator::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(SubnetConsensusSubmission::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(Proposals::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(SubnetNodeNonUniqueParamLastSet::<Test>::iter_prefix(subnet_id).count(), 0);
  assert_eq!(SubnetNodePenalties::<Test>::iter_prefix(subnet_id).count(), 0);

  for n in start..end {
    let _n = n + 1;
    assert_eq!(HotkeySubnetNodeId::<Test>::get(subnet_id, account(subnets*max_subnet_nodes+_n)), None);
    assert_eq!(PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer(subnets*max_subnet_nodes+_n)), Err(()));
  
    let stake_balance = AccountSubnetStake::<Test>::get(account(subnets*max_subnet_nodes+_n), subnet_id);
    assert_ok!(
      Network::remove_stake(
        RuntimeOrigin::signed(account(subnets*max_subnet_nodes+_n)),
        subnet_id,
        account(subnets*max_subnet_nodes+_n),
        stake_balance,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(subnets*max_subnet_nodes+_n), subnet_id);
    if delegate_shares != 0 {
      // increase epoch becuse must have only one unstaking per epoch
      increase_epochs(1);

      assert_ok!(
        Network::remove_delegate_stake(
          RuntimeOrigin::signed(account(subnets*max_subnet_nodes+_n)),
          subnet_id,
          delegate_shares,
        )
      );  
    }
  }

  let epoch_length = EpochLength::get();
  let stake_cooldown_epochs = StakeCooldownEpochs::get();

  let starting_block_number = System::block_number();

  

  // --- Ensure unstaking is stable
  for n in start..end {
    let _n = n + 1;
    System::set_block_number(System::block_number() + ((epoch_length  + 1) * stake_cooldown_epochs));
    let starting_balance = Balances::free_balance(&account(subnets*max_subnet_nodes+_n));
    let unbondings = StakeUnbondingLedger::<Test>::get(account(subnets*max_subnet_nodes+_n));
    // assert_eq!(unbondings.len(), 1);
    // let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    let ledger_balance: u128 = unbondings.values().copied().sum();
    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(subnets*max_subnet_nodes+_n)),
      )
    );
    let ending_balance = Balances::free_balance(&account(subnets*max_subnet_nodes+_n));
    assert_eq!(starting_balance + ledger_balance, ending_balance);
    System::set_block_number(starting_block_number);
  }
}

// pub fn build_for_submit_consensus_data(subnet_id: u32, start: u32, end: u32, start_data: u32, end_data: u32) {
//   let subnet_node_data_vec = get_subnet_node_consensus_data(start_data, end_data);

//   for n in start+1..end+1 {
//     assert_ok!(
//       Network::submit_consensus_data(
//         RuntimeOrigin::signed(account(n)),
//         subnet_id,
//         subnet_node_data_vec.clone(),
//       ) 
//     );
//   }
// }

pub fn increase_epochs(epochs: u32) {
  if epochs == 0 {
      return;
  }

  let block = System::block_number();
  let epoch_length = EpochLength::get();

  let advance_blocks = epoch_length.saturating_mul(epochs);
  let new_block = block.saturating_add(advance_blocks);

  System::set_block_number(new_block);
}

// pub fn increase_subnet_epochs(epochs: u32, subnet_id: u32) {
//   if epochs == 0 {
//       return;
//   }

//   let block = System::block_number();
//   let epoch_length = EpochLength::get();

//   let advance_blocks = epoch_length.saturating_mul(epochs);
//   let new_block = block.saturating_add(advance_blocks);

//   System::set_block_number(new_block);
// }

pub fn set_epoch(epoch: u32) {
  let epoch_length = EpochLength::get();
  System::set_block_number(epoch * epoch_length);
}

pub fn get_epoch() -> u32 {
  let current_block = System::block_number();
  let epoch_length: u32 = EpochLength::get();
  current_block.saturating_div(epoch_length)
}

pub fn set_block_to_subnet_slot_epoch(epoch: u32, subnet_id: u32) {
  let epoch_length = EpochLength::get();
  let slot = SubnetSlot::<Test>::get(subnet_id)
      .expect("SubnetSlot must be assigned before setting block");
  let block = slot + epoch * epoch_length;

  System::set_block_number(block);
}

// pub fn get_subnet_node_consensus_data(
//   subnets: u32,
//   max_subnet_nodes: u32,
//   start: u32, 
//   end: u32
// ) -> Vec<SubnetNodeConsensusData> {
//   // initialize peer consensus data array
//   let mut subnet_node_data: Vec<SubnetNodeConsensusData> = Vec::new();
//   for n in start+1..end+1 {
//     let peer_subnet_node_data: SubnetNodeConsensusData = SubnetNodeConsensusData {
//       peer_id: peer(subnets*max_subnet_nodes+n),
//       score: DEFAULT_SCORE,
//     };

//     subnet_node_data.push(peer_subnet_node_data);
//   }
//   subnet_node_data
// }

pub fn get_subnet_node_consensus_data(
  subnets: u32,
  max_subnet_nodes: u32,
  start: u32, 
  end: u32
) -> Vec<SubnetNodeConsensusData> {
  // initialize peer consensus data array
  let mut subnet_node_data: Vec<SubnetNodeConsensusData> = Vec::new();
  for n in start..end {
    let peer_subnet_node_data: SubnetNodeConsensusData = SubnetNodeConsensusData {
      subnet_node_id: n+1,
      score: DEFAULT_SCORE,
    };

    subnet_node_data.push(peer_subnet_node_data);
  }
  subnet_node_data
}

pub fn get_simulated_consensus_data(
	subnet_id: u32,
	node_count: u32,
) -> ConsensusData<<Test as frame_system::Config>::AccountId> { 
	let mut attests = BTreeMap::new();
	let mut data = Vec::new();

	let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

	let block_number = Network::get_current_block_as_u32();
	let epoch_length = EpochLength::get();
	let epoch = Network::get_current_block_as_u32() / epoch_length;

	for n in 0..node_count {
		// let node_id = subnet_id*max_subnet_nodes+n+1;
		let node_id = subnet_id*max_subnet_nodes-max_subnet_nodes+n+1;

		// Simulate some score and block number
		let score = 1e+18 as u128;

		attests.insert(node_id, block_number);
		data.push(SubnetNodeConsensusData {
			subnet_node_id: node_id,
			score,
		});
	}

	let included_subnet_nodes: Vec<SubnetNode<<Test as frame_system::Config>::AccountId>> = Network::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch);

	ConsensusData {
		validator_id: subnet_id*max_subnet_nodes,
		attests,
		data,
		subnet_nodes: included_subnet_nodes,
		args: None
	}
}

// pub fn subnet_node_data_invalid_scores(start: u32, end: u32) -> Vec<SubnetNodeConsensusData> {
//   // initialize peer consensus data array
//   // let mut subnet_node_data: Vec<SubnetNodeConsensusData<<Test as frame_system::Config>::AccountId>> = Vec::new();
//   let mut subnet_node_data: Vec<SubnetNodeConsensusData> = Vec::new();
//   for n in start+1..end+1 {
//     // let peer_subnet_node_data: SubnetNodeConsensusData<<Test as frame_system::Config>::AccountId> = SubnetNodeConsensusData {
//     //   // account_id: account(n),
//     //   peer_id: peer(n),
//     //   score: 10000000000,
//     // };
//     let peer_subnet_node_data: SubnetNodeConsensusData = SubnetNodeConsensusData {
//       // peer_id: peer(n),
//       subnet_node_id: subnets*max_subnet_nodes+n,
//       score: 10000000000,
//     };
//     subnet_node_data.push(peer_subnet_node_data);
//   }
//   subnet_node_data
// }

pub fn post_successful_add_subnet_node_asserts(
  n: u32, 
  subnet_id: u32, 
  amount: u128
) {
  assert_eq!(Network::account_subnet_stake(account(n), subnet_id), amount);
  // assert_eq!(Network::total_account_stake(account(n)), amount);    
  assert_eq!(Network::total_subnet_nodes(subnet_id), (n + 1) as u32);
}

// check data after adding multiple peers
// each peer must have equal staking amount per subnet
pub fn post_successful_add_subnet_nodes_asserts(
  total_peers: u32,
  stake_per_peer: u128,  
  subnet_id: u32, 
) {
  let amount_staked = total_peers as u128 * stake_per_peer;
  assert_eq!(Network::total_subnet_stake(subnet_id), amount_staked);
}

pub fn post_remove_subnet_node_ensures(n: u32, subnet_id: u32) {
  // ensure SubnetNodesData removed
  let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, account(n));
  assert_eq!(subnet_node_id, Err(()));

  assert_eq!(SubnetNodesData::<Test>::iter_prefix(subnet_id).count(), 0);
  // assert_eq!(subnet_node_hotkey, Err(()));

  // ensure PeerIdSubnetNodeId removed
  let subnet_node_account = PeerIdSubnetNodeId::<Test>::try_get(subnet_id, peer(n));
  assert_eq!(subnet_node_account, Err(()));
}

pub fn post_remove_unstake_ensures(n: u32, subnet_id: u32) {
}

pub fn add_subnet_node(
  account_id: u32, 
  subnet_id: u32,
  peer_id: u32,
  ip: String,
  port: u16,
  amount: u128
) -> Result<(), sp_runtime::DispatchError> {
  Network::add_subnet_node(
    RuntimeOrigin::signed(account(account_id)),
    subnet_id,
    account(account_id),
    peer(peer_id),
    peer(peer_id),
    None,
    0,
    amount,
    None,
    None,
  )
}

pub fn to_bounded<Len: frame_support::traits::Get<u32>>(s: &str) -> BoundedVec<u8, Len> {
  BoundedVec::try_from(s.as_bytes().to_vec()).expect("String too long")
}

pub fn insert_overwatch_node(coldkey_n: u32, hotkey_n: u32) -> u32 {
  let coldkey = account(coldkey_n);
  let hotkey = account(hotkey_n);

  TotalOverwatchNodeUids::<Test>::mutate(|n: &mut u32| *n += 1);
  let current_uid = TotalOverwatchNodeUids::<Test>::get();

  let overwatch_node = OverwatchNode {
    id: current_uid,
    hotkey: hotkey.clone(),
  };

  OverwatchNodes::<Test>::insert(current_uid, overwatch_node);
  HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
  OverwatchNodeIdHotkey::<Test>::insert(current_uid, hotkey.clone());

  let mut hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
  hotkeys.insert(hotkey.clone());
  ColdkeyHotkeys::<Test>::insert(&coldkey.clone(), hotkeys);

  HotkeyOverwatchNodeId::<Test>::insert(&hotkey.clone(), current_uid);

  current_uid
}

pub fn set_stake(account_id: u32, amount: u128) {
    // -- increase account staking balance
  AccountOverwatchStake::<Test>::mutate(account(account_id), |mut n| *n += amount);
  // -- increase total stake
  TotalOverwatchStake::<Test>::mutate(|mut n| *n += amount);
}

pub fn submit_weight(
  epoch: u32,
  subnet_id: u32,
  node_id: u32,
  weight: u128
) {
  OverwatchReveals::<Test>::insert((epoch, subnet_id, node_id), weight);
}

pub fn new_subnet_data(id: u32, state: SubnetState, start_epoch: u32) -> SubnetData {
  SubnetData {
    id,
    name: vec![],
    repo: vec![],
    description: vec![],
    misc: vec![],
    state,
    start_epoch,
  }
}

// Helper to set up a subnet in storage
pub fn insert_subnet(id: u32, state: SubnetState, start_epoch: u32) {
  let data = new_subnet_data(id, state, start_epoch);
  SubnetsData::<Test>::insert(id, data);
}

// Helper to set registration epoch
pub fn set_registration_epoch(id: u32, epoch: u32) {
  SubnetRegistrationEpoch::<Test>::insert(id, epoch);
}

// Helper to set active nodes count
pub fn set_active_nodes(id: u32, count: u32) {
  TotalActiveSubnetNodes::<Test>::insert(id, count);
}

// Helper to set delegate stake balance
pub fn set_delegate_stake(id: u32, stake: u128) {
  TotalSubnetDelegateStakeBalance::<Test>::insert(id, stake);
}

// Helper to set penalties count
pub fn set_penalties(id: u32, count: u32) {
  SubnetPenaltyCount::<Test>::insert(id, count);
}

pub fn run_subnet_consensus_step(subnet_id: u32) {
  let max_subnets = MaxSubnets::<Test>::get();
  let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

  let block_number = Network::get_current_block_as_u32();
  let epoch = Network::get_current_epoch_as_u32();
  
  // set_block_to_subnet_slot_epoch(epoch, subnet_id);

  let subnet_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

  // Network::elect_validator_v3(
  //   subnet_id,
  //   subnet_epoch,
  //   block_number
  // );

  let validator_id = SubnetElectedValidator::<Test>::get(subnet_id, subnet_epoch);
  assert!(validator_id != None, "Validator is None");
  assert!(validator_id != Some(0), "Validator is 0");

  let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

  let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

  let subnet_node_data_vec = get_subnet_node_consensus_data(subnet_id, max_subnet_nodes, 0, total_subnet_nodes);

  assert_ok!(
    Network::validate(
      RuntimeOrigin::signed(validator.clone()), 
      subnet_id,
      subnet_node_data_vec.clone(),
      None,
    )
  );

  for n in 0..total_subnet_nodes {
    let _n = n + 1;
    let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, _n);
    if hotkey.clone() == validator.clone() {
      continue
    }
    assert_ok!(
      Network::attest(
        RuntimeOrigin::signed(hotkey.clone()), 
        subnet_id,
      )
    );
  }

  let submission = SubnetConsensusSubmission::<Test>::get(subnet_id, subnet_epoch).unwrap();
  assert_eq!(submission.attests.len(), total_subnet_nodes as usize);
  
  for n in 0..total_subnet_nodes {
    let _n = n + 1;
    let hotkey = get_hotkey(subnet_id, max_subnet_nodes, max_subnets, _n);
    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
    if hotkey == validator.clone() {
      assert_ne!(submission.attests.get(&(subnet_node_id)), None);
      assert_eq!(submission.attests.get(&(subnet_node_id)), Some(&System::block_number()));
    } else {
      assert_ne!(submission.attests.get(&(subnet_node_id)), None);
      assert_eq!(submission.attests.get(&(subnet_node_id)), Some(&System::block_number()));
    }
  }
}