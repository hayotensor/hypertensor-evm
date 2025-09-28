import { defineChain, http, publicActions, createPublicClient } from "viem"
import { privateKeyToAccount, generatePrivateKey } from 'viem/accounts'
import { ApiPromise } from "@polkadot/api";
import { ethers, getAddress } from "ethers"
import { ETH_LOCAL_URL } from "./config"
import OverwatchNode from "../build/contracts/OverwatchNode.json";
import Subnet from "../build/contracts/Subnet.json";
import Staking from "../build/contracts/Staking.json";
import PeerId from 'peer-id'

export const SEED_PATH = "subnet-name";
export const TEST_PATH = "subnet-test-name";
export const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
export const GENESIS_ACCOUNT_PRIVATE_KEY = "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

export const OVERWATCH_NODE_CONTRACT_ABI = OverwatchNode.abi;
export const OVERWATCH_NODE_CONTRACT_ADDRESS = hash(2050);

export const SUBNET_CONTRACT_ABI = Subnet.abi;
export const SUBNET_CONTRACT_ADDRESS = hash(2049);

export const STAKING_CONTRACT_ABI = Staking.abi;
export const STAKING_CONTRACT_ADDRESS = hash(2048);


export type ClientUrlType = 'http://localhost:9944';

export const chain = (id: number, url: string) => defineChain({
    id: id,
    name: 'hypertensor',
    network: 'hypertensor',
    nativeCurrency: {
        name: 'tensor',
        symbol: 'TENSOR',
        decimals: 18,
    },
    rpcUrls: {
        default: {
            http: [url],
        },
    },
    testnet: true,
})


export async function getPublicClient(url: ClientUrlType) {
    const wallet = createPublicClient({
        chain: chain(42, url),
        transport: http(),

    })

    return wallet.extend(publicActions)
}

/**
 * Generates a random Ethereum wallet
 * @returns wallet keyring
 */
export function generateRandomEthWallet() {
    let privateKey = generatePrivateKey().toString();
    privateKey = privateKey.replace('0x', '');

    const account = privateKeyToAccount(`0x${privateKey}`)
    return account
}


export function generateRandomEthersWallet() {
    const account = ethers.Wallet.createRandom();
    const provider = new ethers.JsonRpcProvider(ETH_LOCAL_URL);

    const wallet = new ethers.Wallet(account.privateKey, provider);
    return wallet;
}

export function hash(n: number) {
  const bytes = new Uint8Array(20); // 20 bytes = H160
  const view = new DataView(bytes.buffer);
  view.setBigUint64(12, BigInt(n)); // store in last 8 bytes, big-endian
  const hex = "0x" + Buffer.from(bytes).toString("hex");
  return getAddress(hex); // optional: applies EIP-55 checksum
}

const characters ='ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';

export function generateRandomString(length: number) {
    let result = ' ';
    const charactersLength = characters.length;
    for ( let i = 0; i < length; i++ ) {
        result += characters.charAt(Math.floor(Math.random() * charactersLength));
    }

    return result;
}

export async function generateRandomEd25519PeerId() {
    const id = await PeerId.create({ bits: 256, keyType: 'Ed25519' })
    return id.toB58String()
}

export async function waitForBlocks(api: ApiPromise, blockCount = 1) {
    let blocksWaited = 0;
    return new Promise(async (resolve) => {
        const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
            blocksWaited++;
            console.log("waitForBlocks", blocksWaited)
            if (blocksWaited >= blockCount) {
                unsubscribe();
                resolve(header);
            }
        });
    })
}
