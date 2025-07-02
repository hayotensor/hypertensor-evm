import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEd25519PeerId, generateRandomEthersWallet, generateRandomString, getPublicClient, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    activateSubnetNode,
    addSubnetNode,
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

    let publicClient: PublicClient;
    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();

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
        peer3 = await generateRandomEd25519PeerId()
        peer4 = await generateRandomEd25519PeerId()

        minStakeAmount = (await api.query.network.minStakeBalance()).toString();
    })

    // Status: passing
    // npm test -- -g "testing add subnet node-Erqf3030mx2sd"
    it("testing add subnet node-0xErqf3030mx2sd", async () => {
        
        await addSubnetNode(
            subnetContract1,
            subnetId,
            wallet1.address,
            peer1,
            peer1,
            delegateRewardRate,
            BigInt(minStakeAmount)
        )

        let subnetNodeId = await api.query.network.hotkeySubnetNodeId(subnetId, wallet1.address);

        const subnetNodeIdOpt = subnetNodeId as Option<any>;
        expect(subnetNodeIdOpt.isSome);

        let subnetNodeExists: boolean = false;
        if (subnetNodeIdOpt.isSome) {
            subnetNodeExists = true;
            const subnetNodeIdUnwrapped = subnetNodeIdOpt.unwrap();
            const human = subnetNodeIdUnwrapped.toHuman();
            const subnetNodeId = human?.toString();
            expect(Number(subnetNodeId)).to.be.greaterThan(0);

            let subnetNodeData = await api.query.network.subnetNodesData(subnetId, subnetNodeId);

            const subnetNodeDataOpt = subnetNodeData as any;
            const subnetNodeDataHuman = subnetNodeDataOpt.toHuman() as any;
            expect(wallet1.address).to.be.equal(subnetNodeDataHuman.hotkey);
            expect(peer1).to.be.equal(subnetNodeDataHuman.peerId);
            expect(peer1).to.be.equal(subnetNodeDataHuman.bootstrapPeerId);
            expect("Validator").to.be.equal(subnetNodeDataHuman.classification.node_class);
            expect(delegateRewardRate).to.be.equal(subnetNodeDataHuman.delegateRewardRate);

            if (delegateRewardRate == "0") {
                expect("0").to.be.equal(subnetNodeDataHuman.lastDelegateRewardRateUpdate);  
            }
        }

        expect(subnetNodeExists);

        let accountSubnetStake = await api.query.network.accountSubnetStake(wallet1.address, subnetId);
        expect(BigInt(accountSubnetStake.toString())).to.be.equal(BigInt(minStakeAmount))
    })

    // Status: passing
    // npm test -- -g "testing register subnet node-04209fwwWERV3"
    it("testing register subnet node-0x04209fwwWERV3", async () => {
        await registerSubnetNode(
            subnetContract2,
            subnetId,
            wallet2.address,
            peer2,
            peer2,
            delegateRewardRate,
            BigInt(minStakeAmount)
        )

        let subnetNodeId = await api.query.network.hotkeySubnetNodeId(subnetId, wallet2.address);

        const subnetNodeIdOpt = subnetNodeId as Option<any>;
        expect(subnetNodeIdOpt.isSome);

        let subnetNodeExists: boolean = false;
        if (subnetNodeIdOpt.isSome) {
            subnetNodeExists = true;
            const subnetNodeIdUnwrapped = subnetNodeIdOpt.unwrap();
            const human = subnetNodeIdUnwrapped.toHuman();
            const subnetNodeId = human?.toString();
            expect(Number(subnetNodeId)).to.be.greaterThan(0);

            let subnetNodeData = await api.query.network.subnetNodesData(subnetId, subnetNodeId);

            const subnetNodeDataOpt = subnetNodeData as any;
            const subnetNodeDataHuman = subnetNodeDataOpt.toHuman() as any;
            expect(wallet2.address).to.be.equal(subnetNodeDataHuman.hotkey);
            expect(peer2).to.be.equal(subnetNodeDataHuman.peerId);
            expect(peer2).to.be.equal(subnetNodeDataHuman.bootstrapPeerId);
            expect("Registered").to.be.equal(subnetNodeDataHuman.classification.node_class);
            expect(delegateRewardRate).to.be.equal(subnetNodeDataHuman.delegateRewardRate);

            if (delegateRewardRate == "0") {
                expect("0").to.be.equal(subnetNodeDataHuman.lastDelegateRewardRateUpdate);  
            }
        }

        expect(subnetNodeExists);

        let accountSubnetStake = await api.query.network.accountSubnetStake(wallet1.address, subnetId);
        expect(BigInt(accountSubnetStake.toString())).to.be.equal(BigInt(minStakeAmount))
    })

    // Status: passing
    // npm test -- -g "testing activate subnet node-q34vvVGWVGVE"
    it("testing activate subnet node-0xq34vvVGWVGVE", async () => {
        await registerSubnetNode(
            subnetContract3,
            subnetId,
            wallet3.address,
            peer3,
            peer3,
            delegateRewardRate,
            BigInt(minStakeAmount)
        )

        let subnetNodeId = await api.query.network.hotkeySubnetNodeId(subnetId, wallet3.address);

        const subnetNodeIdOpt = subnetNodeId as Option<any>;
        expect(subnetNodeIdOpt.isSome);

        let accountSubnetStake = await api.query.network.accountSubnetStake(wallet1.address, subnetId);
        expect(BigInt(accountSubnetStake.toString())).to.be.equal(BigInt(minStakeAmount))

        await activateSubnetNode(
            subnetContract3,
            subnetId,
            subnetNodeIdOpt.unwrap().toHuman().toString(),
        )

        let subnetNodeExists: boolean = false;
        if (subnetNodeIdOpt.isSome) {
            subnetNodeExists = true;
            const subnetNodeIdUnwrapped = subnetNodeIdOpt.unwrap();
            const human = subnetNodeIdUnwrapped.toHuman();
            const subnetNodeId = human?.toString();
            expect(Number(subnetNodeId)).to.be.greaterThan(0);

            let subnetNodeData = await api.query.network.subnetNodesData(subnetId, subnetNodeId);

            const subnetNodeDataOpt = subnetNodeData as any;
            const subnetNodeDataHuman = subnetNodeDataOpt.toHuman() as any;
            expect(wallet3.address).to.be.equal(subnetNodeDataHuman.hotkey);
            expect(peer3).to.be.equal(subnetNodeDataHuman.peerId);
            expect(peer3).to.be.equal(subnetNodeDataHuman.bootstrapPeerId);
            // Will be "Validator" since subnet is still registering
            expect("Validator").to.be.equal(subnetNodeDataHuman.classification.node_class);
            expect(delegateRewardRate).to.be.equal(subnetNodeDataHuman.delegateRewardRate);

            if (delegateRewardRate == "0") {
                expect("0").to.be.equal(subnetNodeDataHuman.lastDelegateRewardRateUpdate);  
            }
        }

        expect(subnetNodeExists);
    })

    // Status: pending
    // npm test -- -g "testing remove subnet node"
    it("testing remove subnet node-0xf56GRTy2", async () => {
        await addSubnetNode(
            subnetContract4,
            subnetId,
            wallet4.address,
            peer4,
            peer4,
            delegateRewardRate,
            BigInt(minStakeAmount)
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

    })

});