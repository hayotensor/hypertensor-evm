import * as assert from "assert";
import { dev } from '@polkadot-api/descriptors';
import { TypedApi, TxCallData, HexString } from 'polkadot-api';
import { KeyPair } from "@polkadot-labs/hdkd-helpers"
import { getAliceSigner, waitForTransactionCompletion, getSignerFromKeypair } from './substrate'
import { convertH160ToSS58, convertPublicKeyToSs58 } from './address-utils'
import { cryptoWaitReady, decodeAddress } from '@polkadot/util-crypto';
import { hexToU8a, u8aToHex } from '@polkadot/util';
import { KeyringPair } from "@polkadot/keyring/types";
import { ApiPromise, Keyring } from "@polkadot/api";
import { Contract, JsonRpcProvider } from "ethers";
import { expect } from "chai";

export async function transferBalanceFromSudo(
  api: ApiPromise,
  papiApi: TypedApi<typeof dev>,
  url: string, 
  who: string, 
  balance: bigint
) {
  console.log("transferBalanceFromSudo", balance)
  const keyring = new Keyring({ type: 'ethereum' });
  const sudoPair: KeyringPair = keyring.addFromUri("0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133");

  const aliceBalance_ = (await papiApi.query.System.Account.getValue(sudoPair.address)).data.free
  expect(Number(aliceBalance_)).to.be.greaterThanOrEqual(0);

  await new Promise<void>((resolve, reject) => {
    api.tx.balances
      .transferKeepAlive(who, balance)
      .signAndSend(sudoPair, async (result) => {

        if (result.status.isInBlock) {
          console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
        } else if (result.status.isFinalized) {
          console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);

          // unsubscribe safely if available
          if (typeof unsub === "function") unsub();

          resolve(); // let outer await continue
        } else if (result.isError) {
          if (typeof unsub === "function") unsub();
          reject(new Error("Transaction failed"));
        }
      })
      .then((u) => {
        // assign the unsubscribe function when available
        unsub = u;
      })
      .catch((err) => {
        reject(err);
      });

    let unsub: () => void; // scoped outside to be accessible
  });

  const balance_ = (await papiApi.query.System.Account.getValue(who)).data.free

  expect(Number(balance_)).to.be.greaterThanOrEqual(Number(balance));
}

export async function transferBalanceFromSudoManual(
  api: ApiPromise,
  papiApi: TypedApi<typeof dev>,
  who: string,
  balance: bigint,
  provider: JsonRpcProvider,
) {
  const keyring = new Keyring({ type: 'ethereum' });
  const sudoPair: KeyringPair = keyring.addFromUri(
    "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133"
  );

  // Optional: check sudo balance before transfer
  const sudoBalance = (await papiApi.query.System.Account.getValue(sudoPair.address)).data.free;
  expect(BigInt(sudoBalance.toString())).to.be.greaterThanOrEqual(Number(balance));

  let finalized = false;
  let unsub: (() => void) | undefined;

  // Send the transfer extrinsic
  unsub = await api.tx.balances
    .transferKeepAlive(who, balance)
    .signAndSend(sudoPair, (result) => {
      console.log(`Status: ${result.status.toString()}`);

      if (result.status.isFinalized) {
        console.log(`Transaction finalized at blockHash: ${result.status.asFinalized}`);
        finalized = true;
        if (unsub) unsub();
      }

      if (result.isError) {
        if (unsub) unsub();
        throw new Error("Transaction failed");
      }
    });

  // Manually seal blocks until finalized
  while (!finalized) {
    await createAndFinalizeBlock(provider)
    await new Promise((r) => setTimeout(r, 10));   // small delay to avoid tight loop
  }

  // Verify the recipient's balance
  const recipientBalance = (await papiApi.query.System.Account.getValue(who)).data.free;
  expect(BigInt(recipientBalance.toString())).to.be.greaterThanOrEqual(Number(balance));
  console.log(`Balance successfully transferred to ${who}`);
}

