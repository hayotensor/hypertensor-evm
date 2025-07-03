use super::mock::*;
use crate::tests::test_utils::*;
use log::info;
use sp_core::U256;

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
    assert_eq!(result, 0); // now this should pass
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
