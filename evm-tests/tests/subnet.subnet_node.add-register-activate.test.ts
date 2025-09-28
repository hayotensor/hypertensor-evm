import * as assert from "assert";
import { getDevnetApi } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEd25519PeerId, generateRandomEthersWallet, generateRandomString, getPublicClient, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    getCurrentRegistrationCost,
    registerSubnet,
    registerSubnetNode,
    removeSubnetNode,
    transferBalanceFromSudo
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";
import { Option } from '@polkadot/types';

// Status: passing
// npm test -- -g "test subnet node entry functions-0xbull3948t92d398"
describe("test subnet node entry functions-0xbull3948t92d398", () => {
    // init eth part
    const wallet0 = generateRandomEthersWallet();
    // subnet registration hotkey
    const wallet1 = generateRandomEthersWallet();
    // node 1 coldkey
    const wallet2 = generateRandomEthersWallet();
    // node 1 hotkey
    const wallet3 = generateRandomEthersWallet();
    // node 2 coldkey
    const wallet4 = generateRandomEthersWallet();
    // node 2 hotkey
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
    const KEY_TYPES = [1, 2]

    const BOOTNODES = [
      "bootnode1",
      "bootnode2"
    ]

    let publicClient: PublicClient;
    let papiApi: TypedApi<typeof dev>
    let api: ApiPromise

    const sudoTransferAmount = BigInt(10000e18)
    const stakeAmount = BigInt(100e18)
    let minStakeAmount: string;

    const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet0);

    const subnetContract1 = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);
    const subnetContract2 = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet2);
    const subnetContract3 = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet3);
    const subnetContract4 = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet4);

    const subnetName = generateRandomString(30)
    const repo = generateRandomString(30)
    const description = generateRandomString(30)
    const misc = generateRandomString(30)
    let subnetId: string;
    let peer1: string;
    let peer2: string;
    let peer3: string;
    let peer4: string;
    const delegateRewardRate = "0";

    // sudo account alice as signer
    let alice: PolkadotSigner;
    before(async () => {
        
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        // init variables got from await and async
        papiApi = await getDevnetApi()

        const provider = new WsProvider(SUB_LOCAL_URL);

        api = await ApiPromise.create({ provider });

        // balance for subnet registerer
        await transferBalanceFromSudo(
            api,
            papiApi,
            SUB_LOCAL_URL,
            wallet0.address,
            sudoTransferAmount,
        )

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

        await transferBalanceFromSudo(
            api,
            papiApi,
            SUB_LOCAL_URL,
            wallet4.address,
            sudoTransferAmount,
        )

        // ==============
        // Register subnet
        // ==============
        const cost = await getCurrentRegistrationCost(subnetContract, api)
        const subnetName = generateRandomString(30)
        const repo = generateRandomString(30)
        const description = generateRandomString(30)
        const misc = generateRandomString(30)
        const churnLimit = await api.query.network.maxChurnLimit();
        const minStake = await api.query.network.networkMinStakeBalance();
        const maxStake = await api.query.network.networkMaxStakeBalance();
        const delegateStakePercentage = await api.query.network.minDelegateStakePercentage();
        const subnetNodeQueueEpochs = await api.query.network.minQueueEpochs();
        const idleClassificationEpochs = await api.query.network.minIdleClassificationEpochs();
        const includedClassificationEpochs = await api.query.network.minIncludedClassificationEpochs();
        const maxNodePenalties = await api.query.network.minMaxSubnetNodePenalties();
        const maxRegisteredNodes = await api.query.network.minMaxRegisteredNodes();

        await registerSubnet(
          subnetContract, 
          wallet1.address,
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
          ALL_ACCOUNTS,
          KEY_TYPES,
          BOOTNODES,
          cost
        )

        subnetId = await subnetContract.getSubnetId(subnetName);

        peer1 = await generateRandomEd25519PeerId()
        peer2 = await generateRandomEd25519PeerId()
        peer3 = await generateRandomEd25519PeerId()
        peer4 = await generateRandomEd25519PeerId()

        minStakeAmount = (await api.query.network.networkMinStakeBalance()).toString();
    })

    // Status: passing
    // npm test -- -g "testing register subnet node-0x04209fwwWERV3"
    it("testing register subnet node-0x04209fwwWERV3", async () => {
        const bootnode = generateRandomString(16)
        const unique = generateRandomString(16)
        const nonUnique = generateRandomString(16)

        await registerSubnetNode(
          subnetContract2, 
          subnetId,
          wallet3.address,
          peer1,
          peer2,
          peer3,
          bootnode,
          delegateRewardRate,
          BigInt(minStakeAmount),
          unique,
          nonUnique,
          "100"
        )
        let subnetNodeId = await api.query.network.hotkeySubnetNodeId(subnetId, wallet3.address);

        const subnetNodeIdOpt = subnetNodeId as Option<any>;
        expect(subnetNodeIdOpt.isSome);

        let subnetNodeExists: boolean = false;
        if (subnetNodeIdOpt.isSome) {
            subnetNodeExists = true;
            const subnetNodeIdUnwrapped = subnetNodeIdOpt.unwrap();
            const human = subnetNodeIdUnwrapped.toHuman();
            const subnetNodeId = human?.toString();
            expect(Number(subnetNodeId)).to.be.greaterThan(0);
            console.log("subnetNodeId", subnetNodeId)

            let subnetNodeData = await api.query.network.subnetNodesData(subnetId, subnetNodeId);

            const subnetNodeDataOpt = subnetNodeData as any;
            const subnetNodeDataHuman = subnetNodeDataOpt.toHuman() as any;
            console.log("subnetNodeDataHuman", subnetNodeDataHuman)
            expect(wallet3.address).to.be.equal(subnetNodeDataHuman.hotkey);
            expect(peer1).to.be.equal(subnetNodeDataHuman.peerId);
            expect(peer2).to.be.equal(subnetNodeDataHuman.bootnodePeerId);
            expect(peer3).to.be.equal(subnetNodeDataHuman.clientPeerId);
            expect(unique).to.be.equal(subnetNodeDataHuman.unique);
            expect(nonUnique).to.be.equal(subnetNodeDataHuman.nonUnique);
            // All nodes are "Validator" if the subnet is in registration
            expect("Validator").to.be.equal(subnetNodeDataHuman.classification.nodeClass);
            expect(delegateRewardRate).to.be.equal(subnetNodeDataHuman.delegateRewardRate);

            if (delegateRewardRate == "0") {
                expect("0").to.be.equal(subnetNodeDataHuman.lastDelegateRewardRateUpdate);  
            }
        }

        expect(subnetNodeExists);

        let accountSubnetStake = await api.query.network.accountSubnetStake(wallet3.address, subnetId);
        expect(BigInt(accountSubnetStake.toString())).to.be.equal(BigInt(minStakeAmount));

        console.log("✅ Subnet node registration testing complete")
    })

    // Status: passing
    // npm test -- -g "testing remove subnet node-0xf56GRTy2"
    it("testing remove subnet node-0xf56GRTy2", async () => {
        const bootnode = generateRandomString(16)
        const unique = generateRandomString(16)
        const nonUnique = generateRandomString(16)

        await registerSubnetNode(
          subnetContract4, 
          subnetId,
          wallet5.address,
          peer1,
          peer2,
          peer3,
          bootnode,
          delegateRewardRate,
          BigInt(minStakeAmount),
          unique,
          nonUnique,
          "100"
        )

        let subnetNodeId: string | undefined;

        let subnetNodeIdFetched = await api.query.network.hotkeySubnetNodeId(subnetId, wallet4.address);

        const subnetNodeIdOpt = subnetNodeIdFetched as Option<any>;
        expect(subnetNodeIdOpt.isSome);

        let subnetNodeExists: boolean = false;
        if (subnetNodeIdOpt.isSome) {
            subnetNodeExists = true;
            const subnetNodeIdUnwrapped = subnetNodeIdOpt.unwrap();
            const human = subnetNodeIdUnwrapped.toHuman();
            subnetNodeId = human?.toString();
            console.log("subnetNodeId", subnetNodeId)

            expect(Number(subnetNodeId)).to.be.greaterThan(0);
        }
        expect(subnetNodeExists);

        expect(typeof subnetNodeId !== 'undefined');

        await removeSubnetNode(
            subnetContract4, 
            subnetId,
            subnetNodeId!,
        )

        const subnetNodeIdAfter = await api.query.network.hotkeySubnetNodeId(subnetId, wallet4.address);
        console.log("subnetNodeIdAfter", subnetNodeIdAfter)

        const subnetNodeIdAfterOpt = subnetNodeIdAfter as Option<any>;
        expect(!subnetNodeIdAfterOpt.isSome);
        expect(subnetNodeIdAfterOpt.isEmpty);

        console.log("✅ Subnet node removal testing complete")
    })

});