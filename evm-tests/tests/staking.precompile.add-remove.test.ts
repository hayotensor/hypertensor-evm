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
    transferBalanceFromSudo
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { AbiItem, PublicClient } from "viem";
import { forceSetBalance } from "../src/test";
import { ApiPromise, WsProvider } from "@polkadot/api";

describe("Test neuron precompile reveal weights", () => {
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

        // convertSs58ToAccountId20(ss58: string)
        // const hotkeySs58 = convertPublicKeyToSs58(hotkey.publicKey)
        // console.log("hotkeySs58       ", hotkeySs58)
        // const hotkeyAccountId20 = convertSs58ToAccountId20(hotkeySs58)
        // const hotkeyAccountId202 = ss58ToAccountId20(hotkeySs58)

        // console.log("hotkeyAccountId20 ", hotkeyAccountId20)
        // console.log("hotkeyAccountId202", hotkeyAccountId202)

        // await forceSetBalanceToSs58Address(api, hotkeyAccountId20)
        // console.log("wallet1.address", wallet1.address)
        // console.log("hotkey.publicKey", hotkey.publicKey)
        // await forceSetBalanceToSs58Address(api, wallet1.address)

        // await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
        // await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
        // await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(proxy.publicKey))
        console.log("wallet1.address", wallet1.address)
        // await forceSetBalanceToEthAddress(api, wallet1.address)
        // await forceSetBalanceToEthAddress(api, wallet2.address)
    })

    it("Can add stake", async () => {
        const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);

        const subnetId = await subnetContract.getSubnetId(SEED_PATH);
        console.log("staking subnetId:       ", subnetId)

        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet1);

        const sharesBefore = await stakingContract.accountSubnetDelegateStakeShares(
            wallet1.address, 
            subnetId
        );
        console.log("staking sharesBefore:  ", sharesBefore)

        const balanceOnChain = (await papiApi.query.System.Account.getValue(wallet1.address)).data.free
        console.log("staking balanceOnChain:  ", balanceOnChain)

        // await stakingContract.addToDelegateStake(subnetId, stakeAmount)
        await addToDelegateStake(
          stakingContract, 
          subnetId,
          stakeAmount
        )

        const sharesAfter = await stakingContract.accountSubnetDelegateStakeShares(wallet1.address, subnetId);

        console.log("staking sharesAfter:   ", sharesAfter)

        const balanceAfter = await stakingContract.accountSubnetDelegateStakeBalance(wallet1.address, subnetId);

        console.log("staking balanceAfter:   ", balanceAfter)

    })
});