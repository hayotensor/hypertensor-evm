use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use log::info;
use crate::inflation::Inflation;
use crate::{
  UtilizationLowerBound,
};
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
fn test_inflation_total() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let inflation = Inflation::default();

    let mut last = inflation.total(0.0);

    for year in &[0.1, 0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 100.0] {
      log::error!("test_inflation_total year {:?}", year);
      let total = inflation.total(*year);
      log::error!("test_inflation_total total {:?}", total);
      assert!(total < last);
      assert!(total >= inflation.terminal);
      last = total;
    }
    assert_eq!(last, inflation.terminal);
    // assert!(false);
  });
}

// #[test]
// fn test_get_epoch_emissions_simple() {
//   new_test_ext().execute_with(|| {
//     let inflation = Inflation::default();

//     // Network::get_epoch_emissions(0);
//     let amount = Network::get_epoch_emissions_w_treasury(0);
//     log::error!("test_get_epoch_emissions amount {:?}", amount);
//     assert!(false);
//   });
// }

// #[test]
// fn test_get_epoch_emissions_factorized() {
//   new_test_ext().execute_with(|| {
//     let inflation = Inflation::default();

//     // Network::get_epoch_emissions(0);
//     let amount = Network::get_epoch_emissions_w_treasury(0);
//     log::error!("test_get_epoch_emissions amount {:?}", amount);
//     // assert!(false);
//   });
// }

// #[test]
// fn test_get_epoch_inflation() {
//   new_test_ext().execute_with(|| {
//     let _ = env_logger::builder().is_test(true).try_init();

//     let mut last = u128::MAX;

//     for subnet_utilization in &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
//       log::error!(" ");

//       log::error!("test_get_epoch_inflation subnet_utilization {:?}", subnet_utilization);

//       let inflation = Network::get_epoch_inflation(
//         0, // epoch
//         *subnet_utilization,
//         0.0, // node utilization
//       );
//       assert!(inflation < last);
//       last = inflation;
//     }
//     // assert!(false);
//   });
// }

#[test]
fn test_get_interest_rate() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let min: f64 = Network::get_percent_as_f64(UtilizationLowerBound::<Test>::get());

    let mut last1 = f64::MAX;

    for subnet_utilization in &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
      log::error!(" ");

      log::error!("test_get_interest_rate subnet_utilization {:?}", subnet_utilization);

      let inflation = Network::get_inflation_rate(
        0, // epoch
        *subnet_utilization,
        0.0, // node utilization
      );

      if *subnet_utilization <= min {
        assert!(inflation <= last1);
      } else {
        assert!(inflation < last1);
      }
      last1 = inflation;
    }

    let mut last2 = f64::MAX;

    for subnet_node_utilization in &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
      log::error!(" ");

      log::error!("test_get_interest_rate subnet_node_utilization {:?}", subnet_node_utilization);

      let inflation = Network::get_inflation_rate(
        0, // epoch
        0.0,
        *subnet_node_utilization, // node utilization
      );
      
      if *subnet_node_utilization <= min {
        assert!(inflation <= last2);
      } else {
        assert!(inflation < last2);
      }
      last2 = inflation;
    }

    // assert!(false);
  });
}


// #[test]
// fn test_inflation_epoch() {
//   new_test_ext().execute_with(|| {
//     let _ = env_logger::builder().is_test(true).try_init();

//     let inflation = Inflation::default();

//     let mut last = inflation.total(0.0);

//     let epochs_per_year = EpochsPerYear::get();
//     log::error!("test_inflation_epoch epochs_per_year {:?}", epochs_per_year);

//     for epoch in &[1, 2, 3, 4, 5, 4, 5, 100] {
//       let year = epoch / epochs_per_year;
//       log::error!("test_inflation_epoch year {:?}", year);

//       log::error!("test_inflation_epoch epoch {:?}", epoch);
//       let total = inflation.epoch(*epoch, epochs_per_year, 1e+9 as u128);
//       log::error!("test_inflation_epoch total {:?}", total);
//       assert!(total < last);
//       assert!(total >= inflation.terminal);
//       last = total;
//     }
//     assert_eq!(last, inflation.terminal);
//     assert!(false);
//   });
// }

#[test]
fn test_inflation_math() {
  new_test_ext().execute_with(|| {
    log::error!("test_inflation_math adassd");
  });
}