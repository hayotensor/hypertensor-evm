// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface Staking  {
  function addToStake(
    uint256 subnetId,
    uint256 subnetNodeId,
    uint256 hotkey,
    uint256 stakeToBeAdded
  ) external;

  function removeStake(
    uint256 subnetId,
    uint256 subnetNodeId,
    uint256 hotkey,
    uint256 stakeToBeRemoved
  ) external;

  function claimUnbondings() external;

  function addToDelegateStake(
    uint256 subnetId,
    uint256 stakeToBeAdded
  ) external;

  function swapDelegateStake(
    uint256 fromSubnetId,
    uint256 toSubnetId,
    uint256 delegateStakeSharesToSwap
  ) external;

  
  function removeDelegateStake(
    uint256 subnetId,
    uint256 sharesToBeRemoved
  ) external;

  function increaseDelegateStake(
    uint256 subnetId,
    uint256 amount
  ) external;

  function addToNodeDelegateStake(
    uint256 subnetId,
    uint256 subnetNodeId,
    uint256 nodeSelegateStakeToBeAdded
  ) external;

  function transferNodeDelegateStake(
    uint256 fromSubnetId,
    uint256 fromSubnetNodeId,
    uint256 toSubnetId,
    uint256 toSubnetNodeId,
    uint256 nodeDelegateStakeSharesToSwap
  ) external;

  function removeNodeDelegateStake(
    uint256 subnetId,
    uint256 subnetNodeId,
    uint256 nodeSelegateStakeSharesToBeRemoved
  ) external;

  function increaseNodeDelegateStake(
    uint256 subnetId,
    uint256 subnetNodeId,
    uint256 amount
  ) external;

  function transferFromNodeToSubnet(
    uint256 fromSubnetId,
    uint256 fromSubnetNodeId,
    uint256 toSubnetId,
    uint256 nodeDelegateStakeSharesToSwap
  ) external;

  function transferFromSubnetToNode(
    uint256 fromSubnetId,
    uint256 toSubnetId,
    uint256 toSubnetNodeId,
    uint256 nodeDelegateStakeSharesToSwap
  ) external;

  function totalSubnetStake(uint256 subnetId) external view returns (uint256);

  function accountSubnetStake(address hotkey, uint256 subnetId) external view returns (uint256);

  function totalSubnetDelegateStakeBalance(uint256 subnetId) external view returns (uint256);

  function totalSubnetDelegateStakeShares(uint256 subnetId) external view returns (uint256);

  function accountSubnetDelegateStakeShares(address hotkey, uint256 subnetId) external view returns (uint256);

  function accountSubnetDelegateStakeBalance(address hotkey, uint256 subnetId) external view returns (uint256);
}