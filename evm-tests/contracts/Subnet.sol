// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface Subnet  {
  function registerSubnet(
    string memory name,
    string memory repo,
    string memory description,
    string memory misc,
    uint256 maxNodeRegistrationEpochs,
    uint256 nodeRegistrationInterval,
    uint256 nodeActivationInterval,
    uint256 nodeQueuePeriod,
    uint256 maxNodePenalties,
    address[] memory initialColdkeys
  ) external payable;

  function canSubnetRegister(uint256) external view returns (bool);

  function registrationCost(uint256) external view returns (uint256);

  function activateSubnet(
    uint256 subnetId
  ) external;

  function removeSubnet(
    uint256 subnetId
  ) external;

  function ownerDeactivateSubnet(
    uint256 subnetId,
    string memory name
  ) external;

  function ownerUpdateRegistrationInterval(
    uint256 subnetId,
    uint256 value
  ) external;

  function ownerRemoveSubnet(
    uint256 subnetId,
    uint256 subnetNodeId
  ) external;

  function addSubnetNode(
    uint256 subnetId,
    address hotkey,
    string memory peer_id,
    string memory bootstrap_peer_id,
    uint256 delegate_reward_rate,
    uint256 stake_to_be_added
  ) external payable;

  function registerSubnetNode(
    uint256 subnetId,
    address hotkey,
    string memory peer_id,
    string memory bootstrap_peer_id,
    uint256 delegate_reward_rate,
    uint256 stake_to_be_added
  ) external payable;

  function activateSubnetNode(
    uint256 subnetId,
    uint256 subnetNodeId
  ) external;

  function removeSubnetNode(
    uint256 subnetId,
    uint256 subnetNodeId
  ) external;

  function getSubnetId(
    string memory name
  ) external view returns (uint256);
}