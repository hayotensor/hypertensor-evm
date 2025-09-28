import { getDevnetApi } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEthersWallet, generateRandomString, getPublicClient, STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    addToDelegateStake,
    getCurrentRegistrationCost,
    registerSubnet,
    swapDelegateStake,
    transferBalanceFromSudo,
    transferDelegateStake
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { AbiItem, PublicClient } from "viem";
import { forceSetBalance } from "../src/test";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";
import { Option } from '@polkadot/types';

// npm test -- -g "test swap and transfer delegate staking-0x454v5v3fc23rh2"
describe("test swap and transfer delegate staking-0x454v5v3fc23rh2", () => {
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

        fromSubnetId = await subnetContract.getSubnetId(subnetName);

        cost = await getCurrentRegistrationCost(subnetContract, api)
        const subnetName2 = generateRandomString(30)
        const repo2 = generateRandomString(30)
        const description2 = generateRandomString(30)
        const misc2 = generateRandomString(30)

        await registerSubnet(
            subnetContract, 
            wallet2.address,
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
            ALL_ACCOUNTS,
            KEY_TYPES,
            BOOTNODES,
            cost
        )

        toSubnetId = await subnetContract.getSubnetId(subnetName2);

    })

    // Status: passing
    // npm test -- -g "testing swap delegate stake-0xHWe2SEv38"
    it("testing swap delegate stake-0xHWe2SEv38", async () => {
        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet1);

        // ==================
        // Add delegate stake
        // ==================
        const sharesBefore = await stakingContract.accountSubnetDelegateStakeShares(
          wallet1.address, 
          fromSubnetId
        );
        const balanceBefore = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, fromSubnetId);

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

        await swapDelegateStake(
          stakingContract, 
          fromSubnetId,
          toSubnetId,
          sharesAfter
        );

        // Ensure in the queue
        const swapCallQueue = await api.query.network.swapCallQueue(nextSwapId);

        expect(swapCallQueue != undefined);

        const swapCallQueueOpt = swapCallQueue as Option<any>;
        expect(swapCallQueueOpt.isSome);

        if (swapCallQueueOpt.isSome) {
            const swapCallQueue = swapCallQueueOpt.unwrap();
            const human = swapCallQueue.toHuman();

            const swapCallQueueId = human.id;
            const accountIdHuman = human.call.SwapToSubnetDelegateStake.accountId;
            const toSubnetIdHuman = human.call.SwapToSubnetDelegateStake.toSubnetId;
            const balanceHuman = human.call.SwapToSubnetDelegateStake.balance;
            const stringWithoutCommas = balanceHuman.replace(/,/g, "");
            expect(Number(swapCallQueueId.toString())).to.be.equal(Number(nextSwapId.toString()));
            expect(accountIdHuman).to.be.equal(wallet1.address);
            expect(Number(toSubnetIdHuman)).to.be.equal(Number(toSubnetId));
            expect(Number(stringWithoutCommas.toString())).to.be.equal(Number(balanceAfter));
        }

        // Ensure shares decreased
        const fromSharesAfter = await stakingContract.accountSubnetDelegateStakeShares(wallet1.address, fromSubnetId);
        const fromBalanceAfter = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, fromSubnetId);

        expect(fromSharesAfter).to.be.lessThan(sharesAfter);
        expect(fromBalanceAfter).to.be.lessThan(balanceAfter);

        console.log("✅ Delegate stake swap testing complete")
      })

    // Status: passing
    // npm test -- -g "testing transfer delegate stake-0xZPQG6sy123"
    it("testing transfer delegate stake-0xZPQG6sy123", async () => {
        let stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet2);

        const sharesBefore = await stakingContract.accountSubnetDelegateStakeShares(wallet2.address, fromSubnetId);
        const balanceBefore = await stakingContract.accountSubnetDelegateStakeBalance(wallet2.address, fromSubnetId);

        // Ensure fresh wallet
        expect(Number(sharesBefore)).to.be.equal(0);
        expect(Number(balanceBefore)).to.be.equal(0);

        // ==================
        // Add delegate stake
        // ==================
        await addToDelegateStake(
          stakingContract, 
          fromSubnetId,
          stakeAmount,
          BigInt(0)
        );

        // =======================
        // Transfer delegate stake 
        //
        // from wallet2 to wallet8
        // =======================

        const sharesBeforeTransfer = await stakingContract.accountSubnetDelegateStakeShares(wallet2.address, fromSubnetId);
        const balanceBeforeTransfer = await stakingContract.accountSubnetDelegateStakeBalance(wallet2.address, fromSubnetId);

        // Ensure stake added
        expect(Number(sharesBeforeTransfer)).to.be.greaterThan(0);
        expect(Number(balanceBeforeTransfer)).to.be.greaterThan(0);

        const toSharesBeforeTransfer = await stakingContract.accountSubnetDelegateStakeShares(wallet8.address, fromSubnetId);
        const toBalanceBeforeTransfer = await stakingContract.accountSubnetDelegateStakeBalance(wallet8.address, fromSubnetId);

        // Ensure fresh wallet
        expect(Number(toSharesBeforeTransfer)).to.be.equal(0);
        expect(Number(toBalanceBeforeTransfer)).to.be.equal(0);

        await transferDelegateStake(
          stakingContract, 
          fromSubnetId,
          wallet8.address,
          sharesBeforeTransfer
        )

        const sharesAfterTransfer = await stakingContract.accountSubnetDelegateStakeShares(wallet2.address, fromSubnetId);

        // From account should be 0
        expect(sharesAfterTransfer).to.be.equal(BigInt(0));

        const toSharesAfter = await stakingContract.accountSubnetDelegateStakeShares(wallet8.address, fromSubnetId);

        // To account should have the shares
        expect(toSharesAfter).to.be.equal(sharesBeforeTransfer);

        console.log("✅ Delegate stake transfer testing complete")
    })
});