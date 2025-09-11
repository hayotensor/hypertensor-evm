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
    pub fn queue_swap(
        account_id: T::AccountId,
        call: QueuedSwapCall<T::AccountId>,
    ) -> DispatchResult {
        let id = NextSwapId::<T>::get();

        let queued_item = QueuedSwapItem {
            id,
            call,
            queued_at_block: Self::get_current_block_as_u32(),
            execute_after_blocks: T::EpochLength::get(),
        };
        
        // Add to data storage
        SwapCallQueue::<T>::insert(&id, &queued_item);
        
        // Add ID to the end of the queue
        SwapQueueOrder::<T>::mutate(|queue| {
            let _ = queue.try_push(id); // Handle error if queue is full
        });
        
        NextSwapId::<T>::mutate(|next_id| *next_id = next_id.saturating_add(1));
        
        // Self::deposit_event(Event::SwapCallQueued { id, who });

        Ok(())
    }
}
