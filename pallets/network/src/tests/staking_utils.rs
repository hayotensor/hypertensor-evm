use super::mock::*;
use crate::tests::test_utils::*;
use sp_core::U256;
use crate::{
  AccountSubnetDelegateStakeShares,
  TotalSubnetDelegateStakeBalance,
  TotalSubnetDelegateStakeShares
};

#[test]
fn test_convert_to_shares_basic() {
  new_test_ext().execute_with(|| {
    let balance = 1_000_000u128;
    let total_shares = 10_000_000u128;
    let total_balance = 100_000_000u128;

    let shares = Network::convert_to_shares(balance, total_shares, total_balance);
    let expected = U256::from(balance) * (U256::from(total_shares) + 10) / (U256::from(total_balance) + 1);

    assert_eq!(U256::from(shares), expected);
  });
}

#[test]
fn test_convert_to_balance_basic() {
  new_test_ext().execute_with(|| {
    let shares = 1_000_000u128;
    let total_shares = 10_000_000u128;
    let total_balance = 100_000_000u128;

    let balance = Network::convert_to_balance(shares, total_shares, total_balance);
    let expected = U256::from(shares) * (U256::from(total_balance) + 1) / (U256::from(total_shares) + 10);

    assert_eq!(U256::from(balance), expected);
  });
}

#[test]
fn test_convert_to_shares_zero_total_shares() {
  new_test_ext().execute_with(|| {
    let balance = 123_456_789u128;
    let shares = Network::convert_to_shares(balance, 0, 0);
    assert_eq!(shares, balance);
  });
}

#[test]
fn test_convert_to_balance_zero_total_shares() {
  new_test_ext().execute_with(|| {
    let shares = 987_654_321u128;
    let balance = Network::convert_to_balance(shares, 0, 0);
    assert_eq!(balance, shares);
  });
}

#[test]
fn test_round_trip_conversion() {
  new_test_ext().execute_with(|| {
    let deposit = 1_000_000u128;
    let total_balance = 10_000_000u128;
    let total_shares = 1_000_000u128;

    let shares = Network::convert_to_shares(deposit, total_shares, total_balance);
    let balance = Network::convert_to_balance(shares, total_shares + shares, total_balance + deposit);

    let diff = if balance > deposit {
        balance - deposit
    } else {
        deposit - balance
    };

    assert!(diff < 10, "Expected close match, got difference {}", diff);
  });
}

#[test]
fn test_convert_to_shares_overflow_caps_to_max() {
  new_test_ext().execute_with(|| {
    let balance = u128::MAX;
    let total_shares = u128::MAX;
    let total_balance = 1;

    let result = Network::convert_to_shares(balance, total_shares, total_balance);
    assert_eq!(result, u128::MAX);
  });
}

#[test]
fn test_convert_to_balance_overflow_caps_to_max() {
  new_test_ext().execute_with(|| {
    let shares = u128::MAX;
    let total_shares = 1;
    let total_balance = u128::MAX;

    let result = Network::convert_to_balance(shares, total_shares, total_balance);
    assert_eq!(result, u128::MAX);
  });
}

// #[test]
// fn test_convert_to_shares_and_balance_roundtrip_large_supply() {
//   new_test_ext().execute_with(|| {
//     // Total supply: 10 billion * 1e18
//     let total_supply: u128 = 10_000_000_000u128
//       .checked_mul(1_000_000_000_000_000_000u128)
//       .expect("overflow multiplying total supply");

//     // Assume total shares == total supply initially (one-to-one)
//     let total_shares = total_supply;

//     // User balance = 10 billion * 1e18 (full supply)
//     let user_balance: u128 = total_supply;

//     // Convert balance -> shares
//     let shares = Network::convert_to_shares(user_balance, total_shares, total_supply);

//     // Convert shares -> balance
//     let balance_back = Network::convert_to_balance(shares, total_shares, total_supply);

//     // Shares must be > 0
//     assert!(shares > 0, "Shares should be greater than zero");

//     // Round-trip balance back should be very close to original balance (within 1 token)
//     let diff = if balance_back > user_balance {
//       balance_back - user_balance
//     } else {
//       user_balance - balance_back
//     };

//     assert!(
//       diff <= 1_000_000_000_000_000_000u128,
//       "Balance round-trip diff too large: {}",
//       diff
//     );
//   });
// }

#[test]
fn test_convert_to_shares_and_balance_roundtrip_large_supply() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;

    // Total supply: 10 billion * 1e18
    let total_supply: u128 = 100_000_000_000u128
      .checked_mul(1_000_000_000_000_000_000u128)
      .expect("overflow multiplying total supply");

    // User balance = 10 billion * 1e18 (full supply)
    let user_balance: u128 = total_supply;

    // Convert balance -> shares
    let shares = Network::convert_to_shares(user_balance, 0, 0);

    Network::increase_account_delegate_stake(
      &account(1),
      subnet_id, 
      user_balance,
      shares,
    );

    // Convert shares -> balance
    let account_shares = AccountSubnetDelegateStakeShares::<Test>::get(&account(1), subnet_id);
    let total_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    let total_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);

    // Shares must be > 0
    assert!(shares > 0, "Shares should be greater than zero");
    assert_eq!(shares, account_shares);

    let balance_back = Network::convert_to_balance(account_shares, total_shares, total_balance);

    assert!(user_balance > balance_back, "Balance back should be less than deposited balance");

    // Round-trip balance back should be very close to original balance (within 1 token)
    let diff = if balance_back > user_balance {
      balance_back - user_balance
    } else {
      user_balance - balance_back
    };

    assert!(
      diff <= 1_000_000_000_000_000_000u128,
      "Balance round-trip diff too large: {}",
      diff
    );

    let real_shares = Network::convert_to_shares(balance_back, total_shares, total_balance);
    assert!(real_shares < shares, "Real shares should be less than equated shares");
  });
}
