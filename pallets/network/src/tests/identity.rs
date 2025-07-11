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
fn test_register_identity() {
  new_test_ext().execute_with(|| {
    assert_ok!(
      Network::register_identity(
        RuntimeOrigin::signed(account(1)),
        hotkey: T::AccountId,
        name: BoundedVec<u8, DefaultMaxUrlLength>,
        url: BoundedVec<u8, DefaultMaxUrlLength>,
        image: BoundedVec<u8, DefaultMaxUrlLength>,
        discord: BoundedVec<u8, DefaultMaxSocialIdLength>,
        x: BoundedVec<u8, DefaultMaxSocialIdLength>,
        telegram: BoundedVec<u8, DefaultMaxSocialIdLength>,
        github: BoundedVec<u8, DefaultMaxUrlLength>,
        hugging_face: BoundedVec<u8, DefaultMaxUrlLength>,
        description: BoundedVec<u8, DefaultMaxVectorLength>,
        misc: BoundedVec<u8, DefaultMaxVectorLength>,
      )
    );

    
  });
}
