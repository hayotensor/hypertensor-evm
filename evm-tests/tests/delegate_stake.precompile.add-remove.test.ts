import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair, waitForFinalizedBlock } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58, convertH160ToSS58, convertSs58ToAccountId20, ss58ToAccountId20 } from "../src/address-utils"
// import { raoToEth, tao } from "../src/balance-math"
import { ethers } from "ethers"
import { generateRandomEthersWallet, getPublicClient, hash, SEED_PATH, STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
// import { convertH160ToPublicKey } from "../src/address-utils"
import {
    addToDelegateStake,
    forceSetBalanceToEthAddress, forceSetBalanceToSs58Address,
    removeDelegateStake,
    transferBalanceFromSudo
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { AbiItem, PublicClient } from "viem";
import { forceSetBalance } from "../src/test";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";

describe("Test staking", () => {
    // init eth part
    const wallet1 = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();
    let publicClient: PublicClient;
    // init substrate part
    const hotkey = getRandomSubstrateKeypair();
    const coldkey = getRandomSubstrateKeypair();
    const proxy = getRandomSubstrateKeypair();

    let papiApi: TypedApi<typeof dev>
    let api: ApiPromise

    const sudoTransferAmount = BigInt(1000e18)
    const stakeAmount = BigInt(100e18)

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
    })

    it("testing add delegate stake", async () => {
        const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);
        const subnetId = await subnetContract.getSubnetId(SEED_PATH);

        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet1);

        const sharesBefore = await stakingContract.accountSubnetDelegateStakeShares(
            wallet1.address, 
            subnetId
        );
        const balanceBefore = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, subnetId);

        await addToDelegateStake(
          stakingContract, 
          subnetId,
          stakeAmount
        )

        const sharesAfter = await stakingContract.accountSubnetDelegateStakeShares(wallet1.address, subnetId);
        const balanceAfter = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, subnetId);

		expect(sharesBefore).to.be.lessThan(sharesAfter);
        expect(balanceBefore).to.be.lessThan(balanceAfter);
    })

    it("testing remove delegate stake", async () => {
        const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);
        const subnetId = await subnetContract.getSubnetId(SEED_PATH);

        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet1);

        // ==================
        // Add delegate stake
        // ==================
        await addToDelegateStake(
          stakingContract, 
          subnetId,
          stakeAmount
        )

        // ==================
        // Remove delegate stake
        // ==================

        const sharesBefore = await stakingContract.accountSubnetDelegateStakeShares(
            wallet1.address, 
            subnetId
        );
        const balanceBefore = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, subnetId);

        await removeDelegateStake(
          stakingContract, 
          subnetId,
          sharesBefore
        )

        const sharesAfter = await stakingContract.accountSubnetDelegateStakeShares(wallet1.address, subnetId);
        const balanceAfter = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, subnetId);

		expect(sharesBefore).to.be.greaterThan(sharesAfter);
        expect(balanceBefore).to.be.greaterThan(balanceAfter);
    })
});