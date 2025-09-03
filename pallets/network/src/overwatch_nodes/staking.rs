// // Copyright (C) Hypertensor.
// // SPDX-License-Identifier: Apache-2.0

// // Licensed under the Apache License, Version 2.0 (the "License");
// // you may not use this file except in compliance with the License.
// // You may obtain a copy of the License at
// //
// // 	http://www.apache.org/licenses/LICENSE-2.0
// //
// // Unless required by applicable law or agreed to in writing, software
// // distributed under the License is distributed on an "AS IS" BASIS,
// // WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// // See the License for the specific language governing permissions and
// // limitations under the License.

// use super::*;
// use sp_runtime::Saturating;

// impl<T: Config> Pallet<T> {
//   pub fn do_add_overwatch_stake(
//     origin: T::RuntimeOrigin,
//     hotkey: T::AccountId,
//     stake_to_be_added: u128,
//   ) -> DispatchResult {
//     let coldkey: T::AccountId = ensure_signed(origin)?;

//     let stake_as_balance = Self::u128_to_balance(stake_to_be_added);

//     ensure!(
//       stake_as_balance.is_some(),
//       Error::<T>::CouldNotConvertToBalance
//     );

//     let account_stake_balance: u128 = AccountOverwatchStake::<T>::get(&hotkey);

//     ensure!(
//       account_stake_balance.saturating_add(stake_to_be_added) >= MinOverwatchStakeBalance::<T>::get(),
//       Error::<T>::MinStakeNotReached
//     );

//     // --- Ensure the callers coldkey has enough stake to perform the transaction.
//     ensure!(
//       Self::can_remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()),
//       Error::<T>::NotEnoughBalanceToStake
//     );
  
//     // to-do: add AddStakeRateLimit instead of universal rate limiter
//     //        this allows peers to come in freely
//     let block: u64 = Self::get_current_block_as_u64();
//     ensure!(
//       !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
//       Error::<T>::TxRateLimitExceeded
//     );

//     // --- Ensure the remove operation from the coldkey is a success.
//     ensure!(
//       Self::remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()) == true,
//       Error::<T>::BalanceWithdrawalError
//     );
  
//     Self::increase_account_overwatch_stake(
//       &hotkey,
//       stake_to_be_added,
//     );

//     // Set last block for rate limiting
//     Self::set_last_tx_block(&coldkey, block);

//     // Self::deposit_event(Event::StakeAdded(coldkey, stake_to_be_added));

//     Ok(())
//   }

//   pub fn do_remove_overwatch_stake(
//     origin: T::RuntimeOrigin, 
//     hotkey: T::AccountId,
//     is_overwatch_node: bool,
//     stake_to_be_removed: u128,
//   ) -> DispatchResult {
//     let coldkey: T::AccountId = ensure_signed(origin)?;

//     // --- Ensure that the stake amount to be removed is above zero.
//     ensure!(
//       stake_to_be_removed > 0,
//       Error::<T>::NotEnoughStakeToWithdraw
//     );

//     let account_stake_balance: u128 = AccountOverwatchStake::<T>::get(&hotkey);

//     // --- Ensure that the account has enough stake to withdraw.
//     ensure!(
//       account_stake_balance >= stake_to_be_removed,
//       Error::<T>::NotEnoughStakeToWithdraw
//     );
    
//     // if user is still a subnet node they must keep the required minimum balance
//     if is_overwatch_node {
//       ensure!(
//         account_stake_balance.saturating_sub(stake_to_be_removed) >= MinOverwatchStakeBalance::<T>::get(),
//         Error::<T>::MinStakeNotReached
//       );  
//     }
  
//     // --- Ensure that we can convert this u128 to a balance.
//     let stake_to_be_removed_as_currency = Self::u128_to_balance(stake_to_be_removed);
//     ensure!(
//       stake_to_be_removed_as_currency.is_some(),
//         Error::<T>::CouldNotConvertToBalance
//     );

//     let block: u64 = Self::get_current_block_as_u64();
//     ensure!(
//       !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
//       Error::<T>::TxRateLimitExceeded
//     );

//     // --- 7. We remove the balance from the hotkey.
//     Self::decrease_account_overwatch_stake(&hotkey, stake_to_be_removed);

//     // --- 9. We add the balancer to the coldkey.  If the above fails we will not credit this coldkey.
//     // Self::add_balance_to_stake_unbonding_ledger(&coldkey, stake_to_be_removed, block).map_err(|e| e)?;

//     // Set last block for rate limiting
//     Self::set_last_tx_block(&coldkey, block);

//     // Self::deposit_event(Event::StakeRemoved(coldkey, stake_to_be_removed));

//     Ok(())
//   }

//   pub fn increase_account_overwatch_stake(
//     hotkey: &T::AccountId,
//     amount: u128,
//   ) {
//     // -- increase account subnet staking balance
//     AccountOverwatchStake::<T>::mutate(hotkey, |mut n| n.saturating_accrue(amount));

//     // -- increase total subnet stake
//     TotalOverwatchStake::<T>::mutate(|mut n| n.saturating_accrue(amount));
//   }
  
//   pub fn decrease_account_overwatch_stake(
//     hotkey: &T::AccountId,
//     amount: u128,
//   ) {
//     // -- decrease account subnet staking balance
//     AccountOverwatchStake::<T>::mutate(hotkey, |mut n| n.saturating_reduce(amount));

//     // -- decrease total subnet stake
//     TotalOverwatchStake::<T>::mutate(|mut n| n.saturating_reduce(amount));
//   }
// }