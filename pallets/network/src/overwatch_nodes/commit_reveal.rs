use super::*;
use sp_runtime::traits::Hash;

impl<T: Config> Pallet<T> {
  pub fn do_commit_ow_weights(
    origin: T::RuntimeOrigin,
    overwatch_node_id: u32,
    mut commit_weights: Vec<OverwatchCommit<T::Hash>>,
  ) -> DispatchResult {
    let key: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_overwatch_node_keys_owner(
        overwatch_node_id, 
        key, 
      ),
      Error::<T>::NotKeyOwner
    );

    // --- Check if we are in commit period
    ensure!(
      Self::in_overwatch_commit_period(),
      Error::<T>::NotCommitPeriod
    );

    Self::perform_commit_ow_weights(
      overwatch_node_id,
      commit_weights,
    )
  }

  pub fn perform_commit_ow_weights(
    overwatch_node_id: u32,
    mut commit_weights: Vec<OverwatchCommit<T::Hash>>,
  ) -> DispatchResult {
    // Remove dups
    commit_weights.dedup_by(|a, b| a.subnet_id == b.subnet_id);

    let subnets: BTreeSet<_> = SubnetsData::<T>::iter().map(|(id, _)| id).collect();

    // Qualify IDs - remove subnet IDs that do not exist
    commit_weights.retain(|x| subnets.contains(&x.subnet_id));

    ensure!(
      !commit_weights.is_empty(),
      Error::<T>::CommitsEmpty
    );

    let overwatch_epoch = Self::get_current_overwatch_epoch_as_u32();

    for commit in commit_weights {
      ensure!(
        !OverwatchCommits::<T>::contains_key((overwatch_epoch, overwatch_node_id, commit.subnet_id)),
        Error::<T>::AlreadyCommitted
      );

      OverwatchCommits::<T>::insert(
        (overwatch_epoch, overwatch_node_id, commit.subnet_id),
        commit.weight,
      );
    }

    Ok(())
  }

  pub fn do_reveal_ow_weights(
    origin: T::RuntimeOrigin,
    overwatch_node_id: u32,
    reveals: Vec<OverwatchReveal>,
  ) -> DispatchResult {
    let key: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_overwatch_node_keys_owner(
        overwatch_node_id, 
        key, 
      ),
      Error::<T>::NotKeyOwner
    );

    // --- Check if we are in reveal period
    ensure!(
      !Self::in_overwatch_commit_period(),
      Error::<T>::NotRevealPeriod
    );

    Self::perform_reveal_ow_weights(
      overwatch_node_id,
      reveals,
    )
  }

  pub fn perform_reveal_ow_weights(
    overwatch_node_id: u32,
    reveals: Vec<OverwatchReveal>,
  ) -> DispatchResult {
    let overwatch_epoch = Self::get_current_overwatch_epoch_as_u32();
    let percentage_factor = Self::percentage_factor_as_u128();

    for reveal in reveals {
      let subnet_id = reveal.subnet_id;
      let weight = reveal.weight;
      ensure!(weight <= percentage_factor, Error::<T>::InvalidWeight);
      let salt = reveal.salt;
      let Some(commit_hash) = OverwatchCommits::<T>::get((overwatch_epoch, overwatch_node_id, subnet_id)) else {
        return Err(Error::<T>::NoCommitFound.into());
      };

      // Reconstruct hash from reveal
      let actual_hash = T::Hashing::hash_of(&(weight, salt.clone()));

      ensure!(
        actual_hash == commit_hash,
        Error::<T>::RevealMismatch
      );

      OverwatchReveals::<T>::insert((overwatch_epoch, subnet_id, overwatch_node_id), weight);
    }

    Ok(())
  }
}