export async function batchTransferBalanceFromSudo(
  api: ApiPromise,
  papiApi: TypedApi<typeof dev>,
  recipients: Array<{ address: string, balance: bigint }>
) {
  const keyring = new Keyring({ type: 'ethereum' });
  const sudoPair: KeyringPair = keyring.addFromUri("0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133");

  const aliceBalance_ = (await papiApi.query.System.Account.getValue(sudoPair.address)).data.free
  expect(Number(aliceBalance_)).to.be.greaterThanOrEqual(0);

  // Create batch of transfer calls
  const transferCalls = recipients.map(({ address, balance }) => 
    api.tx.balances.transferKeepAlive(address, balance)
  );

  await new Promise<void>((resolve, reject) => {
    let unsub: () => void;

    api.tx.utility
      .batch(transferCalls)
      .signAndSend(sudoPair, async (result) => {
        console.log(`Batch transfer status is ${result.status}`);

        if (result.status.isInBlock) {
          console.log(`Batch transaction included at blockHash ${result.status.asInBlock}`);
        } else if (result.status.isFinalized) {
          console.log(`Batch transaction finalized at blockHash ${result.status.asFinalized}`);

          // Check for any failed transfers in the batch
          if (result.dispatchError) {
            if (typeof unsub === "function") unsub();
            reject(new Error(`Batch transaction failed: ${result.dispatchError}`));
            return;
          }

          // unsubscribe safely if available
          if (typeof unsub === "function") unsub();
          resolve();
        } else if (result.isError) {
          if (typeof unsub === "function") unsub();
          reject(new Error("Batch transaction failed"));
        }
      })
      .then((u) => {
        unsub = u;
      })
      .catch((err) => {
        reject(err);
      });
  });

  // Verify all balances after batch transfer
  for (const { address, balance } of recipients) {
    const balance_ = (await papiApi.query.System.Account.getValue(address)).data.free;
    expect(Number(balance_)).to.be.greaterThanOrEqual(Number(balance));
  }
}

export async function batchTransferBalanceFromSudoManual(
  api: ApiPromise,
  papiApi: TypedApi<typeof dev>,
  provider: JsonRpcProvider,
  recipients: Array<{ address: string, balance: bigint }>
) {
  const keyring = new Keyring({ type: 'ethereum' });
  const sudoPair: KeyringPair = keyring.addFromUri(
    "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133"
  );

  // Create batch of transfer calls
  const transferCalls = recipients.map(({ address, balance }) =>
    api.tx.balances.transferKeepAlive(address, balance)
  );

  let finalized = false;
  let unsub: (() => void) | undefined;

  // Submit the batch extrinsic
  unsub = await api.tx.utility
    .batch(transferCalls)
    .signAndSend(sudoPair, (result) => {
      console.log(`Batch status: ${result.status.toString()}`);

      if (result.status.isFinalized) {
        console.log(`Batch finalized at blockHash: ${result.status.asFinalized}`);
        finalized = true;
        if (unsub) unsub();
      }

      if (result.isError) {
        if (unsub) unsub();
        throw new Error('Batch transaction failed');
      }
    });

  // Manually seal blocks until finalized
  while (!finalized) {
    await createAndFinalizeBlock(provider)
    await new Promise((r) => setTimeout(r, 10)); // small delay
  }

  // Verify all balances after batch transfer
  for (const { address, balance } of recipients) {
    const balance_ = (await papiApi.query.System.Account.getValue(address)).data.free;
    expect(Number(balance_)).to.be.greaterThanOrEqual(Number(balance));
  }

  console.log(`Batch transfer to ${recipients.length} recipients completed successfully`);
}

// ==================
// Subnet interaction
// ==================
export async function registerSubnet(
  contract: Contract, 
  maxCost: string,
  name: string,
  repo: string,
  description: string,
  misc: string,
  churnLimit: string,
  minStake: string,
  maxStake: string,
  delegateStakePercentage: string,
  subnetNodeQueueEpochs: string,
  idleClassificationEpochs: string,
  includedClassificationEpochs: string,
  maxNodePenalties: string,
  maxRegisteredNodes: string,
  initialColdkeys: any,
  keyTypes: number[],
  bootnodes: string[],
  fee: bigint,
  provider?: JsonRpcProvider,
  manualSeal?: boolean,
) {
  const tx = await contract.registerSubnet(
    maxCost,
    name,
    repo,
    description,
    misc,
    churnLimit,
    minStake,
    maxStake,
    delegateStakePercentage,
    subnetNodeQueueEpochs,
    idleClassificationEpochs,
    includedClassificationEpochs,
    maxNodePenalties,
    maxRegisteredNodes,
    initialColdkeys,
    keyTypes,
    bootnodes,
    { value: fee }
  );

  if (manualSeal) {
    let receipt = null;
    while (!receipt) {
      // Seal a new block
      await createAndFinalizeBlock(provider!);

      // Try to fetch the receipt
      receipt = await provider!.getTransactionReceipt(tx.hash);
    }
  } else {
    await tx.wait();
  }
}

