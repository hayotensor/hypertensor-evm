use super::mock::*;
use super::test_utils::*;
use crate::{
  Error, 
  TotalSubnetDelegateStakeBalance,
  TotalActiveSubnetNodes,
  TotalActiveNodes,
  TotalDelegateStake,
};

// #[test]
// fn test_calculate_stake_weights_with_node_bias() {
//   new_test_ext().execute_with(|| {
//     let percentage_factor = Network::percentage_factor_as_u128();
//     let alpha = percentage_factor / 2; // alpha = 0.5
//     let subnet_ids = vec![1, 2, 3];

//     // Simulated data:
//     // Subnet 1: 100 TENSOR stake, 10 nodes
//     // Subnet 2: 300 TENSOR stake, 5 nodes
//     // Subnet 3: 100 TENSOR stake, 20 nodes

//     let stake_1 = 100 * 10u128.pow(18);
//     let stake_2 = 300 * 10u128.pow(18);
//     let stake_3 = 100 * 10u128.pow(18);

//     TotalSubnetDelegateStakeBalance::<Test>::insert(1, stake_1);
//     TotalSubnetDelegateStakeBalance::<Test>::insert(2, stake_2);
//     TotalSubnetDelegateStakeBalance::<Test>::insert(3, stake_3);

//     let total_stake = stake_1 + stake_2 + stake_3;
//     TotalDelegateStake::<Test>::put(total_stake);

//     TotalActiveSubnetNodes::<Test>::insert(1, 10);
//     TotalActiveSubnetNodes::<Test>::insert(2, 5);
//     TotalActiveSubnetNodes::<Test>::insert(3, 20);

//     let total_nodes = 10 + 5 + 20;
//     TotalActiveNodes::<Test>::put(total_nodes);

//     // --- Call Function
//     let weights = Network::calculate_emission_weights(
//         &subnet_ids,
//         percentage_factor,
//         total_stake,
//         alpha, // alpha_stake (0.5 in 1e18 form)
//         percentage_factor // alpha_node
//     );

//     // --- Normalize and print
//     let p1 = Network::get_percent_as_f64(*weights.get(&1).unwrap());
//     let p2 = Network::get_percent_as_f64(*weights.get(&2).unwrap());
//     let p3 = Network::get_percent_as_f64(*weights.get(&3).unwrap());

//     let total = p1 + p2 + p3;

//     log::info!("Subnet Weights:");
//     log::info!("  1: {:.4}%", p1 * 100.0);
//     log::info!("  2: {:.4}%", p2 * 100.0);
//     log::info!("  3: {:.4}%", p3 * 100.0);
//     log::info!("Total: {:.4}%", total * 100.0);

//     // --- Ensure it sums to ~100%
//     assert!((total - 1.0).abs() < 0.0001, "Weights must sum to ~1.0");

//     // --- Relative checks
//     assert!(p2 > p1, "Subnet 2 has more stake but fewer nodes than 1");
//     assert!(p3 >= p1, "Subnet 3 has same stake as 1 but more nodes");
//   });
// }

// #[test]
// fn test_calculate_emission_weights_basic() {
//   new_test_ext().execute_with(|| {
//     let _ = env_logger::builder().is_test(true).try_init();

//     let subnet_ids = vec![1, 2, 3];

//     // Total network stake and nodes
//     let total_delegate_stake = 3_000_000_000_000_000_000_000u128; // 3 billion * 1e18
//     let percentage_factor = 1_000_000_000_000_000_000u128; // 1e18

//     // Assign per-subnet stakes and node counts
//     let stakes = vec![
//       1_000_000_000_000_000_000u128, // 1B
//       1_000_000_000_000_000_000u128,
//       1_000_000_000_000_000_000u128,
//     ];
//     let node_counts = vec![10, 20, 30]; // total = 60

