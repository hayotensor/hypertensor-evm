import * as assert from "assert";
import { dev } from '@polkadot-api/descriptors';
import { TypedApi, TxCallData, HexString } from 'polkadot-api';
import { KeyPair } from "@polkadot-labs/hdkd-helpers"
import { getAliceSigner, waitForTransactionCompletion, getSignerFromKeypair } from './substrate'
import { convertH160ToSS58, convertPublicKeyToSs58 } from './address-utils'
import { cryptoWaitReady, decodeAddress } from '@polkadot/util-crypto';
import { hexToU8a, u8aToHex } from '@polkadot/util';
import { KeyringPair } from "@polkadot/keyring/types";
import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { Contract } from "ethers";

// force set balance for a ss58 address
export async function forceSetBalanceToSs58Address(api: TypedApi<typeof dev>, ss58Address: string) {
    let i: HexString = "0";
    const alice = getAliceSigner()
    const balance = BigInt(1000e18)
    // const internalCall = api.tx.Balances.force_set_balance({ who: MultiAddress.Id(ss58Address), new_free: balance })
    

    const decoded = decodeAddress(ss58Address);

    // 3. Truncate or hash it to 20 bytes for AccountId20
    // ⚠️ Choose *only one* strategy, usually truncate
    const accountId20 = decoded.slice(0, 20); // truncate to first 20 bytes
    const hexAddress = u8aToHex(accountId20);

    const address = '0xC0F0f4ab324C46e55D02D0033343B4Be8A55532d'.toLowerCase(); // important!
    const who = hexToU8a(address)

    console.log("who")

    const internalCall = api.tx.Balances.force_set_balance({ who: 'c0f0f4ab324c46e55d02d0033343b4be8a55532d', new_free: balance })

    console.log("internalCall.decodedCall", internalCall.decodedCall)

    const tx = api.tx.Sudo.sudo({ call: internalCall.decodedCall })

    await waitForTransactionCompletion(api, tx, alice)
        .then(() => { })
        .catch((error) => { console.log(`transaction error ${error}`) });

    const balanceOnChain = (await api.query.System.Account.getValue("0xc0f0f4ab324c46e55d02d0033343b4be8a55532d")).data.free
    console.log("forceSetBalanceToSs58Address balanceOnChain", balanceOnChain)
    // check the balance except for sudo account becasue of tx fee
    // if (ss58Address !== convertPublicKeyToSs58(alice.publicKey)) {
    //     assert.equal(balance, balanceOnChain)
    // }
}

// set balance for an eth address
export async function forceSetBalanceToEthAddress(api: TypedApi<typeof dev>, ethAddress: string) {
    const ss58Address = convertH160ToSS58(ethAddress)
    await forceSetBalanceToSs58Address(api, ss58Address)
}

export async function transferBalanceFromSudo(
  api: ApiPromise,
  papiApi: TypedApi<typeof dev>,
  url: string, 
  who: string, 
  balance: bigint
) {
  const keyring = new Keyring({ type: 'ethereum' });
  const sudoPair: KeyringPair = keyring.addFromUri("0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133");

  const alithSs58 = convertH160ToSS58(sudoPair.address)
  console.log("alithSs58", alithSs58)
  // const nonce = await api.query.system.account(sudoPair);
  // const nonce = await api.rpc.system.accountNextIndex(sudoPair.address);

  // let tx = await api.tx.balances.transferKeepAlive(
  //   who,
  //   balance,
  // ).signAndSend(sudoPair);

  // const unsub = await api.tx.balances
  //   .transferKeepAlive(who, balance)
  //   .signAndSend(sudoPair, async (result) => {
  //     console.log(`Current status is ${result.status}`);

  //     if (result.status.isInBlock) {
  //       console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
  //     } else if (result.status.isFinalized) {
  //       console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
  //       const balanceOnChain = (await api.query.system.account(who));
  //       console.log("forceSetBalanceToSs58Address unsub balanceOnChain", balanceOnChain.toJSON())

  //       unsub();
  //     }
  //   })

  await new Promise<void>((resolve, reject) => {
    api.tx.balances
      .transferKeepAlive(who, balance)
      .signAndSend(sudoPair, async (result) => {
        console.log(`Current status is ${result.status}`);

        if (result.status.isInBlock) {
          console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
        } else if (result.status.isFinalized) {
          console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);

          const balanceOnChain = (await api.query.system.account(who));
          console.log("forceSetBalanceToSs58Address unsub balanceOnChain", balanceOnChain.toJSON())

          // unsubscribe safely if available
          if (typeof unsub === "function") unsub();

          resolve(); // let outer await continue
        } else if (result.isError) {
          if (typeof unsub === "function") unsub();
          reject(new Error("Transaction failed"));
        }
      })
      .then((u) => {
        // assign the unsubscribe function when available
        unsub = u;
      })
      .catch((err) => {
        reject(err);
      });

    let unsub: () => void; // scoped outside to be accessible
  });
}

export async function addToDelegateStake(
  contract: Contract, 
  subnetId: string,
  balance: bigint
) {
  const tx = await contract.addToDelegateStake(subnetId, balance, { value: balance });

  const receipt = await tx.wait();
}