export async function activateSubnet(
  contract: Contract, 
  subnetId: string,
) {
  const tx = await contract.activateSubnet(
    subnetId,
  );

  await tx.wait();
}

export async function getCurrentRegistrationCost(
  contract: Contract, 
  api: ApiPromise,
) {
    const ethBlockNumber = await api.rpc.eth.blockNumber()
    const substrateBlockNumber = await api.query.system.number();

    const cost = await contract.getCurrentRegistrationCost(ethBlockNumber.toString());

    return cost
}

export async function getMinSubnetDelegateStakeBalance(
  contract: Contract, 
  subnetId: string
) {
    const minDelegateStake = await contract.getMinSubnetDelegateStakeBalance(subnetId);
    return minDelegateStake
}

// ===========
// Subnet node
// ===========
export async function registerSubnetNode(
  contract: Contract, 
  subnetId: string,
  hotkey: string,
  peerId: string,
  bootnodePeerId: string,
  clientPeerId: string,
  bootnode: string,
  delegateRewardRate: string,
  stakeToBeAdded: bigint,
  unique: string,
  nonUnique: string,
  maxBurnAmount: string,
  provider?: JsonRpcProvider,
  manualSeal?: boolean,
) {
  const tx = await contract.registerSubnetNode(
    subnetId,
    hotkey,
    peerId,
    bootnodePeerId,
    clientPeerId,
    bootnode,
    delegateRewardRate,
    stakeToBeAdded,
    unique,
    nonUnique,
    maxBurnAmount,
    { value: stakeToBeAdded }
  );

  if (manualSeal) {
    let receipt = null;
    while (!receipt) {
      // Seal a new block
      await createAndFinalizeBlock(provider!);

      // Try to fetch the receipt
      receipt = await provider!.getTransactionReceipt(tx.hash);
    }
  } else {
    await tx.wait();
  }
}

export async function activateSubnetNode(
  contract: Contract, 
  subnetId: string,
  subnetNodeId: string,
) {
  const tx = await contract.activateSubnetNode(
    subnetId,
    subnetNodeId,
  );

  await tx.wait();
}

export async function removeSubnetNode(
  contract: Contract, 
  subnetId: string,
  subnetNodeId: string,
) {
  const tx = await contract.removeSubnetNode(
    subnetId,
    subnetNodeId,
  );

  await tx.wait();
}

export async function updateDelegateRewardRate(
  contract: Contract, 
  subnetId: string,
  subnetNodeId: string,
  value: string
) {
  const tx = await contract.updateDelegateRewardRate(
    subnetId,
    subnetNodeId,
    value,
  );

  await tx.wait();
}

export async function updateUnique(
  contract: Contract, 
  subnetId: string,
  subnetNodeId: string,
  value: string
) {
  const tx = await contract.updateUnique(
    subnetId,
    subnetNodeId,
    value,
  );

  await tx.wait();
}

export async function updateNonUnique(
  contract: Contract, 
  subnetId: string,
  subnetNodeId: string,
  value: string
) {
  const tx = await contract.updateNonUnique(
    subnetId,
    subnetNodeId,
    value,
  );

  await tx.wait();
}

export async function updateColdkey(
  contract: Contract, 
  hotkey: string,
  newColdkey: string,
) {
  const tx = await contract.updateColdkey(
    hotkey,
    newColdkey,
  );

  await tx.wait();
}

export async function updateHotkey(
  contract: Contract, 
  oldHotkey: string,
  newHotkey: string,
) {
  const tx = await contract.updateHotkey(
    oldHotkey,
    newHotkey,
  );

  await tx.wait();
}

