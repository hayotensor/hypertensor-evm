import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair, waitForFinalizedBlock } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58, convertH160ToSS58, convertSs58ToAccountId20, ss58ToAccountId20 } from "../src/address-utils"
import { ethers } from "ethers"
import { generateRandomEthersWallet, generateRandomString, getPublicClient, hash, SEED_PATH, STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    addToDelegateStake,
    addToNodeDelegateStake,
    forceSetBalanceToEthAddress, forceSetBalanceToSs58Address,
    getCurrentRegistrationCost,
    registerSubnet,
    removeDelegateStake,
    swapDelegateStake,
    swapNodeDelegateStake,
    transferBalanceFromSudo,
    transferDelegateStake,
    transferNodeDelegateStake
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { AbiItem, PublicClient } from "viem";
import { forceSetBalance } from "../src/test";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";

// npm test -- -g "test delegate staking"
describe("test swap and transfer delegate staking", () => {
    // init eth part
    const wallet1 = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();
    const wallet3 = generateRandomEthersWallet();
    const wallet4 = generateRandomEthersWallet();
    const wallet5 = generateRandomEthersWallet();
    const wallet6 = generateRandomEthersWallet();
    const wallet7 = generateRandomEthersWallet();
    const wallet8 = generateRandomEthersWallet();

    const ALL_ACCOUNTS = [
        wallet1.address,
        wallet2.address,
        wallet3.address,
        wallet4.address,
        wallet5.address,
        wallet6.address,
        wallet7.address,
        wallet8.address,
    ]

    let publicClient: PublicClient;
    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();

    let papiApi: TypedApi<typeof dev>
    let api: ApiPromise

    const sudoTransferAmount = BigInt(10000e18)
    const stakeAmount = BigInt(100e18)

    const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);

    const subnetName = generateRandomString(30)
    const repo = generateRandomString(30)
    const description = generateRandomString(30)
    const misc = generateRandomString(30)
    // from
    let subnetId: string;
    let toSubnetId: string;
    // to
    let subnetNodeId: string;
    let toSubnetNodeId: string;

    // sudo account alice as signer
    let alice: PolkadotSigner;
    before(async () => {
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        // init variables got from await and async
        papiApi = await getDevnetApi()

        const provider = new WsProvider(SUB_LOCAL_URL);

        api = await ApiPromise.create({ provider });

        await transferBalanceFromSudo(
            api,
            papiApi,
            SUB_LOCAL_URL,
            wallet1.address,
            sudoTransferAmount,
        )
        
        await transferBalanceFromSudo(
            api,
            papiApi,
            SUB_LOCAL_URL,
            wallet2.address,
            sudoTransferAmount,
        )

        // ==============
        // Register subnet
        // ==============
        const cost = await getCurrentRegistrationCost(subnetContract, api)

        await registerSubnet(
            subnetContract,
            subnetName, // name: Vec<u8>
            repo, // repo: Vec<u8>
            description, // description: Vec<u8>
            misc, // misc: Vec<u8>
            "16", // maxNodeRegistrationEpochs: u32
            "0", // nodeRegistrationInterval: u32
            "0", // nodeActivationInterval: u32
            "1", // nodeQueuePeriod: u32
            "3", // maxNodePenalties: u32
            ALL_ACCOUNTS, // initialColdkeys: BTreeSet<T::AccountId>
            cost // fee: u32
        )

        subnetId = await subnetContract.getSubnetId(subnetName);
        toSubnetId = "2"
        subnetNodeId = "1"
        toSubnetNodeId = "2"
    })

    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // Requires precompiling/seeding subnet (see `build` in pallets/network/src/lib.rs)
    // - This test requires 2 subnets -
    // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // Status: pending
    // npm test -- -g "testing swap node delegate stake" 
    it("testing swap node delegate stake", async () => {
        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet1);

        // ==================
        // Add delegate stake
        // ==================
        const sharesBefore = await stakingContract.accountNodeDelegateStakeShares(
            wallet1.address, 
            subnetId
        );
        const balanceBefore = await stakingContract.accountNodeDelegateStakeBalance(wallet1.address, subnetId, subnetNodeId);

        await addToNodeDelegateStake(
          stakingContract, 
          subnetId,
          subnetNodeId,
          stakeAmount
        )

        const sharesAfter = await stakingContract.accountNodeDelegateStakeShares(wallet1.address, subnetId, subnetNodeId);
        const balanceAfter = await stakingContract.accountNodeDelegateStakeBalance(wallet1.address, subnetId, subnetNodeId);

        expect(sharesBefore).to.be.lessThan(sharesAfter);
        expect(balanceBefore).to.be.lessThan(balanceAfter);

        // ==================
        // Swap delegate stake
        // ==================
        await swapNodeDelegateStake(
          stakingContract, 
          subnetId, // from
          subnetNodeId, // from
          toSubnetId, // to
          toSubnetNodeId, // to
          sharesBefore
        )

        const toSharesAfter = await stakingContract.accountNodeDelegateStakeShares(wallet1.address, toSubnetId, toSubnetNodeId);
        const toBalanceAfter = await stakingContract.accountNodeDelegateStakeBalance(wallet1.address, toSubnetId, toSubnetNodeId);

        expect(toBalanceAfter).to.be.within(balanceBefore * 0.99, balanceBefore); // Not recommended
    })

    // Status: passing
    // npm test -- -g "testing transfer node delegate stake" 
    it("testing transfer node delegate stake", async () => {
        let stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet2);

        const sharesBeforeAdd = await stakingContract.accountNodeDelegateStakeShares(wallet2.address, subnetId, subnetNodeId);
        const balanceBeforeAdd = await stakingContract.accountNodeDelegateStakeBalance(wallet2.address, subnetId, subnetNodeId);

        // Ensure fresh wallet
        expect(Number(sharesBeforeAdd)).to.be.equal(0);
        expect(Number(balanceBeforeAdd)).to.be.equal(0);

        // ==================
        // Add delegate stake
        // ==================
        await addToNodeDelegateStake(
          stakingContract, 
          subnetId,
          subnetNodeId,
          stakeAmount
        )
 
        // =======================
        // Transfer delegate stake
        // =======================

        const sharesBeforeTransfer = await stakingContract.accountNodeDelegateStakeShares(wallet2.address, subnetId, subnetNodeId);
        const balanceBeforeTransfer = await stakingContract.accountNodeDelegateStakeBalance(wallet2.address, subnetId, subnetNodeId);

        // Ensure stake added
        expect(Number(sharesBeforeTransfer)).to.be.greaterThan(0);
        expect(Number(balanceBeforeTransfer)).to.be.greaterThan(0);

        const toSharesBeforeTransfer = await stakingContract.accountNodeDelegateStakeBalance(wallet8.address, subnetId, subnetNodeId);
        const toBalanceBeforeTransfer = await stakingContract.accountNodeDelegateStakeBalance(wallet8.address, subnetId, subnetNodeId);

        // Ensure fresh wallet
        expect(Number(toSharesBeforeTransfer)).to.be.equal(0);
        expect(Number(toBalanceBeforeTransfer)).to.be.equal(0);

        await transferNodeDelegateStake(
          stakingContract, 
          subnetId,
          subnetNodeId,
          wallet8.address,
          sharesBeforeTransfer
        )

        const sharesAfterTransfer = await stakingContract.accountNodeDelegateStakeShares(wallet2.address, subnetId, subnetNodeId);
        const balanceAfterTransfer = await stakingContract.accountNodeDelegateStakeBalance(wallet2.address, subnetId, subnetNodeId);

        const toSharesAfter = await stakingContract.accountNodeDelegateStakeShares(wallet8.address, subnetId, subnetNodeId);
        const toBalanceAfter = await stakingContract.accountNodeDelegateStakeBalance(wallet8.address, subnetId, subnetNodeId);

        expect(sharesAfterTransfer).to.be.equal(BigInt(0));
        expect(toSharesAfter).to.be.equal(sharesBeforeTransfer);
    })
});