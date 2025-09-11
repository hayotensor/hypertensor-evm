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
use sp_runtime::Saturating;

impl<T: Config> Pallet<T> {
    pub fn do_add_overwatch_stake(
        coldkey: T::AccountId,
        hotkey: T::AccountId,
        stake_to_be_added: u128,
    ) -> DispatchResult {
        let stake_as_balance = Self::u128_to_balance(stake_to_be_added);

        ensure!(
            stake_as_balance.is_some(),
            Error::<T>::CouldNotConvertToBalance
        );

        let account_stake_balance: u128 = AccountOverwatchStake::<T>::get(&hotkey);

        ensure!(
            account_stake_balance.saturating_add(stake_to_be_added)
                >= OverwatchMinStakeBalance::<T>::get(),
            Error::<T>::MinStakeNotReached
        );

        // --- Ensure the callers coldkey has enough stake to perform the transaction.
        ensure!(
            Self::can_remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()),
            Error::<T>::NotEnoughBalanceToStake
        );

        // --- Ensure the remove operation from the coldkey is a success.
        ensure!(
            Self::remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()) == true,
            Error::<T>::BalanceWithdrawalError
        );

        Self::increase_account_overwatch_stake(&hotkey, stake_to_be_added);

        // Self::deposit_event(Event::StakeAdded(subnet_id, coldkey, hotkey, stake_to_be_added));

        Ok(())
    }

    pub fn do_remove_overwatch_stake(
        origin: T::RuntimeOrigin,
        hotkey: T::AccountId,
        is_overwatch_node: bool,
        stake_to_be_removed: u128,
    ) -> DispatchResult {
        let coldkey: T::AccountId = ensure_signed(origin)?;

        // --- Ensure that the stake amount to be removed is above zero.
        ensure!(stake_to_be_removed > 0, Error::<T>::AmountZero);

        let account_stake_balance: u128 = AccountOverwatchStake::<T>::get(&hotkey);

        // --- Ensure that the account has enough stake to withdraw.
        ensure!(
            account_stake_balance >= stake_to_be_removed,
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // if user is still an overwatch node they must keep the required minimum balance
        if is_overwatch_node {
            ensure!(
                account_stake_balance.saturating_sub(stake_to_be_removed)
                    >= OverwatchMinStakeBalance::<T>::get(),
                Error::<T>::MinStakeNotReached
            );
        }

        // --- Ensure that we can convert this u128 to a balance.
        let stake_to_be_removed_as_currency = Self::u128_to_balance(stake_to_be_removed);
        ensure!(
            stake_to_be_removed_as_currency.is_some(),
            Error::<T>::CouldNotConvertToBalance
        );

        let block: u32 = Self::get_current_block_as_u32();

        // --- 7. We remove the balance from the hotkey.
        Self::decrease_account_overwatch_stake(&hotkey, stake_to_be_removed);

        // --- 9. We add the balancer to the coldkey.  If the above fails we will not credit this coldkey.
        // Self::add_balance_to_unbonding_ledger(
        //   &coldkey,
        //   stake_to_be_removed,
        //   StakeCooldownEpochs::<T>::get(),
        //   block
        // ).map_err(|e| e)?;

        let result = Self::add_balance_to_unbonding_ledger_v2(
            &coldkey,
            stake_to_be_removed,
            StakeCooldownEpochs::<T>::get() * T::EpochLength::get(),
            block,
        )
        .map_err(|e| e)?;

        // Self::deposit_event(Event::StakeRemoved(subnet_id, coldkey, hotkey, stake_to_be_removed));

        Ok(())
    }

    pub fn do_swap_overwatch_hotkey_balance(old_hotkey: &T::AccountId, new_hotkey: &T::AccountId) {
        Self::swap_account_overwatch_stake(old_hotkey, new_hotkey)
    }

    pub fn increase_account_overwatch_stake(hotkey: &T::AccountId, amount: u128) {
        // -- increase account overwatch staking balance
        AccountOverwatchStake::<T>::mutate(hotkey, |mut n| n.saturating_accrue(amount));

        // -- increase total overwatch stake
        TotalOverwatchStake::<T>::mutate(|mut n| n.saturating_accrue(amount));
    }

    pub fn decrease_account_overwatch_stake(hotkey: &T::AccountId, amount: u128) {
        // -- decrease account overwatch staking balance
        AccountOverwatchStake::<T>::mutate(hotkey, |mut n| n.saturating_reduce(amount));

        // -- decrease total overwatch stake
        TotalOverwatchStake::<T>::mutate(|mut n| n.saturating_reduce(amount));
    }

    fn swap_account_overwatch_stake(old_hotkey: &T::AccountId, new_hotkey: &T::AccountId) {
        // --- swap old_hotkey overwatch staking balance
        let old_hotkey_stake_balance = AccountOverwatchStake::<T>::take(old_hotkey);
        // --- Redundant take of new hotkeys stake balance
        // --- New hotkey is always checked before updating
        let new_hotkey_stake_balance = AccountOverwatchStake::<T>::take(new_hotkey);
        AccountOverwatchStake::<T>::insert(
            new_hotkey,
            old_hotkey_stake_balance.saturating_add(new_hotkey_stake_balance),
        );
    }
}
