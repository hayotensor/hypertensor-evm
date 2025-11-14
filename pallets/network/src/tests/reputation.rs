use super::mock::*;
use crate::tests::test_utils::*;
use crate::{ColdkeyReputation, Reputation};

#[test]
fn test_increase_coldkey_reputation_with_weight_factor() {
    new_test_ext().execute_with(|| {
        let coldkey: AccountId = account(1);
        let epoch = 1;
        let min_attestation = 660_000_000_000_000_000u128; // 66%
        let attestation = 900_000_000_000_000_000u128; // 90%
        let weight_factor = 500_000_000_000_000_000u128; // 0.5

        // Set initial reputation
        ColdkeyReputation::<Test>::insert(
            &coldkey,
            Reputation {
                start_epoch: 0,
                score: 500_000_000_000_000_000,
                lifetime_node_count: 0,
                total_active_nodes: 0,
                total_increases: 0,
                total_decreases: 0,
                average_attestation: 0,
                last_validator_epoch: 0,
                ow_score: 500_000_000_000_000_000,
            },
        );

        Network::increase_coldkey_reputation(
            coldkey.clone(),
            attestation,
            min_attestation,
            weight_factor,
            epoch,
        );

        let rep = ColdkeyReputation::<Test>::get(&coldkey);

        assert_eq!(rep.total_increases, 1);
        assert_eq!(rep.last_validator_epoch, epoch);
        assert_eq!(rep.average_attestation, attestation);
        assert!(rep.score > 500_000_000_000_000_000); // score increased
    });
}

#[test]
fn test_average_attestation_over_multiple_increases() {
    new_test_ext().execute_with(|| {
        let coldkey: AccountId = account(1);
        let min_attestation = 660_000_000_000_000_000u128;
        let weight_factor = 500_000_000_000_000_000u128; // 0.5
        let perc = Network::percentage_factor_as_u128(); // 1e18

        // Step 1: insert initial rep
        ColdkeyReputation::<Test>::insert(
            &coldkey,
            Reputation {
                start_epoch: 0,
                score: 500_000_000_000_000_000,
                lifetime_node_count: 0,
                total_active_nodes: 0,
                total_increases: 0,
                total_decreases: 0,
                average_attestation: 0,
                last_validator_epoch: 0,
                ow_score: 500_000_000_000_000_000,
            },
        );

        // Step 1: 90%
        let att1 = 900_000_000_000_000_000u128;
        Network::increase_coldkey_reputation(
            coldkey.clone(),
            att1,
            min_attestation,
            weight_factor,
            1,
        );
        let rep1 = ColdkeyReputation::<Test>::get(coldkey.clone());
        assert_eq!(rep1.average_attestation, att1);
        assert_eq!(rep1.total_increases, 1);

        // Step 2: 70%
        let att2 = 700_000_000_000_000_000u128;
        Network::increase_coldkey_reputation(
            coldkey.clone(),
            att2,
            min_attestation,
            weight_factor,
            2,
        );
        let rep2 = ColdkeyReputation::<Test>::get(coldkey.clone());
        let expected_avg2 = (att1 + att2) / 2;
        assert_eq!(rep2.average_attestation, expected_avg2);
        assert_eq!(rep2.total_increases, 2);

        // Step 3: 100%
        let att3 = 1_000_000_000_000_000_000u128;
        Network::increase_coldkey_reputation(
            coldkey.clone(),
            att3,
            min_attestation,
            weight_factor,
            3,
        );
        let rep3 = ColdkeyReputation::<Test>::get(coldkey.clone());
        let expected_avg3 = (expected_avg2 * 2 + att3) / 3;
        assert_eq!(rep3.average_attestation, expected_avg3);
        assert_eq!(rep3.total_increases, 3);

        // Step 4: 80%
        let att4 = 800_000_000_000_000_000u128;
        Network::increase_coldkey_reputation(
            coldkey.clone(),
            att4,
            min_attestation,
            weight_factor,
            4,
        );
        let rep4 = ColdkeyReputation::<Test>::get(coldkey.clone());
        let expected_avg4 = (expected_avg3 * 3 + att4) / 4;
        assert_eq!(rep4.average_attestation, expected_avg4);
        assert_eq!(rep4.total_increases, 4);
    });
}

#[test]
fn test_single_decrease_updates_average_and_weight() {
    new_test_ext().execute_with(|| {
        let coldkey: AccountId = account(1);
        let min_attestation = 660_000_000_000_000_000u128;
        let attestation = 500_000_000_000_000_000u128; // 50%
        let weight_factor = 500_000_000_000_000_000u128; // 0.5
        let start_score = 800_000_000_000_000_000u128;

        ColdkeyReputation::<Test>::insert(
            &coldkey,
            Reputation {
                start_epoch: 0,
                score: start_score,
                lifetime_node_count: 0,
                total_active_nodes: 0,
                total_increases: 0,
                total_decreases: 0,
                average_attestation: 0,
                last_validator_epoch: 0,
                ow_score: 500_000_000_000_000_000,
            },
        );

        Network::decrease_coldkey_reputation(
            coldkey.clone(),
            attestation,
            min_attestation,
            weight_factor,
            1,
        );

        let rep = ColdkeyReputation::<Test>::get(&coldkey);
        assert_eq!(rep.total_decreases, 1);
        assert_eq!(rep.average_attestation, attestation);
        assert!(rep.score < start_score);
        assert_eq!(rep.last_validator_epoch, 1);
    });
}

