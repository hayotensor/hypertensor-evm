import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEd25519PeerId, generateRandomEthersWallet, generateRandomString, getPublicClient, STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    addSubnetNode,
    addToStake,
    getCurrentRegistrationCost,
    registerSubnet,
    removeStake,
    transferBalanceFromSudo
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";
import { Option } from '@polkadot/types';

// npm test -- -g "test node staking-0x65683fx2"
describe("test node staking-0x65683fx2", () => {
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

    const subnetContract1 = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);
    const subnetContract2 = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet2);
    const subnetContract3 = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet2);

    const stakingContract1 = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet1);
    const stakingContract2 = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet2);
    const stakingContract3 = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet3);

    const subnetName = generateRandomString(30)
    const repo = generateRandomString(30)
    const description = generateRandomString(30)
    const misc = generateRandomString(30)
    let subnetId: string;
    let subnetNodeId1: string;
    let subnetNodeId2: string;
    let subnetNodeId3: string;

    let peer1: string;
    let peer2: string;
    let peer3: string;

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
        
        await transferBalanceFromSudo(
            api,
            papiApi,
            SUB_LOCAL_URL,
            wallet3.address,
            sudoTransferAmount,
        )

        // ==============
        // Register subnet
        // ==============
        const cost = await getCurrentRegistrationCost(subnetContract1, api)

        await registerSubnet(
            subnetContract1,
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

        subnetId = await subnetContract1.getSubnetId(subnetName);

        peer1 = await generateRandomEd25519PeerId()
        peer2 = await generateRandomEd25519PeerId()
        peer3 = await generateRandomEd25519PeerId()

        // ================
        // Add subnet nodes
        // ================

        minStakeAmount = (await api.query.network.minStakeBalance()).toString();

        // Add subnet node 1
        await addSubnetNode(
            subnetContract1,
            subnetId,
            wallet1.address,
            peer1,
            peer1,
            "0",
            BigInt(minStakeAmount)
        )

        let subnetNodeId1Fetched = await api.query.network.hotkeySubnetNodeId(subnetId, wallet1.address);

        const subnetNodeId1Opt = subnetNodeId1Fetched as Option<any>;
        expect(subnetNodeId1Opt.isSome);

        let subnetNode1Exists: boolean = false;
        if (subnetNodeId1Opt.isSome) {
            subnetNode1Exists = true;
            const subnetNodeId2Unwrapped = subnetNodeId1Opt.unwrap();
            const human = subnetNodeId2Unwrapped.toHuman();
            subnetNodeId1 = human?.toString();
            expect(Number(subnetNodeId1)).to.be.greaterThan(0);
        }
        expect(subnetNode1Exists);

        // Add subnet node 2
        await addSubnetNode(
            subnetContract2,
            subnetId,
            wallet2.address,
            peer2,
            peer2,
            "0",
            BigInt(minStakeAmount)
        )

        let subnetNodeId2Fetched = await api.query.network.hotkeySubnetNodeId(subnetId, wallet2.address);

        const subnetNodeId2Opt = subnetNodeId2Fetched as Option<any>;
        expect(subnetNodeId2Opt.isSome);

        let subnetNode2Exists: boolean = false;
        if (subnetNodeId2Opt.isSome) {
            subnetNode2Exists = true;
            const subnetNodeId2Unwrapped = subnetNodeId2Opt.unwrap();
            const human = subnetNodeId2Unwrapped.toHuman();
            subnetNodeId2 = human?.toString();
            expect(Number(subnetNodeId2)).to.be.greaterThan(0);
        }
        expect(subnetNode2Exists);

        // Add subnet node 3
        await addSubnetNode(
            subnetContract3,
            subnetId,
            wallet3.address,
            peer3,
            peer3,
            "0",
            BigInt(minStakeAmount)
        )

        let subnetNodeId3Fetched = await api.query.network.hotkeySubnetNodeId(subnetId, wallet3.address);

        const subnetNodeId3Opt = subnetNodeId3Fetched as Option<any>;
        expect(subnetNodeId3Opt.isSome);

        let subnetNode3Exists: boolean = false;
        if (subnetNodeId3Opt.isSome) {
            subnetNode3Exists = true;
            const subnetNodeId3Unwrapped = subnetNodeId3Opt.unwrap();
            const human = subnetNodeId3Unwrapped.toHuman();
            subnetNodeId3 = human?.toString();
            expect(Number(subnetNodeId3)).to.be.greaterThan(0);
        }
        expect(subnetNode3Exists);
    })

    // Status: passing
    // npm test -- -g "testing add subnet node stake-0xpqlaz0185"
    it("testing add subnet node stake-0xpqlaz0185", async () => {
        let accountSubnetStakePre = await api.query.network.accountSubnetStake(wallet1.address, subnetId);

        // =========
        // Add stake
        // =========
        await addToStake(
          stakingContract1, 
          subnetId,
          subnetNodeId1,
          wallet1.address,
          stakeAmount
        )

        let accountSubnetStakePost = await api.query.network.accountSubnetStake(wallet1.address, subnetId);
        expect(Number(accountSubnetStakePre.toString())).to.be.lessThan(Number(accountSubnetStakePost.toString()))
    })

    // Status: passing
    // npm test -- -g "testing remove subnet node stake-0xnvhgyt926v"
    it("testing remove subnet node stake-0xnvhgyt926v", async () => {
        let accountSubnetStakePre = await api.query.network.accountSubnetStake(wallet2.address, subnetId);

        // =========
        // Add stake
        // =========
        await addToStake(
            stakingContract2, 
            subnetId,
            subnetNodeId2,
            wallet2.address,
            stakeAmount
        )

        let accountSubnetStakePost = await api.query.network.accountSubnetStake(wallet2.address, subnetId);
        expect(Number(accountSubnetStakePre.toString())).to.be.lessThan(Number(accountSubnetStakePost.toString()))

        // =========
        // Remove stake
        // =========
        await removeStake(
            stakingContract2, 
            subnetId,
            wallet2.address,
            stakeAmount
        )

        let accountSubnetStakeAfterRemoval = await api.query.network.accountSubnetStake(wallet2.address, subnetId);
        expect(Number(accountSubnetStakePre.toString())).to.be.equal(Number(accountSubnetStakeAfterRemoval.toString()))
    })
});