import { getDevnetApi } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEd25519PeerId, generateRandomEthersWallet, generateRandomString, getPublicClient, STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    addToDelegateStake,
    getCurrentRegistrationCost,
    registerSubnet,
    registerSubnetNode,
    swapDelegateStake,
    transferBalanceFromSudo,
    updateSwapQueue
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";
import { Option } from '@polkadot/types';

// npm test -- -g "test swap and transfer delegate staking-0xrh2"
describe("test swap and transfer delegate staking-0xrh2", () => {
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
    let fromSubnetId: string;
    let toSubnetId: string;
    let subnetNodeId: string;

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

        fromSubnetId = await subnetContract.getSubnetId(subnetName);

        cost = await getCurrentRegistrationCost(subnetContract, api)
        const subnetName2 = generateRandomString(30)
        const repo2 = generateRandomString(30)
        const description2 = generateRandomString(30)
        const misc2 = generateRandomString(30)

        await registerSubnet(
            subnetContract, 
            cost,
            subnetName2,
            repo2,
            description2,
            misc2,
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

        toSubnetId = await subnetContract.getSubnetId(subnetName2);

        // ================
        // Add subnet nodes
        // ================
        let peer1 = await generateRandomEd25519PeerId()
        let peer2 = await generateRandomEd25519PeerId()
        let peer3 = await generateRandomEd25519PeerId()
        const delegateRewardRate = "0";
        
        const bootnode = generateRandomString(16)
        const unique = generateRandomString(16)
        const nonUnique = generateRandomString(16)

        await registerSubnetNode(
          subnetContract, 
          fromSubnetId,
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

        let subnetNodeIdFetched = await api.query.network.hotkeySubnetNodeId(fromSubnetId, wallet2.address);

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
    // npm test -- -g "testing update swap queue to subnet delegate stake-0xH3284oufDWe28"
    it("testing update swap queue to subnet delegate stake-0xH3284oufDWe28", async () => {
        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet1);

        // ==================
        // Add delegate stake
        // ==================
        const sharesBefore = await stakingContract.accountSubnetDelegateStakeShares(
          wallet1.address, 
          fromSubnetId
        );
        const balanceBefore = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, fromSubnetId);

        console.log("adding to delegate stake")

        await addToDelegateStake(
          stakingContract, 
          fromSubnetId,
          stakeAmount,
          BigInt(0)
        );

        const sharesAfter = await stakingContract.accountSubnetDelegateStakeShares(wallet1.address, fromSubnetId);
        const balanceAfter = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, fromSubnetId);

        expect(sharesBefore).to.be.lessThan(sharesAfter);
        expect(sharesBefore).to.not.equal(0);
        expect(balanceBefore).to.be.lessThan(balanceAfter);

        // ==================
        // Swap delegate stake
        // ==================
        const nextSwapId = await api.query.network.nextSwapId();

        console.log("swap delegate stake")

        await swapDelegateStake(
          stakingContract, 
          fromSubnetId,
          toSubnetId,
          sharesAfter
        );

        // Ensure shares decreased
        const fromSharesAfter = await stakingContract.accountSubnetDelegateStakeShares(wallet1.address, fromSubnetId);
        const fromBalanceAfter = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, fromSubnetId);

        expect(fromSharesAfter).to.be.lessThan(sharesAfter);
        expect(fromBalanceAfter).to.be.lessThan(balanceAfter);

        console.log("calling get swap queue")

        // Ensure in the queue
        const swapCallQueue = await api.query.network.swapCallQueue(nextSwapId);

        expect(swapCallQueue != undefined);

        const swapCallQueueOpt = swapCallQueue as Option<any>;
        expect(swapCallQueueOpt.isSome);

        let swapQueueId = 0;

        if (swapCallQueueOpt.isSome) {
            const swapCallQueue = swapCallQueueOpt.unwrap();
            const human = swapCallQueue.toHuman();
            const callType = Object.keys(human.call);
            expect(callType[0]).to.be.equal("SwapToSubnetDelegateStake");

            const swapCallQueueId = human.id;
            swapQueueId = swapCallQueueId;
            const accountIdHuman = human.call.SwapToSubnetDelegateStake.accountId;
            const toSubnetIdHuman = human.call.SwapToSubnetDelegateStake.toSubnetId;
            const balanceHuman = human.call.SwapToSubnetDelegateStake.balance;
            const stringWithoutCommas = balanceHuman.replace(/,/g, "");
            expect(Number(swapCallQueueId.toString())).to.be.equal(Number(nextSwapId.toString()));
            expect(accountIdHuman).to.be.equal(wallet1.address);
            expect(Number(toSubnetIdHuman)).to.be.equal(Number(toSubnetId));
            expect(Number(stringWithoutCommas.toString())).to.be.greaterThan(0);
        }

        // Ensure `getQueuedSwapCall` works
        let evmSwapQueue = await stakingContract.getQueuedSwapCall(swapQueueId);
        let _id = evmSwapQueue[0]
        let _account_id = evmSwapQueue[1]
        let _call_type = evmSwapQueue[2]
        let _to_subnet_id = evmSwapQueue[3]
        let _to_subnet_node_id = evmSwapQueue[4]
        let _balance = evmSwapQueue[5]
        let _queued_at_block = evmSwapQueue[6]
        let _execute_after_blocks = evmSwapQueue[7]
        expect(Number(_id) == swapQueueId);
        expect(_account_id == wallet1.address);
        expect(Number(_call_type) == 0); // SwapToSubnetDelegateStake
        expect(Number(_queued_at_block) > 0);
        expect(Number(_execute_after_blocks) > 0);
        expect(Number(_balance) > 0);
        expect(Number(_to_subnet_id) == Number(toSubnetId));
        expect(Number(_to_subnet_node_id) == 0);

        // Update the queue
        // Update back to the from subnet ID
        await updateSwapQueue(
          stakingContract, 
          swapQueueId.toString(),
          "0",
          fromSubnetId.toString(),
          "0"
        );

        const swapCallQueueAfter = await api.query.network.swapCallQueue(nextSwapId);

        expect(swapCallQueueAfter != undefined);

        const swapCallQueueAfterOpt = swapCallQueueAfter as Option<any>;
        expect(swapCallQueueAfterOpt.isSome);

        swapQueueId = 0;

        if (swapCallQueueAfterOpt.isSome) {
            const swapCallQueue = swapCallQueueAfterOpt.unwrap();
            const human = swapCallQueue.toHuman();
            const callType = Object.keys(human.call);
            expect(callType[0]).to.be.equal("SwapToSubnetDelegateStake");
            const swapCallQueueId = human.id;
            swapQueueId = swapCallQueueId;
            const accountIdHuman = human.call.SwapToSubnetDelegateStake.accountId;
            const toSubnetIdHuman = human.call.SwapToSubnetDelegateStake.toSubnetId;
            const balanceHuman = human.call.SwapToSubnetDelegateStake.balance;
            const stringWithoutCommas = balanceHuman.replace(/,/g, "");
            expect(Number(swapCallQueueId.toString())).to.be.equal(Number(nextSwapId.toString()));
            expect(accountIdHuman).to.be.equal(wallet1.address);
            expect(Number(toSubnetIdHuman)).to.be.equal(Number(fromSubnetId));
            expect(Number(stringWithoutCommas.toString())).to.be.greaterThan(0);
        }

        // Ensure `getQueuedSwapCall` works
        evmSwapQueue = await stakingContract.getQueuedSwapCall(swapQueueId);
        _id = evmSwapQueue[0]
        _account_id = evmSwapQueue[1]
        _call_type = evmSwapQueue[2]
        _to_subnet_id = evmSwapQueue[3]
        _to_subnet_node_id = evmSwapQueue[4]
        _balance = evmSwapQueue[5]
        _queued_at_block = evmSwapQueue[6]
        _execute_after_blocks = evmSwapQueue[7]
        expect(Number(_id) == swapQueueId);
        expect(_account_id == wallet1.address);
        expect(Number(_call_type) == 0); // SwapToSubnetDelegateStake
        expect(Number(_queued_at_block) > 0);
        expect(Number(_execute_after_blocks) > 0);
        expect(Number(_balance) > 0);
        expect(Number(_to_subnet_id) == Number(fromSubnetId));
        expect(Number(_to_subnet_node_id) == 0);

        // Update the queue
        // Update to to node delegate staking to node ID 
        await updateSwapQueue(
          stakingContract, 
          swapQueueId.toString(),
          "1",
          fromSubnetId.toString(),
          subnetNodeId.toString()
        );

        const swapCallQueueAfter2 = await api.query.network.swapCallQueue(nextSwapId);

        expect(swapCallQueueAfter2 != undefined);

        const swapCallQueueAfter2Opt = swapCallQueueAfter2 as Option<any>;
        expect(swapCallQueueAfter2Opt.isSome);


        if (swapCallQueueAfter2Opt.isSome) {
            const swapCallQueue = swapCallQueueAfter2Opt.unwrap();
            const human = swapCallQueue.toHuman();
            const callType = Object.keys(human.call);
            expect(callType[0]).to.be.equal("SwapToNodeDelegateStake");
            const swapCallQueueId = human.id;
            const accountIdHuman = human.call.SwapToNodeDelegateStake.accountId;
            const toSubnetIdHuman = human.call.SwapToNodeDelegateStake.toSubnetId;
            const toSubnetNodeIdHuman = human.call.SwapToNodeDelegateStake.toSubnetId;
            const balanceHuman = human.call.SwapToNodeDelegateStake.balance;
            const stringWithoutCommas = balanceHuman.replace(/,/g, "");
            expect(Number(swapCallQueueId.toString())).to.be.equal(Number(nextSwapId.toString()));
            expect(accountIdHuman).to.be.equal(wallet1.address);
            expect(Number(toSubnetIdHuman)).to.be.equal(Number(fromSubnetId));
            expect(Number(toSubnetNodeIdHuman)).to.be.equal(Number(subnetNodeId));
            expect(Number(stringWithoutCommas.toString())).to.be.greaterThan(0);
        }

        evmSwapQueue = await stakingContract.getQueuedSwapCall(swapQueueId);
        _id = evmSwapQueue[0]
        _account_id = evmSwapQueue[1]
        _call_type = evmSwapQueue[2]
        _to_subnet_id = evmSwapQueue[3]
        _to_subnet_node_id = evmSwapQueue[4]
        _balance = evmSwapQueue[5]
        _queued_at_block = evmSwapQueue[6]
        _execute_after_blocks = evmSwapQueue[7]
        expect(Number(_id) == swapQueueId);
        expect(_account_id == wallet1.address);
        expect(Number(_call_type) == 1); // SwapToSubnetDelegateStake
        expect(Number(_queued_at_block) > 0);
        expect(Number(_execute_after_blocks) > 0);
        expect(Number(_balance) > 0);
        expect(Number(_to_subnet_id) == Number(fromSubnetId));
        expect(Number(_to_subnet_node_id) == Number(subnetNodeId));

        console.log("✅ Update delegate stake swap queue testing complete")
      })
});