import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { SubmittableExtrinsic } from '@polkadot/api/types';
import { KeyringPair } from '@polkadot/keyring/types';
import { AddressOrPair, SubmittableResultValue } from '@polkadot/api/types';
import { ss58ToEthAddress } from './address-utils';

export async function forceSetBalance(
  wsEndpoint: string,
  sudoUri: string,
  targetSs58: string,
  newFree: bigint,
  // newReserved: bigint
): Promise<void> {
  const provider = new WsProvider(wsEndpoint);

  const api = await ApiPromise.create({
    provider,
    // types: {
    //   AccountId: 'AccountId20',
    //   LookupSource: 'AccountId20',
    //   Balance: 'u128',
    // },
  });

  const keyring = new Keyring({ type: 'ethereum' });
  const sudoPair: KeyringPair = keyring.addFromUri("0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133");

  // const innerCall = await api.tx.balances.transferKeepAlive(
  //   "c0f0f4ab324c46e55d02d0033343b4be8a55532d",
  //   newFree,
  // ).signAndSend(sudoPair);

  await api.tx.balances.forceSetBalance(
    "0xc0f0f4ab324c46e55d02d0033343b4be8a55532d",
    newFree,
  ).signAndSend(sudoPair);


  // let call = await api.tx.balances.forceSetBalance(
  //   "0xc0f0f4ab324c46e55d02d0033343b4be8a55532d",
  //   newFree,
  // )

  // const tx = api.tx.Sudo.sudo({ call: call.data. }).signAndSend(sudoPair);

  // const internalCall = api.tx.Balances.force_set_balance({ who: 'c0f0f4ab324c46e55d02d0033343b4be8a55532d', new_free: newFree })
  // const tx = api.tx.Sudo.sudo({ call: internalCall. }).signAndSend(sudoPair);

}
