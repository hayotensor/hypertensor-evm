use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use crate::{
    ColdkeyIdentity, ColdkeyIdentityNameOwner, DefaultMaxSocialIdLength, DefaultMaxUrlLength,
    DefaultMaxVectorLength, Error, HotkeyOwner,
};
use frame_support::storage::bounded_vec::BoundedVec;
use frame_support::{assert_err, assert_ok};
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

#[test]
fn test_register_or_update_identity() {
    new_test_ext().execute_with(|| {
        increase_epochs(1);

        let coldkey = account(99);
        let hotkey = account(98);

        HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
        let name = to_bounded::<DefaultMaxVectorLength>("name");
        let url = to_bounded::<DefaultMaxUrlLength>("url");
        let image = to_bounded::<DefaultMaxUrlLength>("image");
        let discord = to_bounded::<DefaultMaxSocialIdLength>("discord");
        let x = to_bounded::<DefaultMaxSocialIdLength>("x");
        let telegram = to_bounded::<DefaultMaxSocialIdLength>("telegram");
        let github = to_bounded::<DefaultMaxUrlLength>("github");
        let hugging_face = to_bounded::<DefaultMaxUrlLength>("hugging_face");
        let description = to_bounded::<DefaultMaxVectorLength>("description");
        let misc = to_bounded::<DefaultMaxVectorLength>("misc");
        assert_ok!(Network::register_or_update_identity(
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
        ));

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

        assert_eq!(
            ColdkeyIdentityNameOwner::<Test>::get(name.clone()),
            coldkey.clone()
        );

        assert_eq!(
            *network_events().last().unwrap(),
            Event::IdentityRegistered {
                coldkey: coldkey.clone(),
                identity: coldkey_identity,
            }
        );
    });
}

#[test]
fn test_register_or_update_identity_try_stealing_error() {
    new_test_ext().execute_with(|| {
        increase_epochs(1);

        let coldkey = account(99);
        let hotkey = account(98);

        HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
        let name = to_bounded::<DefaultMaxVectorLength>("name");
        let url = to_bounded::<DefaultMaxUrlLength>("url");
        let image = to_bounded::<DefaultMaxUrlLength>("image");
        let discord = to_bounded::<DefaultMaxSocialIdLength>("discord");
        let x = to_bounded::<DefaultMaxSocialIdLength>("x");
        let telegram = to_bounded::<DefaultMaxSocialIdLength>("telegram");
        let github = to_bounded::<DefaultMaxUrlLength>("github");
        let hugging_face = to_bounded::<DefaultMaxUrlLength>("hugging_face");
        let description = to_bounded::<DefaultMaxVectorLength>("description");
        let misc = to_bounded::<DefaultMaxVectorLength>("misc");
        assert_ok!(Network::register_or_update_identity(
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
        ));

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
        assert_eq!(
            ColdkeyIdentityNameOwner::<Test>::get(name.clone()),
            coldkey.clone()
        );

        assert_eq!(
            *network_events().last().unwrap(),
            Event::IdentityRegistered {
                coldkey: coldkey.clone(),
                identity: coldkey_identity,
            }
        );

        let coldkey = account(199);
        let hotkey = account(198);

        HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());

        // Try stealing identity from a new key
        assert_err!(
            Network::register_or_update_identity(
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
            ),
            Error::<Test>::IdentityTaken
        );
    });
}

#[test]
fn test_register_or_update_identity_update_name() {
    new_test_ext().execute_with(|| {
        increase_epochs(1);

        let coldkey = account(99);
        let hotkey = account(98);

        HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
        let name = to_bounded::<DefaultMaxVectorLength>("name");
        let url = to_bounded::<DefaultMaxUrlLength>("url");
        let image = to_bounded::<DefaultMaxUrlLength>("image");
        let discord = to_bounded::<DefaultMaxSocialIdLength>("discord");
        let x = to_bounded::<DefaultMaxSocialIdLength>("x");
        let telegram = to_bounded::<DefaultMaxSocialIdLength>("telegram");
        let github = to_bounded::<DefaultMaxUrlLength>("github");
        let hugging_face = to_bounded::<DefaultMaxUrlLength>("hugging_face");
        let description = to_bounded::<DefaultMaxVectorLength>("description");
        let misc = to_bounded::<DefaultMaxVectorLength>("misc");
        assert_ok!(Network::register_or_update_identity(
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
        ));

        let coldkey_identity = ColdkeyIdentity::<Test>::get(coldkey);
        assert_eq!(coldkey_identity.name, name);
        assert_eq!(
            ColdkeyIdentityNameOwner::<Test>::get(name.clone()),
            coldkey.clone()
        );

        // new name should override previous
        let new_name = to_bounded::<DefaultMaxVectorLength>("new_name");
        assert_ok!(Network::register_or_update_identity(
            RuntimeOrigin::signed(coldkey.clone()),
            hotkey.clone(),
            new_name.clone(),
            url.clone(),
            image.clone(),
            discord.clone(),
            x.clone(),
            telegram.clone(),
            github.clone(),
            hugging_face.clone(),
            description.clone(),
            misc.clone(),
        ));

        // Ensure old name is removed
        assert_eq!(
            ColdkeyIdentityNameOwner::<Test>::try_get(name.clone()),
            Err(())
        );

        // Ensure new name is the identity name and new name is the key
        let coldkey_identity = ColdkeyIdentity::<Test>::get(coldkey);
        assert_eq!(coldkey_identity.name, new_name);
        assert_eq!(
            ColdkeyIdentityNameOwner::<Test>::get(new_name.clone()),
            coldkey.clone()
        );
    });
}

