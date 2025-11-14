import * as assert from "assert";
import { getDevnetApi } from "../src/substrate"
import { dev } from "@polkadot-api/descriptors"
import { PolkadotSigner, TypedApi } from "polkadot-api";
import { ethers } from "ethers"
import { generateRandomEd25519PeerId, generateRandomEthersWallet, generateRandomString, getPublicClient, SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS } from "../src/utils"
import {
    batchTransferBalanceFromSudo,
    getCurrentRegistrationCost,
    registerOrUpdateIdentity,
    registerSubnet,
    registerSubnetNode,
    removeIdentity,
} from "../src/network"
import { ETH_LOCAL_URL, SUB_LOCAL_URL } from "../src/config";
import { PublicClient } from "viem";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { expect } from "chai";
import { Option } from '@polkadot/types';

// npm test -- -g "test node update parameters-0xdgahRTH"
describe("test identities-0xDANBre34", () => {
    // init eth part
    const wallet0 = generateRandomEthersWallet();
    const wallet1 = generateRandomEthersWallet();
    const wallet2 = generateRandomEthersWallet();
    const wallet3 = generateRandomEthersWallet();
    const wallet4 = generateRandomEthersWallet();
    const wallet5 = generateRandomEthersWallet();
    const wallet6 = generateRandomEthersWallet();
    const wallet7 = generateRandomEthersWallet();
    const wallet8 = generateRandomEthersWallet();

    const ALL_ACCOUNTS = [
      wallet0.address,
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

    const subnetContract = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet0);
    const subnetContract1 = new ethers.Contract(SUBNET_CONTRACT_ADDRESS, SUBNET_CONTRACT_ABI, wallet1);

    let subnetId: string;
    let subnetNodeId1: string;

    // sudo account alice as signer
    let alice: PolkadotSigner;
    before(async () => {
        
        publicClient = await getPublicClient(ETH_LOCAL_URL)
        // init variables got from await and async
        papiApi = await getDevnetApi()

        const provider = new WsProvider(SUB_LOCAL_URL);

        api = await ApiPromise.create({ provider });

        const recipients = ALL_ACCOUNTS.map(address => ({
            address: address,
            balance: BigInt(sudoTransferAmount + BigInt(500))
        }));

        await batchTransferBalanceFromSudo(
          api,
          papiApi,
          recipients
        )

        // ==============
        // Register subnet
        // ==============
        let cost = await getCurrentRegistrationCost(subnetContract, api)
        const subnetName = generateRandomString(30)
        const repo = generateRandomString(30)
        const description = generateRandomString(30)
        const misc = generateRandomString(30)
        const minStake = await api.query.network.minSubnetMinStake();
        const maxStake = await api.query.network.networkMaxStakeBalance();
        const delegateStakePercentage = await api.query.network.minDelegateStakePercentage();

        await registerSubnet(
            subnetContract, 
            cost,
            subnetName,
            repo,
            description,
            misc,
            minStake.toString(),
            maxStake.toString(),
            delegateStakePercentage.toString(),
            initialColdkeys,
            KEY_TYPES,
            BOOTNODES,
            cost,
        )

        subnetId = await subnetContract.getSubnetId(subnetName);

        // ================
        // Add subnet nodes
        // ================

        // ================
        // Subnet node 1
        // ================
        let peer1 = await generateRandomEd25519PeerId()
        let peer2 = await generateRandomEd25519PeerId()
        let peer3 = await generateRandomEd25519PeerId()
        const delegateRewardRate = "0";
        
        const bootnode = generateRandomString(16)
        const unique = generateRandomString(16)
        const nonUnique = generateRandomString(16)

        await registerSubnetNode(
          subnetContract1, 
          subnetId,
          wallet4.address,
          peer1,
          peer2,
          peer3,
          bootnode,
          delegateRewardRate,
          BigInt(minStake.toString()),
          unique,
          nonUnique,
          "100"
        )

        let subnetNodeId1Fetched = await api.query.network.hotkeySubnetNodeId(subnetId, wallet4.address);

        const subnetNodeId1Opt = subnetNodeId1Fetched as Option<any>;
        expect(subnetNodeId1Opt.isSome);

        let subnetNode1Exists: boolean = false;
        if (subnetNodeId1Opt.isSome) {
            subnetNode1Exists = true;
            const subnetNodeId2Unwrapped = subnetNodeId1Opt.unwrap();
            const human = subnetNodeId2Unwrapped.toHuman();
            subnetNodeId1 = human?.toString();
            expect(Number(subnetNodeId1)).to.be.greaterThan(0);
        }
        expect(subnetNode1Exists);
    })

    // Status: passing
    // npm test -- -g "testing register and remove identity-0xA4hdrg"
    it("testing register and remove identity-0xA4hdrg", async () => {
        const newName = generateRandomString(16)
        const newUrl = generateRandomString(16)
        const newImage = generateRandomString(16)
        const newDiscord = generateRandomString(16)
        const newX = generateRandomString(16)
        const newTelegram = generateRandomString(16)
        const newGithub = generateRandomString(16)
        const newHuggingFace = generateRandomString(16)
        const newDescription = generateRandomString(16)
        const newMisc = generateRandomString(16)

        await registerOrUpdateIdentity(
          subnetContract1, 
          wallet4.address,
          newName,
          newUrl,
          newImage,
          newDiscord,
          newX,
          newTelegram,
          newGithub,
          newHuggingFace,
          newDescription,
          newMisc,
        )

        let newColdkeyIdentity = await api.query.network.coldkeyIdentity(wallet1.address);
        let newColdkeyIdentityOpt = newColdkeyIdentity as Option<any>;
        expect(newColdkeyIdentityOpt.isSome);
        if (newColdkeyIdentityOpt.isSome) {
            const data = newColdkeyIdentityOpt.unwrap();
            const human = data.toHuman();
            expect(human.name == newName);
            expect(human.url == newUrl);
            expect(human.image == newImage);
            expect(human.discord == newDiscord);
            expect(human.x == newX);
            expect(human.telegram == newTelegram);
            expect(human.github == newGithub);
            expect(human.huggingFace == newHuggingFace);
            expect(human.description == newDescription);
            expect(human.misc == newMisc);
        }

        await removeIdentity(subnetContract1)
        newColdkeyIdentity = await api.query.network.coldkeyIdentity(wallet1.address);
        newColdkeyIdentityOpt = newColdkeyIdentity as Option<any>;
        expect(newColdkeyIdentityOpt.isSome == false);

        console.log("✅ Registering identity testing complete")
    })
});