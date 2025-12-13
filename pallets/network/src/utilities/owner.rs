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
use libm::{ceil, log};

impl<T: Config> Pallet<T> {
    /// Owner pause subnet for up to max period
    pub fn do_owner_pause_subnet(origin: T::RuntimeOrigin, subnet_id: u32) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            Self::is_subnet_active(subnet_id).unwrap_or(false),
            Error::<T>::SubnetMustBeActive
        );

        let epoch = Self::get_current_epoch_as_u32();

        // Ensure subnet pause period has been reached to pause again
        ensure!(
            PreviousSubnetPauseEpoch::<T>::get(subnet_id) + SubnetPauseCooldownEpochs::<T>::get()
                <= epoch,
            Error::<T>::SubnetPauseCooldownActive
        );

        SubnetsData::<T>::try_mutate_exists(subnet_id, |maybe_params| -> DispatchResult {
            let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetId)?;

            // Update state
            params.state = SubnetState::Paused;

            // We use the current epoch as the `start_epoch` when pausing
            // This enables us to know the delta when reactivating for updating the node registration pool node start epochs
            // see `do_owner_unpause_subnet`
            params.start_epoch = epoch;

            Ok(())
        })?;

        Self::deposit_event(Event::SubnetPaused {
            subnet_id: subnet_id,
            owner: coldkey,
        });

        Ok(())
    }

    pub fn do_owner_unpause_subnet(origin: T::RuntimeOrigin, subnet_id: u32) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            Self::is_subnet_paused(subnet_id).unwrap_or(false),
            Error::<T>::SubnetMustBePaused
        );

        let epoch = Self::get_current_epoch_as_u32();

        // If the subnet is passed the max pause epochs, validators via on_initialize already
        // unpaused it. If not, we allow the owner to unpause

        // A subnet can only pause if it's active, so we re-activate it back in the Active state
        SubnetsData::<T>::try_mutate_exists(subnet_id, |maybe_params| -> DispatchResult {
            let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetId)?;

            let pause_epoch = params.start_epoch;

            // Epochs the subnet was paused for
            let delta = epoch.saturating_sub(pause_epoch).saturating_add(1); // Add +1 to offset the subnet slots

            // Update each registration queued node
            for (subnet_id, uid, _) in RegisteredSubnetNodesData::<T>::iter() {
                RegisteredSubnetNodesData::<T>::mutate(subnet_id, uid, |subnet_node| {
                    let curr_start_epoch = subnet_node.classification.start_epoch;
                    subnet_node.classification.start_epoch = curr_start_epoch.saturating_add(delta);
                });
            }

            // Update state
            params.state = SubnetState::Active;

            params.start_epoch = epoch + 1;

            Ok(())
        })?;

        PreviousSubnetPauseEpoch::<T>::insert(subnet_id, epoch);

        // Modify max subnet epoch if owner called for emergency validators
        EmergencySubnetNodeElectionData::<T>::mutate_exists(subnet_id, |maybe_data| {
            if let Some(data) = maybe_data {
                data.max_emergency_validators_epoch = Self::get_current_subnet_epoch_as_u32(
                    subnet_id,
                )
                .saturating_add(Self::percent_mul(
                    data.target_emergency_validators_epochs as u128,
                    MaxEmergencyValidatorEpochsMultiplier::<T>::get(),
                ) as u32);
            }
        });

        Self::deposit_event(Event::SubnetUnpaused {
            subnet_id: subnet_id,
            owner: coldkey,
        });

        Ok(())
    }

    pub fn do_owner_set_emergency_validator_set(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        mut subnet_node_ids: Vec<u32>,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            Self::is_subnet_paused(subnet_id).unwrap_or(false),
            Error::<T>::SubnetMustBePaused
        );

        subnet_node_ids.dedup_by(|a, b| a == b);

        let subnet_epoch = Self::get_current_subnet_epoch_as_u32(subnet_id);

        subnet_node_ids.retain(|id| match SubnetNodesData::<T>::try_get(subnet_id, id) {
            Ok(subnet_node) => {
                subnet_node.has_classification(&SubnetNodeClass::Validator, subnet_epoch)
            }
            Err(()) => false,
        });

        ensure!(
            subnet_node_ids.len() as u32 >= MinSubnetNodes::<T>::get(),
            Error::<T>::InvalidMinEmergencySubnetNodes
        );

        ensure!(
            subnet_node_ids.len() as u32 <= MaxEmergencySubnetNodes::<T>::get(),
            Error::<T>::InvalidMaxEmergencySubnetNodes
        );

        let target_emergency_epochs = Self::get_max_steps_for_node_removal(subnet_id);

        EmergencySubnetNodeElectionData::<T>::insert(
            subnet_id,
            EmergencySubnetValidatorData {
                subnet_node_ids: subnet_node_ids.clone(),
                target_emergency_validators_epochs: target_emergency_epochs,
                total_epochs: 0,
                max_emergency_validators_epoch: 0,
            },
        );

        Self::deposit_event(Event::SubnetForked {
            subnet_id: subnet_id,
            owner: coldkey,
            subnet_node_ids,
        });

        Ok(())
    }

    /// Get the required epochs to have a node removed based on not being in consensus data
    /// based on the `AbsentDecreaseReputationFactor`
    fn get_max_steps_for_node_removal(subnet_id: u32) -> u32 {
        let one: f64 = Self::get_percent_as_f64(Self::percentage_factor_as_u128());

        // Based on network min max parameters
        let min_min_reputation: f64 =
            Self::get_percent_as_f64(MinMinSubnetNodeReputation::<T>::get());
        let min_absent_factor: f64 = Self::get_percent_as_f64(MinNodeReputationFactor::<T>::get());

        let r = one - min_absent_factor;
        let n = ceil(log(min_min_reputation / one) / log(r)) as u32 + 1;

        // Subnet parameters
        let min_reputation: f64 =
            Self::get_percent_as_f64(MinSubnetNodeReputation::<T>::get(subnet_id));
        let absent_factor: f64 =
            Self::get_percent_as_f64(AbsentDecreaseReputationFactor::<T>::get(subnet_id));

        let r2 = one - absent_factor;
        let n2 = ceil(log(min_reputation / one) / log(r2)) as u32 + 1;

        // Redundantly check steps
        if n < n2 {
            return n;
        }

        n2
    }

    pub fn do_owner_revert_emergency_validator_set(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        EmergencySubnetNodeElectionData::<T>::remove(subnet_id);

        Self::deposit_event(Event::SubnetForkRevert {
            subnet_id: subnet_id,
            owner: coldkey,
        });

        Ok(())
    }

    pub fn do_owner_deactivate_subnet(origin: T::RuntimeOrigin, subnet_id: u32) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        // Redundant
        ensure!(
            SubnetsData::<T>::contains_key(subnet_id),
            Error::<T>::InvalidSubnetId
        );

        Self::do_remove_subnet(subnet_id, SubnetRemovalReason::Owner);

        Ok(())
    }

    pub fn do_owner_update_name(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: Vec<u8>,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            !SubnetName::<T>::contains_key(&value),
            Error::<T>::SubnetNameExist
        );

        let mut prev_name: Vec<u8> = Vec::new();
        SubnetsData::<T>::try_mutate_exists(subnet_id, |maybe_params| -> DispatchResult {
            let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetId)?;

            prev_name = params.name.clone();

            SubnetName::<T>::remove(&prev_name);

            params.name = value.clone();

            Ok(())
        })?;

        SubnetName::<T>::insert(&value, subnet_id);

        Self::deposit_event(Event::SubnetNameUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            prev_value: prev_name,
            value: value,
        });

        Ok(())
    }

    pub fn do_owner_update_repo(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: Vec<u8>,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            !SubnetRepo::<T>::contains_key(&value),
            Error::<T>::SubnetRepoExist
        );

        let mut prev_repo: Vec<u8> = Vec::new();
        SubnetsData::<T>::try_mutate_exists(subnet_id, |maybe_params| -> DispatchResult {
            let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetId)?;

            prev_repo = params.repo.clone();

            SubnetRepo::<T>::remove(&prev_repo);

            params.repo = value.clone();

            Ok(())
        })?;

        SubnetRepo::<T>::insert(&value, subnet_id);

        Self::deposit_event(Event::SubnetRepoUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            prev_value: prev_repo,
            value: value,
        });

        Ok(())
    }

    pub fn do_owner_update_description(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: Vec<u8>,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        let mut prev_description: Vec<u8> = Vec::new();
        SubnetsData::<T>::try_mutate_exists(subnet_id, |maybe_params| -> DispatchResult {
            let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetId)?;

            prev_description = params.description.clone();
            params.description = value.clone();

            Ok(())
        })?;

        Self::deposit_event(Event::SubnetDescriptionUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            prev_value: prev_description,
            value: value,
        });

        Ok(())
    }

    pub fn do_owner_update_misc(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: Vec<u8>,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        let mut prev_misc: Vec<u8> = Vec::new();
        SubnetsData::<T>::try_mutate_exists(subnet_id, |maybe_params| -> DispatchResult {
            let params = maybe_params.as_mut().ok_or(Error::<T>::InvalidSubnetId)?;

            prev_misc = params.misc.clone();
            params.misc = value.clone();

            Ok(())
        })?;

        Self::deposit_event(Event::SubnetMiscUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            prev_value: prev_misc,
            value: value,
        });

        Ok(())
    }

    pub fn do_owner_update_churn_limit(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u32,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value >= MinChurnLimit::<T>::get() && value <= MaxChurnLimit::<T>::get(),
            Error::<T>::InvalidChurnLimit
        );

        ChurnLimit::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::ChurnLimitUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value: value,
        });

        Ok(())
    }

    pub fn do_owner_update_churn_limit_multiplier(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u32,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value >= MinChurnLimitMultiplier::<T>::get()
                && value <= MaxChurnLimitMultiplier::<T>::get(),
            Error::<T>::InvalidChurnLimitMultiplier
        );

        ChurnLimitMultiplier::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::ChurnLimitMultiplierUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value: value,
        });

        Ok(())
    }

    pub fn do_owner_update_registration_queue_epochs(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u32,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value >= MinQueueEpochs::<T>::get() && value <= MaxQueueEpochs::<T>::get(),
            Error::<T>::InvalidRegistrationQueueEpochs
        );

        SubnetNodeQueueEpochs::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::RegistrationQueueEpochsUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value: value,
        });

        Ok(())
    }

    pub fn do_owner_update_idle_classification_epochs(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u32,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value >= MinIdleClassificationEpochs::<T>::get()
                && value <= MaxIdleClassificationEpochs::<T>::get(),
            Error::<T>::InvalidIdleClassificationEpochs
        );

        IdleClassificationEpochs::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::IdleClassificationEpochsUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value: value,
        });

        Ok(())
    }

    pub fn do_owner_update_included_classification_epochs(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u32,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value >= MinIncludedClassificationEpochs::<T>::get()
                && value <= MaxIncludedClassificationEpochs::<T>::get(),
            Error::<T>::InvalidIncludedClassificationEpochs
        );

        IncludedClassificationEpochs::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::IncludedClassificationEpochsUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value: value,
        });

        Ok(())
    }

    pub fn do_owner_add_or_update_initial_coldkeys(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        coldkeys: BTreeMap<T::AccountId, u32>,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            !Self::is_subnet_active(subnet_id).unwrap_or(false),
            Error::<T>::SubnetMustBeRegistering
        );

        ensure!(
            coldkeys.values().all(|&value| value >= 1),
            Error::<T>::InvalidSubnetRegistrationInitialColdkeys
        );

        SubnetRegistrationInitialColdkeys::<T>::mutate(subnet_id, |accounts| {
            let accounts_set = accounts.get_or_insert_with(BTreeMap::new);
            accounts_set.extend(coldkeys.iter().map(|(k, v)| (k.clone(), *v)));
        });

        Self::deposit_event(Event::AddSubnetRegistrationInitialColdkeys {
            subnet_id: subnet_id,
            owner: coldkey,
            coldkeys: coldkeys,
        });

        Ok(())
    }

    pub fn do_owner_remove_initial_coldkeys(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        coldkeys: BTreeSet<T::AccountId>,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            !Self::is_subnet_active(subnet_id).unwrap_or(false),
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

        Self::deposit_event(Event::RemoveSubnetRegistrationInitialColdkeys {
            subnet_id: subnet_id,
            owner: coldkey,
            coldkeys: coldkeys,
        });

        Ok(())
    }

    pub fn do_owner_update_key_types(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: BTreeSet<KeyType>,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        SubnetKeyTypes::<T>::insert(subnet_id, &value);

        Self::deposit_event(Event::SubnetKeyTypesUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value: value,
        });

        Ok(())
    }

    pub fn do_owner_update_min_max_stake(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        min: u128,
        max: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(min <= max, Error::<T>::InvalidValues);

        ensure!(
            min >= MinSubnetMinStake::<T>::get() && min <= MaxSubnetMinStake::<T>::get(),
            Error::<T>::InvalidSubnetMinStake
        );

        ensure!(
            max <= NetworkMaxStakeBalance::<T>::get(),
            Error::<T>::InvalidSubnetMaxStake
        );

        SubnetMinStakeBalance::<T>::insert(subnet_id, min);
        SubnetMaxStakeBalance::<T>::insert(subnet_id, max);

        Self::deposit_event(Event::SubnetMinMaxStakeBalanceUpdate {
            subnet_id: subnet_id,
            owner: coldkey.clone(),
            min: min,
            max: max,
        });

        Ok(())
    }

    /// Update delegate stake percentage
    ///
    /// This function can only be called by the current owner of the subnet.  
    ///
    /// # Parameters
    /// - `origin`: The caller, must be the current subnet owner.
    /// - `subnet_id`: The ID of the subnet.
    /// - `value`: The new percentage (1e18 = 1.0) share of rewards to delegate stakers.
    ///
    /// # Errors
    /// - [`NotSubnetOwner`]: Caller is not the owner of the subnet.
    /// - [`DelegateStakePercentageUpdateTooSoon`]: Updated too soon.
    /// - [`DelegateStakePercentageAbsDiffTooLarge`]: Value change too large.
    /// - [`InvalidDelegateStakePercentage`]: Value is not in allowable range.
    pub fn do_owner_update_delegate_stake_percentage(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        let block = Self::get_current_block_as_u32();
        let last_update = LastSubnetDelegateStakeRewardsUpdate::<T>::get(subnet_id);
        let update_period = SubnetDelegateStakeRewardsUpdatePeriod::<T>::get();

        ensure!(
            last_update + update_period < block,
            Error::<T>::DelegateStakePercentageUpdateTooSoon
        );

        let current_rate = SubnetDelegateStakeRewardsPercentage::<T>::get(subnet_id);
        let max_change = MaxSubnetDelegateStakeRewardsPercentageChange::<T>::get();

        ensure!(
            current_rate.abs_diff(value) <= max_change,
            Error::<T>::DelegateStakePercentageAbsDiffTooLarge
        );

        ensure!(
            value >= MinDelegateStakePercentage::<T>::get()
                && value <= MaxDelegateStakePercentage::<T>::get()
                && value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidDelegateStakePercentage
        );

        LastSubnetDelegateStakeRewardsUpdate::<T>::insert(subnet_id, block);
        SubnetDelegateStakeRewardsPercentage::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::SubnetDelegateStakeRewardsPercentageUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value: value,
        });

        Ok(())
    }

    /// Update maximum registered nodes
    ///
    /// This function can only be called by the current owner of the subnet.  
    ///
    /// # Parameters
    /// - `origin`: The caller, must be the current subnet owner.
    /// - `subnet_id`: The ID of the subnet.
    /// - `value`: The new number maximum registered nodes.
    ///
    /// # Errors
    /// - [`NotSubnetOwner`]: Caller is not the owner of the subnet.
    /// - [`InvalidMaxRegisteredNodes`]: Value is not in allowable range.
    pub fn do_owner_update_max_registered_nodes(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u32,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value >= MinMaxRegisteredNodes::<T>::get()
                && value <= MaxMaxRegisteredNodes::<T>::get()
                && value <= TargetNodeRegistrationsPerEpoch::<T>::get(subnet_id),
            Error::<T>::InvalidMaxRegisteredNodes
        );

        MaxRegisteredNodes::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::MaxRegisteredNodesUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value: value,
        });

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
    /// - `subnet_id`: The ID of the subnet.
    /// - `new_owner`: The `AccountId` of the new proposed owner.
    ///
    /// # Undoing a Transfer
    /// To cancel a pending transfer, the current owner may call this function
    /// again with a zero address, effectively invalidating the pending owner.
    ///
    /// # Errors
    /// - [`NotSubnetOwner`]: Caller is not the owner of the subnet.
    pub fn do_transfer_subnet_ownership(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        new_owner: T::AccountId,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        PendingSubnetOwner::<T>::insert(subnet_id, &new_owner);

        Self::deposit_event(Event::TransferPendingSubnetOwner {
            subnet_id: subnet_id,
            owner: coldkey,
            new_owner: new_owner,
        });

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

        ensure!(coldkey == pending_owner, Error::<T>::NotPendingSubnetOwner);

        SubnetOwner::<T>::try_mutate_exists(subnet_id, |maybe_owner| -> DispatchResult {
            let owner = maybe_owner.as_mut().ok_or(Error::<T>::InvalidSubnetId)?;
            *owner = pending_owner;
            Ok(())
        })?;

        PendingSubnetOwner::<T>::remove(subnet_id);

        Self::deposit_event(Event::AcceptPendingSubnetOwner {
            subnet_id: subnet_id,
            new_owner: coldkey,
        });

        Ok(())
    }

    pub fn do_owner_add_bootnode_access(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        new_account: T::AccountId,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        Self::deposit_event(Event::AddSubnetBootnodeAccess {
            subnet_id: subnet_id,
            owner: coldkey,
            new_account: new_account.clone(),
        });

        SubnetBootnodeAccess::<T>::try_mutate(subnet_id, |access_list| {
            if !access_list.insert(new_account) {
                return Err(Error::<T>::InBootnodeAccessList.into());
            }
            ensure!(
                access_list.len() <= MaxSubnetBootnodeAccess::<T>::get() as usize,
                Error::<T>::MaxSubnetBootnodeAccess
            );
            Ok(())
        })
    }

    pub fn do_owner_remove_bootnode_access(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        remove_account: T::AccountId,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        Self::deposit_event(Event::RemoveSubnetBootnodeAccess {
            subnet_id: subnet_id,
            owner: coldkey,
            remove_account: remove_account.clone(),
        });

        SubnetBootnodeAccess::<T>::try_mutate(subnet_id, |access_list| {
            if !access_list.remove(&remove_account) {
                return Err(Error::<T>::NotInAccessList.into());
            }
            Ok(())
        })
    }

    pub fn do_owner_update_target_node_registrations_per_epoch(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u32,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        let max_registrations = MaxRegisteredNodes::<T>::get(subnet_id);
        ensure!(
            value <= max_registrations && value > 0,
            Error::<T>::InvalidTargetNodeRegistrationsPerEpoch
        );

        TargetNodeRegistrationsPerEpoch::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::TargetNodeRegistrationsPerEpochUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value,
        });

        Ok(())
    }

    pub fn do_owner_update_node_burn_rate_alpha(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        NodeBurnRateAlpha::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::NodeBurnRateAlphaUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value,
        });

        Ok(())
    }

    pub fn do_owner_update_queue_immunity_epochs(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u32,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        QueueImmunityEpochs::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::QueueImmunityEpochsUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value,
        });

        Ok(())
    }

    pub fn do_owner_update_min_subnet_node_reputation(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        ensure!(
            value >= MinMinSubnetNodeReputation::<T>::get()
                && value <= MaxMinSubnetNodeReputation::<T>::get(),
            Error::<T>::MinSubnetNodeReputation
        );

        MinSubnetNodeReputation::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::MinSubnetNodeReputationUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value,
        });

        Ok(())
    }

    pub fn do_owner_update_subnet_node_min_weight_decrease_reputation_threshold(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value <= MaxSubnetNodeMinWeightDecreaseReputationThreshold::<T>::get(),
            Error::<T>::InvalidPercent
        );

        SubnetNodeMinWeightDecreaseReputationThreshold::<T>::insert(subnet_id, value);

        Self::deposit_event(
            Event::SubnetNodeMinWeightDecreaseReputationThresholdUpdate {
                subnet_id: subnet_id,
                owner: coldkey,
                value,
            },
        );

        Ok(())
    }

    pub fn do_owner_update_absent_decrease_reputation_factor(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        ensure!(
            value >= MinNodeReputationFactor::<T>::get()
                && value <= MaxNodeReputationFactor::<T>::get(),
            Error::<T>::InvalidAbsentDecreaseReputationFactor
        );

        ensure!(
            !EmergencySubnetNodeElectionData::<T>::contains_key(subnet_id),
            Error::<T>::EmergencyValidatorsSet
        );

        AbsentDecreaseReputationFactor::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::AbsentDecreaseReputationFactorUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value,
        });

        Ok(())
    }

    pub fn do_owner_update_included_increase_reputation_factor(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        ensure!(
            value >= MinNodeReputationFactor::<T>::get()
                && value <= MaxNodeReputationFactor::<T>::get(),
            Error::<T>::InvalidIncludedIncreaseReputationFactor
        );

        IncludedIncreaseReputationFactor::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::IncludedIncreaseReputationFactorUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value,
        });

        Ok(())
    }

    pub fn do_owner_update_below_min_weight_decrease_reputation_factor(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        ensure!(
            value >= MinNodeReputationFactor::<T>::get()
                && value <= MaxNodeReputationFactor::<T>::get(),
            Error::<T>::InvalidBelowMinWeightDecreaseReputationFactor
        );

        ensure!(
            !EmergencySubnetNodeElectionData::<T>::contains_key(subnet_id),
            Error::<T>::EmergencyValidatorsSet
        );

        BelowMinWeightDecreaseReputationFactor::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::BelowMinWeightDecreaseReputationFactorUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value,
        });

        Ok(())
    }

    pub fn do_owner_update_non_attestor_decrease_reputation_factor(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        ensure!(
            value >= MinNodeReputationFactor::<T>::get()
                && value <= MaxNodeReputationFactor::<T>::get(),
            Error::<T>::InvalidNonAttestorDecreaseReputationFactor
        );

        ensure!(
            !EmergencySubnetNodeElectionData::<T>::contains_key(subnet_id),
            Error::<T>::EmergencyValidatorsSet
        );

        NonAttestorDecreaseReputationFactor::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::NonAttestorDecreaseReputationFactorUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value,
        });

        Ok(())
    }

    pub fn do_owner_update_non_consensus_attestor_decrease_reputation_factor(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        ensure!(
            value >= MinNodeReputationFactor::<T>::get()
                && value <= MaxNodeReputationFactor::<T>::get(),
            Error::<T>::InvalidNonConsensusAttestorDecreaseReputationFactor
        );

        ensure!(
            !EmergencySubnetNodeElectionData::<T>::contains_key(subnet_id),
            Error::<T>::EmergencyValidatorsSet
        );

        NonConsensusAttestorDecreaseReputationFactor::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::NonConsensusAttestorDecreaseReputationFactorUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value,
        });

        Ok(())
    }

    pub fn do_owner_update_validator_absent_decrease_reputation_factor(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        ensure!(
            value >= MinNodeReputationFactor::<T>::get()
                && value <= MaxNodeReputationFactor::<T>::get(),
            Error::<T>::InvalidNonValidatorAbsentSubnetNodeReputationFactor
        );

        ensure!(
            !EmergencySubnetNodeElectionData::<T>::contains_key(subnet_id),
            Error::<T>::EmergencyValidatorsSet
        );

        ValidatorAbsentSubnetNodeReputationFactor::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::ValidatorAbsentSubnetNodeReputationFactorUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value,
        });

        Ok(())
    }

    pub fn do_owner_update_validator_non_consensus_decrease_reputation_factor(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        ensure!(
            value >= MinNodeReputationFactor::<T>::get()
                && value <= MaxNodeReputationFactor::<T>::get(),
            Error::<T>::InvalidValidatorNonConsensusSubnetNodeReputationFactor
        );

        ensure!(
            !EmergencySubnetNodeElectionData::<T>::contains_key(subnet_id),
            Error::<T>::EmergencyValidatorsSet
        );

        ValidatorNonConsensusSubnetNodeReputationFactor::<T>::insert(subnet_id, value);

        Self::deposit_event(
            Event::ValidatorNonConsensusSubnetNodeReputationFactorUpdate {
                subnet_id: subnet_id,
                owner: coldkey,
                value,
            },
        );

        Ok(())
    }
}