export async function updatePeerId(
  contract: Contract, 
  subnetId: string,
  subnetNodeId: string,
  newPeerId: string
) {
  const tx = await contract.updatePeerId(
    subnetId,
    subnetNodeId,
    newPeerId
  );

  await tx.wait();
}

export async function updateBootnode(
  contract: Contract, 
  subnetId: string,
  subnetNodeId: string,
  newBootnode: string
) {
  const tx = await contract.updateBootnode(
    subnetId,
    subnetNodeId,
    newBootnode
  );

  await tx.wait();
}

export async function updateBootnodePeerId(
  contract: Contract, 
  subnetId: string,
  subnetNodeId: string,
  newPeerId: string
) {
  const tx = await contract.updateBootnodePeerId(
    subnetId,
    subnetNodeId,
    newPeerId
  );

  await tx.wait();
}

export async function updateClientPeerId(
  contract: Contract, 
  subnetId: string,
  subnetNodeId: string,
  newPeerId: string
) {
  const tx = await contract.updateClientPeerId(
    subnetId,
    subnetNodeId,
    newPeerId
  );

  await tx.wait();
}

// =====
// Identities
// =====
export async function registerOrUpdateIdentity(
  contract: Contract, 
  hotkey: string,
  name: string,
  url: string,
  image: string,
  discord: string,
  x: string,
  telegram: string,
  github: string,
  huggingFace: string,
  description: string,
  misc: string,
) {
  const tx = await contract.registerOrUpdateIdentity(
    hotkey,
    name,
    url,
    image,
    discord,
    x,
    telegram,
    github,
    huggingFace,
    description,
    misc
  );

  await tx.wait();
}

export async function removeIdentity(
  contract: Contract, 
) {
  const tx = await contract.removeIdentity();

  await tx.wait();
}

// ==============
// Subnet node stake
// ==============

export async function addToStake(
  contract: Contract, 
  subnetId: string,
  subnetNodeId: string,
  hotkey: string,
  balance: bigint
) {
  const tx = await contract.addToStake(
    subnetId, 
    subnetNodeId,
    hotkey,
    balance, 
    { value: balance }
  );

  await tx.wait();
}

export async function removeStake(
  contract: Contract, 
  subnetId: string,
  hotkey: string,
  balance: bigint
) {
  const tx = await contract.removeStake(
    subnetId, 
    hotkey,
    balance, 
    { value: balance }
  );

  await tx.wait();
}

export async function claimUnbondings(contract: Contract) {
  const tx = await contract.claimUnbondings();
  await tx.wait();
}

// ==============
// Delegate stake
// ==============

export async function addToDelegateStake(
  contract: Contract, 
  subnetId: string,
  balance: bigint,
  fee: bigint
) {
  const tx = await contract.addToDelegateStake(subnetId, balance, { value: fee });

  await tx.wait();
}

export async function removeDelegateStake(
  contract: Contract, 
  subnetId: string,
  shares: bigint
) {
  const tx = await contract.removeDelegateStake(subnetId, shares);

  await tx.wait();
}

export async function swapDelegateStake(
  contract: Contract, 
  fromSubnetId: string,
  toSubnetId: string,
  shares: bigint
) {
  const tx = await contract.swapDelegateStake(fromSubnetId, toSubnetId, shares);

  await tx.wait();
}

export async function transferDelegateStake(
  contract: Contract, 
  subnetId: string,
  toAccount: string,
  shares: bigint
) {
  const tx = await contract.transferDelegateStake(subnetId, toAccount, shares);

  await tx.wait();
}

// ===================
// Node delegate stake
// ===================

export async function addToNodeDelegateStake(
  contract: Contract, 
  subnetId: string,
  subnetNodeId: string,
  stakeAmount: bigint
) {
  const tx = await contract.addToNodeDelegateStake(subnetId, subnetNodeId, stakeAmount);

  await tx.wait();
}

export async function removeNodeDelegateStake(
  contract: Contract, 
  subnetId: string,
  subnetNodeId: string,
  shares: bigint
) {
  const tx = await contract.removeNodeDelegateStake(subnetId, subnetNodeId, shares);

  await tx.wait();
}

