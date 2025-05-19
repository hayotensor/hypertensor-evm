import { ApiPromise, WsProvider, Keyring, ApiRx } from '@polkadot/api';
import { decodeAddress, encodeAddress } from '@polkadot/util-crypto';
import { RPC_PORT, PORT } from '../tests/util';
import { dev } from '@polkadot-api/descriptors';
import { createClient, InvalidTxError, Transaction, TransactionValidityError } from 'polkadot-api';
import { withPolkadotSdkCompat } from 'polkadot-api/polkadot-sdk-compat';
import { getWsProvider } from 'polkadot-api/ws-provider/web';
import { ss58ToEthAddress } from './address-utils';
import { getAliceSigner } from './substrate';
import { ALITH_ACCOUNT, GENESIS_ACCOUNT } from '../tests/config';

export async function forceSetBalanceToSs58Address(
  // api: ApiPromise, 
  ss58Address: string
) {
  const alice = getAliceSigner()
  const client = createClient(
    withPolkadotSdkCompat(getWsProvider(`ws://127.0.0.1:9944`)),
  );

  // To interact with the chain, obtain the `TypedApi`, which provides
  // the types for all available calls in that chain
  const api = client.getTypedApi(dev);

  const keyring = new Keyring({ type: 'sr25519' });
  const sudo = keyring.addFromUri('//Alice');

  const sudoEthAddress = ss58ToEthAddress(sudo.address)
  console.log("sudoEthAddress", sudoEthAddress)
  console.log("alice address ", alice)

  const alith = keyring.addFromUri('//Alith');
  console.log("alith", alith)
  const sudoAlithAddress = ss58ToEthAddress(alith.address)
  console.log("sudoAlithAddress", sudoAlithAddress)

  // ===================
  // transfer_keep_alive
  // ===================
  const currAlithBalanace = (await api.query.System.Account.getValue(ALITH_ACCOUNT)).data.free
  console.log("currAlithBalanace", currAlithBalanace)

  const ethAddress = ss58ToEthAddress(ss58Address)
  console.log("ethAddress", ethAddress)
  
  const balance = BigInt(1000e18)

  const call = api.tx.Balances.transfer_keep_alive({ 
    // if use ss58Address `Invalid length found on EthAddress`
    // if use EthAddress `Execution aborted due to trap: wasm trap: wasm `unreachable` instruction executed`
    dest: ethAddress, 
    value: balance
   })

  call.signAndSubmit(alice)
    .then(() => {
      console.log("tx went well")
    })
    .catch((err) => {
      console.log("tx error", err)
      if (err instanceof InvalidTxError) {
        const typedErr: TransactionValidityError<typeof dev> = err.error
        console.log(typedErr)
      }
    })


  // ===================
  // force_set_balance
  // ===================

  // const balance = BigInt(1000e18)
  
  // const ethAddress = ss58ToEthAddress(ss58Address)
  // console.log("ethAddress", ethAddress)
  // console.log("ethAddress", GENESIS_ACCOUNT)

  // // 
  // const currBalanace = (await api.query.System.Account.getValue(GENESIS_ACCOUNT)).data.free
  // console.log("currBalanace", currBalanace)

  // const call = api.tx.Balances.force_set_balance({ 
  //   // if use ss58Address `Invalid length found on EthAddress`
  //   // if use EthAddress `Execution aborted due to trap: wasm trap: wasm `unreachable` instruction executed`
  //   who: ethAddress, 
  //   new_free: balance
  //  })

  // const tx = api.tx.Sudo.sudo({ call: call.decodedCall })

  // tx.signAndSubmit(alice)
  //   .then(() => {
  //     console.log("tx went well")
  //   })
  //   .catch((err) => {
  //     console.log("tx error", err)
  //     if (err instanceof InvalidTxError) {
  //       const typedErr: TransactionValidityError<typeof dev> = err.error
  //       console.log(typedErr)
  //     }
  //   })















  // console.log("after setBalanceCall")

  // const newBalance = (await api.query.System.Account.getValue(ss58Address)).data.free
  // console.log("newBalance", newBalance)

  // const provider = new WsProvider('ws://localhost:9944'); // Replace with your endpoint
  // const api = await ApiPromise.create({ provider });

  // // Create keyring and load Sudo account
  // const keyring = new Keyring({ type: 'sr25519' });
  // const sudo = keyring.addFromUri('//Alice'); // or your own sudo mnemonic

  // // Recipient SS58 address
  // const recipient = '5FHneW46xGXgs5mUiveU4sbTyGBzmstxk94rZZuD6YbN7hHf'; // replace

  // // Amount to set (in raw units, e.g., Planck)
  // const amount = BigInt(1_000_000_000_000_000); // e.g., 1 DOT

  // // Create the inner call
  // const setBalanceCall = api.tx.balances.forceSetBalance(recipient, amount);

  // // Wrap it in sudo
  // const tx = api.tx.sudo.sudo(setBalanceCall);

  // // Sign and send
  // const unsub = await tx.signAndSend(sudo, ({ status, dispatchError }) => {
  //   if (status.isInBlock) {
  //     console.log('Included in block:', status.asInBlock.toHex());
  //   } else if (status.isFinalized) {
  //     console.log('Finalized in block:', status.asFinalized.toHex());
  //     unsub();
  //   }

  //   if (dispatchError) {
  //     if (dispatchError.isModule) {
  //       const decoded = api.registry.findMetaError(dispatchError.asModule);
  //       console.error(`Error: ${decoded.section}.${decoded.name} - ${decoded.docs.join(' ')}`);
  //     } else {
  //       console.error(`Error: ${dispatchError.toString()}`);
  //     }
  //   }
  // });

}
