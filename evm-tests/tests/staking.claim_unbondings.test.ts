import * as assert from "assert";
import { getDevnetApi } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEthersWallet, generateRandomString, getPublicClient, STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS, waitForBlocks } from "../src/utils"
import {
    addToDelegateStake,
    claimUnbondings,
    getCurrentRegistrationCost,
    registerSubnet,
    removeDelegateStake,
    transferBalanceFromSudo,
    waitForFinalizedBalance
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";

// Status: passing
// npm test -- -g "test claim unbondings-0x310crc12"
describe("test claim unbondings-0x310crc12", () => {
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

    const KEY_TYPES = [1, 2]

    const BOOTNODES = [
        generateRandomString(6),
        generateRandomString(6)
    ]


    let publicClient: PublicClient;
    let papiApi: TypedApi<typeof dev>
    let api: ApiPromise

    const sudoTransferAmount = BigInt(10000e18)
    const stakeAmount = BigInt(100e18)

    const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);
    let subnetId: string;

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

        // ==============
        // Register subnet
        // ==============
        const cost = await getCurrentRegistrationCost(subnetContract, api)
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
    })

    // Status: passing
    // npm test -- -g "testing claim unbondings-0xwvdvih3209556wv"
    it("testing claim unbondings-0xwvdvih3209556wv", async () => {
        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet1);

        const sharesBeforeDelegateStake = await stakingContract.accountSubnetDelegateStakeShares(
            wallet1.address, 
            subnetId
        );
        const balanceBeforeDelegateStake = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, subnetId);

        // Ensure fresh wallet
        expect(Number(sharesBeforeDelegateStake)).to.be.equal(0);
        expect(Number(balanceBeforeDelegateStake)).to.be.equal(0);

        // ==================
        // Add delegate stake
        // ==================
        await addToDelegateStake(
          stakingContract, 
          subnetId,
          stakeAmount,
          BigInt(stakeAmount)
        )

        // =====================
        // Remove delegate stake
        // =====================

        const sharesAfterDelegateStake = await stakingContract.accountSubnetDelegateStakeShares(
            wallet1.address, 
            subnetId
        );
        const balanceAfterDelegateStake = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, subnetId);

        // Ensure there is a balance
        expect(Number(sharesAfterDelegateStake)).to.not.equal(0);
        expect(Number(balanceAfterDelegateStake)).to.not.equal(0);

        await removeDelegateStake(
          stakingContract, 
          subnetId,
          sharesAfterDelegateStake
        )

        const sharesAfterRemove = await stakingContract.accountSubnetDelegateStakeShares(wallet1.address, subnetId);
        const balanceAfterRemove = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, subnetId);

        // After removal, staking balances and shares should decrease
        expect(sharesAfterDelegateStake).to.be.greaterThan(sharesAfterRemove);
        expect(balanceAfterDelegateStake).to.be.greaterThan(balanceAfterRemove);

        const unbondings = (await api.query.network.stakeUnbondingLedgerV2(wallet1.address)).toHuman();

        const beforeFinalizedBalance = await waitForFinalizedBalance(
            papiApi, 
            wallet1.address, 
            (await papiApi.query.System.Account.getValue(wallet1.address)).data.free
        );

        await claimUnbondings(
          stakingContract
        )

        const afterFinalizedBalance = await waitForFinalizedBalance(
            papiApi, 
            wallet1.address, 
            (await papiApi.query.System.Account.getValue(wallet1.address)).data.free
        );

        expect(Number(afterFinalizedBalance)).to.be.greaterThan(Number(beforeFinalizedBalance));

        console.log("✅ Claim unbondings testing complete")
    })
});