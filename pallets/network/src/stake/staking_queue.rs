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
use sp_core::U256;

impl<T: Config> Pallet<T> {
    /// Queue a swap call
    ///
    /// # Description
    ///
    /// Queues a swap call to be executed after a certain number of blocks.
    ///
    /// Only callable by
    /// - `do_swap_delegate_stake`
    /// - `do_swap_node_delegate_stake`
    /// - `do_swap_from_node_to_subnet`
    /// - `do_swap_from_subnet_to_node`
    ///
    /// # Arguments
    ///
    /// * `account_id` - Account ID of the caller.
    /// * `call` - Swap call to queue.
    ///
    pub fn queue_swap(
        account_id: T::AccountId,
        call: QueuedSwapCall<T::AccountId>,
    ) -> DispatchResult {
        let id = NextSwapQueueId::<T>::get();

        let queued_item = QueuedSwapItem {
            id,
            call: call.clone(),
            queued_at_block: Self::get_current_block_as_u32(),
            execute_after_blocks: T::EpochLength::get(),
        };

        // Add to data storage
        SwapCallQueue::<T>::insert(&id, &queued_item);

        // Add ID to the end of the queue
        SwapQueueOrder::<T>::mutate(|queue| {
            let _ = queue.try_push(id); // Handle error if queue is full
        });

        NextSwapQueueId::<T>::mutate(|next_id| *next_id = next_id.saturating_add(1));

        Self::deposit_event(Event::SwapCallQueued {
            id,
            account_id,
            call: call.clone(),
        });

        Ok(())
    }

    pub fn do_update_swap_queue(
        key: T::AccountId,
        id: u32,
        new_call: QueuedSwapCall<T::AccountId>,
    ) -> DispatchResult {
        SwapCallQueue::<T>::mutate(&id, |item_opt| -> DispatchResult {
            let item = item_opt.as_mut().ok_or(Error::<T>::SwapCallNotFound)?;
            let call_balance = item.call.get_queue_balance();

            match new_call {
                QueuedSwapCall::SwapToSubnetDelegateStake {
                    account_id,
                    to_subnet_id,
                    balance: _,
                } => {
                    ensure!(&account_id == &key, Error::<T>::NotKeyOwner);
                    ensure!(
                        SubnetsData::<T>::contains_key(to_subnet_id),
                        Error::<T>::InvalidSubnetId
                    );

                    // Update queue balance "to" subnet
                    item.call = QueuedSwapCall::SwapToSubnetDelegateStake {
                        account_id,
                        to_subnet_id,
                        balance: call_balance,
                    };

                    Self::deposit_event(Event::SwapCallQueueUpdated {
                        id,
                        account_id: key,
                        call: item.call.clone(),
                    });
                }
                QueuedSwapCall::SwapToNodeDelegateStake {
                    account_id,
                    to_subnet_id,
                    to_subnet_node_id,
                    balance: _,
                } => {
                    ensure!(&account_id == &key, Error::<T>::NotKeyOwner);
                    ensure!(
                        Self::get_subnet_node(to_subnet_id, to_subnet_node_id,).is_some(),
                        Error::<T>::InvalidSubnetNodeId
                    );

                    // Update queue balance "to" subnet node
                    item.call = QueuedSwapCall::SwapToNodeDelegateStake {
                        account_id,
                        to_subnet_id,
                        to_subnet_node_id,
                        balance: call_balance,
                    };

                    Self::deposit_event(Event::SwapCallQueueUpdated {
                        id,
                        account_id: key,
                        call: item.call.clone(),
                    });
                }
            }
            Ok(())
        })?;

        Ok(())
    }
}