#[test]
fn test_register_or_update_identity_not_key_owner_error() {
    new_test_ext().execute_with(|| {
        let coldkey = account(99);
        let hotkey = account(98);
        let fake_hotkey = account(97);

        HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
        let name = to_bounded::<DefaultMaxVectorLength>("name");
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
            Network::register_or_update_identity(
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

#[test]
fn test_remove_identity() {
    new_test_ext().execute_with(|| {
        increase_epochs(1);

        let coldkey = account(99);
        let hotkey = account(98);

        HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
        let name = to_bounded::<DefaultMaxVectorLength>("name");
        let url = to_bounded::<DefaultMaxUrlLength>("url");
        let image = to_bounded::<DefaultMaxUrlLength>("image");
        let discord = to_bounded::<DefaultMaxSocialIdLength>("discord");
        let x = to_bounded::<DefaultMaxSocialIdLength>("x");
        let telegram = to_bounded::<DefaultMaxSocialIdLength>("telegram");
        let github = to_bounded::<DefaultMaxUrlLength>("github");
        let hugging_face = to_bounded::<DefaultMaxUrlLength>("hugging_face");
        let description = to_bounded::<DefaultMaxVectorLength>("description");
        let misc = to_bounded::<DefaultMaxVectorLength>("misc");
        assert_ok!(Network::register_or_update_identity(
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
        ));

        let coldkey_identity = ColdkeyIdentity::<Test>::get(coldkey.clone());
        assert_eq!(coldkey_identity.name, name.clone());
        assert_eq!(coldkey_identity.url, url);
        assert_eq!(coldkey_identity.image, image);
        assert_eq!(coldkey_identity.discord, discord);
        assert_eq!(coldkey_identity.x, x);
        assert_eq!(coldkey_identity.telegram, telegram);
        assert_eq!(coldkey_identity.github, github);
        assert_eq!(coldkey_identity.hugging_face, hugging_face);
        assert_eq!(coldkey_identity.description, description);
        assert_eq!(coldkey_identity.misc, misc);

        assert_ok!(Network::remove_identity(RuntimeOrigin::signed(
            coldkey.clone()
        )));

        assert_eq!(
            *network_events().last().unwrap(),
            Event::IdentityRemoved {
                coldkey: coldkey.clone(),
                identity: coldkey_identity,
            }
        );

        assert_eq!(ColdkeyIdentity::<Test>::try_get(coldkey.clone()), Err(()));
        assert_eq!(
            ColdkeyIdentityNameOwner::<Test>::try_get(name.clone()),
            Err(())
        );
    });
}

#[test]
fn test_remove_identity_readd() {
    new_test_ext().execute_with(|| {
        let coldkey = account(99);
        let hotkey = account(98);

        HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());
        let name = to_bounded::<DefaultMaxVectorLength>("name");
        let url = to_bounded::<DefaultMaxUrlLength>("url");
        let image = to_bounded::<DefaultMaxUrlLength>("image");
        let discord = to_bounded::<DefaultMaxSocialIdLength>("discord");
        let x = to_bounded::<DefaultMaxSocialIdLength>("x");
        let telegram = to_bounded::<DefaultMaxSocialIdLength>("telegram");
        let github = to_bounded::<DefaultMaxUrlLength>("github");
        let hugging_face = to_bounded::<DefaultMaxUrlLength>("hugging_face");
        let description = to_bounded::<DefaultMaxVectorLength>("description");
        let misc = to_bounded::<DefaultMaxVectorLength>("misc");
        assert_ok!(Network::register_or_update_identity(
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
        ));

        let coldkey_identity = ColdkeyIdentity::<Test>::get(coldkey.clone());
        assert_eq!(coldkey_identity.name, name.clone());
        assert_eq!(coldkey_identity.url, url);
        assert_eq!(coldkey_identity.image, image);
        assert_eq!(coldkey_identity.discord, discord);
        assert_eq!(coldkey_identity.x, x);
        assert_eq!(coldkey_identity.telegram, telegram);
        assert_eq!(coldkey_identity.github, github);
        assert_eq!(coldkey_identity.hugging_face, hugging_face);
        assert_eq!(coldkey_identity.description, description);
        assert_eq!(coldkey_identity.misc, misc);

        assert_ok!(Network::remove_identity(RuntimeOrigin::signed(
            coldkey.clone()
        )));

        assert_eq!(ColdkeyIdentity::<Test>::try_get(coldkey.clone()), Err(()));
        assert_eq!(
            ColdkeyIdentityNameOwner::<Test>::try_get(name.clone()),
            Err(())
        );

        let coldkey = account(199);
        let hotkey = account(198);

        HotkeyOwner::<Test>::insert(hotkey.clone(), coldkey.clone());

        assert_ok!(Network::register_or_update_identity(
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
        ));
    });
}
