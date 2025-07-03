import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEthersWallet, generateRandomString, getPublicClient, STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    addToDelegateStake,
    getCurrentRegistrationCost,
    registerSubnet,
    removeDelegateStake,
    transferBalanceFromSudo
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";

// Status: pending
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

    let publicClient: PublicClient;
    let papiApi: TypedApi<typeof dev>
    let api: ApiPromise

    const sudoTransferAmount = BigInt(10000e18)
    const stakeAmount = BigInt(100e18)

    const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);

    const subnetName = generateRandomString(30)
    const repo = generateRandomString(30)
    const description = generateRandomString(30)
    const misc = generateRandomString(30)
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
    })

    // Status: pending
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
          stakeAmount
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

        // TODO: Sudo decrease unbondings period before removing stake

        await removeDelegateStake(
          stakingContract, 
          subnetId,
          sharesAfterDelegateStake
        )

        const sharesAfterRemove = await stakingContract.accountSubnetDelegateStakeShares(wallet1.address, subnetId);
        const balanceAfterRemove = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, subnetId);

		expect(sharesAfterDelegateStake).to.be.greaterThan(sharesAfterRemove);
        expect(balanceAfterDelegateStake).to.be.greaterThan(balanceAfterRemove);

        console.log("balanceAfterDelegateStake", balanceAfterDelegateStake)

        const unbondings = (await api.query.network.stakeUnbondingLedger(wallet1.address)).toHuman();
        console.log("unbondings", unbondings)

        // TODO: Increase block promise await
    })
});