//     // Set mock storage values
//     for (i, subnet_id) in subnet_ids.iter().enumerate() {
//       TotalSubnetDelegateStakeBalance::<Test>::insert(subnet_id, stakes[i]);
//       TotalActiveSubnetNodes::<Test>::insert(subnet_id, node_counts[i]);
//     }

//     // Total active nodes
//     TotalActiveNodes::<Test>::put(60u32);

//     // alpha = 0.5
//     let alpha_stake = 500_000_000_000_000_000u128;
//     let alpha_node = 1_000_000_000_000_000_000u128;

//     let weights = Network::calculate_emission_weights(
//       &subnet_ids,
//       percentage_factor,
//       total_delegate_stake,
//       alpha_stake,
//       alpha_node,
//     );

//     // Output weights for visual inspection (optional)
//     for (subnet_id, weight) in &weights {
//       log::error!("Subnet {} has normalized weight: {}", subnet_id, weight);
//       log::info!("Subnet {} has normalized weight: {}", subnet_id, weight);
//     }

//     // Assert weights are in expected proportions
//     let weight_sum: u128 = weights.values().sum();
//     assert_eq!(weight_sum, percentage_factor); // normalized

//     // Check that weights are ordered by node count (since stake is equal)
//     assert!(weights[&1] < weights[&2]);
//     assert!(weights[&2] < weights[&3]);
//   });
// }

// #[test]
// fn test_stake_weights_zero_stake() {
//   new_test_ext().execute_with(|| {
//     let subnet_ids = vec![1, 2];
//     TotalSubnetDelegateStakeBalance::<Test>::insert(1, 0);
//     TotalSubnetDelegateStakeBalance::<Test>::insert(2, 1_000_000_000_000_000_000u128);

//     TotalActiveSubnetNodes::<Test>::insert(1, 10);
//     TotalActiveSubnetNodes::<Test>::insert(2, 10);

//     TotalActiveNodes::<Test>::put(20);
//     let total_delegate_stake = 1_000_000_000_000_000_000u128;

//     let weights = Network::calculate_emission_weights(
//       &subnet_ids,
//       1_000_000_000_000_000_000,
//       total_delegate_stake,
//       500_000_000_000_000_000,
//       1_000_000_000_000_000_000,
//     );

//     assert_eq!(weights[&1], 0); // zero stake means zero weight
//   });
// }

// #[test]
// fn test_stake_weights_zero_nodes() {
//   new_test_ext().execute_with(|| {
//     let subnet_ids = vec![1, 2];
//     TotalSubnetDelegateStakeBalance::<Test>::insert(1, 1_000_000_000_000_000_000u128);
//     TotalSubnetDelegateStakeBalance::<Test>::insert(2, 1_000_000_000_000_000_000u128);

//     TotalActiveSubnetNodes::<Test>::insert(1, 0); // test 0
//     TotalActiveSubnetNodes::<Test>::insert(2, 10);

//     TotalActiveNodes::<Test>::put(10);
//     let total_delegate_stake = 2_000_000_000_000_000_000u128;

//     let weights = Network::calculate_emission_weights(
//       &subnet_ids,
//       1_000_000_000_000_000_000,
//       total_delegate_stake,
//       500_000_000_000_000_000,
//       1_000_000_000_000_000_000,
//     );

//     assert!(weights[&1] < weights[&2]); // node count influences this
//   });
// }

