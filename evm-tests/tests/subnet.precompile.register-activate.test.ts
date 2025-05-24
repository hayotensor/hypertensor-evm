import * as assert from "assert";
import { getDevnetApi, getRandomSubstrateKeypair, waitForFinalizedBlock } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { convertPublicKeyToSs58, convertH160ToSS58, convertSs58ToAccountId20, ss58ToAccountId20 } from "../src/address-utils"
import { ethers } from "ethers"
import { generateRandomEthersWallet, generateRandomString, getPublicClient, hash, SEED_PATH, STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS, TEST_PATH } from "../src/utils"
import { Option } from '@polkadot/types';
import {
    addToDelegateStake,
    forceSetBalanceToEthAddress, forceSetBalanceToSs58Address,
    getCurrentRegistrationCost,
    registerSubnet,
    removeDelegateStake,
    swapDelegateStake,
    transferBalanceFromSudo
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { AbiItem, PublicClient } from "viem";
import { forceSetBalance } from "../src/test";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";

// npm test -- -g "Test subnet register activate"
describe("Test subnet register activate", () => {
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

    // npm test -- -g "testing register subnet"
    // Status: passing
    it("testing register subnet", async () => {
        const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);

        const subnetName = generateRandomString(30)
        const repo = generateRandomString(30)
        const description = generateRandomString(30)
        const misc = generateRandomString(30)

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

        const subnetId = await subnetContract.getSubnetId(subnetName);
        expect(BigInt(subnetId)).to.not.equal(BigInt(0))

        const subnetData = await api.query.network.subnetsData(subnetId)
        expect(subnetData != undefined);

        const subnetDataOpt = subnetData as Option<any>;
        expect(subnetDataOpt.isSome);

        if (subnetDataOpt.isSome) {
            const subnetData = subnetDataOpt.unwrap();
            const human = subnetData.toHuman();

            const subnetIdStored = human.id;
            const subnetNameStored = human.name;
            const repoStored = human.repo;
            const descriptionStored = human.description;
            const miscStored = human.misc;

            expect(Number(subnetIdStored)).to.equal(Number(subnetId))
            expect(subnetNameStored).to.equal(subnetName)
            expect(repoStored).to.equal(repo)
            expect(descriptionStored).to.equal(description)
            expect(miscStored).to.equal(misc)
        }
    })


    // TODO
    it("testing activate subnet", async () => {
        const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);
    })
});