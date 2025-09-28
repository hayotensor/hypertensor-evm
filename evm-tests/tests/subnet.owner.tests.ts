import * as assert from "assert";
import { getDevnetApi } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEd25519PeerId, generateRandomEthersWallet, generateRandomString, getPublicClient, STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS, waitForBlocks } from "../src/utils"
import { Option } from '@polkadot/types';
import {
    activateSubnet,
    addToDelegateStake,
    batchTransferBalanceFromSudo,
    getCurrentRegistrationCost,
    getMinSubnetDelegateStakeBalance,
    ownerAddInitialColdkeys,
    ownerDeactivateSubnet,
    ownerPauseSubnet,
    ownerRemoveInitialColdkeys,
    ownerUnpauseSubnet,
    ownerUpdateChurnLimit,
    ownerUpdateDelegateStakePercentage,
    ownerUpdateDescription,
    ownerUpdateIdleClassificationEpochs,
    ownerUpdateIncludedClassificationEpochs,
    ownerUpdateKeyTypes,
    ownerUpdateMaxNodePenalties,
    ownerUpdateMaxRegisteredNodes,
    ownerUpdateMaxStake,
    ownerUpdateMinStake,
    ownerUpdateMisc,
    ownerUpdateName,
    ownerUpdateRegistrationQueueEpochs,
    ownerUpdateRepo,
    registerSubnet,
    registerSubnetNode,
    transferBalanceFromSudo
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";

// npm test -- -g "Test subnet register activate-0xuhnrfvok"
describe("Test subnet owner-0xuhnrfvok", () => {
    // init eth part
    const wallet1 = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();
    const wallet3 = generateRandomEthersWallet();
    const wallet4 = generateRandomEthersWallet();
    const wallet5 = generateRandomEthersWallet();
    const wallet6 = generateRandomEthersWallet();
    const wallet7 = generateRandomEthersWallet();
    const wallet8 = generateRandomEthersWallet();

    const ALL_WALLETS = new Map([
        [wallet1, wallet2],
        [wallet3, wallet4],
        [wallet5, wallet6],
        [wallet7, wallet8],
    ]);

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

    const KEY_TYPES = [1, 2]

    const BOOTNODES = [
        generateRandomString(6),
        generateRandomString(6)
    ]

    let publicClient: PublicClient;
    let papiApi: TypedApi<typeof dev>
    let api: ApiPromise

    const sudoTransferAmount = BigInt(1000e18)

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

    // Status: passing
    // npm test -- -g "testing subnet owner functions-0xARAD3gb3"
    it("testing subnet owner functions-0xARAD3gb3", async () => {
        const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);
        
        const cost = await getCurrentRegistrationCost(subnetContract, api)
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

        const palletSubnetId = await api.query.network.subnetName(subnetName);
        expect(palletSubnetId != undefined);

        const subnetId = await subnetContract.getSubnetId(subnetName);
        expect(BigInt(subnetId)).to.not.equal(BigInt(0))

        const minStakeAmount = (await api.query.network.networkMinStakeBalance()).toString();
        const delegateRewardRate = "0";

        const coldkeys = Array.from(ALL_WALLETS.keys());
        const recipients = coldkeys.map(wallet => ({
            address: wallet.address,
            balance: BigInt(minStakeAmount + BigInt(500))
        }));

        await batchTransferBalanceFromSudo(
            api,
            papiApi,
            recipients
        )
        // Get enough nodes registered to meet min nodes requirement
        await Promise.all([...ALL_WALLETS.entries()].map(async ([coldkey, hotkey]) => {
            console.log("registering node coldkey", coldkey.address)
            console.log("registering node hotkey", hotkey.address)
            const accountSubnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, coldkey);

            const peer1 = await generateRandomEd25519PeerId()
            const peer2 = await generateRandomEd25519PeerId()
            const peer3 = await generateRandomEd25519PeerId()
            const bootnode = generateRandomString(5)
            const unique = generateRandomString(5)
            const nonUnique = generateRandomString(5)

            await registerSubnetNode(
                accountSubnetContract, 
                subnetId,
                hotkey.address,
                peer1,
                peer2,
                peer3,
                bootnode,
                delegateRewardRate,
                BigInt(minStakeAmount),
                unique,
                nonUnique,
                "100"
            );
        }));

        // get enough delegate stake to meet min delegate stake requirement
        const minDelegateStake = await getMinSubnetDelegateStakeBalance(
          subnetContract, 
          subnetId.toString()
        );

        const delegateStakerBalanceBefore = (await papiApi.query.System.Account.getValue(wallet1.address)).data.free

        await transferBalanceFromSudo(
            api,
            papiApi,
            SUB_LOCAL_URL,
            wallet1.address,
            BigInt(minDelegateStake + BigInt(100000)),
        )

        const delegateStakerBalance = (await papiApi.query.System.Account.getValue(wallet1.address)).data.free
        expect(Number(delegateStakerBalance)).to.be.greaterThanOrEqual(Number(minDelegateStake));

        // Delegate stake
        const stakingContract = new ethers.Contract(STAKING_CONTRACT_ADDRESS, STAKING_CONTRACT_ABI, wallet1);
        await addToDelegateStake(
            stakingContract, 
            subnetId,
            minDelegateStake,
            BigInt(0)
        );

        //
        // Subnet owner functions
        //

        const newSubnetName = generateRandomString(30)
        const newRepo = generateRandomString(30)
        const newDescription = generateRandomString(30)
        const newMisc = generateRandomString(30)
        const newChurnLimit = (BigInt((await api.query.network.maxChurnLimit()).toString()) - BigInt(1)).toString();
        const newMinStake = (BigInt((await api.query.network.networkMinStakeBalance()).toString()) + BigInt(1)).toString();
        const newMaxStake = (BigInt((await api.query.network.networkMaxStakeBalance()).toString()) - BigInt(1)).toString();
        const newDelegateStakePercentage = (BigInt((await api.query.network.minDelegateStakePercentage()).toString()) + BigInt(1)).toString();
        const newSubnetNodeQueueEpochs = (BigInt((await api.query.network.minQueueEpochs()).toString()) + BigInt(1)).toString();
        const newIdleClassificationEpochs = (BigInt((await api.query.network.minIdleClassificationEpochs()).toString()) + BigInt(1)).toString();
        const newIncludedClassificationEpochs = (BigInt((await api.query.network.minIncludedClassificationEpochs()).toString()) + BigInt(1)).toString();
        const newMaxNodePenalties = (BigInt((await api.query.network.minMaxSubnetNodePenalties()).toString()) + BigInt(1)).toString();
        const newMaxRegisteredNodes = (BigInt((await api.query.network.minMaxRegisteredNodes()).toString()) + BigInt(1)).toString();

        const wallet9 = generateRandomEthersWallet();
        const wallet10 = generateRandomEthersWallet();
        const addColdkeys = [wallet9.address, wallet10.address]
        const removeColdkeys = [wallet10.address]

        const newKeyTypes = [3]
        // 0 => Some(KeyType::Rsa),
        // 1 => Some(KeyType::Ed25519),
        // 2 => Some(KeyType::Secp256k1),
        // 3 => Some(KeyType::Ecdsa),
        await ownerUpdateName(subnetContract, subnetId, newSubnetName)
        let subnetData = await api.query.network.subnetsData(subnetId)
        expect(subnetData != undefined);
        let subnetDataOpt = subnetData as Option<any>;
        expect(subnetDataOpt.isSome);
        if (subnetDataOpt.isSome) {
            const subnetData = subnetDataOpt.unwrap();
            const human = subnetData.toHuman();
            const subnetNameStored = human.name;
            expect(subnetNameStored).to.equal(newSubnetName)
        }

        await ownerUpdateRepo(subnetContract, subnetId, newRepo)
        subnetData = await api.query.network.subnetsData(subnetId)
        expect(subnetData != undefined);
        subnetDataOpt = subnetData as Option<any>;
        expect(subnetDataOpt.isSome);
        if (subnetDataOpt.isSome) {
            const subnetData = subnetDataOpt.unwrap();
            const human = subnetData.toHuman();
            const repoStored = human.repo;
            expect(repoStored).to.equal(newRepo)
        }

        await ownerUpdateDescription(subnetContract, subnetId, newDescription)
        subnetData = await api.query.network.subnetsData(subnetId)
        expect(subnetData != undefined);
        subnetDataOpt = subnetData as Option<any>;
        expect(subnetDataOpt.isSome);
        if (subnetDataOpt.isSome) {
            const subnetData = subnetDataOpt.unwrap();
            const human = subnetData.toHuman();
            const descriptionStored = human.description;
            expect(descriptionStored).to.equal(newDescription)
        }

        await ownerUpdateMisc(subnetContract, subnetId, newMisc)
        subnetData = await api.query.network.subnetsData(subnetId)
        expect(subnetData != undefined);
        subnetDataOpt = subnetData as Option<any>;
        expect(subnetDataOpt.isSome);
        if (subnetDataOpt.isSome) {
            const subnetData = subnetDataOpt.unwrap();
            const human = subnetData.toHuman();
            const miscStored = human.misc;
            expect(miscStored).to.equal(newMisc)
        }

        await ownerUpdateChurnLimit(subnetContract, subnetId, newChurnLimit)
        expect((await api.query.network.churnLimit(subnetId)).toString()).to.be.equal(newChurnLimit)

        await ownerUpdateRegistrationQueueEpochs(subnetContract, subnetId, newSubnetNodeQueueEpochs)
        expect((await api.query.network.subnetNodeQueueEpochs(subnetId)).toString()).to.be.equal(newSubnetNodeQueueEpochs)

        await ownerUpdateIdleClassificationEpochs(subnetContract, subnetId, newIdleClassificationEpochs)
        expect((await api.query.network.idleClassificationEpochs(subnetId)).toString()).to.be.equal(newIdleClassificationEpochs)

        await ownerUpdateIncludedClassificationEpochs(subnetContract, subnetId, newIncludedClassificationEpochs)
        expect((await api.query.network.includedClassificationEpochs(subnetId)).toString()).to.be.equal(newIncludedClassificationEpochs)

        await ownerUpdateMaxNodePenalties(subnetContract, subnetId, newMaxNodePenalties)
        expect((await api.query.network.maxSubnetNodePenalties(subnetId)).toString()).to.be.equal(newMaxNodePenalties)

        await ownerAddInitialColdkeys(subnetContract, subnetId, addColdkeys)
        let currentColdkeys = await api.query.network.subnetRegistrationInitialColdkeys(subnetId)
        expect(currentColdkeys != undefined);
        let currentColdkeysOpt = currentColdkeys as Option<any>;
        expect(currentColdkeysOpt.isSome);
        if (currentColdkeysOpt.isSome) {
            const currentColdkeys = currentColdkeysOpt.unwrap();
            const human = currentColdkeys.toHuman();
            expect(addColdkeys.every(value => human.includes(value)))
        }

        await ownerRemoveInitialColdkeys(subnetContract, subnetId, removeColdkeys)
        currentColdkeys = await api.query.network.subnetRegistrationInitialColdkeys(subnetId)
        expect(currentColdkeys != undefined);
        currentColdkeysOpt = currentColdkeys as Option<any>;
        expect(currentColdkeysOpt.isSome);
        if (currentColdkeysOpt.isSome) {
            const currentColdkeys = currentColdkeysOpt.unwrap();
            const human = currentColdkeys.toHuman();
            expect([addColdkeys[0]].every(value => !human.includes(value)))
        }

        await ownerUpdateKeyTypes(subnetContract, subnetId, newKeyTypes)
        const currentKeyTypes = await api.query.network.subnetKeyTypes(subnetId)
        expect(currentKeyTypes != undefined);
        expect(currentKeyTypes.toHuman() == newKeyTypes)

        await ownerUpdateMinStake(subnetContract, subnetId, newMinStake)
        expect((await api.query.network.subnetMinStakeBalance(subnetId)).toString()).to.be.equal(newMinStake)

        await ownerUpdateMaxStake(subnetContract, subnetId, newMaxStake)
        expect((await api.query.network.subnetMaxStakeBalance(subnetId)).toString()).to.be.equal(newMaxStake)

        // Updating requires waiting 1 block, must be greater than min update period
        await waitForBlocks(api, 1);
        await ownerUpdateDelegateStakePercentage(subnetContract, subnetId, newDelegateStakePercentage)
        expect((await api.query.network.subnetDelegateStakeRewardsPercentage(subnetId)).toString()).to.be.equal(newDelegateStakePercentage)

        await ownerUpdateMaxRegisteredNodes(subnetContract, subnetId, newMaxRegisteredNodes)
        expect((await api.query.network.maxRegisteredNodes(subnetId)).toString()).to.be.equal(newMaxRegisteredNodes)

        // ================
        // Activate subnet before calling pause, unpause, and deactivate (required to pause and unpause)
        // ================

        await activateSubnet(
            subnetContract, 
            subnetId,
        )

        subnetData = await api.query.network.subnetsData(subnetId)
        
        expect(subnetData != undefined);

        subnetDataOpt = subnetData as Option<any>;
        expect(subnetDataOpt.isSome);

        if (subnetDataOpt.isSome) {
            const subnetData = subnetDataOpt.unwrap();
            const human = subnetData.toHuman();
            expect(human.state).to.equal("Active")
        }


        await ownerPauseSubnet(subnetContract, subnetId)
        subnetData = await api.query.network.subnetsData(subnetId)
        expect(subnetData != undefined);
        subnetDataOpt = subnetData as Option<any>;
        expect(subnetDataOpt.isSome);
        if (subnetDataOpt.isSome) {
            const subnetData = subnetDataOpt.unwrap();
            const human = subnetData.toHuman();
            expect(human.state).to.equal("Paused")
        }

        await ownerUnpauseSubnet(subnetContract, subnetId)
        subnetData = await api.query.network.subnetsData(subnetId)
        expect(subnetData != undefined);
        subnetDataOpt = subnetData as Option<any>;
        expect(subnetDataOpt.isSome);
        if (subnetDataOpt.isSome) {
            const subnetData = subnetDataOpt.unwrap();
            const human = subnetData.toHuman();
            expect(human.state).to.equal("Active")
        }
        
        await ownerDeactivateSubnet(subnetContract, subnetId)
        subnetData = await api.query.network.subnetsData(subnetId)
        expect(subnetData == undefined);

        console.log("✅ Subnet owner functions testing complete")
    })
});