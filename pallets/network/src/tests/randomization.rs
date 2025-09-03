use super::mock::*;
use crate::tests::test_utils::*;
use frame_support::traits::OnInitialize;
use sp_runtime::traits::Header;

///
///
///
///
///
///
///
/// Randomization
///
///
///
///
///
///
///

pub fn setup_blocks(blocks: u32) {
    let mut parent_hash = System::parent_hash();

    for i in 1..(blocks + 1) {
        System::reset_events();
        System::initialize(&i, &parent_hash, &Default::default());
        InsecureRandomnessCollectiveFlip::on_initialize(i);

        let header = System::finalize();
        parent_hash = header.hash();
        System::set_block_number(*header.number());
    }
}

#[test]
fn test_randomness() {
    new_test_ext().execute_with(|| {
        setup_blocks(38);
        let gen_rand_num = Network::generate_random_number(1);
        let rand_num = Network::get_random_number_with_max(96, 0);
    });
}

#[test]
fn test_random_number_within_range() {
    new_test_ext().execute_with(|| {
        let max = 100;
        let seed = 123;

        let random = Network::get_random_number_with_max(max, seed);
        assert!(random < max);
    });
}

#[test]
fn test_random_number_zero_max() {
    new_test_ext().execute_with(|| {
        let max = 0;
        let seed = 123;

        let random = Network::get_random_number_with_max(max, seed);
        assert_eq!(random, 0);
    });
}

#[test]
fn test_random_number_is_deterministic_with_mocked_randomness() {
    new_test_ext().execute_with(|| {
        let r1 = Network::generate_random_number(111);
        let r2 = Network::generate_random_number(111);
        assert_eq!(r1, r2); // StaticRandomness always returns same result
    });
}
