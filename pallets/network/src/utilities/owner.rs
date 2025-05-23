// Copyright (C) Hypertensor.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;

impl<T: Config> Pallet<T> {
  /// Owner pause subnet for up to max period
  pub fn do_owner_pause_subnet(origin: T::RuntimeOrigin, subnet_id: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    Ok(())
  }

  pub fn do_owner_deactivate_subnet(origin: T::RuntimeOrigin, subnet_id: u32, name: Vec<u8>) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    // TODO: check name is subnet name

    Self::do_remove_subnet(
      name,
      SubnetRemovalReason::Owner,
    ).map_err(|e| e)?;

    Ok(())
  }

  pub fn do_owner_remove_subnet_node(origin: T::RuntimeOrigin, subnet_id: u32, subnet_node_id: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );


    Ok(())
  }

  pub fn do_owner_update_registration_interval(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      value <= MaxSubnetRegistrationInterval::<T>::get(),
      Error::<T>::MaxSubnetRegistration
    );

    SubnetNodeRegistrationInterval::<T>::insert(subnet_id, value);

    Self::deposit_event(Event::SubnetEntryIntervalUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      value: value 
    });

    Ok(())
  }

  pub fn do_owner_update_activation_interval(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      value <= MaxSubnetActivationInterval::<T>::get(),
      Error::<T>::MaxSubnetActivation
    );

    SubnetNodeActivationInterval::<T>::insert(subnet_id, value);

    Self::deposit_event(Event::SubnetEntryIntervalUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      value: value 
    });

    Ok(())
  }

  pub fn do_owner_add_to_initial_coldkeys(origin: T::RuntimeOrigin, subnet_id: u32, coldkeys: BTreeSet<T::AccountId>) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      !Self::is_subnet_active(subnet_id),
      Error::<T>::SubnetMustBeRegistering
    );

    Ok(())
  }

  pub fn do_owner_remove_from_initial_coldkeys(origin: T::RuntimeOrigin, subnet_id: u32, coldkeys: BTreeSet<T::AccountId>) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      !Self::is_subnet_active(subnet_id),
      Error::<T>::SubnetMustBeRegistering
    );

    Ok(())
  }

  pub fn do_owner_set_max_subnet_registration_epochs(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      !Self::is_subnet_active(subnet_id),
      Error::<T>::SubnetMustBeRegistering
    );

    SubnetNodeRegistrationEpochs::<T>::insert(subnet_id, value);

    Ok(())
  }

  pub fn do_owner_update_queue_period(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    SubnetNodeQueuePeriod::<T>::insert(subnet_id, value);

    Ok(())
  }

  pub fn do_owner_update_included_period(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    Ok(())
  }

  /// Gives owner the ability to rearrange the queue, for instance, the owner can order the queue based on
  /// a validators performance
  pub fn do_owner_rearrange_queue(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    Ok(())
  }

  /// Update max subnet node penalties
  pub fn do_owner_update_max_penalties(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    Ok(())
  }

  pub fn do_transfer_subnet_ownership(origin: T::RuntimeOrigin, subnet_id: u32, new_owner: T::AccountId) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    PendingSubnetOwner::<T>::insert(subnet_id, new_owner);

    Ok(())
  }

  pub fn do_accept_subnet_ownership(origin: T::RuntimeOrigin, subnet_id: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    // Ensure is pending subnet owner
    // let pending_owner: T::AccountId = PendingSubnetOwner::<T>::get(subnet_id);
    let pending_owner: T::AccountId = match PendingSubnetOwner::<T>::try_get(subnet_id) {
      Ok(pending_owner) => pending_owner,
      Err(()) => return Err(Error::<T>::NoPendingSubnetOwner.into()),
    };

    ensure!(
      coldkey == pending_owner,
      Error::<T>::NotPendingSubnetOwner
    );

    SubnetOwner::<T>::try_mutate_exists(
      subnet_id,
      |maybe_owner| -> DispatchResult {
        let owner = maybe_owner.as_mut().ok_or(Error::<T>::InvalidSubnetId)?;
        *owner = pending_owner;
        Ok(())
      }
    )?;

    Ok(())
  }

}