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

    // Remove dups
    commit_weights.dedup_by(|a, b| a.subnet_id == b.subnet_id);

    let subnets: BTreeSet<_> = SubnetsData::<T>::iter().map(|(id, _)| id).collect();

    // Qualify IDs
    commit_weights.retain(|x| subnets.contains(&x.subnet_id));

    ensure!(
      !commit_weights.is_empty(),
      Error::<T>::CommitsEmpty
    );

    let epoch: u32 = Self::get_current_epoch_as_u32();

    for commit in commit_weights {
      ensure!(
        !OverwatchCommits::<T>::contains_key((epoch, overwatch_node_id, commit.subnet_id)),
        Error::<T>::AlreadyCommitted
      );

      OverwatchCommits::<T>::insert(
        (epoch, overwatch_node_id, commit.subnet_id),
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

    let epoch: u32 = Self::get_current_epoch_as_u32();
    let percentage_factor = Self::percentage_factor_as_u128();

    for reveal in reveals {
      let subnet_id = reveal.subnet_id;
      let weight = reveal.weight;
      ensure!(weight <= percentage_factor, Error::<T>::InvalidWeight);
      let salt = reveal.salt;
      let Some(commit_hash) = OverwatchCommits::<T>::get((epoch, overwatch_node_id, subnet_id)) else {
        return Err(Error::<T>::NoCommitFound.into());
      };

      // Reconstruct hash from reveal
      let actual_hash = T::Hashing::hash_of(&(weight, salt.clone()));

      ensure!(
        actual_hash == commit_hash,
        Error::<T>::RevealMismatch
      );

      OverwatchReveals::<T>::insert((epoch, subnet_id, overwatch_node_id), weight);
    }

    Ok(())
  }
}