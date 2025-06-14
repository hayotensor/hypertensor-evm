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

// npm test -- -g "test delegate staking-0xDy454g"
describe("test delegate staking-0xDy454g", () => {
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
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();

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

    // Status: passing
    // npm test -- -g "testing add delegate stake-0xpf3mf"
    it("testing add delegate stake-0xpf3mf", async () => {
        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet1);

        // ==================
        // Add delegate stake
        // ==================
        const sharesBefore = await stakingContract.accountSubnetDelegateStakeShares(
            wallet1.address, 
            subnetId
        );
        const balanceBefore = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, subnetId);

        expect(Number(sharesBefore)).to.be.equal(0);
        expect(Number(balanceBefore)).to.be.equal(0);

        await addToDelegateStake(
          stakingContract, 
          subnetId,
          stakeAmount
        )

        const sharesAfter = await stakingContract.accountSubnetDelegateStakeShares(wallet1.address, subnetId);
        const balanceAfter = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, subnetId);

        expect(Number(sharesAfter)).to.be.greaterThan(0);
        expect(Number(balanceAfter)).to.be.greaterThan(0);
		expect(sharesBefore).to.be.lessThan(sharesAfter);
        expect(balanceBefore).to.be.lessThan(balanceAfter);
    })

    // Status: passing
    // npm test -- -g "testing remove delegate stake-0xe5"
    it("testing remove delegate stake-0xe5", async () => {
        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet2);

        const sharesBeforeDelegateStake = await stakingContract.accountSubnetDelegateStakeShares(
            wallet2.address, 
            subnetId
        );
        const balanceBeforeDelegateStake = await stakingContract.accountSubnetDelegateStakeBalance(wallet2.address, subnetId);

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
            wallet2.address, 
            subnetId
        );
        const balanceAfterDelegateStake = await stakingContract.accountSubnetDelegateStakeBalance(wallet2.address, subnetId);

        // Ensure there is a balance
        expect(Number(sharesAfterDelegateStake)).to.not.equal(0);
        expect(Number(balanceAfterDelegateStake)).to.not.equal(0);

        await removeDelegateStake(
          stakingContract, 
          subnetId,
          sharesAfterDelegateStake
        )

        const sharesAfterRemove = await stakingContract.accountSubnetDelegateStakeShares(wallet2.address, subnetId);
        const balanceAfterRemove = await stakingContract.accountSubnetDelegateStakeBalance(wallet2.address, subnetId);

		expect(sharesAfterDelegateStake).to.be.greaterThan(sharesAfterRemove);
        expect(balanceAfterDelegateStake).to.be.greaterThan(balanceAfterRemove);
    })
});