export async function swapNodeDelegateStake(
  contract: Contract, 
  fromSubnetId: string,
  fromSubnetNodeId: string,
  toSubnetId: string,
  toSubnetNodeId: string,
  shares: bigint
) {
  const tx = await contract.swapNodeDelegateStake(
    fromSubnetId,
    fromSubnetNodeId,
    toSubnetId,
    toSubnetNodeId,
    shares
  );

  await tx.wait();
}

export async function transferNodeDelegateStake(
  contract: Contract, 
  subnetId: string,
  subnetNodeId: string,
  toAccountId: string,
  shares: bigint
) {
  const tx = await contract.transferNodeDelegateStake(
    subnetId,
    subnetNodeId,
    toAccountId,
    shares,
  );

  await tx.wait();
}


export async function updateSwapQueue(
  contract: Contract, 
  id: string,
  callType: string,
  toSubnetId: string,
  toSubnetNodeId: string,
) {
  const tx = await contract.updateSwapQueue(
    id,
    callType,
    toSubnetId,
    toSubnetNodeId
  );

  await tx.wait();
}

export async function ownerPauseSubnet(
  contract: Contract, 
  subnetId: string,
) {
  const tx = await contract.ownerPauseSubnet(
    subnetId
  );

  await tx.wait();
}

export async function ownerUnpauseSubnet(
  contract: Contract, 
  subnetId: string,
) {
  const tx = await contract.ownerUnpauseSubnet(
    subnetId
  );

  await tx.wait();
}

export async function ownerDeactivateSubnet(
  contract: Contract, 
  subnetId: string,
) {
  const tx = await contract.ownerDeactivateSubnet(
    subnetId
  );

  await tx.wait();
}

