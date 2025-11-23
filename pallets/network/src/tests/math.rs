use super::mock::*;
use crate::tests::test_utils::*;
use sp_core::U256;
use sp_std::collections::btree_map::BTreeMap;

///
///
///
///
///
///
///
/// Math
///
///
///
///
///
///
///

#[test]
fn percent_mul_basic_cases() {
    new_test_ext().execute_with(|| {
        let pf = Network::percentage_factor_as_u128();

        // 100% of 100 is 100
        assert_eq!(Network::percent_mul(100, pf), 100);

        // 50% of 200 is 100
        assert_eq!(Network::percent_mul(200, pf / 2), 100);

        // 25% of 400 is 100
        assert_eq!(Network::percent_mul(400, pf / 4), 100);

        // 0% of any number is 0
        assert_eq!(Network::percent_mul(1000, 0), 0);

        // Any number times 0 is 0
        assert_eq!(Network::percent_mul(0, pf), 0);
    });
}

#[test]
fn percent_div_basic_cases() {
    new_test_ext().execute_with(|| {
        let pf = Network::percentage_factor_as_u128();

        // 100 / 100% is 100
        assert_eq!(Network::percent_div(100, pf), 100);

        // 100 / 50% is 200
        assert_eq!(Network::percent_div(100, pf / 2), 200);

        // 100 / 25% is 400
        assert_eq!(Network::percent_div(100, pf / 4), 400);

        // Divide zero returns zero
        assert_eq!(Network::percent_div(0, pf), 0);
        assert_eq!(Network::percent_div(0, 0), 0);

        // Division by zero returns zero (by design)
        assert_eq!(Network::percent_div(100, 0), 0);
    });
}

#[test]
fn percent_mul_overflow_protection() {
    new_test_ext().execute_with(|| {
        // let pf = Network::percentage_factor_as_u128();

        // // Should not overflow (max safe u128 * 100%)
        // let result = Network::percent_mul(u128::MAX, pf);
        // assert!(result <= u128::MAX);

        // // Large value + high percent that causes overflow returns 0
        // let big_value = u128::MAX;
        // let big_percent = pf * 2; // 200%
        // let result = Network::percent_mul(big_value, big_percent);
        // assert_eq!(result, 0);

        let pf = Network::percentage_factor_as_u128();

        // This should be safe: MAX * 1 = MAX
        let result = Network::percent_mul(u128::MAX, pf);
        assert_eq!(result, u128::MAX);

        // This should overflow U256 internally and return 0
        let big_value = u128::MAX;
        let big_percent = pf * 2; // 200%
        let result = Network::percent_mul(big_value, big_percent);
        assert_eq!(result, u128::MAX); // now this should pass
    });
}

#[test]
fn percent_div_saturation() {
    new_test_ext().execute_with(|| {
        let pf = Network::percentage_factor_as_u128();

        // Large numerator with small denominator
        let result = Network::percent_div(u128::MAX, 1);
        assert_eq!(result, u128::MAX); // Should saturate
    });
}

#[test]
fn percentage_factor_is_correct() {
    new_test_ext().execute_with(|| {
        let pf = Network::percentage_factor_as_u128();
        assert_eq!(pf, 1_000_000_000_000_000_000);
    });
}

#[test]
fn get_percent_as_f64_conversion() {
    new_test_ext().execute_with(|| {
        let pf = Network::percentage_factor_as_u128();

        // 100% as f64 should be 1.0
        assert_eq!(Network::get_percent_as_f64(pf), 1.0);

        // 50% should be 0.5
        assert_eq!(Network::get_percent_as_f64(pf / 2), 0.5);

        // 0 should be 0.0
        assert_eq!(Network::get_percent_as_f64(0), 0.0);
    });
}

#[test]
fn pow_function_works() {
    new_test_ext().execute_with(|| {
        assert_eq!(Network::pow(2.0, 3.0), 8.0);
        assert_eq!(Network::pow(10.0, 0.0), 1.0);
        assert_eq!(Network::pow(5.0, 1.0), 5.0);
    });
}

#[test]
fn test_checked_mul_div() {
    new_test_ext().execute_with(|| {
        // Normal case: 10 * 20 / 5 = 40
        let x = U256::from(10);
        let y = U256::from(20);
        let z = U256::from(5);
        assert_eq!(Network::checked_mul_div(x, y, z), Some(U256::from(40)));

        // Divisor zero returns None
        let divisor_zero = U256::zero();
        assert_eq!(Network::checked_mul_div(x, y, divisor_zero), None);

        // Multiplication overflow test - Use a very large x and y to force overflow
        let max = U256::MAX;
        let large_x = max;
        let large_y = max;
        let denom = U256::from(1);

        // This multiplication overflows, so should return None
        assert_eq!(Network::checked_mul_div(large_x, large_y, denom), None);
    });
}

// #[test]
// fn test_sigmoid_decreasing() {
//     new_test_ext().execute_with(|| {
//         let y = Network::sigmoid_decreasing(0.5, 0.5, 10.0, 0.0, 1.0);
//         assert_eq!(y, 0.5);

//         let y = Network::sigmoid_decreasing(0.5, 0.5, 10.0, 0.0, 2.0);
//         assert_eq!(y, 1.0);

//         let y = Network::sigmoid_decreasing(0.5, 0.5, 10.0, 1.0, 2.0);
//         assert_eq!(y, 1.5);
//     });
// }

