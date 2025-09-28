// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface OverwatchNode  {
  function registerOverwatchNode(address hotkey, uint256 stakeToBeAdded) external payable;
  function removeOverwatchNode(uint256 overwatchNodeId) external;
  function anyoneRemoveOverwatchNode(uint256 overwatchNodeId) external;
  function setOverwatchNodePeerId(uint256 subnetId, uint256 overwatchNodeId, string memory peerId) external;
  function addToOverwatchStake(uint256 overwatchNodeId, address hotkey, uint256 stakeToBeAdded) external payable;
  function removeOverwatchStake(address hotkey, uint256 stakeToBeRemoved) external payable;
}