export async function ownerUpdateName(
  contract: Contract, 
  subnetId: string,
  value: string
) {
  const tx = await contract.ownerUpdateName(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerUpdateRepo(
  contract: Contract, 
  subnetId: string,
  value: string
) {
  const tx = await contract.ownerUpdateRepo(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerUpdateDescription(
  contract: Contract, 
  subnetId: string,
  value: string
) {
  const tx = await contract.ownerUpdateDescription(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerUpdateMisc(
  contract: Contract, 
  subnetId: string,
  value: string
) {
  const tx = await contract.ownerUpdateMisc(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerUpdateChurnLimit(
  contract: Contract, 
  subnetId: string,
  value: string
) {
  const tx = await contract.ownerUpdateChurnLimit(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerUpdateRegistrationQueueEpochs(
  contract: Contract, 
  subnetId: string,
  value: string
) {
  const tx = await contract.ownerUpdateRegistrationQueueEpochs(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerUpdateIdleClassificationEpochs(
  contract: Contract, 
  subnetId: string,
  value: string
) {
  const tx = await contract.ownerUpdateIdleClassificationEpochs(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerUpdateIncludedClassificationEpochs(
  contract: Contract, 
  subnetId: string,
  value: string
) {
  const tx = await contract.ownerUpdateIncludedClassificationEpochs(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerUpdateMaxNodePenalties(
  contract: Contract, 
  subnetId: string,
  value: string
) {
  const tx = await contract.ownerUpdateMaxNodePenalties(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerAddOrUpdateInitialColdkeys(
  contract: Contract, 
  subnetId: string,
  coldkeys: any
) {
  const tx = await contract.ownerAddOrUpdateInitialColdkeys(
    subnetId,
    coldkeys
  );

  await tx.wait();
}

export async function ownerRemoveInitialColdkeys(
  contract: Contract, 
  subnetId: string,
  coldkeys: string[]
) {
  const tx = await contract.ownerRemoveInitialColdkeys(
    subnetId,
    coldkeys
  );

  await tx.wait();
}

export async function ownerUpdateKeyTypes(
  contract: Contract, 
  subnetId: string,
  keyTypes: number[],
) {
  const tx = await contract.ownerUpdateKeyTypes(
    subnetId,
    keyTypes
  );

  await tx.wait();
}

export async function ownerUpdateMinStake(
  contract: Contract, 
  subnetId: string,
  value: string
) {
  const tx = await contract.ownerUpdateMinStake(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerUpdateMaxStake(
  contract: Contract, 
  subnetId: string,
  value: string
) {
  const tx = await contract.ownerUpdateMaxStake(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerUpdateDelegateStakePercentage(
  contract: Contract, 
  subnetId: string,
  value: string
) {
  const tx = await contract.ownerUpdateDelegateStakePercentage(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerUpdateMaxRegisteredNodes(
  contract: Contract, 
  subnetId: string,
  value: string,
) {
  const tx = await contract.ownerUpdateMaxRegisteredNodes(
    subnetId,
    value
  );

  await tx.wait();
}

export async function transferSubnetOwnership(
  contract: Contract, 
  subnetId: string,
  address: string,
) {
  const tx = await contract.transferSubnetOwnership(
    subnetId,
    address
  );

  await tx.wait();
}

export async function acceptSubnetOwnership(
  contract: Contract, 
  subnetId: string,
) {
  const tx = await contract.acceptSubnetOwnership(
    subnetId,
  );

  await tx.wait();
}

export async function ownerAddBootnodeAccess(
  contract: Contract, 
  subnetId: string,
  newAccount: string,
) {
  const tx = await contract.ownerAddBootnodeAccess(
    subnetId,
    newAccount
  );

  await tx.wait();
}

export async function ownerUpdateTargetNodeRegistrationsPerEpoch(
  contract: Contract, 
  subnetId: string,
  value: string,
) {
  const tx = await contract.ownerUpdateTargetNodeRegistrationsPerEpoch(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerUpdateNodeBurnRateAlpha(
  contract: Contract, 
  subnetId: string,
  value: string,
) {
  const tx = await contract.ownerUpdateNodeBurnRateAlpha(
    subnetId,
    value
  );

  await tx.wait();
}

export async function ownerUpdateQueueImmunityEpochs(
  contract: Contract, 
  subnetId: string,
  value: string,
) {
  const tx = await contract.ownerUpdateQueueImmunityEpochs(
    subnetId,
    value
  );

  await tx.wait();
}

export async function updateBootnodes(
  contract: Contract, 
  subnetId: string,
  add: any,
  remove: any
) {
  const tx = await contract.updateBootnodes(
    subnetId,
    add,
    remove
  );

  await tx.wait();
}

// =======
// Overwatch node
// =======
export async function registerOverwatchNode(
  contract: Contract, 
  hotkey: string,
  stakeToBeAdded: bigint,
  provider?: JsonRpcProvider,
  manualSeal?: boolean,
) {
  const tx = await contract.registerOverwatchNode(
    hotkey,
    stakeToBeAdded
  );

  if (manualSeal) {
    let receipt = null;
    while (!receipt) {
      // Seal a new block
      await createAndFinalizeBlock(provider!);

      // Try to fetch the receipt
      receipt = await provider!.getTransactionReceipt(tx.hash);
    }
  } else {
    await tx.wait();
  }
}

export async function removeOverwatchNode(
  contract: Contract, 
  overwatchNodeId: string,
  provider?: JsonRpcProvider,
  manualSeal?: boolean,
) {
  const tx = await contract.removeOverwatchNode(
    overwatchNodeId
  );

  if (manualSeal) {
    let receipt = null;
    while (!receipt) {
      // Seal a new block
      await createAndFinalizeBlock(provider!);

      // Try to fetch the receipt
      receipt = await provider!.getTransactionReceipt(tx.hash);
    }
  } else {
    await tx.wait();
  }
}

export async function anyoneRemoveOverwatchNode(
  contract: Contract, 
  overwatchNodeId: string,
  provider?: JsonRpcProvider,
  manualSeal?: boolean,
) {
  const tx = await contract.anyoneRemoveOverwatchNode(
    overwatchNodeId
  );

  if (manualSeal) {
    let receipt = null;
    while (!receipt) {
      // Seal a new block
      await createAndFinalizeBlock(provider!);

      // Try to fetch the receipt
      receipt = await provider!.getTransactionReceipt(tx.hash);
    }
  } else {
    await tx.wait();
  }
}

export async function setOverwatchNodePeerId(
  contract: Contract, 
  subnetId: string,
  overwatchNodeId: string,
  peerId: string,
  provider?: JsonRpcProvider,
  manualSeal?: boolean,
) {
  const tx = await contract.setOverwatchNodePeerId(
    subnetId,
    overwatchNodeId,
    peerId,
  );

  if (manualSeal) {
    let receipt = null;
    while (!receipt) {
      // Seal a new block
      await createAndFinalizeBlock(provider!);

      // Try to fetch the receipt
      receipt = await provider!.getTransactionReceipt(tx.hash);
    }
  } else {
    await tx.wait();
  }
}

export async function addToOverwatchStake(
  contract: Contract, 
  overwatchNodeId: string,
  hotkey: string,
  stakeToBeAdded: bigint,
  provider?: JsonRpcProvider,
  manualSeal?: boolean,
) {
  const tx = await contract.addToOverwatchStake(
    overwatchNodeId,
    hotkey,
    stakeToBeAdded
  );

  if (manualSeal) {
    let receipt = null;
    while (!receipt) {
      // Seal a new block
      await createAndFinalizeBlock(provider!);

      // Try to fetch the receipt
      receipt = await provider!.getTransactionReceipt(tx.hash);
    }
  } else {
    await tx.wait();
  }
}

export async function removeOverwatchStake(
  contract: Contract, 
  hotkey: string,
  stakeToBeRemoved: bigint,
  provider?: JsonRpcProvider,
  manualSeal?: boolean,
) {
  const tx = await contract.removeOverwatchStake(
    hotkey,
    stakeToBeRemoved,
  );

  if (manualSeal) {
    let receipt = null;
    while (!receipt) {
      // Seal a new block
      await createAndFinalizeBlock(provider!);

      // Try to fetch the receipt
      receipt = await provider!.getTransactionReceipt(tx.hash);
    }
  } else {
    await tx.wait();
  }
}

export async function commitOverwatchSubnetWeights(
  contract: Contract, 
  overwatchNodeId: string,
  commits: any,
  provider?: JsonRpcProvider,
  manualSeal?: boolean,
) {
  const tx = await contract.commitOverwatchSubnetWeights(overwatchNodeId, commits);

  if (manualSeal) {
    let receipt = null;
    while (!receipt) {
      // Seal a new block
      await createAndFinalizeBlock(provider!);

      // Try to fetch the receipt
      receipt = await provider!.getTransactionReceipt(tx.hash);
    }
  } else {
    await tx.wait();
  }
}

export async function revealOverwatchSubnetWeights(
  contract: Contract, 
  overwatchNodeId: string,
  reveals: any,
  provider?: JsonRpcProvider,
  manualSeal?: boolean,
) {
  const tx = await contract.revealOverwatchSubnetWeights(overwatchNodeId, reveals);

  if (manualSeal) {
    let receipt = null;
    while (!receipt) {
      // Seal a new block
      await createAndFinalizeBlock(provider!);

      // Try to fetch the receipt
      receipt = await provider!.getTransactionReceipt(tx.hash);
    }
  } else {
    await tx.wait();
  }
}

/**
 * Waits for the next finalized block and returns the finalized free balance for an account.
 * @param papiApi The polkadot-api instance
 * @param address The account address (SS58 or H160 depending on runtime)
 */
// export async function waitForFinalizedBalance(papiApi: any, address: string) {
//   return new Promise<bigint>((resolve, reject) => {
//     const unsub = papiApi.rpc.chain.subscribeFinalizedHeads(async (header: any) => {
//       try {
//         const finalizedHash = header.hash;
//         const accountData = await papiApi.query.System.Account.getValue(address, finalizedHash);
//         const freeBalance = accountData.data.free;
//         unsub(); // stop listening after first finalized block
//         resolve(freeBalance);
//       } catch (err) {
//         unsub();
//         reject(err);
//       }
//     });
//   });
// }
export async function waitForFinalizedBalance(papiApi: any, address: string, lastBalance: bigint) {
  while (true) {
    // Query latest System.Account
    const accountData = await papiApi.query.System.Account.getValue(address);
    const freeBalance = accountData.data.free;
    if (freeBalance !== lastBalance) return freeBalance;
    await new Promise((r) => setTimeout(r, 1000)); // wait 1s
  }
}

/**
 * Advance the chain by `numBlocks` blocks.
 * Requires the node to have the `manual-seal` pallet.
 *
 * @param api - Connected ApiPromise instance
 * @param numBlocks - Number of blocks to produce
 */
export async function advanceBlocks(api: ApiPromise, numBlocks: number): Promise<void> {
  for (let i = 0; i < numBlocks; i++) {
    // true, true => finalize block, include pending extrinsics
    await api.rpc.engine.createBlock(true, true);
  }

  const latestHash = await api.rpc.chain.getBlockHash();
  const latestNumber = await api.rpc.chain.getHeader(latestHash).then(h => h.number.toNumber());

  console.log(`Advanced ${numBlocks} blocks. Current block: ${latestNumber}`);
}

export async function createAndFinalizeBlock(provider: JsonRpcProvider, finalize = true) {
  const request = {
    jsonrpc: "2.0",
    id: Date.now(),
    method: "engine_createBlock",
    params: [true, finalize, null],
  };

  const response = await provider.send(request.method, request.params);

  if (!response) {
    throw new Error(`engine_createBlock failed: ${JSON.stringify(response)}`);
  }

  // optional delay to avoid tight loop
  await new Promise<void>((resolve) => setTimeout(resolve, 500));
}

export async function createAndFinalizeBlocks(provider: JsonRpcProvider, numBlocks: number, finalize = true) {
  for (let i = 0; i < numBlocks; i++) {
    const request = {
      jsonrpc: "2.0",
      id: Date.now() + i, // Unique ID for each request
      method: "engine_createBlock",
      params: [true, finalize, null],
    };

    const response = await provider.send(request.method, request.params);

    if (!response) {
      throw new Error(`engine_createBlock failed on block ${i + 1}: ${JSON.stringify(response)}`);
    }

    console.log(`Created block ${i + 1}/${numBlocks}`);

    // Optional delay between blocks
    if (i < numBlocks - 1) { // Don't delay after the last block
      await new Promise<void>((resolve) => setTimeout(resolve, 500));
    }
  }
}

export async function calculateRevealBlock(
  api: ApiPromise,
  epoch: number
): Promise<number> {
  // Get configuration values from storage
  const epochLength = Number(api.consts.network.epochLength.toString());
  const multiplier = Number((await api.query.network.overwatchEpochLengthMultiplier()).toString());
  const cutoffPercentage = Number((await api.query.network.overwatchCommitCutoffPercent()).toString());
  
  console.log('Configuration:');
  console.log('  Epoch Length:', epochLength);
  console.log('  Multiplier:', multiplier);
  console.log('  Cutoff Percentage:', cutoffPercentage);
  
  // Calculate overwatch epoch length
  const overwatchEpochLength = epochLength * multiplier;
  
  // Calculate block increase cutoff
  // percent_mul formula: (value * percentage) / 10_000
  const blockIncreaseCutoff = Math.floor(
    (overwatchEpochLength * cutoffPercentage) / 1e18
  );
  
  // Calculate target block number
  const revealBlock = epoch * multiplier * epochLength + blockIncreaseCutoff;
  
  console.log('Calculations:');
  console.log('  Overwatch Epoch Length:', overwatchEpochLength);
  console.log('  Block Increase Cutoff:', blockIncreaseCutoff);
  console.log('  Target Reveal Block:', revealBlock);
  
  return revealBlock;
}

// Advance to the reveal block for a given epoch
export async function advanceToRevealBlock(
  api: ApiPromise,
  provider: JsonRpcProvider,
  epoch: number
): Promise<number> {
  // Get current block number
  const currentBlock = Number((await api.query.system.number()).toString());
  console.log('Current block:', currentBlock);
  
  // Calculate target reveal block
  const revealBlock = await calculateRevealBlock(api, epoch);
  console.log('Target reveal block:', revealBlock);
  
  // Calculate how many blocks to advance
  const blocksToAdvance = revealBlock - currentBlock;
  
  if (blocksToAdvance <= 0) {
    console.log('Already at or past reveal block');
    return currentBlock;
  }
  
  console.log(`Advancing ${blocksToAdvance} blocks...`);
  
  // Advance blocks
  await createAndFinalizeBlocks(provider, blocksToAdvance);
  
  // Verify we reached the target
  const newBlock = Number((await api.query.system.number()).toString());
  console.log('New block number:', newBlock);
  
  return newBlock;
}
