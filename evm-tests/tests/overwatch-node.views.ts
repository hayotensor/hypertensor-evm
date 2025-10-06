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

// npm test -- -g "test overwatch view functions-0xfff90000"
describe("test overwatch view functions-0xfff90000", () => {
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
            cost,
            ethersProvider,
            true
        )

        subnetId = await subnetContract.getSubnetId(subnetName);

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

        overwatchMinStake = await api.query.network.overwatchCommits();
    })

    // Status: passing
    // npm test -- -g "testing overwatch view functions-0xdgewve2346"
    it("testing overwatch view functions-0xdgewve2346", async () => {
        let stakeBalance = await api.query.network.accountOverwatchStake(wallet5.address);
        expect(BigInt(stakeBalance.toString())).to.equal(BigInt(overwatchMinStake!.toString()))

        let precompileStakeBalance = await overwatchNodeContract1.accountOverwatchStake(wallet5.address);
        expect(BigInt(stakeBalance.toString())).to.be.equal(BigInt(precompileStakeBalance.toString()))


        let totalOverwatchStake = await api.query.network.totalOverwatchStake();
        expect(BigInt(totalOverwatchStake.toString())).to.not.equal(BigInt(0))

        let precompiletotalOverwatchStake = await overwatchNodeContract1.totalOverwatchStake();
        expect(BigInt(totalOverwatchStake.toString())).to.be.equal(BigInt(precompiletotalOverwatchStake.toString()))

        
        let maxOverwatchNodes = await api.query.network.maxOverwatchNodes();
        expect(Number(maxOverwatchNodes.toString())).to.not.equal(0)

        let precompileMaxOverwatchNodes = await overwatchNodeContract1.maxOverwatchNodes();
        expect(Number(maxOverwatchNodes.toString())).to.be.equal(Number(precompileMaxOverwatchNodes.toString()))


        let totalOverwatchNodes = await api.query.network.totalOverwatchNodes();
        expect(Number(totalOverwatchNodes.toString())).to.not.equal(0)

        let precompileTotalOverwatchNodes = await overwatchNodeContract1.totalOverwatchNodes();
        expect(Number(totalOverwatchNodes.toString())).to.be.equal(Number(precompileTotalOverwatchNodes.toString()))


        let totalOverwatchNodeUids = await api.query.network.totalOverwatchNodeUids();
        expect(Number(totalOverwatchNodeUids.toString())).to.not.equal(0)

        let precompileTotalOverwatchNodeUids = await overwatchNodeContract1.totalOverwatchNodeUids();
        expect(Number(totalOverwatchNodeUids.toString())).to.be.equal(Number(precompileTotalOverwatchNodeUids.toString()))


        let overwatchEpochLengthMultiplier = await api.query.network.overwatchEpochLengthMultiplier();
        expect(Number(overwatchEpochLengthMultiplier)).to.not.equal(0)

        let precompileOverwatchEpochLengthMultiplier = await overwatchNodeContract1.overwatchEpochLengthMultiplier();
        expect(Number(overwatchEpochLengthMultiplier.toString())).to.be.equal(Number(precompileOverwatchEpochLengthMultiplier.toString()))


        let overwatchMinStakeBalance = await api.query.network.overwatchMinStakeBalance();
        expect(Number(overwatchMinStakeBalance.toString())).to.not.equal(Number(0))

        let precompileOverwatchMinStakeBalance = await overwatchNodeContract1.overwatchMinStakeBalance();
        expect(overwatchMinStakeBalance.toString()).to.be.equal(precompileOverwatchMinStakeBalance.toString())


        let overwatchNodeBlacklist = await api.query.network.overwatchNodeBlacklist(wallet5.address);
        expect(overwatchNodeBlacklist.toString()).to.equal("false")

        let precompileOverwatchNodeBlacklist = await overwatchNodeContract1.overwatchNodeBlacklist(wallet5.address);
        expect(overwatchNodeBlacklist.toString()).to.be.equal(precompileOverwatchNodeBlacklist.toString());

        let precompileGetCurrentOverwatchEpoch = await overwatchNodeContract1.getCurrentOverwatchEpoch();
        expect(Number(precompileGetCurrentOverwatchEpoch.toString())).to.be.greaterThan(Number(0))
        
        console.log("✅ Registering overwatch node testing complete")
    })
});