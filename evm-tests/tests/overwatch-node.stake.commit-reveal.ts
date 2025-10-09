import { getDevnetApi } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { TypedApi } from "polkadot-api";
import { blake2AsU8a } from '@polkadot/util-crypto';
import { u8aConcat, u8aToHex } from '@polkadot/util';
import { ethers } from "ethers"
import { generateRandomEd25519PeerId, generateRandomEthersWallet, generateRandomString, getPublicClient, OVERWATCH_NODE_CONTRACT_ABI, OVERWATCH_NODE_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    advanceToRevealBlock,
    batchTransferBalanceFromSudoManual,
    commitOverwatchSubnetWeights,
    createAndFinalizeBlock,
    createAndFinalizeBlocks,
    getCurrentRegistrationCost,
    registerOverwatchNode,
    registerSubnet,
    registerSubnetNode,
    revealOverwatchSubnetWeights,
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";
import { Option } from '@polkadot/types';

// npm test -- -g "test overwatch view functions-0xfff90000"
describe("test overwatch commit reveal-0xfff90000", () => {
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
    let ethersProvider: ethers.JsonRpcProvider;

    const sudoTransferAmount = BigInt(10000e18)
    let overwatchMinStake;

    const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet0);
    const subnetContract1 = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);

    const overwatchNodeContract1 = new ethers.Contract(OVERWATCH_NODE_CONTRACT_ADDRESS, OVERWATCH_NODE_CONTRACT_ABI, wallet1);

    let subnetId1: string;
    let subnetId2: string;
    let subnetNodeId1: string;
    let overwatchNodeId: string;
    before(async () => {
        
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        // init variables got from await and async
        papiApi = await getDevnetApi()

        const provider = new WsProvider(SUB_LOCAL_URL);
        ethersProvider = new ethers.JsonRpcProvider(ETH_LOCAL_URL);

        api = await ApiPromise.create({ provider });

        await createAndFinalizeBlock(ethersProvider)
        const recipients = ALL_ACCOUNTS.map(address => ({
            address: address,
            balance: BigInt(sudoTransferAmount + BigInt(500))
        }));

        // await api.rpc.engine.createBlock(true, true)
        await batchTransferBalanceFromSudoManual(
          api,
          papiApi,
          ethersProvider,
          recipients
        )

        // ==============
        // Register subnet
        // ==============
        let cost = await getCurrentRegistrationCost(subnetContract, api)
        let subnetName = generateRandomString(30)
        let repo = generateRandomString(30)
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
            cost,
            ethersProvider,
            true
        )

        subnetId1 = await subnetContract.getSubnetId(subnetName);

        await createAndFinalizeBlock(ethersProvider)

        cost = await getCurrentRegistrationCost(subnetContract1, api)
        subnetName = generateRandomString(30)
        repo = generateRandomString(30)

        await registerSubnet(
            subnetContract1, 
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
            cost,
            ethersProvider,
            true
        )

        subnetId2 = await subnetContract1.getSubnetId(subnetName);

        await createAndFinalizeBlock(ethersProvider)

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
            subnetId1,
            wallet4.address,
            peer1,
            peer2,
            peer3,
            bootnode,
            delegateRewardRate,
            BigInt(minStake.toString()),
            unique,
            nonUnique,
            "100",
            ethersProvider,
            true
        )

        await createAndFinalizeBlock(ethersProvider)

        let subnetNodeId1Fetched = await api.query.network.hotkeySubnetNodeId(subnetId1, wallet4.address);

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

        let overwatch_epochs = await api.query.network.overwatchEpochLengthMultiplier();

        await createAndFinalizeBlocks(ethersProvider, Number(overwatch_epochs.toString()) * 300)
        
        overwatchMinStake = await api.query.network.overwatchMinStakeBalance();

        await registerOverwatchNode(
            overwatchNodeContract1, 
            wallet5.address,
            BigInt(overwatchMinStake.toString()),
            ethersProvider,
            true
        )
        await createAndFinalizeBlock(ethersProvider)

        let hotkeyOverwatchNodeId = await api.query.network.hotkeyOverwatchNodeId(wallet5.address);
        let hotkeyOverwatchNodeIdOpt = hotkeyOverwatchNodeId as Option<any>;
        expect(hotkeyOverwatchNodeIdOpt.isSome);
        if (hotkeyOverwatchNodeIdOpt.isSome) {
            const data = hotkeyOverwatchNodeIdOpt.unwrap();
            const human = data.toHuman();
            expect(Number(human)).to.not.equal(0);
            overwatchNodeId = human?.toString();
        }
    })

    // Status: passing
    // npm test -- -g "testing overwatch commit-0x9"
    it("testing overwatch commit-0x9", async () => {
        const weight = BigInt(1e18);
        const saltString = 'secret-salt';
        const salt = new Uint8Array(Buffer.from(saltString));

        const saltArray = Array.from(salt);

        const tupleType = api.registry.createType('(u128, Vec<u8>)', [weight, saltArray]);
        const encodedTuple = tupleType.toU8a();

        // Blake2-256 hash
        const commitHash = u8aToHex(blake2AsU8a(encodedTuple, 256));

        // Phase 1: Commit
        const commits = [
            { subnetId: Number(subnetId1), weight: commitHash },
            { subnetId: Number(subnetId2), weight: commitHash },
        ];

        let currentOverwatchEpoch = await overwatchNodeContract1.getCurrentOverwatchEpoch();

        await commitOverwatchSubnetWeights(
          overwatchNodeContract1, 
          overwatchNodeId!.toString(),
          commits,
          ethersProvider,
          true,
        );

        let overwatchCommits = await api.query.network.overwatchCommits(currentOverwatchEpoch.toString(), overwatchNodeId!.toString(), subnetId1);
        const overwatchCommitsOpt = overwatchCommits as Option<any>;
        expect(overwatchCommitsOpt.isSome);

        let overwatchCommitsOptExists: boolean = false;
        if (overwatchCommitsOpt.isSome) {
            overwatchCommitsOptExists = true;
            const overwatchCommitsUnwrapped = overwatchCommitsOpt.unwrap();
            const human = overwatchCommitsUnwrapped.toHuman();
        }
        expect(overwatchCommitsOptExists).to.equal(true)

        let precompileCurrentOverwatchEpochS1 = await overwatchNodeContract1.overwatchCommits(currentOverwatchEpoch.toString(), overwatchNodeId!.toString(), subnetId1);
        expect(precompileCurrentOverwatchEpochS1.toString()).to.equal(commits[0].weight.toString())

        let precompileCurrentOverwatchEpochS2 = await overwatchNodeContract1.overwatchCommits(currentOverwatchEpoch.toString(), overwatchNodeId!.toString(), subnetId2);
        expect(precompileCurrentOverwatchEpochS2.toString()).to.equal(commits[1].weight.toString())

        console.log("✅ Overwat node commit testing complete")
    })

    // Status: pending
    // npm test -- -g "testing overwatch reveal-0x9910169321"
    it("testing overwatch reveal-0x9910169321", async () => {
        const weight = BigInt(1e18);
        const saltString = 'secret-salt';
        const salt = new Uint8Array(Buffer.from(saltString));

        const saltArray = Array.from(salt);

        const tupleType = api.registry.createType('(u128, Vec<u8>)', [weight, saltArray]);
        const encodedTuple = tupleType.toU8a();

        // Blake2-256 hash
        const commitHash = u8aToHex(blake2AsU8a(encodedTuple, 256));

        // Phase 1: Commit
        const commits = [
            { subnetId: Number(subnetId1), weight: commitHash },
            { subnetId: Number(subnetId2), weight: commitHash },
        ];

        const reveals = [
            { 
                subnetId: Number(subnetId1), 
                weight: weight,
                salt: saltArray  // Use the same saltArray
            },
            { 
                subnetId: Number(subnetId2), 
                weight: weight,
                salt: saltArray
            },
        ];

        let currentOverwatchEpoch = await overwatchNodeContract1.getCurrentOverwatchEpoch();

        await commitOverwatchSubnetWeights(
          overwatchNodeContract1, 
          overwatchNodeId!.toString(),
          commits,
          ethersProvider,
          true,
        );

        let precompileCommitS1 = await overwatchNodeContract1.overwatchCommits(currentOverwatchEpoch.toString(), overwatchNodeId!.toString(), subnetId1);
        expect(precompileCommitS1.toString()).to.equal(commits[0].weight.toString())

        let precompileCommitS2 = await overwatchNodeContract1.overwatchCommits(currentOverwatchEpoch.toString(), overwatchNodeId!.toString(), subnetId2);
        expect(precompileCommitS2.toString()).to.equal(commits[1].weight.toString())

        currentOverwatchEpoch = await overwatchNodeContract1.getCurrentOverwatchEpoch();

        await advanceToRevealBlock(
          api,
          ethersProvider,
          Number(currentOverwatchEpoch.toString())
        )
        
        await revealOverwatchSubnetWeights(
          overwatchNodeContract1, 
          overwatchNodeId!.toString(),
          reveals,
          ethersProvider,
          true,
        );

        let precompileRevealS1 = await overwatchNodeContract1.overwatchReveals(currentOverwatchEpoch.toString(), subnetId1, overwatchNodeId!.toString());
        expect(precompileRevealS1.toString()).to.equal(reveals[0].weight.toString())

        let precompileRevealS2 = await overwatchNodeContract1.overwatchReveals(currentOverwatchEpoch.toString(), subnetId2, overwatchNodeId!.toString());
        expect(precompileRevealS2.toString()).to.equal(reveals[1].weight.toString())

        console.log("✅ Overwat node reveal testing complete")
    })
});