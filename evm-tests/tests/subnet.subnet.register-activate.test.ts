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
    registerSubnet,
    registerSubnetNode,
    transferBalanceFromSudo
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";

// npm test -- -g "Test subnet register activate-0xuhnrfvok"
describe("Test subnet register activate-0xuhnrfvok", () => {
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
    // npm test -- -g "testing register subnet-0xzmghoq5702"
    it("testing register subnet-0xzmghoq5702", async () => {
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
            console.log("human", human)
            
            expect(Number(subnetIdStored)).to.equal(Number(subnetId))
            expect(subnetNameStored).to.equal(subnetName)
            expect(repoStored).to.equal(repo)
            expect(descriptionStored).to.equal(description)
            expect(miscStored).to.equal(misc)
            expect(human.state).to.equal("Registered")
        }

        console.log("✅ Subnet registration testing complete")
    })


    // Status: passing
    // npm test -- -g "testing activate subnet-0xg2fE$g"
    it("testing activate subnet-0xg2fE$g", async () => {
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

        await activateSubnet(
            subnetContract, 
            subnetId,
        )

        const subnetData = await api.query.network.subnetsData(subnetId)
        
        expect(subnetData != undefined);

        const subnetDataOpt = subnetData as Option<any>;
        expect(subnetDataOpt.isSome);

        if (subnetDataOpt.isSome) {
            const subnetData = subnetDataOpt.unwrap();
            const human = subnetData.toHuman();
            expect(human.state).to.equal("Active")
        }

        console.log("✅ Subnet activation testing complete")
    })
});