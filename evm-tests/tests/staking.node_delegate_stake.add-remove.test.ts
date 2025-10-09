import * as assert from "assert";
import { getDevnetApi } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEd25519PeerId, generateRandomEthersWallet, generateRandomString, getPublicClient, STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    addToNodeDelegateStake,
    getCurrentRegistrationCost,
    registerSubnet,
    registerSubnetNode,
    removeNodeDelegateStake,
    transferBalanceFromSudo,
    waitForFinalizedBalance
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";
import { Option } from '@polkadot/types';

// npm test -- -g "test node delegate staking-0x835yv"
describe("test node delegate staking-0x835yv", () => {
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
    const initialColdkeys = [
        {
            coldkey: wallet1.address,
            count: 1
        },
        {
            coldkey: wallet2.address,
            count: 1
        },
        {
            coldkey: wallet3.address,
            count: 1
        },
        {
            coldkey: wallet4.address,
            count: 1
        },
        {
            coldkey: wallet5.address,
            count: 1
        },
        {
            coldkey: wallet6.address,
            count: 1
        },
        {
            coldkey: wallet7.address,
            count: 1
        },
        {
            coldkey: wallet8.address,
            count: 1
        },
    ];

    let publicClient: PublicClient;
    // init substrate part
    const KEY_TYPES = [1, 2]

    const BOOTNODES = [
        generateRandomString(6),
        generateRandomString(6)
    ]

    let papiApi: TypedApi<typeof dev>
    let api: ApiPromise

    const sudoTransferAmount = BigInt(10000e18)
    const stakeAmount = BigInt(100e18)

    const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);

    let subnetId: string;
    let subnetNodeId: string;
    let peer1: string;
    let peer2: string;
    let peer3: string;

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
        let cost = await getCurrentRegistrationCost(subnetContract, api)
        const subnetName = generateRandomString(30)
        const repo = generateRandomString(30)
        const description = generateRandomString(30)
        const misc = generateRandomString(30)
        const churnLimit = await api.query.network.maxChurnLimit();
        const minStake = await api.query.network.minSubnetMinStake();
        const maxStake = await api.query.network.networkMaxStakeBalance();
        const delegateStakePercentage = await api.query.network.minDelegateStakePercentage();
        const subnetNodeQueueEpochs = await api.query.network.minQueueEpochs();
        const idleClassificationEpochs = await api.query.network.minIdleClassificationEpochs();
        const includedClassificationEpochs = await api.query.network.minIncludedClassificationEpochs();
        const maxNodePenalties = await api.query.network.minMaxSubnetNodePenalties();
        const maxRegisteredNodes = await api.query.network.minMaxRegisteredNodes();

        await registerSubnet(
            subnetContract, 
            cost,
            subnetName,
            repo,
            description,
            misc,
            churnLimit.toString(),
            minStake.toString(),
            maxStake.toString(),
            delegateStakePercentage.toString(),
            subnetNodeQueueEpochs.toString(),
            idleClassificationEpochs.toString(),
            includedClassificationEpochs.toString(),
            maxNodePenalties.toString(),
            maxRegisteredNodes.toString(),
            initialColdkeys,
            KEY_TYPES,
            BOOTNODES,
            cost
        )

        subnetId = await subnetContract.getSubnetId(subnetName);

        peer1 = await generateRandomEd25519PeerId()
        peer2 = await generateRandomEd25519PeerId()
        peer3 = await generateRandomEd25519PeerId()
        const delegateRewardRate = "0";

        // ================
        // Add subnet nodes
        // ================

        const bootnode = generateRandomString(16)
        const unique = generateRandomString(16)
        const nonUnique = generateRandomString(16)

        await registerSubnetNode(
          subnetContract, 
          subnetId,
          wallet2.address,
          peer1,
          peer2,
          peer3,
          bootnode,
          delegateRewardRate,
          BigInt(minStake.toString()),
          unique,
          nonUnique,
          "100"
        )
        console.log("registered node id")


        let subnetNodeIdFetched = await api.query.network.hotkeySubnetNodeId(subnetId, wallet2.address);

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
        expect(subnetNodeId != undefined);
    })

    // Status: passing
    // npm test -- -g "testing add node delegate stake-0xw98546"
    it("testing add node delegate stake-0xw98546", async () => {
        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet1);

        const sharesBeforeAdd = await stakingContract.accountNodeDelegateStakeShares(wallet1.address, subnetId, subnetNodeId);
        const balanceBeforeAdd = await stakingContract.accountNodeDelegateStakeBalance(wallet1.address, subnetId, subnetNodeId);

        // Ensure fresh wallet
        expect(Number(sharesBeforeAdd)).to.be.equal(0);
        expect(Number(balanceBeforeAdd)).to.be.equal(0);

        const beforeFinalizedBalance = await waitForFinalizedBalance(
            papiApi, 
            wallet1.address, 
            (await papiApi.query.System.Account.getValue(wallet1.address)).data.free
        );

        // ==================
        // Add delegate stake
        // ==================
        await addToNodeDelegateStake(
          stakingContract, 
          subnetId,
          subnetNodeId,
          stakeAmount
        )

        const afterFinalizedBalance = await waitForFinalizedBalance(
            papiApi, 
            wallet1.address, 
            (await papiApi.query.System.Account.getValue(wallet1.address)).data.free
        );

        expect(Number(beforeFinalizedBalance)).to.be.greaterThan(Number(afterFinalizedBalance));

        const sharesAfterAdd = await stakingContract.accountNodeDelegateStakeShares(wallet1.address, subnetId, subnetNodeId);
        const balanceAfterAdd = await stakingContract.accountNodeDelegateStakeBalance(wallet1.address, subnetId, subnetNodeId);

        expect(Number(sharesAfterAdd)).to.be.greaterThan(0);
        expect(Number(balanceAfterAdd)).to.be.greaterThan(0);

        expect(sharesBeforeAdd).to.be.lessThan(sharesAfterAdd);
        expect(balanceBeforeAdd).to.be.lessThan(balanceAfterAdd);
    })

    // Status: passing
    // npm test -- -g "testing remove node delegate stake-0x0987s"
    it("testing remove node delegate stake-0x0987s", async () => {
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