#[test]
fn test_stake_and_node_dominance_is_dampened_2_nets() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let subnet_ids = vec![1, 2];

    // Assign stakes: Subnet 1 has 90%, Subnet 2 has 10%
    let stake_a = 9_000_000_000_000_000_000u128;
    let stake_b = 1_000_000_000_000_000_000u128;
    let total_delegate_stake = stake_a + stake_b;

    // Assign node counts: Subnet 1 has 90%, Subnet 2 has 10%
    let nodes_a = 90u32;
    let nodes_b = 10u32;
    let total_nodes = nodes_a + nodes_b;

    // Insert into storage
    TotalSubnetDelegateStakeBalance::<Test>::insert(1, stake_a);
    TotalSubnetDelegateStakeBalance::<Test>::insert(2, stake_b);
    TotalActiveSubnetNodes::<Test>::insert(1, nodes_a);
    TotalActiveSubnetNodes::<Test>::insert(2, nodes_b);
    TotalActiveNodes::<Test>::put(total_nodes);

    // Constants
    let percentage_factor = 1_000_000_000_000_000_000u128;
    let alpha =   500_000_000_000_000_000u128; // = 0.5

    // Calculate weights
    let weights = Network::calculate_emission_weights(
      &subnet_ids,
      percentage_factor,
      total_delegate_stake,
      alpha,
    );

    let weight_a = weights[&1];
    let weight_b = weights[&2];

    // Assert weights sum to 1e18
    assert_eq!(weight_a + weight_b, percentage_factor);

    // Subnet 1 has 90% stake/node ratio — but should get < 90% weight
    assert!(weight_a < 900_000_000_000_000_000u128);

    // Subnet 2 has 10% — but should get > 10% weight
    assert!(weight_b > 100_000_000_000_000_000u128);
  });
}

#[test]
fn test_stake_and_node_dominance_is_dampened_3_nets() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let subnet_ids = vec![1, 2, 3];

    // Assign stakes: Subnet 1 has 90%, Subnet 2 has 10%
    let stake_a = 9_000_000_000_000_000_000u128;
    let stake_b = 500_000_000_000_000_000u128;
    let stake_c = 500_000_000_000_000_000u128;
    let total_delegate_stake = stake_a + stake_b + stake_c;

    // Assign node counts: Subnet 1 has 90%, Subnet 2 has 10%
    let nodes_a = 90u32;
    let nodes_b = 5u32;
    let nodes_c = 5u32;
    let total_nodes = nodes_a + nodes_b + nodes_c;

    // Insert into storage
    TotalSubnetDelegateStakeBalance::<Test>::insert(1, stake_a);
    TotalSubnetDelegateStakeBalance::<Test>::insert(2, stake_b);
    TotalSubnetDelegateStakeBalance::<Test>::insert(3, stake_c);
    TotalActiveSubnetNodes::<Test>::insert(1, nodes_a);
    TotalActiveSubnetNodes::<Test>::insert(2, nodes_b);
    TotalActiveSubnetNodes::<Test>::insert(3, nodes_c);
    TotalActiveNodes::<Test>::put(total_nodes);

    // Constants
    let percentage_factor = 1_000_000_000_000_000_000u128;
    let alpha = 500_000_000_000_000_000u128; // = 0.5

    // Calculate weights
    let weights = Network::calculate_emission_weights(
      &subnet_ids,
      percentage_factor,
      total_delegate_stake,
      alpha,
    );

    let weight_a = weights[&1];
    let weight_b = weights[&2];
    let weight_c = weights[&3];

    log::info!("Subnet 1 (dominant) weight: {}", weight_a);
    log::info!("Subnet 2 (minor) weight: {}", weight_b);
    log::info!("Subnet 3 (minor) weight: {}", weight_c);
    log::error!("Subnet 1 (dominant) weight: {}", weight_a);
    log::error!("Subnet 2 (minor) weight: {}", weight_b);
    log::error!("Subnet 3 (minor) weight: {}", weight_c);

    // Assert weights sum to 1e18
    assert_eq!(weight_a + weight_b + weight_b, percentage_factor);

    // Subnet 1 has 90% stake/node ratio — but should get < 90% weight
    assert!(weight_a < 900_000_000_000_000_000u128);

    // Subnet 2 has 5% — but should get > 5% weight
    assert!(weight_b > 50_000_000_000_000_000u128);

    // Subnet 3 has 5% — but should get > 5% weight
    assert!(weight_b > 50_000_000_000_000_000u128);

    assert!(false);
  });
}
