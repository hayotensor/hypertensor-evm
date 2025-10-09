use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use crate::{
    AccountOverwatchStake, AccountSubnetStake, ColdkeyHotkeys, ColdkeyIdentity,
    ColdkeyIdentityNameOwner, ColdkeyReputation, DefaultMaxSocialIdLength, DefaultMaxUrlLength,
    DefaultMaxVectorLength, Error, HotkeyOverwatchNodeId, HotkeyOwner, HotkeySubnetId,
    HotkeySubnetNodeId, MaxSubnetNodes, MaxSubnets, MinActiveNodeStakeEpochs, MinSubnetMinStake,
    OverwatchMinStakeBalance, OverwatchNodeIdHotkey, OverwatchNodes, StakeUnbondingLedger,
    SubnetName, SubnetNodeIdHotkey, SubnetNodesData, TotalActiveSubnets, TotalSubnetNodes,
};
use frame_support::traits::Currency;
use frame_support::{assert_err, assert_ok};
use sp_std::collections::btree_map::BTreeMap;

#[test]
fn test_update_coldkey() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();

        let deposit_amount: u128 = 10000000000000000000000;
        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        let end = 16;

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

        let new_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let new_coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let new_coldkey_2 = get_coldkey(subnets, max_subnet_nodes, end + 2);
        let fake_coldkey = get_coldkey(subnets, max_subnet_nodes, end + 3);

        // Insert overwatch node with coldkey
        let overwatch_node_id = insert_overwatch_node(max_subnet_nodes + end * subnets, 0);
        set_overwatch_stake(
            max_subnet_nodes + end * subnets,
            OverwatchMinStakeBalance::<Test>::get(),
        );

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
        let starting_account_subnet_stake =
            AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

        // add extra stake and then add to ledger to check if it swapped
        let add_stake_amount = 10e+18 as u128;
        let remove_stake_amount = 10e+18 as u128;
        let _ = Balances::deposit_creating(&coldkey.clone(), deposit_amount);

        // Insert identity

        let name = to_bounded::<DefaultMaxVectorLength>("name");
        let url = to_bounded::<DefaultMaxUrlLength>("url");
        let image = to_bounded::<DefaultMaxUrlLength>("image");
        let discord = to_bounded::<DefaultMaxSocialIdLength>("discord");
        let x = to_bounded::<DefaultMaxSocialIdLength>("x");
        let telegram = to_bounded::<DefaultMaxSocialIdLength>("telegram");
        let github = to_bounded::<DefaultMaxUrlLength>("github");
        let hugging_face = to_bounded::<DefaultMaxUrlLength>("hugging_face");
        let description = to_bounded::<DefaultMaxVectorLength>("description");
        let misc = to_bounded::<DefaultMaxVectorLength>("misc");
        assert_ok!(Network::register_or_update_identity(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            name.clone(),
            url.clone(),
            image.clone(),
            discord.clone(),
            x.clone(),
            telegram.clone(),
            github.clone(),
            hugging_face.clone(),
            description.clone(),
            misc.clone(),
        ));

        let starting_rep = ColdkeyReputation::<Test>::get(coldkey.clone());

        //
        //
        // Coldkey = same
        // Hotkey  = same
        //
        //

        assert_ok!(Network::add_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey_subnet_node_id,
            hotkey.clone(),
            add_stake_amount,
        ));

        let stake_balance = AccountSubnetStake::<Test>::get(&hotkey.clone(), subnet_id);
        assert_eq!(
            stake_balance,
            starting_account_subnet_stake + add_stake_amount
        );

        let min_stake_epochs = MinActiveNodeStakeEpochs::<Test>::get();

        increase_epochs(min_stake_epochs + 1);

        assert_ok!(Network::remove_stake(
            RuntimeOrigin::signed(coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            remove_stake_amount,
        ));

        let original_unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(coldkey.clone());
        let original_ledger_balance: u128 = original_unbondings.values().copied().sum();
        assert_eq!(original_unbondings.len() as u32, 1);
        assert_eq!(original_ledger_balance, remove_stake_amount);

        // Update the coldkey to unused key
        //
        //
        // Coldkey = coldkey
        // Hotkey  = hotkey
        //
        //

        // Updating coldkey to new_coldkey
        assert_ok!(Network::update_coldkey(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            new_coldkey.clone(), // new_coldkey
        ));

        assert_eq!(
            *network_events().last().unwrap(),
            Event::UpdateColdkey {
                coldkey: coldkey.clone(),
                new_coldkey: new_coldkey.clone(),
            }
        );

        //
        // Check old coldkey
        //

        // check old coldkey balance is now removed because it was swapped to the new one
        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(coldkey.clone());
        let ledger_balance: u128 = unbondings.values().copied().sum();
        assert_eq!(unbondings.len() as u32, 0);
        assert_eq!(ledger_balance, 0);

        // old coldkey shouldn't have the hotkeys any longer
        let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
        assert_eq!(hotkeys.len(), 0);

        let coldkey_identity = ColdkeyIdentity::<Test>::try_get(coldkey.clone());
        assert_eq!(coldkey_identity, Err(()));

        let rep = ColdkeyReputation::<Test>::try_get(coldkey.clone());
        assert_eq!(rep, Err(()));

        //
        // Check new coldkey
        //
        let new_unbondings: BTreeMap<u32, u128> =
            StakeUnbondingLedger::<Test>::get(new_coldkey.clone());
        let new_ledger_balance: u128 = new_unbondings.values().copied().sum();
        assert_eq!(
            new_unbondings.len() as u32,
            original_unbondings.len() as u32
        );
        assert_eq!(new_ledger_balance, original_ledger_balance);

        let subnet_node_id_hotkey =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_id_hotkey, hotkey.clone());

        let subnet_node_data =
            SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_data.hotkey, hotkey.clone());

        let key_owner = HotkeyOwner::<Test>::get(hotkey.clone());
        assert_eq!(key_owner, new_coldkey.clone());

        // Simple check the overwatch hotkey is the same
        let ow_node_id = HotkeyOverwatchNodeId::<Test>::get(&account(0));
        assert_eq!(overwatch_node_id, ow_node_id.unwrap());

        let hotkeys = ColdkeyHotkeys::<Test>::get(&new_coldkey.clone());
        assert_eq!(hotkeys.len(), 2); // 1 subnet node, 1 overwatch node

        let coldkey_identity = ColdkeyIdentity::<Test>::get(new_coldkey.clone());
        assert_eq!(coldkey_identity.name, name);
        assert_eq!(coldkey_identity.url, url);
        assert_eq!(coldkey_identity.image, image);
        assert_eq!(coldkey_identity.discord, discord);
        assert_eq!(coldkey_identity.x, x);
        assert_eq!(coldkey_identity.telegram, telegram);
        assert_eq!(coldkey_identity.github, github);
        assert_eq!(coldkey_identity.hugging_face, hugging_face);
        assert_eq!(coldkey_identity.description, description);
        assert_eq!(coldkey_identity.misc, misc);

        let coldkey_identity_name_owner = ColdkeyIdentityNameOwner::<Test>::get(name.clone());
        assert_eq!(coldkey_identity_name_owner, new_coldkey);

        let rep = ColdkeyReputation::<Test>::get(new_coldkey.clone());
        assert_eq!(rep, starting_rep);

        //
        // Coldkey is updated, shouldn't be able to make changes anywhere using coldkey
        //

        let add_stake_amount: u128 = 1000000000000000000000;
        let _ = Balances::deposit_creating(&coldkey.clone(), add_stake_amount);

        assert_err!(
            Network::add_stake(
                RuntimeOrigin::signed(coldkey.clone()),
                subnet_id,
                hotkey_subnet_node_id,
                hotkey.clone(),
                add_stake_amount,
            ),
            Error::<Test>::NotKeyOwner,
        );

        assert_err!(
            Network::remove_stake(
                RuntimeOrigin::signed(coldkey.clone()),
                subnet_id,
                hotkey.clone(),
                1000,
            ),
            Error::<Test>::NotKeyOwner
        );

        // // `do_pause_subnet_node` allows both hotkey and coldkey
        // // old_coldkey shouldn't work
        // assert_err!(
        //     Network::do_pause_subnet_node(
        //         RuntimeOrigin::signed(fake_coldkey.clone()),
        //         subnet_id,
        //         hotkey_subnet_node_id
        //     ),
        //     Error::<Test>::NotKeyOwner
        // );

        assert_err!(
            Network::update_coldkey(
                RuntimeOrigin::signed(coldkey.clone()),
                hotkey.clone(),
                new_coldkey.clone(), // new_coldkey
                                     // None,
            ),
            Error::<Test>::NotKeyOwner
        );

        // new hotkey is 2
        assert_err!(
            Network::update_hotkey(
                RuntimeOrigin::signed(fake_coldkey.clone()),
                hotkey.clone(),
                account(1), // use non registered hotkey to get correct error
            ),
            Error::<Test>::NotKeyOwner
        );

        // Use new coldkey
        let add_stake_amount: u128 = 10e+18 as u128;
        let _ = Balances::deposit_creating(&new_coldkey.clone(), add_stake_amount + 500);

        // add stake with new_coldkey
        assert_ok!(Network::add_stake(
            RuntimeOrigin::signed(new_coldkey.clone()),
            subnet_id,
            hotkey_subnet_node_id,
            hotkey.clone(),
            add_stake_amount,
        ));

        // remove stake with new_coldkey
        assert_ok!(Network::remove_stake(
            RuntimeOrigin::signed(new_coldkey.clone()),
            subnet_id,
            hotkey.clone(),
            add_stake_amount,
        ));

        // // `do_pause_subnet_node` allows both hotkey and coldkey
        // assert_ok!(Network::do_pause_subnet_node(
        //     RuntimeOrigin::signed(new_coldkey.clone()),
        //     subnet_id,
        //     hotkey_subnet_node_id
        // ));

        assert_ok!(Network::update_hotkey(
            RuntimeOrigin::signed(new_coldkey.clone()),
            hotkey.clone(),     // old_hotkey
            new_hotkey.clone(), // new hotkey
        ));

        assert_ok!(Network::update_coldkey(
            RuntimeOrigin::signed(new_coldkey.clone()),
            new_hotkey.clone(),
            new_coldkey_2.clone(), // new_coldkey
        ));

        assert_err!(
            Network::update_coldkey(
                RuntimeOrigin::signed(new_coldkey.clone()),
                new_hotkey.clone(),
                new_coldkey_2.clone(), // new_coldkey
            ),
            Error::<Test>::NotKeyOwner
        );
    })
}

