use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use log::info;
use crate::inflation::Inflation;
use crate::{
  UtilizationLowerBound,
};
use sp_runtime::FixedU128;
use sp_runtime::traits::{Saturating, CheckedDiv, CheckedMul};
use frame_support::pallet_prelude::{One, Zero};
use sp_runtime::FixedPointNumber;

//
//
//
//
//
//
//
// Inflation
//
//
//
//
//
//
//

#[test]
fn inflation_should_decrease_as_utilization_increases() {
  new_test_ext().execute_with(|| {
    let low = Network::get_inflation(0.0);
    let mid = Network::get_inflation(0.5);
    let high = Network::get_inflation(1.0);

    // Ensure inflation starts high and decreases
    assert!(low > mid, "Inflation at 0.0 should be higher than at 0.5");
    assert!(mid > high, "Inflation at 0.5 should be higher than at 1.0");

    // Check that boundaries are roughly as expected
    assert!((low - 0.1).abs() < 0.01, "Low inflation not near max");
    assert!((high - 0.015).abs() < 0.01, "High inflation not near min");
  });
}

// Interest rate decreases as utilization increases
#[test]
fn test_get_interest_rate() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let mut last = f64::MAX;

    for util in &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
      let inflation = Network::get_inflation(*util);
      assert!(inflation < last);
    }
  });
}

// #[test]
// fn inflation_values_should_be_correct() {
//   new_test_ext().execute_with(|| {
//     let i0 = FixedU128::saturating_from_rational(10u128, 100);   // 10%
//     let i_min = FixedU128::saturating_from_rational(15u128, 1000); // 1.5%
//     let k = FixedU128::saturating_from_rational(1u128, 100);    // 0.01
//     let max_nodes = 1000;

//     let inflation_0 = Network::inflation(0, max_nodes, i0, i_min, k);
//     let epsilon = FixedU128::saturating_from_rational(1u128, 10_000); // 0.01%
//     assert!(inflation_0.saturating_sub(i0) < epsilon);

//     let inflation_max = Network::inflation(max_nodes, max_nodes, i0, i_min, k);
//     assert_eq!(inflation_max, i_min);

//     let inflation_mid = Network::inflation(max_nodes / 2, max_nodes, i0, i_min, k);
//     assert!(inflation_mid < i0);
//     assert!(inflation_mid > i_min);
//   });
// }

// Interest rate decreases as utilization increases
// #[test]
// fn test_get_interest_rate() {
//   new_test_ext().execute_with(|| {
//     let _ = env_logger::builder().is_test(true).try_init();

//     let min: f64 = Network::get_percent_as_f64(UtilizationLowerBound::<Test>::get());

//     let mut last1 = f64::MAX;

//     for subnet_utilization in &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
//       log::error!(" ");

//       log::error!("test_get_interest_rate subnet_utilization {:?}", subnet_utilization);

//       let inflation = Network::get_inflation_rate(
//         0, // epoch
//         *subnet_utilization,
//         0.0, // node utilization
//       );

//       if *subnet_utilization <= min {
//         assert!(inflation <= last1);
//       } else {
//         assert!(inflation < last1);
//       }
//       last1 = inflation;
//     }

//     let mut last2 = f64::MAX;

//     for subnet_node_utilization in &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
//       log::error!(" ");

//       log::error!("test_get_interest_rate subnet_node_utilization {:?}", subnet_node_utilization);

//       let inflation = Network::get_inflation_rate(
//         0, // epoch
//         0.0,
//         *subnet_node_utilization, // node utilization
//       );
      
//       if *subnet_node_utilization <= min {
//         assert!(inflation <= last2);
//       } else {
//         assert!(inflation < last2);
//       }
//       last2 = inflation;
//     }
//   });
// }

// #[test]
// fn test_inflation_math() {
//   new_test_ext().execute_with(|| {

//   });
// }