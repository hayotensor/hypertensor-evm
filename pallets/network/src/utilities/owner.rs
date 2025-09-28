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
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        ensure!(
            Self::is_subnet_active(subnet_id).unwrap_or(false),
            Error::<T>::SubnetMustBeActive
        );

        let epoch = Self::get_current_epoch_as_u32();

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

        Self::deposit_event(Event::SubnetUnpaused {
            subnet_id: subnet_id,
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

        Self::do_remove_subnet_v2(subnet_id, SubnetRemovalReason::Owner);

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

    // pub fn do_owner_update_activation_grace_epochs(
    //     origin: T::RuntimeOrigin,
    //     subnet_id: u32,
    //     value: u32,
    // ) -> DispatchResult {
    //     let coldkey: T::AccountId = ensure_signed(origin)?;

    //     ensure!(
    //         Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
    //         Error::<T>::NotSubnetOwner
    //     );

    //     ensure!(
    //         value >= MinActivationGraceEpochs::<T>::get()
    //             && value <= MaxActivationGraceEpochs::<T>::get(),
    //         Error::<T>::InvalidActivationGraceEpochs
    //     );

    //     ActivationGraceEpochs::<T>::insert(subnet_id, value);

    //     Self::deposit_event(Event::ActivationGraceEpochsUpdate {
    //         subnet_id: subnet_id,
    //         owner: coldkey,
    //         value: value,
    //     });

    //     Ok(())
    // }

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

    pub fn do_owner_update_max_node_penalties(
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
            value >= MinMaxSubnetNodePenalties::<T>::get()
                && value <= MaxMaxSubnetNodePenalties::<T>::get(),
            Error::<T>::InvalidMaxSubnetNodePenalties
        );

        MaxSubnetNodePenalties::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::MaxSubnetNodePenaltiesUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value: value,
        });

        Ok(())
    }

    pub fn do_owner_add_initial_coldkeys(
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

        SubnetRegistrationInitialColdkeys::<T>::mutate(subnet_id, |accounts| {
            let accounts_set = accounts.get_or_insert_with(BTreeSet::new);
            accounts_set.extend(coldkeys.iter().cloned());
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

    // pub fn do_owner_update_node_removal_reputation_score_percentage_delta(origin: T::RuntimeOrigin, subnet_id: u32, value: u128) -> DispatchResult {
    //   let coldkey: T::AccountId = ensure_signed(origin)?;

    //   ensure!(
    //     Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
    //     Error::<T>::NotSubnetOwner
    //   );

    //   ensure!(
    //     value <= Self::percentage_factor_as_u128(),
    //     Error::<T>::InvalidNodeRemovalReputationScorePercentageDelta
    //   );

    //   NodeRemovalReputationScorePercentageDelta::<T>::insert(subnet_id, value);

    //   Ok(())
    // }

    // pub fn do_owner_update_node_removal_reputation_score_min(origin: T::RuntimeOrigin, subnet_id: u32, value: u128) -> DispatchResult {
    //   let coldkey: T::AccountId = ensure_signed(origin)?;

    //   ensure!(
    //     Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
    //     Error::<T>::NotSubnetOwner
    //   );

    //   ensure!(
    //     value <= Self::percentage_factor_as_u128(),
    //     Error::<T>::InvalidNodeRemovalReputationScorePercentageDelta
    //   );

    //   NodeRemovalReputationScoreMin::<T>::insert(subnet_id, value);

    //   Ok(())
    // }

    /// Owner activates to remove subnet node from its subnet
    ///
    /// This function can only be called by the current owner of the subnet.  
    ///
    /// This pushes a registered node to the front of the queue and activates them on the following epoch
    ///
    /// # Parameters
    /// - `origin`: The caller, must be the current subnet owner.
    /// - `subnet_id`: The ID of the subnet.
    /// - `activate_subnet_node_id`: The ID of the subnet node to activate.
    /// - `remove_subnet_node_id`: The ID of the subnet node to remove.
    ///       - Only removes node if the subnet is at maximum node slots.
    ///
    /// # Errors
    /// - [`NotSubnetOwner`]: Caller is not the owner of the subnet.
    /// - [`SubnetNotActive`]: Subnet must be active.
    pub fn do_owner_activate_subnet_node(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        activate_subnet_node_id: u32,
        remove_subnet_node_id: u32,
    ) -> DispatchResult {
        // must have EnableOwnerActivation?
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        let subnet = match SubnetsData::<T>::try_get(subnet_id) {
            Ok(subnet) => subnet,
            Err(()) => return Err(Error::<T>::InvalidSubnetId.into()),
        };

        let subnet_epoch: u32 = Self::get_current_subnet_epoch_as_u32(subnet_id);

        ensure!(
            subnet.state == SubnetState::Active,
            Error::<T>::SubnetNotActive
        );

        let total_nodes = TotalActiveSubnetNodes::<T>::get(subnet_id);
        let max_nodes = MaxSubnetNodes::<T>::get();

        // --- If subnet is full, the owner must supply a node to remove
        if total_nodes >= max_nodes {
            // --- Ensure node exists
            let subnet_node = match SubnetNodesData::<T>::try_get(subnet_id, remove_subnet_node_id)
            {
                Ok(subnet_node) => subnet_node,
                Err(()) => return Err(Error::<T>::InvalidSubnetNodeId.into()),
            };
            // --- Remove node
            Self::perform_remove_subnet_node(subnet_id, remove_subnet_node_id);
        }

        let mut subnet_node =
            RegisteredSubnetNodesData::<T>::take(subnet_id, activate_subnet_node_id);
        let activating_subnet_node_coldkey = HotkeyOwner::<T>::get(&subnet_node.hotkey);

        Self::perform_activate_subnet_node(
            activating_subnet_node_coldkey,
            subnet_id,
            subnet.state,
            subnet_node,
            subnet_epoch,
        )
        .map_err(|e| e)?;

        Ok(())
    }

    // /// Owner calls to remove subnet node from its subnet
    // ///
    // /// This function can only be called by the current owner of the subnet.
    // ///
    // /// # Parameters
    // /// - `origin`: The caller, must be the current subnet owner.
    // /// - `subnet_id`: The ID of the subnet.
    // /// - `subnet_node_id`: The ID of the subnet node.
    // ///
    // /// # Errors
    // /// - [`NotSubnetOwner`]: Caller is not the owner of the subnet.
    // /// - See `perform_remove_subnet_node`
    // pub fn do_owner_remove_subnet_node(origin: T::RuntimeOrigin, subnet_id: u32, subnet_node_id: u32) -> DispatchResult {
    //   let coldkey: T::AccountId = ensure_signed(origin)?;

    //   ensure!(
    //     Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
    //     Error::<T>::NotSubnetOwner
    //   );

    //   Self::perform_remove_subnet_node(subnet_id, subnet_node_id);

    //   Ok(())
    // }

    /// Update minimum stake balance per subnet node
    ///
    /// This function can only be called by the current owner of the subnet.  
    ///
    /// # Parameters
    /// - `origin`: The caller, must be the current subnet owner.
    /// - `subnet_id`: The ID of the subnet.
    /// - `value`: *.
    ///
    /// # Errors
    /// - [`NotSubnetOwner`]: Caller is not the owner of the subnet.
    /// - [`InvalidSubnetMinStake`]: Value is not in allowable range.
    /// - [`InvalidSubnetStakeParameters`]: Min stake `value` is greater than max stake value, min must be less than max.
    pub fn do_owner_update_min_stake(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        // ensure!(
        //     value >= MinSubnetMinStake::<T>::get() && value <= MaxSubnetMinStake::<T>::get(),
        //     Error::<T>::InvalidSubnetMinStake
        // );
        ensure!(
            value >= NetworkMinStakeBalance::<T>::get()
                && value <= NetworkMaxStakeBalance::<T>::get(),
            Error::<T>::InvalidSubnetMinStake
        );

        // --- Can not be greater than max xstake
        ensure!(
            SubnetMaxStakeBalance::<T>::get(subnet_id) >= value,
            Error::<T>::InvalidSubnetStakeParameters
        );

        SubnetMinStakeBalance::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::SubnetMinStakeBalanceUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value: value,
        });

        Ok(())
    }

    /// Update maximum stake balance per subnet node
    ///
    /// This function can only be called by the current owner of the subnet.  
    ///
    /// # Parameters
    /// - `origin`: The caller, must be the current subnet owner.
    /// - `subnet_id`: The ID of the subnet.
    /// - `value`: *.
    ///
    /// # Errors
    /// - [`NotSubnetOwner`]: Caller is not the owner of the subnet.
    /// - [`InvalidSubnetMinStake`]: Value is not in allowable range.
    /// - [`InvalidSubnetStakeParameters`]: Max stake `value` is less than min stake value, max must be greater than min.
    pub fn do_owner_update_max_stake(
        origin: T::RuntimeOrigin,
        subnet_id: u32,
        value: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        ensure!(
            Self::is_subnet_owner(&coldkey, subnet_id).unwrap_or(false),
            Error::<T>::NotSubnetOwner
        );

        // ensure!(
        //     value >= MinSubnetMaxStake::<T>::get() && value <= MaxSubnetMaxStake::<T>::get(),
        //     Error::<T>::InvalidSubnetMaxStake
        // );
        ensure!(
            value >= NetworkMinStakeBalance::<T>::get()
                && value <= NetworkMaxStakeBalance::<T>::get(),
            Error::<T>::InvalidSubnetMaxStake
        );

        // Cannot be less than min stake
        ensure!(
            SubnetMinStakeBalance::<T>::get(subnet_id) <= value,
            Error::<T>::InvalidSubnetStakeParameters
        );

        SubnetMaxStakeBalance::<T>::insert(subnet_id, value);

        Self::deposit_event(Event::SubnetMaxStakeBalanceUpdate {
            subnet_id: subnet_id,
            owner: coldkey,
            value: value,
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

    pub fn do_owner_update_target_registrations_per_epoch(
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
            value <= max_registrations,
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
}
