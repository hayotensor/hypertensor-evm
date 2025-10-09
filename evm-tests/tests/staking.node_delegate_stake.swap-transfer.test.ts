import * as assert from "assert";
import { getDevnetApi } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEd25519PeerId, generateRandomEthersWallet, generateRandomString, getPublicClient, STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    addToNodeDelegateStake,
    batchTransferBalanceFromSudo,
    getCurrentRegistrationCost,
    registerSubnet,
    registerSubnetNode,
    swapNodeDelegateStake,
    transferNodeDelegateStake
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { AbiItem, PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";
import { Option } from '@polkadot/types';

// npm test -- -g "test swap and transfer delegate staking-0x0101d"
describe("test swap and transfer delegate staking-0x0101d", () => {
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

    // from
    let fromSubnetId: string;
    let toSubnetId: string;
    // to
    let fromSubnetNodeId: string;
    let toSubnetNodeId: string;

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

        fromSubnetId = await subnetContract.getSubnetId(subnetName);

        // ================
        // Add subnet nodes
        // ================
        const delegateRewardRate = "0";

        // from 
        const peer1 = await generateRandomEd25519PeerId()
        const peer2 = await generateRandomEd25519PeerId()
        const peer3 = await generateRandomEd25519PeerId()

        const bootnode = generateRandomString(16)
        const unique = generateRandomString(16)
        const nonUnique = generateRandomString(16)

        const fromSubnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);

        await registerSubnetNode(
          fromSubnetContract, 
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

        let fromSubnetNodeIdFetched = await api.query.network.hotkeySubnetNodeId(fromSubnetId, wallet2.address);

        const fromSubnetNodeIdOpt = fromSubnetNodeIdFetched as Option<any>;
        expect(fromSubnetNodeIdOpt.isSome);

        let fromSubnetNodeExists: boolean = false;
        if (fromSubnetNodeIdOpt.isSome) {
            fromSubnetNodeExists = true;
            const subnetNodeIdUnwrapped = fromSubnetNodeIdOpt.unwrap();
            const human = subnetNodeIdUnwrapped.toHuman();
            fromSubnetNodeId = human?.toString();
            expect(Number(fromSubnetNodeId)).to.be.greaterThan(0);
        }
        expect(fromSubnetNodeExists);
        expect(fromSubnetNodeId != undefined);

        // to
        const toUnique = generateRandomString(16)
        const peer4 = await generateRandomEd25519PeerId()
        const peer5 = await generateRandomEd25519PeerId()
        const peer6 = await generateRandomEd25519PeerId()

        const toSubnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet3);

        await registerSubnetNode(
          toSubnetContract, 
          fromSubnetId,
          wallet4.address,
          peer4,
          peer5,
          peer6,
          bootnode,
          delegateRewardRate,
          BigInt(minStake.toString()),
          toUnique,
          nonUnique,
          "100"
        )

        let toSubnetNodeIdFetched = await api.query.network.hotkeySubnetNodeId(fromSubnetId, wallet4.address);

        const toSubnetNodeIdOpt = toSubnetNodeIdFetched as Option<any>;
        expect(toSubnetNodeIdOpt.isSome);

        let toSubnetNodeExists: boolean = false;
        if (toSubnetNodeIdOpt.isSome) {
            toSubnetNodeExists = true;
            const subnetNodeIdUnwrapped = toSubnetNodeIdOpt.unwrap();
            const human = subnetNodeIdUnwrapped.toHuman();
            toSubnetNodeId = human?.toString();
            expect(Number(toSubnetNodeId)).to.be.greaterThan(0);
        }
        expect(toSubnetNodeExists);
        expect(toSubnetNodeId != undefined);
      })

    // Status: passing
    // npm test -- -g "testing swap node delegate stake-0x9284jw4"
    it("testing swap node delegate stake-0x9284jw4", async () => {
        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet1);

        // ==================
        // Add delegate stake
        // ==================
        const sharesBefore = await stakingContract.accountNodeDelegateStakeShares(
            wallet1.address, 
            fromSubnetId,
            fromSubnetNodeId
        );
        const balanceBefore = await stakingContract.accountNodeDelegateStakeBalance(wallet1.address, fromSubnetId, fromSubnetNodeId);

        await addToNodeDelegateStake(
          stakingContract, 
          fromSubnetId,
          fromSubnetNodeId,
          stakeAmount
        )

        const sharesAfter = await stakingContract.accountNodeDelegateStakeShares(wallet1.address, fromSubnetId, fromSubnetNodeId);
        const balanceAfter = await stakingContract.accountNodeDelegateStakeBalance(wallet1.address, fromSubnetId, fromSubnetNodeId);

        expect(sharesBefore).to.be.lessThan(sharesAfter);
        expect(balanceBefore).to.be.lessThan(balanceAfter);

        // ==================
        // Swap delegate stake
        // ==================
        await swapNodeDelegateStake(
          stakingContract, 
          fromSubnetId, // from
          fromSubnetNodeId, // from
          fromSubnetId, // to
          toSubnetNodeId, // to
          sharesBefore
        )

        const toBalanceAfter = await stakingContract.accountNodeDelegateStakeBalance(wallet1.address, fromSubnetId, toSubnetNodeId);

        expect(toBalanceAfter).to.be.within(balanceBefore * 0.99, balanceBefore); // Not recommended

        console.log("✅ Swap node delegate stake testing complete")
    })

    // Status: passing
    // npm test -- -g "testing transfer node delegate stake-0xing9592" 
    it("testing transfer node delegate stake-0xing9592", async () => {
        let stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet2);

        const sharesBeforeAdd = await stakingContract.accountNodeDelegateStakeShares(wallet2.address, fromSubnetId, fromSubnetNodeId);
        const balanceBeforeAdd = await stakingContract.accountNodeDelegateStakeBalance(wallet2.address, fromSubnetId, fromSubnetNodeId);

        // Ensure fresh wallet
        expect(Number(sharesBeforeAdd)).to.be.equal(0);
        expect(Number(balanceBeforeAdd)).to.be.equal(0);

        // ==================
        // Add delegate stake
        // ==================
        await addToNodeDelegateStake(
          stakingContract, 
          fromSubnetId,
          fromSubnetNodeId,
          stakeAmount
        )
 
        // =======================
        // Transfer delegate stake
        // =======================

        const sharesBeforeTransfer = await stakingContract.accountNodeDelegateStakeShares(wallet2.address, fromSubnetId, fromSubnetNodeId);
        const balanceBeforeTransfer = await stakingContract.accountNodeDelegateStakeBalance(wallet2.address, fromSubnetId, fromSubnetNodeId);

        // Ensure stake added
        expect(Number(sharesBeforeTransfer)).to.be.greaterThan(0);
        expect(Number(balanceBeforeTransfer)).to.be.greaterThan(0);

        const toSharesBeforeTransfer = await stakingContract.accountNodeDelegateStakeBalance(wallet8.address, fromSubnetId, fromSubnetNodeId);
        const toBalanceBeforeTransfer = await stakingContract.accountNodeDelegateStakeBalance(wallet8.address, fromSubnetId, fromSubnetNodeId);

        // Ensure fresh wallet
        expect(Number(toSharesBeforeTransfer)).to.be.equal(0);
        expect(Number(toBalanceBeforeTransfer)).to.be.equal(0);

        await transferNodeDelegateStake(
          stakingContract, 
          fromSubnetId,
          fromSubnetNodeId,
          wallet8.address,
          sharesBeforeTransfer
        )

        const sharesAfterTransfer = await stakingContract.accountNodeDelegateStakeShares(wallet2.address, fromSubnetId, fromSubnetNodeId);
        const balanceAfterTransfer = await stakingContract.accountNodeDelegateStakeBalance(wallet2.address, fromSubnetId, fromSubnetNodeId);

        const toSharesAfter = await stakingContract.accountNodeDelegateStakeShares(wallet8.address, fromSubnetId, fromSubnetNodeId);
        const toBalanceAfter = await stakingContract.accountNodeDelegateStakeBalance(wallet8.address, fromSubnetId, fromSubnetNodeId);

        expect(sharesAfterTransfer).to.be.equal(BigInt(0));
        expect(toSharesAfter).to.be.equal(sharesBeforeTransfer);

        console.log("✅ Transfer node delegate stake testing complete")
    })
});