#[test]
fn test_average_attestation_over_multiple_decreases() {
    new_test_ext().execute_with(|| {
        let coldkey: AccountId = account(1);
        let min_attestation = 660_000_000_000_000_000u128;
        let weight_factor = 500_000_000_000_000_000u128; // 0.5
        let start_score = 900_000_000_000_000_000u128;

        // Initial insert
        ColdkeyReputation::<Test>::insert(
            &coldkey,
            Reputation {
                start_epoch: 0,
                score: start_score,
                lifetime_node_count: 0,
                total_active_nodes: 0,
                total_increases: 0,
                total_decreases: 0,
                average_attestation: 0,
                last_validator_epoch: 0,
                ow_score: 500_000_000_000_000_000,
            },
        );

        // Step 1: 50%
        let att1 = 500_000_000_000_000_000u128;
        Network::decrease_coldkey_reputation(
            coldkey.clone(),
            att1,
            min_attestation,
            weight_factor,
            1,
        );
        let rep1 = ColdkeyReputation::<Test>::get(coldkey.clone());
        assert_eq!(rep1.average_attestation, att1);

        // Step 2: 40%
        let att2 = 400_000_000_000_000_000u128;
        Network::decrease_coldkey_reputation(
            coldkey.clone(),
            att2,
            min_attestation,
            weight_factor,
            2,
        );
        let rep2 = ColdkeyReputation::<Test>::get(coldkey.clone());
        let expected_avg2 = (att1 + att2) / 2;
        assert_eq!(rep2.average_attestation, expected_avg2);

        // Step 3: 60%
        let att3 = 600_000_000_000_000_000u128;
        Network::decrease_coldkey_reputation(
            coldkey.clone(),
            att3,
            min_attestation,
            weight_factor,
            3,
        );
        let rep3 = ColdkeyReputation::<Test>::get(coldkey.clone());
        let expected_avg3 = (expected_avg2 * 2 + att3) / 3;
        assert_eq!(rep3.average_attestation, expected_avg3);

        // Confirm all other reputation fields are tracking
        assert_eq!(rep3.total_decreases, 3);
        assert_eq!(rep3.last_validator_epoch, 3);
        assert!(rep3.score < start_score); // score has gone down over 3 decreases
    });
}

#[test]
fn test_increase_node_reputation_basic() {
    new_test_ext().execute_with(|| {
        let new = Network::get_increase_reputation(500000000000000000, 100000000000000000);
        assert_eq!(new, 550000000000000000);

        let new = Network::get_increase_reputation(900000000000000000, 500000000000000000);
        assert_eq!(new, 950000000000000000);

        let new = Network::get_increase_reputation(
            Network::percentage_factor_as_u128(),
            500000000000000000,
        );
        assert_eq!(new, Network::percentage_factor_as_u128());

        let new = Network::get_increase_reputation(0, Network::percentage_factor_as_u128());
        assert_eq!(new, Network::percentage_factor_as_u128());
    });
}

#[test]
fn test_decrease_node_reputation_basic() {
    new_test_ext().execute_with(|| {
        let new = Network::get_decrease_reputation(500000000000000000, 100000000000000000);
        assert_eq!(new, 450000000000000000);

        let new = Network::get_decrease_reputation(900000000000000000, 500000000000000000);
        assert_eq!(new, 450000000000000000);

        let new = Network::get_decrease_reputation(
            Network::percentage_factor_as_u128(),
            Network::percentage_factor_as_u128(),
        );
        assert_eq!(new, 0);

        let new = Network::get_decrease_reputation(0, 800000000000000000);
        assert_eq!(new, 0);
    });
}

#[test]
fn test_reputation_bounds() {
    new_test_ext().execute_with(|| {
        let new = Network::get_increase_reputation(
            Network::percentage_factor_as_u128() - 1,
            Network::percentage_factor_as_u128(),
        );
        assert_eq!(new, Network::percentage_factor_as_u128());

        let new = Network::get_decrease_reputation(1, Network::percentage_factor_as_u128());
        assert_eq!(new, 0);
    });
}

#[test]
fn test_factor_clamping() {
    new_test_ext().execute_with(|| {
        let over_factor = Network::percentage_factor_as_u128() * 10;
        let new_inc = Network::get_increase_reputation(500000000000000000, over_factor);
        let new_dec = Network::get_decrease_reputation(500000000000000000, over_factor);
        assert_eq!(new_inc, Network::percentage_factor_as_u128());
        assert_eq!(new_dec, 0);
    });
}
