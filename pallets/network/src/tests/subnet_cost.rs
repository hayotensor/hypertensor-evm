use super::mock::*;
use crate::tests::test_utils::*;
use crate::{
    Error, MaxSubnetRegistrationFee, MinSubnetRegistrationFee, SubnetRegistrationInterval,
};
use frame_support::traits::Currency;
use frame_support::{assert_err, assert_ok};

// #[test]
// fn registration_cost_no_registration_various_epochs() {
//     new_test_ext().execute_with(|| {
//         // No registration yet
//         LastSubnetRegistrationEpoch::<Test>::put(0);
//         MinSubnetRegistrationFee::<Test>::put(100);
//         MaxSubnetRegistrationFee::<Test>::put(1000);
//         SubnetRegistrationInterval::<Test>::put(100);

//         // Epoch 0: full cost
//         assert_eq!(Network::registration_cost(0), 1000);

//         // Epoch 10: 10% into period
//         assert_eq!(Network::registration_cost(10), 910);

//         // Epoch 99: near end of period
//         assert_eq!(Network::registration_cost(99), 109);

//         // Epoch 100+: minimum fee
//         assert_eq!(Network::registration_cost(100), 100);
//         assert_eq!(Network::registration_cost(150), 100);
//         assert_eq!(Network::registration_cost(500), 100);
//     });
// }

// #[test]
// fn registration_cost_with_prior_registration() {
//     new_test_ext().execute_with(|| {
//         MinSubnetRegistrationFee::<Test>::put(100);
//         MaxSubnetRegistrationFee::<Test>::put(1000);
//         SubnetRegistrationInterval::<Test>::put(100);

//         // Simulate a prior registration at epoch 200
//         LastSubnetRegistrationEpoch::<Test>::put(200);

//         // Interval is 100 → next registration range is [200, 300)

//         assert_eq!(Network::registration_cost(200), 1000); // beginning of new interval
//         assert_eq!(Network::registration_cost(250), 550); // halfway through
//         assert_eq!(Network::registration_cost(299), 109); // near end
//         assert_eq!(Network::registration_cost(300), 100); // minimum fee after interval
//         assert_eq!(Network::registration_cost(350), 100); // still minimum
//     });
// }

// #[test]
// fn registration_cost_respects_min_fee() {
//     new_test_ext().execute_with(|| {
//         LastSubnetRegistrationEpoch::<Test>::put(0);
//         SubnetRegistrationInterval::<Test>::put(1);
//         MinSubnetRegistrationFee::<Test>::put(999);
//         MaxSubnetRegistrationFee::<Test>::put(1000);

//         assert_eq!(Network::registration_cost(1), 999);
//         assert_eq!(Network::registration_cost(2), 999);
//     });
// }

// #[test]
// fn no_registration_yet_epoch_0() {
//     new_test_ext().execute_with(|| {
//         LastSubnetRegistrationEpoch::<Test>::put(0);
//         assert_eq!(Network::get_next_registration_epoch(0), 0);
//     });
// }

// #[test]
// fn no_registration_yet_epoch_1010() {
//     new_test_ext().execute_with(|| {
//         LastSubnetRegistrationEpoch::<Test>::put(0);
//         assert_eq!(Network::get_next_registration_epoch(1010), 1000);
//     });
// }

// #[test]
// fn last_registered_at_30_next_should_be_100() {
//     new_test_ext().execute_with(|| {
//         LastSubnetRegistrationEpoch::<Test>::put(30);
//         assert_eq!(Network::get_next_registration_epoch(40), 100);
//     });
// }

// #[test]
// fn last_registered_at_100_next_should_be_200() {
//     new_test_ext().execute_with(|| {
//         LastSubnetRegistrationEpoch::<Test>::put(100);
//         assert_eq!(Network::get_next_registration_epoch(150), 200);
//     });
// }

// #[test]
// fn no_registration_yet_epoch_199_should_be_100() {
//     new_test_ext().execute_with(|| {
//         LastSubnetRegistrationEpoch::<Test>::put(0);
//         assert_eq!(Network::get_next_registration_epoch(199), 100);
//     });
// }
