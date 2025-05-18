// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface Subnet  {
  function registerSubnet(
    string memory path,
    uint256 maxNodeRegistrationEpochs,
    uint256 nodeRegistrationInterval,
    uint256 nodeActivationInterval,
    uint256 nodeQueuePeriod,
    uint256 maxNodePenalties,
    address[] memory coldkeys
  ) external payable;

  function activateSubnet(
    uint256 subnetId
  ) external;

  function removeSubnet(
    uint256 subnetId
  ) external;

  function ownerDeactivateSubnet(
    uint256 subnetId,
    string memory path
  ) external;

  function ownerUpdateRegistrationInterval(
    uint256 subnetId,
    uint256 value
  ) external;

  function ownerRemoveSubnet(
    uint256 subnetId,
    uint256 subnetNodeId
  ) external;

  function getSubnetId(
    string memory path
  ) external view returns (uint256);
}