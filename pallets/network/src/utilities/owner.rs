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

    // SubnetsData::<T>::mutate(subnet_id, |maybe_data| {
    //   if let Some(data) = maybe_data {
    //     data.name = value.clone();
    //   }
    // });

    Ok(())
  }

  pub fn do_owner_deactivate_subnet(origin: T::RuntimeOrigin, subnet_id: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    Self::do_remove_subnet(
      subnet_id,
      SubnetRemovalReason::Owner,
    ).map_err(|e| e)?;

    Ok(())
  }

  pub fn do_owner_update_name(origin: T::RuntimeOrigin, subnet_id: u32, value: Vec<u8>) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      !SubnetName::<T>::contains_key(&value),
      Error::<T>::SubnetNameExist
    );

    let mut prev_name: Vec<u8> = Vec::new();
    SubnetsData::<T>::mutate(subnet_id, |maybe_data| {
      if let Some(data) = maybe_data {
        prev_name = data.name.clone();
        data.name = value.clone();
      }
    });

    SubnetName::<T>::insert(&value, subnet_id);

    Self::deposit_event(Event::SubnetNameUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      prev_value: prev_name,
      value: value 
    });

    Ok(())
  }

  pub fn do_owner_update_repo(origin: T::RuntimeOrigin, subnet_id: u32, value: Vec<u8>) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      !SubnetRepo::<T>::contains_key(&value),
      Error::<T>::SubnetRepoExist
    );

    let mut prev_repo: Vec<u8> = Vec::new();
    SubnetsData::<T>::mutate(subnet_id, |maybe_data| {
      if let Some(data) = maybe_data {
        prev_repo = data.repo.clone();
        data.repo = value.clone();
      }
    });

    SubnetRepo::<T>::insert(&value, subnet_id);

    Self::deposit_event(Event::SubnetRepoUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      prev_value: prev_repo,
      value: value 
    });

    Ok(())
  }

  pub fn do_owner_update_description(origin: T::RuntimeOrigin, subnet_id: u32, value: Vec<u8>) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    let mut prev_description: Vec<u8> = Vec::new();
    SubnetsData::<T>::mutate(subnet_id, |maybe_data| {
      if let Some(data) = maybe_data {
        prev_description = data.description.clone();
        data.description = value.clone();
      }
    });

    Self::deposit_event(Event::SubnetDescriptionUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      prev_value: prev_description,
      value: value 
    });

    Ok(())
  }

  pub fn do_owner_update_misc(origin: T::RuntimeOrigin, subnet_id: u32, value: Vec<u8>) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    let mut prev_misc: Vec<u8> = Vec::new();
    SubnetsData::<T>::mutate(subnet_id, |maybe_data| {
      if let Some(data) = maybe_data {
        prev_misc = data.misc.clone();
        data.misc = value.clone();
      }
    });

    Self::deposit_event(Event::SubnetMiscUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      prev_value: prev_misc,
      value: value 
    });

    Ok(())
  }

  pub fn do_owner_update_churn_limit(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      value >= MinChurnLimit::<T>::get() && 
      value <= MaxChurnLimit::<T>::get(),
      Error::<T>::InvalidChurnLimit
    );

    ChurnLimit::<T>::insert(subnet_id, value);

    Self::deposit_event(Event::ChurnLimitUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      value: value 
    });

    Ok(())
  }

  pub fn do_owner_update_registration_queue_epochs(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      value >= MinRegistrationQueueEpochs::<T>::get() &&
      value <= MaxRegistrationQueueEpochs::<T>::get(),
      Error::<T>::InvalidRegistrationQueueEpochs
    );

    RegistrationQueueEpochs::<T>::insert(subnet_id, value);

    Self::deposit_event(Event::RegistrationQueueEpochsUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      value: value 
    });


    Ok(())
  }

  pub fn do_owner_update_activation_grace_epochs(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      value >= MinActivationGraceEpochs::<T>::get() &&
      value <= MaxActivationGraceEpochs::<T>::get(),
      Error::<T>::InvalidActivationGraceEpochs
    );

    ActivationGraceEpochs::<T>::insert(subnet_id, value);

    Self::deposit_event(Event::ActivationGraceEpochsUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      value: value 
    });

    Ok(())
  }

  pub fn do_owner_update_queue_classification_epochs(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      value >= MinQueueClassificationEpochs::<T>::get() &&
      value <= MaxQueueClassificationEpochs::<T>::get(),
      Error::<T>::InvalidQueueClassificationEpochs
    );

    QueueClassificationEpochs::<T>::insert(subnet_id, value);

    Self::deposit_event(Event::QueueClassificationEpochsUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      value: value 
    });

    Ok(())
  }

  pub fn do_owner_update_included_classification_epochs(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      value >= MinIncludedClassificationEpochs::<T>::get() &&
      value <= MaxIncludedClassificationEpochs::<T>::get(),
      Error::<T>::InvalidIncludedClassificationEpochs
    );

    IncludedClassificationEpochs::<T>::insert(subnet_id, value);

    Self::deposit_event(Event::IncludedClassificationEpochsUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      value: value 
    });

    Ok(())
  }

  pub fn do_owner_update_max_node_penalties(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    MaxSubnetNodePenalties::<T>::insert(subnet_id, value);

    Self::deposit_event(Event::MaxSubnetNodePenaltiesUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      value: value 
    });

    Ok(())
  }

  pub fn do_owner_add_initial_coldkeys(origin: T::RuntimeOrigin, subnet_id: u32, coldkeys: BTreeSet<T::AccountId>) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      !Self::is_subnet_active(subnet_id),
      Error::<T>::SubnetMustBeRegistering
    );

    SubnetRegistrationInitialColdkeys::<T>::mutate(subnet_id, |accounts| {
      let accounts_set = accounts.get_or_insert_with(BTreeSet::new);
      accounts_set.extend(coldkeys.iter().cloned());
    });

    Ok(())
  }

  pub fn do_owner_remove_initial_coldkeys(origin: T::RuntimeOrigin, subnet_id: u32, coldkeys: BTreeSet<T::AccountId>) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      !Self::is_subnet_active(subnet_id),
      Error::<T>::SubnetMustBeRegistering
    );

    SubnetRegistrationInitialColdkeys::<T>::mutate(subnet_id, |maybe_accounts| {
      if let Some(existing_accounts) = maybe_accounts {
        // Remove all accounts that exist in coldkeys
        for account in &coldkeys {
          existing_accounts.remove(account);
        }
        
        // Clean up if the set becomes empty
        if existing_accounts.is_empty() {
          *maybe_accounts = None;
        }
      }
    });

    Ok(())
  }

  pub fn do_owner_update_node_removal_system(origin: T::RuntimeOrigin, subnet_id: u32, value: NodeRemovalSystem) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    SubnetNodeRemovalSystem::<T>::insert(subnet_id, value);

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

  pub fn do_owner_update_min_stake(origin: T::RuntimeOrigin, subnet_id: u32, value: u128) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
    	value >= MinSubnetMinStake::<T>::get() &&
    	value <= MaxSubnetMinStake::<T>::get(),
    	Error::<T>::InvalidSubnetMinStake
    );

    ensure!(
      SubnetMaxStakeBalance::<T>::get(subnet_id) >= value,
      Error::<T>::InvalidSubnetStakeParameters
    );

    SubnetMinStakeBalance::<T>::insert(subnet_id, value);

    Ok(())
  }

  pub fn do_owner_update_max_stake(origin: T::RuntimeOrigin, subnet_id: u32, value: u128) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
    	value >= MinSubnetMaxStake::<T>::get() &&
    	value <= MaxSubnetMaxStake::<T>::get(),
    	Error::<T>::InvalidSubnetMinStake
    );

    ensure!(
      SubnetMinStakeBalance::<T>::get(subnet_id) <= value,
      Error::<T>::InvalidSubnetStakeParameters
    );

    SubnetMaxStakeBalance::<T>::insert(subnet_id, value);

    Ok(())
  }

  pub fn do_owner_update_delegate_stake_percentage(origin: T::RuntimeOrigin, subnet_id: u32, value: u128) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
      value >= MinDelegateStakePercentage::<T>::get() &&
      value <= MaxDelegateStakePercentage::<T>::get() &&
      value <= Self::percentage_factor_as_u128(),
      Error::<T>::InvalidMinDelegateStakePercentage
    );

    SubnetDelegateStakeRewardsPercentage::<T>::insert(subnet_id, value);

    Ok(())
  }

  pub fn do_owner_update_max_registered_nodes(
    origin: T::RuntimeOrigin, 
    subnet_id: u32, 
    value: u32
  ) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    ensure!(
    	value >= MinMaxRegisteredNodes::<T>::get(),
    	Error::<T>::InvalidMaxRegisteredNodes
    );

    MaxRegisteredNodes::<T>::insert(subnet_id, value);

    Ok(())
  }


  /// Initiates the transfer of a subnet's ownership to a new account using a 2-step model.
  ///
  /// This function can only be called by the current owner of the subnet.  
  /// It sets a pending owner, who must later explicitly accept the transfer via
  /// [`do_accept_subnet_ownership`]. Ownership is not transferred immediately.
  ///
  /// # Parameters
  /// - `origin`: The caller, must be the current subnet owner.
  /// - `subnet_id`: The ID of the subnet being transferred.
  /// - `new_owner`: The `AccountId` of the new proposed owner.
  ///
  /// # Undoing a Transfer
  /// To cancel a pending transfer, the current owner may call this function
  /// again with a zero address, effectively invalidating the pending owner.
  ///
  /// # Errors
  /// - [`NotSubnetOwner`]: Caller is not the owner of the subnet.
  pub fn do_transfer_subnet_ownership(origin: T::RuntimeOrigin, subnet_id: u32, new_owner: T::AccountId) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    PendingSubnetOwner::<T>::insert(subnet_id, new_owner);

    Ok(())
  }

  /// Accepts ownership of a subnet that was previously offered via a transfer.
  ///
  /// This function must be called by the account set as the `PendingSubnetOwner`
  /// for the specified subnet. Upon successful execution, the caller becomes
  /// the new `SubnetOwner`.
  ///
  /// # Parameters
  /// - `origin`: The caller, must match the pending owner.
  /// - `subnet_id`: The ID of the subnet being claimed.
  ///
  /// # Errors
  /// - [`NoPendingSubnetOwner`]: No transfer was initiated.
  /// - [`NotPendingSubnetOwner`]: Caller is not the designated pending owner.
  /// - [`InvalidSubnetId`]: Subnet does not exist or has no registered owner.
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

    PendingSubnetOwner::<T>::remove(subnet_id);

    Ok(())
  }
}