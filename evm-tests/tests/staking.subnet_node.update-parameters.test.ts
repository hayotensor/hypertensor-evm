import * as assert from "assert";
import { getDevnetApi } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEd25519PeerId, generateRandomEthersWallet, generateRandomString, getPublicClient, STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    batchTransferBalanceFromSudo,
    getCurrentRegistrationCost,
    registerSubnet,
    registerSubnetNode,
    updateBootnode,
    updateBootnodePeerId,
    updateClientPeerId,
    updateColdkey,
    updateDelegateRewardRate,
    updateHotkey,
    updateNonUnique,
    updatePeerId,
    updateUnique,
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";
import { Option } from '@polkadot/types';

// npm test -- -g "test node update parameters-0xdgahRTH"
describe("test node update parameters-0xdgahRTH", () => {
    // init eth part
    const wallet0 = generateRandomEthersWallet();
    const wallet1 = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();
    const wallet3 = generateRandomEthersWallet();
    const wallet4 = generateRandomEthersWallet();
    const wallet5 = generateRandomEthersWallet();
    const wallet6 = generateRandomEthersWallet();
    const wallet7 = generateRandomEthersWallet();
    const wallet8 = generateRandomEthersWallet();

    const ALL_ACCOUNTS = [
      wallet0.address,
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

    const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet0);
    const subnetContract1 = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);

    let subnetId: string;
    let subnetNodeId1: string;

    // sudo account alice as signer
    let alice: PolkadotSigner;
    before(async () => {
        
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        // init variables got from await and async
        papiApi = await getDevnetApi()

        const provider = new WsProvider(SUB_LOCAL_URL);

        api = await ApiPromise.create({ provider });

        const recipients = ALL_ACCOUNTS.map(address => ({
            address: address,
            balance: BigInt(sudoTransferAmount + BigInt(500))
        }));

        await batchTransferBalanceFromSudo(
          api,
          papiApi,
          recipients
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

        // ================
        // Add subnet nodes
        // ================

        // ================
        // Subnet node 1
        // ================
        let peer1 = await generateRandomEd25519PeerId()
        let peer2 = await generateRandomEd25519PeerId()
        let peer3 = await generateRandomEd25519PeerId()
        const delegateRewardRate = "0";
        
        const bootnode = generateRandomString(16)
        const unique = generateRandomString(16)
        const nonUnique = generateRandomString(16)

        await registerSubnetNode(
          subnetContract1, 
          subnetId,
          wallet4.address,
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

        let subnetNodeId1Fetched = await api.query.network.hotkeySubnetNodeId(subnetId, wallet4.address);

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
    })

    // Status: passing
    // npm test -- -g "testing update node parameters-0xpdgaa663uF"
    it("testing update node parameters-0xpdgaa663uF", async () => {
        const newDelegateRewardRate = "1"
        await updateDelegateRewardRate(
            subnetContract1, 
            subnetId,
            subnetNodeId1,
            newDelegateRewardRate
        )
        let nodeData = await api.query.network.subnetNodesData(subnetId, subnetNodeId1);
        let nodeDataOpt = nodeData as Option<any>;
        expect(nodeDataOpt.isSome);
        if (nodeDataOpt.isSome) {
            const nodeData = nodeDataOpt.unwrap();
            const human = nodeData.toHuman();
            expect(Number(human.delegateRewardRate)).to.equal(Number(newDelegateRewardRate));
        }

        const newUnique = generateRandomString(16)
        await updateUnique(
            subnetContract1, 
            subnetId,
            subnetNodeId1,
            newUnique
        )
        nodeData = await api.query.network.subnetNodesData(subnetId, subnetNodeId1);
        nodeDataOpt = nodeData as Option<any>;
        expect(nodeDataOpt.isSome);
        if (nodeDataOpt.isSome) {
            const nodeData = nodeDataOpt.unwrap();
            const human = nodeData.toHuman();
            expect(human.unique == newUnique);
        }

        const newNonUnique = generateRandomString(16)
        await updateNonUnique(
            subnetContract1, 
            subnetId,
            subnetNodeId1,
            newNonUnique
        )
        nodeData = await api.query.network.subnetNodesData(subnetId, subnetNodeId1);
        nodeDataOpt = nodeData as Option<any>;
        expect(nodeDataOpt.isSome);
        if (nodeDataOpt.isSome) {
            const nodeData = nodeDataOpt.unwrap();
            const human = nodeData.toHuman();
            expect(human.nonUnique == newNonUnique);
        }

        let newPeerId = await generateRandomEd25519PeerId()
        await updatePeerId(
            subnetContract1, 
            subnetId,
            subnetNodeId1,
            newPeerId
        )

        const newBootnode = generateRandomString(6)
        await updateBootnode(
            subnetContract1, 
            subnetId,
            subnetNodeId1,
            newBootnode
        )
        nodeData = await api.query.network.subnetNodesData(subnetId, subnetNodeId1);
        nodeDataOpt = nodeData as Option<any>;
        expect(nodeDataOpt.isSome);
        if (nodeDataOpt.isSome) {
            const nodeData = nodeDataOpt.unwrap();
            const human = nodeData.toHuman();
            expect(human.bootnode == newBootnode);
        }

        newPeerId = await generateRandomEd25519PeerId()
        await updateBootnodePeerId(
            subnetContract1, 
            subnetId,
            subnetNodeId1,
            newPeerId
        )
        nodeData = await api.query.network.subnetNodesData(subnetId, subnetNodeId1);
        nodeDataOpt = nodeData as Option<any>;
        expect(nodeDataOpt.isSome);
        if (nodeDataOpt.isSome) {
            const nodeData = nodeDataOpt.unwrap();
            const human = nodeData.toHuman();
            expect(human.bootnodePeerId == newPeerId);
        }

        newPeerId = await generateRandomEd25519PeerId()
        await updateClientPeerId(
            subnetContract1, 
            subnetId,
            subnetNodeId1,
            newPeerId
        )
        nodeData = await api.query.network.subnetNodesData(subnetId, subnetNodeId1);
        nodeDataOpt = nodeData as Option<any>;
        expect(nodeDataOpt.isSome);
        if (nodeDataOpt.isSome) {
            const nodeData = nodeDataOpt.unwrap();
            const human = nodeData.toHuman();
            expect(human.clientePeerId == newPeerId);
        }

        const newHotkey = generateRandomEthersWallet();
        await updateHotkey(
            subnetContract1, 
            wallet4.address,
            newHotkey.address,
        )
        let hotkeyOwner = await api.query.network.hotkeyOwner(newHotkey.address);
        expect(hotkeyOwner.toHuman() == wallet4.address);

        const newColdkey = generateRandomEthersWallet();
        await updateColdkey(
            subnetContract1, 
            newHotkey.address,
            newColdkey.address,
        )
        hotkeyOwner = await api.query.network.hotkeyOwner(newHotkey.address);
        expect(hotkeyOwner.toHuman() == newColdkey.address);

        console.log("✅ Updating node parameters testing complete")
    })
});