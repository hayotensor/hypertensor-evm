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
    ownerAddOrUpdateInitialColdkeys,
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
    transferSubnetOwnership,
    acceptSubnetOwnership,
    ownerAddBootnodeAccess,
    ownerUpdateTargetNodeRegistrationsPerEpoch,
    ownerUpdateNodeBurnRateAlpha,
    ownerUpdateQueueImmunityEpochs,
    updateBootnodes,
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
        const minStake = await api.query.network.minSubnetMinStake();
        const maxStake = await api.query.network.networkMaxStakeBalance();
        const delegateStakePercentage = await api.query.network.minDelegateStakePercentage();
        const subnetNodeQueueEpochs = await api.query.network.minQueueEpochs();
        const idleClassificationEpochs = await api.query.network.minIdleClassificationEpochs();
        const includedClassificationEpochs = await api.query.network.minIncludedClassificationEpochs();
        const maxNodePenalties = await api.query.network.minMaxSubnetNodePenalties();
        const maxRegisteredNodes = await api.query.network.maxMaxRegisteredNodes();

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

        const palletSubnetId = await api.query.network.subnetName(subnetName);
        expect(palletSubnetId != undefined);

        const subnetId = await subnetContract.getSubnetId(subnetName);
        expect(BigInt(subnetId)).to.not.equal(BigInt(0))

        const minStakeAmount = (await api.query.network.minSubnetMinStake()).toString();
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

        //
        // Subnet owner functions
        //

        const newSubnetName = generateRandomString(30)
        const newRepo = generateRandomString(30)
        const newDescription = generateRandomString(30)
        const newMisc = generateRandomString(30)
        const newChurnLimit = (BigInt((await api.query.network.maxChurnLimit()).toString()) - BigInt(1)).toString();
        const newMinStake = (BigInt((await api.query.network.minSubnetMinStake()).toString()) + BigInt(1)).toString();
        const newMaxStake = (BigInt((await api.query.network.networkMaxStakeBalance()).toString()) - BigInt(1)).toString();
        const newDelegateStakePercentage = (BigInt((await api.query.network.minDelegateStakePercentage()).toString()) + BigInt(1)).toString();
        const newSubnetNodeQueueEpochs = (BigInt((await api.query.network.minQueueEpochs()).toString()) + BigInt(1)).toString();
        const newIdleClassificationEpochs = (BigInt((await api.query.network.minIdleClassificationEpochs()).toString()) + BigInt(1)).toString();
        const newIncludedClassificationEpochs = (BigInt((await api.query.network.minIncludedClassificationEpochs()).toString()) + BigInt(1)).toString();
        const newMaxNodePenalties = (BigInt((await api.query.network.minMaxSubnetNodePenalties()).toString()) + BigInt(1)).toString();
        const newMaxRegisteredNodes = (BigInt((await api.query.network.minMaxRegisteredNodes()).toString()) + BigInt(1)).toString();
        const newTargetNodeRegistrationsPerEpoch = (BigInt(newMaxRegisteredNodes) - BigInt(1)).toString();
        const newNodeBurnRateAlpha = (BigInt((await api.query.network.nodeBurnRateAlpha(subnetId)).toString()) - BigInt(1)).toString();
        const newQueueImmunityEpochs = (BigInt((await api.query.network.queueImmunityEpochs(subnetId)).toString()) - BigInt(1)).toString();

        const wallet9 = generateRandomEthersWallet();
        const wallet10 = generateRandomEthersWallet();

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

        const addColdkeys = [
            {
                coldkey: wallet9.address,
                count: 5
            },
            {
                coldkey: wallet10.address,
                count: 3
            }
        ];

        await ownerAddOrUpdateInitialColdkeys(subnetContract, subnetId, addColdkeys)
        let currentColdkeys = await api.query.network.subnetRegistrationInitialColdkeys(subnetId)
        expect(currentColdkeys != undefined);
        let currentColdkeysOpt = currentColdkeys as Option<any>;
        expect(currentColdkeysOpt.isSome);
        if (currentColdkeysOpt.isSome) {
            const coldkeysMap = currentColdkeysOpt.unwrap();
            
            // Convert to a plain object for easier comparison
            const coldkeysObj = coldkeysMap.toJSON();
            
            // Check each coldkey exists with the correct count
            for (const entry of addColdkeys) {
                expect(coldkeysObj[entry.coldkey]).to.equal(entry.count);
            }
            
            // Or check all at once
            initialColdkeys.forEach(entry => {
                expect(coldkeysObj[entry.coldkey]).to.equal(entry.count);
            });

            addColdkeys.forEach(entry => {
                expect(coldkeysObj[entry.coldkey]).to.equal(entry.count);
            });
        }

        const removeColdkeys = [wallet10.address]

        await ownerRemoveInitialColdkeys(subnetContract, subnetId, removeColdkeys)
        currentColdkeys = await api.query.network.subnetRegistrationInitialColdkeys(subnetId)
        expect(currentColdkeys != undefined);
        currentColdkeysOpt = currentColdkeys as Option<any>;
        expect(currentColdkeysOpt.isSome);
        if (currentColdkeysOpt.isSome) {
            const coldkeysMap = currentColdkeysOpt.unwrap();
            const coldkeysJson = coldkeysMap.toJSON();
            
            // Check that wallet10 doesn't exist
            expect(coldkeysJson[wallet10.address]).to.equal(undefined);
        }

        await ownerUpdateKeyTypes(subnetContract, subnetId, newKeyTypes)
        const currentKeyTypes = await api.query.network.subnetKeyTypes(subnetId)
        expect(currentKeyTypes != undefined);
        expect(currentKeyTypes.toHuman() == newKeyTypes)


        await ownerUpdateMinStake(subnetContract, subnetId, newMinStake)
        expect((await api.query.network.subnetMinStakeBalance(subnetId)).toString()).to.be.equal(newMinStake)


        await ownerUpdateMaxStake(subnetContract, subnetId, newMaxStake)
        expect((await api.query.network.subnetMaxStakeBalance(subnetId)).toString()).to.be.equal(newMaxStake)


        const lastSubnetDelegateStakeRewardsUpdate = Number((await api.query.network.lastSubnetDelegateStakeRewardsUpdate(subnetId)).toString());

        // Updating requires to be greater than min update period
        const subnetDelegateStakeRewardsUpdatePeriod = Number((await api.query.network.subnetDelegateStakeRewardsUpdatePeriod()).toString());

        await waitForBlocks(api, subnetDelegateStakeRewardsUpdatePeriod + 2);
        await ownerUpdateDelegateStakePercentage(subnetContract, subnetId, newDelegateStakePercentage)
        expect((await api.query.network.subnetDelegateStakeRewardsPercentage(subnetId)).toString()).to.be.equal(newDelegateStakePercentage)


        await ownerUpdateMaxRegisteredNodes(subnetContract, subnetId, newMaxRegisteredNodes)
        expect((await api.query.network.maxRegisteredNodes(subnetId)).toString()).to.be.equal(newMaxRegisteredNodes)


        await ownerUpdateTargetNodeRegistrationsPerEpoch(subnetContract, subnetId, newTargetNodeRegistrationsPerEpoch)
        expect((await api.query.network.targetNodeRegistrationsPerEpoch(subnetId)).toString()).to.be.equal(newTargetNodeRegistrationsPerEpoch)


        await ownerUpdateNodeBurnRateAlpha(subnetContract, subnetId, newNodeBurnRateAlpha)
        expect((await api.query.network.nodeBurnRateAlpha(subnetId)).toString()).to.be.equal(newNodeBurnRateAlpha)


        await ownerUpdateQueueImmunityEpochs(subnetContract, subnetId, newQueueImmunityEpochs)
        expect((await api.query.network.queueImmunityEpochs(subnetId)).toString()).to.be.equal(newQueueImmunityEpochs)


        const addBootnodes = [generateRandomString(6)]
        const removeBootnodes = [BOOTNODES[0]];

        await updateBootnodes(
            subnetContract,
            subnetId,
            addBootnodes,
            removeBootnodes
        )
        const newBootnodes = await api.query.network.subnetBootnodes(subnetId)
        expect(newBootnodes != undefined);
        const newBootnodesOpt = newBootnodes as Option<any>;
        expect(newBootnodesOpt.isSome);
        if (newBootnodesOpt.isSome) {
            const bootnodesMap = newBootnodesOpt.unwrap();
            const bootnodesJson = bootnodesMap.toJSON();

            expect(bootnodesJson.includes(BOOTNODES[0])).to.equal(false);

            BOOTNODES.slice(1).forEach(bootnode => {
                expect(bootnodesJson.has(bootnode)).to.equal(true);
            });

            addBootnodes.forEach(bootnode => {
                expect(bootnodesJson.includes(bootnode)).to.equal(true);
            });
        }

        const newAccessWallet = generateRandomEthersWallet();
        await ownerAddBootnodeAccess(
            subnetContract,
            subnetId,
            newAccessWallet.address,
        )
        const newAccess = await api.query.network.subnetBootnodeAccess(subnetId)
        expect(newAccess != undefined);
        const newAccessOpt = newAccess as Option<any>;
        expect(newAccessOpt.isSome);
        if (newAccessOpt.isSome) {
            const newAccessMap = newAccessOpt.unwrap();
            const newAccessMapJson = newAccessMap.toJSON();

            const accessSet = new Set(newAccessMapJson.map((addr: string) => addr.toLowerCase()));
            expect(accessSet.has(newAccessWallet.address.toLowerCase())).to.equal(true);
        }


        // ================
        // Activate subnet before calling pause, unpause, and deactivate (required to pause and unpause)
        // ================
        // get enough delegate stake to meet min delegate stake requirement
        let minDelegateStake = await getMinSubnetDelegateStakeBalance(
          subnetContract, 
          subnetId.toString()
        );

        if (Number(minDelegateStake) < 1000) {
            minDelegateStake = BigInt(1e18);
        }

        await transferBalanceFromSudo(
            api,
            papiApi,
            SUB_LOCAL_URL,
            wallet1.address,
            BigInt(minDelegateStake + BigInt(1000000)),
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

        const newOwner = generateRandomEthersWallet();
        await transferSubnetOwnership(subnetContract, subnetId, newOwner.address)

        await transferBalanceFromSudo(
            api,
            papiApi,
            SUB_LOCAL_URL,
            newOwner.address,
            BigInt(1e18),
        )

        const newOwnerSubnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, newOwner);
        await acceptSubnetOwnership(newOwnerSubnetContract, subnetId)
        const subnetOwner = (await api.query.network.subnetOwner(subnetId)).toString()
        expect(subnetOwner).to.equal(newOwner.address)

        await ownerDeactivateSubnet(newOwnerSubnetContract, subnetId)
        subnetData = await api.query.network.subnetsData(subnetId)
        expect(subnetData == undefined);

        console.log("✅ Subnet owner functions testing complete")
    })
});