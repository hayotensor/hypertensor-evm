// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface IOverwatchNode  {
  struct OverwatchCommit {
    uint256 subnetId;
    bytes32 weight;
  }
  struct OverwatchReveal {
    uint256 subnetId;
    uint256 weight;
    uint8[] salt;
  }
  function registerOverwatchNode(address hotkey, uint256 stakeToBeAdded) external payable;
  function removeOverwatchNode(uint256 overwatchNodeId) external;
  function anyoneRemoveOverwatchNode(uint256 overwatchNodeId) external;
  function setOverwatchNodePeerId(uint256 subnetId, uint256 overwatchNodeId, string memory peerId) external;
  function addToOverwatchStake(uint256 overwatchNodeId, address hotkey, uint256 stakeToBeAdded) external payable;
  function removeOverwatchStake(address hotkey, uint256 stakeToBeRemoved) external payable;
  function commitOverwatchSubnetWeights(
    uint256 overwatchNodeId,
    OverwatchCommit[] calldata commits
  ) external;
  function revealOverwatchSubnetWeights(
      uint256 overwatchNodeId,
      OverwatchReveal[] calldata reveals
  ) external;
  function accountOverwatchStake(address hotkey) external view returns (uint256);
  function totalOverwatchStake() external view returns (uint256);
  function overwatchNodeBlacklist(address coldkey) external view returns (bool);
  function maxOverwatchNodes() external view returns (uint256);
  function totalOverwatchNodes() external view returns (uint256);
  function totalOverwatchNodeUids() external view returns (uint256);
  function overwatchEpochLengthMultiplier() external view returns (uint256);
  function overwatchCommitCutoffPercent() external view returns (uint256);
  function overwatchNodes(uint256 overwatchNodeId) external view returns (uint256, address);
  function overwatchNodeIdHotkey(uint256 overwatchNodeId) external view returns (address);
  function hotkeyOverwatchNodeId(address) external view returns (uint256);
  function peerIdOverwatchNode(uint256 subnetId, string memory peerId) external view returns (uint256);
  function overwatchCommits(uint256 overwatchEpoch, uint256 overwatchNodeId, uint256 subnetId) external view returns (bytes32);
  function overwatchReveals(uint256 overwatchEpoch, uint256 subnetId, uint256 overwatchNodeId) external view returns (uint256);
  function overwatchNodePenalties(uint256 overwatchNodeId) external view returns (uint256);
  function maxOverwatchNodePenalties() external view returns (uint256);
  function overwatchSubnetWeights(uint256 overwatchEpoch, uint256 subnetId) external view returns (uint256);
  function overwatchNodeWeights(uint256 overwatchEpoch, uint256 overwatchNodeId) external view returns (uint256);
  function overwatchMinDiversificationRatio() external view returns (uint256);
  function overwatchMinRepScore() external view returns (uint256);
  function overwatchMinAvgAttestationRatio() external view returns (uint256);
  function overwatchMinAge() external view returns (uint256);
  function overwatchMinStakeBalance() external view returns (uint256);
  function testOverwatchBalance() external view returns (uint256);
  function getCurrentOverwatchEpoch() external view returns (uint256);
}