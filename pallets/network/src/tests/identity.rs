use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use log::info;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use frame_support::{
	storage::bounded_vec::BoundedVec,
};
use crate::{
  Error,
  ColdkeyIdentity,
  HotkeyOwner,
  DefaultMaxUrlLength,
  DefaultMaxSocialIdLength,
  DefaultMaxVectorLength,
};
//
//
//
//
//
//
//
// Identity
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
    let coldkey = account(99);
    let hotkey = account(98);

    HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
    let name = to_bounded::<DefaultMaxUrlLength>("name");
    let url = to_bounded::<DefaultMaxUrlLength>("url");
    let image = to_bounded::<DefaultMaxUrlLength>("image");
    let discord = to_bounded::<DefaultMaxSocialIdLength>("discord");
    let x = to_bounded::<DefaultMaxSocialIdLength>("x");
    let telegram = to_bounded::<DefaultMaxSocialIdLength>("telegram");
    let github = to_bounded::<DefaultMaxUrlLength>("github");
    let hugging_face = to_bounded::<DefaultMaxUrlLength>("hugging_face");
    let description = to_bounded::<DefaultMaxVectorLength>("description");
    let misc = to_bounded::<DefaultMaxVectorLength>("misc");
    assert_ok!(
      Network::register_identity(
        RuntimeOrigin::signed(coldkey.clone()),
        hotkey.clone(),
        name.clone(),
        url.clone(),
        image.clone(),
        discord.clone(),
        x.clone(),
        telegram.clone(),
        github.clone(),
        hugging_face.clone(),
        description.clone(),
        misc.clone(),
      )
    );

    let coldkey_identity = ColdkeyIdentity::<Test>::get(coldkey);
    assert_eq!(coldkey_identity.name, name);
    assert_eq!(coldkey_identity.url, url);
    assert_eq!(coldkey_identity.image, image);
    assert_eq!(coldkey_identity.discord, discord);
    assert_eq!(coldkey_identity.x, x);
    assert_eq!(coldkey_identity.telegram, telegram);
    assert_eq!(coldkey_identity.github, github);
    assert_eq!(coldkey_identity.hugging_face, hugging_face);
    assert_eq!(coldkey_identity.description, description);
    assert_eq!(coldkey_identity.misc, misc);
  });
}

#[test]
fn test_register_identity_not_key_owner_error() {
  new_test_ext().execute_with(|| {
    let coldkey = account(99);
    let hotkey = account(98);
    let fake_hotkey = account(97);

    HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
    let name = to_bounded::<DefaultMaxUrlLength>("name");
    let url = to_bounded::<DefaultMaxUrlLength>("url");
    let image = to_bounded::<DefaultMaxUrlLength>("image");
    let discord = to_bounded::<DefaultMaxSocialIdLength>("discord");
    let x = to_bounded::<DefaultMaxSocialIdLength>("x");
    let telegram = to_bounded::<DefaultMaxSocialIdLength>("telegram");
    let github = to_bounded::<DefaultMaxUrlLength>("github");
    let hugging_face = to_bounded::<DefaultMaxUrlLength>("hugging_face");
    let description = to_bounded::<DefaultMaxVectorLength>("description");
    let misc = to_bounded::<DefaultMaxVectorLength>("misc");
    assert_err!(
      Network::register_identity(
        RuntimeOrigin::signed(coldkey.clone()),
        fake_hotkey.clone(),
        name.clone(),
        url.clone(),
        image.clone(),
        discord.clone(),
        x.clone(),
        telegram.clone(),
        github.clone(),
        hugging_face.clone(),
        description.clone(),
        misc.clone(),
      ),
      Error::<Test>::NotKeyOwner
    );
  });
}