#[test]
fn test_update_overwatch_coldkey() {
    new_test_ext().execute_with(|| {})
}

#[test]
fn test_update_coldkey_key_taken_err() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();
        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();

        let end = 16;

        let account_n = max_subnet_nodes + end * subnets;

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        assert_err!(
            Network::update_coldkey(
                RuntimeOrigin::signed(account(account_n)),
                account(2),
                account(account_n),
                // None,
            ),
            Error::<Test>::NotKeyOwner
        );
    });
}

#[test]
fn test_update_hotkey() {
    new_test_ext().execute_with(|| {
        let subnet_name: Vec<u8> = "subnet-name".into();

        let deposit_amount: u128 = 10000000000000000000000;
        let amount: u128 = 1000000000000000000000;

        let stake_amount: u128 = MinSubnetMinStake::<Test>::get();

        let subnets = TotalActiveSubnets::<Test>::get() + 1;
        let max_subnet_nodes = MaxSubnetNodes::<Test>::get();
        let max_subnets = MaxSubnets::<Test>::get();

        let end = 16;

        let coldkey = get_coldkey(subnets, max_subnet_nodes, end);
        let hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end);

        let new_hotkey = get_hotkey(subnets, max_subnet_nodes, max_subnets, end + 1);
        let new_coldkey = get_coldkey(subnets, max_subnet_nodes, end + 1);
        let new_coldkey_2 = get_coldkey(subnets, max_subnet_nodes, end + 2);
        let fake_coldkey = get_coldkey(subnets, max_subnet_nodes, end + 3);

        // Insert overwatch node with coldkey
        let ow_hotkey_n = 1;
        let ow_hotkey = account(ow_hotkey_n);
        let overwatch_node_id =
            insert_overwatch_node(max_subnet_nodes + end * subnets, ow_hotkey_n);

        set_overwatch_stake(ow_hotkey_n, OverwatchMinStakeBalance::<Test>::get());

        build_activated_subnet_new(subnet_name.clone(), 0, end, deposit_amount, stake_amount);

        let name = to_bounded::<DefaultMaxVectorLength>("name");
        let url = to_bounded::<DefaultMaxUrlLength>("url");
        let image = to_bounded::<DefaultMaxUrlLength>("image");
        let discord = to_bounded::<DefaultMaxSocialIdLength>("discord");
        let x = to_bounded::<DefaultMaxSocialIdLength>("x");
        let telegram = to_bounded::<DefaultMaxSocialIdLength>("telegram");
        let github = to_bounded::<DefaultMaxUrlLength>("github");
        let hugging_face = to_bounded::<DefaultMaxUrlLength>("hugging_face");
        let description = to_bounded::<DefaultMaxVectorLength>("description");
        let misc = to_bounded::<DefaultMaxVectorLength>("misc");
        assert_ok!(Network::register_or_update_identity(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            name.clone(),
            url.clone(),
            image.clone(),
            discord.clone(),
            x.clone(),
            telegram.clone(),
            github.clone(),
            hugging_face.clone(),
            description.clone(),
            misc.clone(),
        ));

        let subnet_id = SubnetName::<Test>::get(subnet_name.clone()).unwrap();
        let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

        let hotkey_subnet_node_id =
            HotkeySubnetNodeId::<Test>::get(subnet_id, hotkey.clone()).unwrap();
        let starting_account_subnet_stake =
            AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);

        assert_ok!(Network::update_hotkey(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            new_hotkey.clone(),
        ));

        assert_eq!(
            *network_events().last().unwrap(),
            Event::UpdateHotkey {
                hotkey: hotkey.clone(),
                new_hotkey: new_hotkey.clone(),
            }
        );

        //
        // Old hotkey
        //
        let hotkeys = ColdkeyHotkeys::<Test>::get(&coldkey.clone());
        assert_eq!(hotkeys.contains(&hotkey.clone()), false);

        let account_subnet_stake = AccountSubnetStake::<Test>::get(hotkey.clone(), subnet_id);
        assert_eq!(account_subnet_stake, 0);

        let hotkey_subnet_id = HotkeySubnetId::<Test>::get(&hotkey.clone());
        assert_eq!(hotkey_subnet_id, None);

        assert_eq!(HotkeyOwner::<Test>::try_get(hotkey.clone()), Err(()));

        //
        // New hotkey
        //
        assert_eq!(hotkeys.contains(&new_hotkey.clone()), true);

        let subnet_node_id_hotkey =
            SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_id_hotkey, new_hotkey.clone());

        let subnet_node_data =
            SubnetNodesData::<Test>::try_get(subnet_id, hotkey_subnet_node_id).unwrap();
        assert_eq!(subnet_node_data.hotkey, new_hotkey.clone());

        let key_owner = HotkeyOwner::<Test>::get(new_hotkey.clone());
        assert_eq!(key_owner, coldkey.clone());

        let account_subnet_stake = AccountSubnetStake::<Test>::get(new_hotkey.clone(), subnet_id);
        assert_eq!(account_subnet_stake, starting_account_subnet_stake);

        let hotkey_subnet_id = HotkeySubnetId::<Test>::get(&new_hotkey.clone());
        assert_eq!(hotkey_subnet_id, Some(subnet_id));

        //
        // Update overwatch node hotkey
        //
        let ow_new_hotkey = account(ow_hotkey_n + 1);

        let starting_account_overwatch_stake =
            AccountOverwatchStake::<Test>::get(ow_hotkey.clone());
        assert!(starting_account_overwatch_stake > 0);

        assert_ok!(Network::update_hotkey(
            RuntimeOrigin::signed(coldkey.clone()),
            ow_hotkey.clone(),
            ow_new_hotkey.clone(),
        ));

        //
        // Old ow hotkey should be removed
        //
        assert_eq!(AccountOverwatchStake::<Test>::get(ow_hotkey.clone()), 0);

        let ow_node_id = HotkeyOverwatchNodeId::<Test>::try_get(&ow_hotkey.clone());
        assert_eq!(ow_node_id, Err(()));

        //
        // New ow node ID updated to new hotkey
        //
        let account_overwatch_stake = AccountOverwatchStake::<Test>::get(ow_new_hotkey.clone());
        assert_eq!(account_overwatch_stake, starting_account_overwatch_stake);

        let ow_node_id = HotkeyOverwatchNodeId::<Test>::get(&ow_new_hotkey.clone());
        assert_eq!(ow_node_id, Some(overwatch_node_id));

        let overwatch_node_hotkey = OverwatchNodeIdHotkey::<Test>::get(overwatch_node_id);
        assert_eq!(overwatch_node_hotkey, Some(ow_new_hotkey.clone()));

        let overwatch_node = OverwatchNodes::<Test>::get(overwatch_node_id);
        assert_eq!(overwatch_node.unwrap().hotkey, ow_new_hotkey.clone());
    })
}
