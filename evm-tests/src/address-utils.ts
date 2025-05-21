import { Address } from "viem"
import { encodeAddress } from "@polkadot/util-crypto";
import { ss58Address } from "@polkadot-labs/hdkd-helpers";
import { hexToU8a, u8aConcat, u8aToHex } from "@polkadot/util";
import { blake2AsU8a, decodeAddress } from "@polkadot/util-crypto";
import { Binary } from "polkadot-api";
import { SS58_PREFIX } from "./config"

export function toViemAddress(address: string): Address {
    let addressNoPrefix = address.replace("0x", "")
    return `0x${addressNoPrefix}`
}

export function convertH160ToSS58(ethAddress: string) {
    // get the public key
    const hash = convertH160ToPublicKey(ethAddress);

    // Convert the hash to SS58 format
    const ss58Address = encodeAddress(hash, SS58_PREFIX);
    return ss58Address;
}

export function convertPublicKeyToSs58(publickey: Uint8Array) {
    return ss58Address(publickey, SS58_PREFIX);
}

export function toAccountId20Compatible(address: string): `0x${string}` {
  const h160 = new Uint8Array(Buffer.from(address.replace(/^0x/, ''), 'hex'));
  if (h160.length !== 20) throw new Error('Must be H160');
  const accountId32 = u8aConcat(new Uint8Array(12), h160); // 12 zeros + 20 bytes = 32 bytes
  return u8aToHex(accountId32);
}

export function convertSs58ToAccountId20(ss58: string): string {
  const fullAccountId = decodeAddress(ss58); // 32 bytes
  const truncated = fullAccountId.slice(-20); // Get last 20 bytes
  return u8aToHex(truncated); // Return as 0x hex string
}

export function ss58ToAccountId20(ss58: string): `0x${string}` {
  const decoded = decodeAddress(ss58);
  const h160 = decoded.slice(-20);
  const accountId32 = u8aConcat(new Uint8Array(12), h160);
  return u8aToHex(accountId32) as `0x${string}`;
}

export function convertH160ToPublicKey(ethAddress: string) {
    const prefix = "evm:";
    const prefixBytes = new TextEncoder().encode(prefix);
    const addressBytes = hexToU8a(
        ethAddress.startsWith("0x") ? ethAddress : `0x${ethAddress}`
    );
    const combined = new Uint8Array(prefixBytes.length + addressBytes.length);

    // Concatenate prefix and Ethereum address
    combined.set(prefixBytes);
    combined.set(addressBytes, prefixBytes.length);

    // Hash the combined data (the public key)
    const hash = blake2AsU8a(combined);
    return hash;
}

export function ss58ToEthAddress(ss58Address: string) {
    // Decode the SS58 address to a Uint8Array public key
    const publicKey = decodeAddress(ss58Address);

    // Take the first 20 bytes of the hashed public key for the Ethereum address
    const ethereumAddressBytes = publicKey.slice(0, 20);

    // Convert the 20 bytes into an Ethereum H160 address format (Hex string)
    const ethereumAddress = '0x' + Buffer.from(ethereumAddressBytes).toString('hex');

    return ethereumAddress;
}

export function ss58ToH160(ss58Address: string): Binary {
    // Decode the SS58 address to a Uint8Array public key
    const publicKey = decodeAddress(ss58Address);

    // Take the first 20 bytes of the hashed public key for the Ethereum address
    const ethereumAddressBytes = publicKey.slice(0, 20);


    return new Binary(ethereumAddressBytes);
}

export function ethAddressToH160(ethAddress: string): Binary {
    // Decode the SS58 address to a Uint8Array public key
    const publicKey = hexToU8a(ethAddress);

    // Take the first 20 bytes of the hashed public key for the Ethereum address
    // const ethereumAddressBytes = publicKey.slice(0, 20);


    return new Binary(publicKey);
}