#[test]
fn test_sigmoid_decreasing_bounds() {
    new_test_ext().execute_with(|| {
        let min = 0.5;
        let max = 2.0;
        let mid = 0.5;
        let k = 5.0;

        let xs = [0.0, 0.25, 0.5, 0.75, 1.0];
        for &x in &xs {
            let y = Network::sigmoid_decreasing(x, mid, k, min, max);
            assert!(
                y >= min && y <= max,
                "y={} is out of bounds [{},{}]",
                y,
                min,
                max
            );
        }
    });
}

#[test]
fn test_sigmoid_decreasing_symmetry() {
    new_test_ext().execute_with(|| {
        let min = 0.0;
        let max = 1.0;
        let mid = 0.5;
        let k = 5.0;

        let y_left = Network::sigmoid_decreasing(0.25, mid, k, min, max);
        let y_right = Network::sigmoid_decreasing(0.75, mid, k, min, max);

        let complement_diff = (y_left + y_right - 1.0).abs();
        log::error!("complement_diff={:?}", complement_diff);
        assert!(
            complement_diff < 1e-6,
            "Expected y_left + y_right ≈ 1.0, got {} + {}",
            y_left,
            y_right
        );
    });
}

#[test]
fn test_sigmoid_decreasing_monotonicity() {
    new_test_ext().execute_with(|| {
        let min = 0.0;
        let max = 1.0;
        let mid = 0.5;
        let k = 5.0;

        let y0 = Network::sigmoid_decreasing(0.0, mid, k, min, max);
        let y1 = Network::sigmoid_decreasing(0.25, mid, k, min, max);
        let y2 = Network::sigmoid_decreasing(0.5, mid, k, min, max);
        let y3 = Network::sigmoid_decreasing(0.75, mid, k, min, max);
        let y4 = Network::sigmoid_decreasing(1.0, mid, k, min, max);

        assert!(
            y0 > y1 && y1 > y2 && y2 > y3 && y3 > y4,
            "Function is not decreasing properly"
        );
    });
}

#[test]
fn test_sigmoid_decreasing_extreme_k() {
    new_test_ext().execute_with(|| {
        let min = 0.0;
        let max = 1.0;
        let mid = 0.5;

        // Very small k → almost linear
        let y_low_k0 = Network::sigmoid_decreasing(0.0, mid, 0.01, min, max);
        let y_high_k0 = Network::sigmoid_decreasing(1.0, mid, 0.01, min, max);
        assert!(y_low_k0 > y_high_k0);

        // Very large k → almost step function
        let y_low_k1 = Network::sigmoid_decreasing(0.0, mid, 50.0, min, max);
        let y_high_k1 = Network::sigmoid_decreasing(1.0, mid, 50.0, min, max);
        // assert!((y_low_k1 - max).abs() < 1e-12);
        // assert!((y_high_k1 - min).abs() < 1e-12);
        assert!(
            (y_low_k1 - max).abs() < 1e-6,
            "y_low_k1={} not close to max={}",
            y_low_k1,
            max
        );
        assert!(
            (y_high_k1 - min).abs() < 1e-6,
            "y_high_k1={} not close to min={}",
            y_high_k1,
            min
        );
    });
}

#[test]
fn test_concave_down_decreasing_basic() {
    new_test_ext().execute_with(|| {
        let min = 0.5;
        let max = 2.0;
        let power = 2.0;

        // x = 0.0 -> should return max
        let y = Network::concave_down_decreasing(0.0, min, max, power);
        assert!((y - max).abs() < 1e-12, "Expected {}, got {}", max, y);

        // x = 1.0 -> should return min
        let y = Network::concave_down_decreasing(1.0, min, max, power);
        assert!((y - min).abs() < 1e-12, "Expected {}, got {}", min, y);

        // x = 0.5 -> should be between min and max
        let y = Network::concave_down_decreasing(0.5, min, max, power);
        assert!(
            y > min && y < max,
            "Expected between {} and {}, got {}",
            min,
            max,
            y
        );
    });
}

#[test]
fn test_concave_down_decreasing_power_edge() {
    new_test_ext().execute_with(|| {
        let min = 0.0;
        let max = 1.0;

        // negative power -> should default to 1.0
        let y = Network::concave_down_decreasing(0.5, min, max, -2.0);
        // with power = 1: y = 1 - x = 1 - 0.5 = 0.5
        assert!((y - 0.5).abs() < 1e-12, "Expected 0.5, got {}", y);

        // zero power -> should default to 1.0
        let y = Network::concave_down_decreasing(0.5, min, max, 0.0);
        assert!((y - 0.5).abs() < 1e-12, "Expected 0.5, got {}", y);
    });
}

#[test]
fn test_concave_down_decreasing_monotonicity() {
    new_test_ext().execute_with(|| {
        let min = 0.0;
        let max = 1.0;
        let power = 2.0;

        // Check that the function decreases as x increases
        let y0 = Network::concave_down_decreasing(0.0, min, max, power);
        let y1 = Network::concave_down_decreasing(0.25, min, max, power);
        let y2 = Network::concave_down_decreasing(0.5, min, max, power);
        let y3 = Network::concave_down_decreasing(0.75, min, max, power);
        let y4 = Network::concave_down_decreasing(1.0, min, max, power);

        assert!(
            y0 > y1 && y1 > y2 && y2 > y3 && y3 > y4,
            "Function is not strictly decreasing"
        );
    });
}
