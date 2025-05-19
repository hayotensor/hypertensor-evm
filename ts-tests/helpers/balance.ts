// import { getAliceSigner } from "./substrate";
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { decodeAddress, encodeAddress } from '@polkadot/util-crypto';
import { RPC_PORT, PORT } from '../tests/util';
import { dev } from '@polkadot-api/descriptors';
import { createClient, InvalidTxError, Transaction, TransactionValidityError } from 'polkadot-api';
import { withPolkadotSdkCompat } from 'polkadot-api/polkadot-sdk-compat';
import { getWsProvider } from 'polkadot-api/ws-provider/web';
import { ss58ToEthAddress } from './address-utils';
import { getAliceSigner } from './substrate';

export async function forceSetBalanceToSs58Address(
  // api: ApiPromise, 
  ss58Address: string
) {
  const alice = getAliceSigner()
  const client = createClient(
    // The Polkadot SDK nodes may have compatibility issues; using this enhancer is recommended.
    // Refer to the Requirements page for additional details
    withPolkadotSdkCompat(getWsProvider(`ws://127.0.0.1:9944`)),
  );

  // To interact with the chain, obtain the `TypedApi`, which provides
  // the types for all available calls in that chain
  const api = client.getTypedApi(dev);

  // const provider = new WsProvider(`ws://127.0.0.1:9944`);

  // const api = await ApiPromise.create({ provider });

  const keyring = new Keyring({ type: 'sr25519' });
  const sudo = keyring.addFromUri('//Alice');

  const sudoEthAddress = ss58ToEthAddress(sudo.address)
  console.log("sudoEthAddress", sudoEthAddress)

  console.log("after sudo")

  const balance = BigInt(1000e18)
  
  const ethAddress = ss58ToEthAddress(ss58Address)
  console.log("ethAddress", ethAddress)

  const call = api.tx.Balances.force_set_balance({ 
    // if use ss58Address `Invalid length found on EthAddress`
    // if use EthAddress `Execution aborted due to trap: wasm trap: wasm `unreachable` instruction executed`
    who: ss58Address, 
    new_free: balance
   })

  const tx = api.tx.Sudo.sudo({ call: call.decodedCall })

  tx.signAndSubmit(alice)
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

  console.log("after setBalanceCall")

  // const newBalance = (await api.query.System.Account.getValue(ss58Address)).data.free
  // console.log("newBalance", newBalance)
}
