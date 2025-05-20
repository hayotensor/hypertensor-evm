// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface Staking  {
  function addToStake(
    uint256 subnetId,
    uint256 subnetNodeId,
    bytes32 hotkey,
    uint256 stakeToBeAdded
  ) external payable;

  function removeStake(
    uint256 subnetId,
    uint256 subnetNodeId,
    bytes32 hotkey,
    uint256 stakeToBeRemoved
  ) external;

  function claimUnbondings() external;

  function addToDelegateStake(
    uint256 subnetId,
    uint256 stakeToBeAdded
  ) external payable;

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
  ) external payable;

  function addToNodeDelegateStake(
    uint256 subnetId,
    uint256 subnetNodeId,
    uint256 nodeDelegateStakeToBeAdded
  ) external payable;

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
    uint256 nodeDelegateStakeSharesToBeRemoved
  ) external;

  function increaseNodeDelegateStake(
    uint256 subnetId,
    uint256 subnetNodeId,
    uint256 amount
  ) external payable;

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

  function accountSubnetStake(bytes32 hotkey, uint256 subnetId) external view returns (uint256);

  function totalSubnetDelegateStakeBalance(uint256 subnetId) external view returns (uint256);

  function totalSubnetDelegateStakeShares(uint256 subnetId) external view returns (uint256);

  function accountSubnetDelegateStakeShares(bytes32 hotkey, uint256 subnetId) external view returns (uint256);

  function accountSubnetDelegateStakeBalance(bytes32 hotkey, uint256 subnetId) external view returns (uint256);
}