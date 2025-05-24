import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEd25519PeerId, generateRandomEthersWallet, generateRandomString, getPublicClient, STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    addSubnetNode,
    addToDelegateStake,
    addToNodeDelegateStake,
    getCurrentRegistrationCost,
    registerSubnet,
    removeDelegateStake,
    removeNodeDelegateStake,
    transferBalanceFromSudo
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";
import { Option } from '@polkadot/types';

// npm test -- -g "test node delegate staking"
describe("test node delegate staking", () => {
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
    let minStakeAmount: string;

    const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);

    const subnetName = generateRandomString(30)
    const repo = generateRandomString(30)
    const description = generateRandomString(30)
    const misc = generateRandomString(30)
    let subnetId: string;
    let subnetNodeId: string;
    let peer1: string;
    let peer2: string;

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

        peer1 = await generateRandomEd25519PeerId()
        peer2 = await generateRandomEd25519PeerId()

        // ================
        // Add subnet nodes
        // ================

        minStakeAmount = (await api.query.network.minStakeBalance()).toString();

        await addSubnetNode(
            subnetContract,
            subnetId,
            wallet1.address,
            peer1,
            peer1,
            "0",
            BigInt(minStakeAmount)
        )

        let subnetNodeIdFetched = await api.query.network.hotkeySubnetNodeId(subnetId, wallet1.address);

        const subnetNodeIdOpt = subnetNodeIdFetched as Option<any>;
        expect(subnetNodeIdOpt.isSome);

        let subnetNodeExists: boolean = false;
        if (subnetNodeIdOpt.isSome) {
            subnetNodeExists = true;
            const subnetNodeIdUnwrapped = subnetNodeIdOpt.unwrap();
            const human = subnetNodeIdUnwrapped.toHuman();
            subnetNodeId = human?.toString();
            expect(Number(subnetNodeId)).to.be.greaterThan(0);
        }
        expect(subnetNodeExists);
    })

    // Status: passing
    // npm test -- -g "testing add node delegate stake"
    it("testing add node delegate stake", async () => {
        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet1);

        const sharesBeforeAdd = await stakingContract.accountNodeDelegateStakeShares(wallet1.address, subnetId, subnetNodeId);
        const balanceBeforeAdd = await stakingContract.accountNodeDelegateStakeBalance(wallet1.address, subnetId, subnetNodeId);

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

        const sharesAfterAdd = await stakingContract.accountNodeDelegateStakeShares(wallet1.address, subnetId, subnetNodeId);
        const balanceAfterAdd = await stakingContract.accountNodeDelegateStakeBalance(wallet1.address, subnetId, subnetNodeId);

        expect(Number(sharesAfterAdd)).to.be.greaterThan(0);
        expect(Number(balanceAfterAdd)).to.be.greaterThan(0);

		expect(sharesBeforeAdd).to.be.lessThan(sharesAfterAdd);
        expect(balanceBeforeAdd).to.be.lessThan(balanceAfterAdd);
    })

    // Status: passing
    // npm test -- -g "testing remove node delegate stake"
    it("testing remove node delegate stake", async () => {
        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet2);

        const sharesBeforeAdd = await stakingContract.accountNodeDelegateStakeShares(wallet2.address, subnetId, subnetNodeId);
        const balanceBeforeAdd = await stakingContract.accountNodeDelegateStakeBalance(wallet2.address, subnetId, subnetNodeId);

        // Ensure fresh wallet
        expect(Number(sharesBeforeAdd)).to.be.equal(0);
        expect(Number(balanceBeforeAdd)).to.be.equal(0);

        // ==================
        // Add node delegate stake
        // ==================
        await addToNodeDelegateStake(
          stakingContract, 
          subnetId,
          subnetNodeId,
          stakeAmount
        )

        // ==========================
        // Remove node delegate stake
        // ==========================

        const sharesAfterAdd = await stakingContract.accountNodeDelegateStakeShares(
            wallet2.address, 
            subnetId,
            subnetNodeId
        );
        const balanceAfterAdd = await stakingContract.accountNodeDelegateStakeBalance(wallet2.address, subnetId, subnetNodeId);

        // Ensure stake added
        expect(Number(sharesAfterAdd)).to.be.greaterThan(0);
        expect(Number(balanceAfterAdd)).to.be.greaterThan(0);

        await removeNodeDelegateStake(
          stakingContract, 
          subnetId,
          subnetNodeId,
          sharesAfterAdd
        )

        const sharesAfter = await stakingContract.accountNodeDelegateStakeShares(wallet2.address, subnetId, subnetNodeId);
        const balanceAfter = await stakingContract.accountNodeDelegateStakeBalance(wallet2.address, subnetId, subnetNodeId);

		expect(sharesAfterAdd).to.be.greaterThan(sharesAfter);
        expect(balanceAfterAdd).to.be.greaterThan(balanceAfter);
    })
});