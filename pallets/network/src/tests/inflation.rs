use super::mock::*;
use crate::inflation::Inflation;
use crate::tests::test_utils::*;
use crate::{SigmoidMidpoint, SigmoidSteepness};
use frame_support::pallet_prelude::{One, Zero};
use sp_runtime::traits::{CheckedDiv, CheckedMul, Saturating};
use sp_runtime::FixedPointNumber;
use sp_runtime::FixedU128;

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
        let low = Network::get_inflation(0.0, 1.0);
        let mid = Network::get_inflation(0.5, 1.0);
        let high = Network::get_inflation(1.0, 1.0);

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
            let inflation = Network::get_inflation(*util, 1.0);
            assert!(inflation < last);
            last = inflation;
        }
    });
}

#[test]
fn test_get_interest_rate_year() {
    new_test_ext().execute_with(|| {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut last = f64::MAX;

        for year in &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
            let inflation = Network::get_inflation(0.0, *year);
            assert!(inflation < last);
            last = inflation;
        }
    });
}

#[test]
fn test_get_interest_rate_total() {
    new_test_ext().execute_with(|| {
        let _ = env_logger::builder().is_test(true).try_init();

        let mut last_v_r = f64::MAX;
        let mut last_f_r = f64::MAX;

        let inflation = Inflation::default();

        let mid = Network::get_percent_as_f64(SigmoidMidpoint::<Test>::get());
        let k = SigmoidSteepness::<Test>::get() as f64;

        for u in &[0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
            let (validator_rate, foundation_rate) = {
                let inflation = Inflation::default();
                (
                    (inflation).validator(*u, mid, k, 1.0),
                    (inflation).foundation(*u, mid, k, 1.0),
                )
            };

            assert!(validator_rate < last_v_r);
            assert!(foundation_rate < last_f_r);

            last_v_r = validator_rate;
            last_f_r = foundation_rate;
        }
    });
}
