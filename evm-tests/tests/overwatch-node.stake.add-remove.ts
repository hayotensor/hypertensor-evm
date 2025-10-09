import { getDevnetApi } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEd25519PeerId, generateRandomEthersWallet, generateRandomString, getPublicClient, OVERWATCH_NODE_CONTRACT_ABI, OVERWATCH_NODE_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    addToOverwatchStake,
    batchTransferBalanceFromSudoManual,
    createAndFinalizeBlock,
    createAndFinalizeBlocks,
    getCurrentRegistrationCost,
    registerOverwatchNode,
    registerSubnet,
    registerSubnetNode,
    removeOverwatchStake,
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";
import { Option } from '@polkadot/types';

// npm test -- -g "test overwatch nodes-0xDDDDDJUUK9996"
describe("test overwatch nodes-0xDDDDDJUUK9996", () => {
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

    const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet0);
    const subnetContract1 = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);

    const overwatchNodeContract1 = new ethers.Contract(OVERWATCH_NODE_CONTRACT_ADDRESS, OVERWATCH_NODE_CONTRACT_ABI, wallet1);

    let subnetId: string;
    let subnetNodeId1: string;
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

        console.log("registering subnet")

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

        subnetId = await subnetContract.getSubnetId(subnetName);
        console.log("subnetId", subnetId)

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

        console.log("registering node")

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
            "100",
            ethersProvider,
            true
        )
        console.log("registering node complete")

        await createAndFinalizeBlock(ethersProvider)

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
    // npm test -- -g "testing add overwatch stake-0xdgewve2346"
    it("testing add overwatch stake-0xdgewve2346", async () => {
        let overwatch_epochs = await api.query.network.overwatchEpochLengthMultiplier();

        await createAndFinalizeBlocks(ethersProvider, Number(overwatch_epochs.toString()) * 300)
        
        const minStake = await api.query.network.overwatchMinStakeBalance();

        await registerOverwatchNode(
            overwatchNodeContract1, 
            wallet5.address,
            BigInt(minStake.toString()),
            ethersProvider,
            true
        )
        await createAndFinalizeBlock(ethersProvider)

        let overwatchNodeId;
        let hotkeyOverwatchNodeId = await api.query.network.hotkeyOverwatchNodeId(wallet5.address);
        let hotkeyOverwatchNodeIdOpt = hotkeyOverwatchNodeId as Option<any>;
        expect(hotkeyOverwatchNodeIdOpt.isSome);
        if (hotkeyOverwatchNodeIdOpt.isSome) {
            const data = hotkeyOverwatchNodeIdOpt.unwrap();
            const human = data.toHuman();
            overwatchNodeId = human;
            expect(Number(human)).to.not.equal(0);
        }

        const stakeAmount = BigInt(10e18);
        await batchTransferBalanceFromSudoManual(
            api,
            papiApi,
            ethersProvider,
            [{
                address: wallet5.address,
                balance: stakeAmount
            }]
        )

        await addToOverwatchStake(
            overwatchNodeContract1, 
            overwatchNodeId,
            wallet5.address,
            stakeAmount,
            ethersProvider,
            true
        );

        let stakeBalance = await api.query.network.accountOverwatchStake(wallet5.address);
        expect(BigInt(stakeBalance.toString())).to.equal(stakeAmount)

        let precompileStakeBalance = await overwatchNodeContract1.accountOverwatchStake(wallet5.address);
        expect(BigInt(stakeBalance.toString())).to.be.equal(BigInt(precompileStakeBalance.toString()))

        console.log("✅ Registering overwatch node testing complete")
    })

    // Status: passing
    // npm test -- -g "testing remove overwatch stake-0xSERBMS5sS"
    it("testing remove overwatch stake-0xSERBMS5sS", async () => {
        let overwatch_epochs = await api.query.network.overwatchEpochLengthMultiplier();

        await createAndFinalizeBlocks(ethersProvider, Number(overwatch_epochs.toString()) * 300)
        
        const minStake = await api.query.network.overwatchMinStakeBalance();

        await registerOverwatchNode(
            overwatchNodeContract1, 
            wallet5.address,
            BigInt(minStake.toString()),
            ethersProvider,
            true
        )
        await createAndFinalizeBlock(ethersProvider)

        let afterRegisterStakeBalance = await api.query.network.accountOverwatchStake(wallet5.address);
        expect(BigInt(afterRegisterStakeBalance.toString())).to.equal(BigInt(minStake.toString()))
        let afterRegisterPrecompileStakeBalance = await overwatchNodeContract1.accountOverwatchStake(wallet5.address);
        expect(BigInt(afterRegisterPrecompileStakeBalance.toString())).to.be.equal(BigInt(minStake.toString()))

        let overwatchNodeId;
        let hotkeyOverwatchNodeId = await api.query.network.hotkeyOverwatchNodeId(wallet5.address);
        let hotkeyOverwatchNodeIdOpt = hotkeyOverwatchNodeId as Option<any>;
        expect(hotkeyOverwatchNodeIdOpt.isSome);
        if (hotkeyOverwatchNodeIdOpt.isSome) {
            const data = hotkeyOverwatchNodeIdOpt.unwrap();
            const human = data.toHuman();
            overwatchNodeId = human;
            expect(Number(human)).to.not.equal(0);
        }

        const stakeAmount = BigInt(10e18);
        await batchTransferBalanceFromSudoManual(
            api,
            papiApi,
            ethersProvider,
            [{
                address: wallet5.address,
                balance: stakeAmount
            }]
        )

        await addToOverwatchStake(
            overwatchNodeContract1, 
            overwatchNodeId,
            wallet5.address,
            stakeAmount,
            ethersProvider,
            true
        )

        let beforeStakeBalance = await api.query.network.accountOverwatchStake(wallet5.address);
        expect(BigInt(beforeStakeBalance.toString())).to.equal(BigInt(afterRegisterStakeBalance.toString()) + stakeAmount)
        let beforePrecompileStakeBalance = await overwatchNodeContract1.accountOverwatchStake(wallet5.address);
        expect(BigInt(beforeStakeBalance.toString())).to.be.equal(BigInt(afterRegisterStakeBalance.toString()) + stakeAmount)

        await removeOverwatchStake(
            overwatchNodeContract1, 
            wallet5.address,
            BigInt(1e18),
            ethersProvider,
            true
        )

        let stakeBalance = await api.query.network.accountOverwatchStake(wallet5.address);
        let precompileStakeBalance = await overwatchNodeContract1.accountOverwatchStake(wallet5.address);
        expect(Number(beforeStakeBalance.toString())).to.be.greaterThan(Number(stakeBalance.toString()))
        expect(Number(beforePrecompileStakeBalance.toString())).to.be.greaterThan(Number(precompileStakeBalance.toString()))

        console.log("✅ Remove overwatch node testing complete")
    })
});