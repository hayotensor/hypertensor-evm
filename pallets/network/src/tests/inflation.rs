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

// Interest rate decreases as utilization increases
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
  });
}

#[test]
fn test_inflation_math() {
  new_test_ext().execute_with(|| {

  });
}