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
    pub fn do_pause() -> DispatchResult {
        TxPause::<T>::put(true);
        Ok(())
    }

    pub fn do_unpause() -> DispatchResult {
        TxPause::<T>::put(false);
        Ok(())
    }

    pub fn do_set_proposal_min_subnet_nodes(value: u32) -> DispatchResult {
        ProposalMinSubnetNodes::<T>::put(value);
        Ok(())
    }

    pub fn do_set_subnet_owner_percentage(value: u128) -> DispatchResult {
        SubnetOwnerPercentage::<T>::put(value);
        Ok(())
    }

    pub fn do_set_max_subnets(value: u32) -> DispatchResult {
        // Account for the first 3 block steps in an epoch
        // Do not go over epoch length - 3 to ensure each subnet has a slot in each epoch
        ensure!(
            value <= T::EpochLength::get() - 3,
            Error::<T>::InvalidMaxSubnets
        );

        MaxSubnets::<T>::set(value);

        Self::deposit_event(Event::SetMaxSubnets(value));

        Ok(())
    }

    pub fn do_set_min_subnet_nodes(value: u32) -> DispatchResult {
        ensure!(
            value > 0 && value < MaxSubnetNodes::<T>::get(),
            Error::<T>::InvalidMinSubnetNodes
        );

        MinSubnetNodes::<T>::set(value);
        Self::deposit_event(Event::SetMinSubnetNodes(value));
        Ok(())
    }

    pub fn do_set_max_subnet_nodes(value: u32) -> DispatchResult {
        ensure!(
            value > MinSubnetNodes::<T>::get(),
            Error::<T>::InvalidMaxSubnetNodes
        );

        MaxSubnetNodes::<T>::set(value);

        Self::deposit_event(Event::SetMaxSubnetNodes(value));

        Ok(())
    }

    pub fn do_set_tx_rate_limit(value: u32) -> DispatchResult {
        TxRateLimit::<T>::set(value);

        Self::deposit_event(Event::SetTxRateLimit(value));

        Ok(())
    }

    pub fn do_set_min_subnet_delegate_stake_factor(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        MinSubnetDelegateStakeFactor::<T>::set(value);

        Self::deposit_event(Event::SetMinSubnetDelegateStakeFactor(value));

        Ok(())
    }

    pub fn do_collective_remove_subnet(subnet_id: u32) -> DispatchResult {
        Self::do_remove_subnet(subnet_id, SubnetRemovalReason::Council);
        Ok(())
    }

    pub fn do_collective_remove_subnet_node(subnet_id: u32, subnet_node_id: u32) -> DispatchResult {
        Self::do_remove_subnet_node(subnet_id, subnet_node_id)
    }

    pub fn do_collective_remove_overwatch_node(overwatch_node_id: u32) -> DispatchResult {
        Ok(())
    }

    /// Temporary solution until network maturity
    pub fn do_collective_blacklist_overwatch_node(coldkey: T::AccountId) -> DispatchResult {
        Ok(())
    }

    pub fn do_set_sigmoid_midpoint(value: u128) -> DispatchResult {
        ensure!(
            value <= Self::percentage_factor_as_u128(),
            Error::<T>::InvalidPercent
        );

        SigmoidMidpoint::<T>::put(value);

        Ok(())
    }

    pub fn do_set_sigmoid_steepness(value: u128) -> DispatchResult {
        SigmoidSteepness::<T>::put(value);
        Ok(())
    }

    pub fn do_set_overwatch_epoch_length_multiplier(value: u32) -> DispatchResult {
        OverwatchEpochLengthMultiplier::<T>::put(value);
        Ok(())
    }

    pub fn do_set_min_subnet_registration_epochs(value: u32) -> DispatchResult {
        let registration_epochs = SubnetRegistrationEpochs::<T>::get();
        // Must be less than the registration period itself
        ensure!(
            value < registration_epochs,
            Error::<T>::InvalidMinSubnetRegistrationEpochs
        );
        MinSubnetRegistrationEpochs::<T>::put(value);
        Ok(())
    }

    pub fn do_set_subnet_registration_epochs(value: u32) -> DispatchResult {
        let min_registration_epochs = MinSubnetRegistrationEpochs::<T>::get();
        ensure!(
            value > min_registration_epochs,
            Error::<T>::InvalidSubnetRegistrationEpochs
        );
        SubnetRegistrationEpochs::<T>::put(value);
        Ok(())
    }
}
