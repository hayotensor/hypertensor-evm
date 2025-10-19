use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use crate::SubnetSlot;
use frame_support::{assert_err, assert_ok};

#[test]
fn test_get_current_subnet_epoch_as_u32() {
    new_test_ext().execute_with(|| {
        let subnet_id = 1;
        let slot = 5;
        let epoch_length = EpochLength::get();

        SubnetSlot::<Test>::insert(subnet_id, slot);
        let current_epoch = Network::get_current_subnet_epoch_as_u32(subnet_id);

        // Epoch 0
        System::set_block_number(slot);
        assert_eq!(Network::get_current_subnet_epoch_as_u32(subnet_id), 0);

        System::set_block_number(epoch_length + slot - 1);
        assert_eq!(Network::get_current_subnet_epoch_as_u32(subnet_id), 0);

        // Epoch 1
        System::set_block_number(epoch_length + slot);
        assert_eq!(Network::get_current_subnet_epoch_as_u32(subnet_id), 1);

        log::error!("subnet epoch {:?}", epoch_length * 2 + slot - 1);
        log::error!("subnet epoch {:?}", epoch_length + slot + epoch_length - 1);

        System::set_block_number(epoch_length * 2 + slot - 1);
        assert_eq!(Network::get_current_subnet_epoch_as_u32(subnet_id), 1);

        // Epoch 2
        System::set_block_number(epoch_length * 2 + slot);
        assert_eq!(Network::get_current_subnet_epoch_as_u32(subnet_id), 2);

        System::set_block_number(epoch_length * 3 + slot - 1);
        assert_eq!(Network::get_current_subnet_epoch_as_u32(subnet_id), 2);

        // Epoch 3
        System::set_block_number(epoch_length * 3 + slot);
        assert_eq!(Network::get_current_subnet_epoch_as_u32(subnet_id), 3);

        System::set_block_number(epoch_length * 4 + slot - 1);
        assert_eq!(Network::get_current_subnet_epoch_as_u32(subnet_id), 3);
    })
}
