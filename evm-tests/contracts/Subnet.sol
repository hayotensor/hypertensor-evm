// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

interface Subnet  {
  function registerSubnet(
    address hotkey,
    uint256 maxCost,
    string memory name,
    string memory repo,
    string memory description,
    string memory misc,
    uint256 churnLimit,
    uint256 minStake,
    uint256 maxStake,
    uint256 delegateStakePercentage,
    uint256 subnetNodeQueueEpochs,
    uint256 idleClassificationEpochs,
    uint256 includedClassificationEpochs,
    uint256 maxNodePenalties,
    uint256 maxRegisteredNodes,
    address[] memory initialColdkeys,
    uint256[] memory keyTypes,
    string[] memory bootnodes
  ) external payable;

  function registerSubnetNode(
    uint256 subnetId,
    address hotkey,
    string memory peerId,
    string memory bootnodePeerId,
    string memory clientPeerId,
    string memory bootnode,
    uint256 delegateRewardRate,
    uint256 stakeToBeAdded,
    string memory unique,
    string memory nonUnique,
    uint256 maxBurnAmount
  ) external payable;

  function getCurrentRegistrationCost(uint256) external view returns (uint256);

  function activateSubnet(
    uint256 subnetId
  ) external;

  function removeSubnet(
    uint256 subnetId
  ) external;

  function getSubnetId(
    string memory name
  ) external view returns (uint256);

  function getMinSubnetDelegateStakeBalance(
    uint256 subnetId
  ) external view returns (uint256);

  function updateDelegateRewardRate(
    uint256 subnetId,
    uint256 subnetNodeId,
    uint256 newDelegateRewardRate
  ) external;

  function updateUnique(
    uint256 subnetId,
    uint256 subnetNodeId,
    string memory newUnique
  ) external;

  function updateNonUnique(
    uint256 subnetId,
    uint256 subnetNodeId,
    string memory newNonUnique
  ) external;

  function updateColdkey(
    address hotkey,
    address newColdkey
  ) external;

  function updateHotkey(
    address oldHotkey,
    address newHotkey
  ) external;

  function updatePeerId(
    uint256 subnetId,
    uint256 subnetNodeId,
    string memory newPeerId
  ) external;

  function updateBootnode(
    uint256 subnetId,
    uint256 subnetNodeId,
    string memory newBootnode
  ) external;

  function updateBootnodePeerId(
    uint256 subnetId,
    uint256 subnetNodeId,
    string memory newPeerId
  ) external;

  function updateClientPeerId(
    uint256 subnetId,
    uint256 subnetNodeId,
    string memory newPeerId
  ) external;

  function registerOrUpdateIdentity(
    address hotkey,
    string memory name,
    string memory url,
    string memory image,
    string memory discord,
    string memory x,
    string memory telegram,
    string memory github,
    string memory hugging_face,
    string memory description,
    string memory misc
  ) external;

  function removeIdentity() external;
  
  function ownerPauseSubnet(uint256 subnetId) external;

  function ownerUnpauseSubnet(uint256 subnetId) external;

  function ownerDeactivateSubnet(uint256 subnetId) external;

  function ownerUpdateName(uint256 subnetId, string memory value) external;

  function ownerUpdateRepo(uint256 subnetId, string memory value) external;

  function ownerUpdateDescription(uint256 subnetId, string memory value) external;

  function ownerUpdateMisc(uint256 subnetId, string memory value) external;

  function ownerUpdateChurnLimit(uint256 subnetId, uint256 value) external;

  function ownerUpdateRegistrationQueueEpochs(uint256 subnetId, uint256 value) external;

  function ownerUpdateIdleClassificationEpochs(uint256 subnetId, uint256 value) external;

  function ownerUpdateIncludedClassificationEpochs(uint256 subnetId, uint256 value) external;

  function ownerUpdateMaxNodePenalties(uint256 subnetId, uint256 value) external;

  function ownerAddInitialColdkeys(uint256 subnetId, address[] memory coldkeys) external;

  function ownerRemoveInitialColdkeys(uint256 subnetId, address[] memory coldkeys) external;

  function ownerUpdateKeyTypes(uint256 subnetId, uint256[] memory keyTypes) external;

  function ownerUpdateMinStake(uint256 subnetId, uint256 value) external;

  function ownerUpdateMaxStake(uint256 subnetId, uint256 value) external;

  function ownerUpdateDelegateStakePercentage(uint256 subnetId, uint256 value) external;

  function ownerUpdateMaxRegisteredNodes(uint256 subnetId, uint256 value) external;

  function transferSubnetOwnership(uint256 subnetId, address newOwner) external;
  
  function acceptSubnetOwnership(uint256 subnetId) external;
  
  function ownerAddBootnodeAccess(uint256 subnetId, address newAccount) external;
  
  function ownerUpdateTargetRegistrationsPerEpoch(uint256 subnetId, uint256 value) external;
  
  function ownerUpdateNodeBurnRateAlpha(uint256 subnetId, uint256 value) external;
  
  function ownerUpdateQueueImmunityEpochs(uint256 subnetId, uint256 value) external;
}