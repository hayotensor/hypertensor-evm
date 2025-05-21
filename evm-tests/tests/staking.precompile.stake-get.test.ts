// import * as assert from "assert";
// import { getDevnetApi, getRandomSubstrateKeypair } from "../src/substrate"
// import { dev } from "@polkadot-api/descriptors"
// import { TypedApi } from "polkadot-api";
// import { convertPublicKeyToSs58 } from "../src/address-utils"
// import { tao } from "../src/balance-math"
// import {
//     forceSetBalanceToSs58Address, addNewSubnetwork, addStake,
// } from "../src/subtensor"
// import { ethers } from "ethers";
// import { generateRandomEthersWallet } from "../src/utils"
// import { ISTAKING_V2_ADDRESS, IStakingV2ABI } from "../src/contracts/staking"
// import { log } from "console";

// describe("Test staking precompile get methods", () => {
//     const hotkey = getRandomSubstrateKeypair();
//     const coldkey = getRandomSubstrateKeypair();
//     const wallet1 = generateRandomEthersWallet();

//     let api: TypedApi<typeof dev>

//     before(async () => {
//         api = await getDevnetApi()
//         await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(hotkey.publicKey))
//         await forceSetBalanceToSs58Address(api, convertPublicKeyToSs58(coldkey.publicKey))
//         let netuid = await addNewSubnetwork(api, hotkey, coldkey)
//         console.log("will test in subnet: ", netuid)
//     })

//     it("Staker receives rewards", async () => {
//         let netuid = (await api.query.SubtensorModule.TotalNetworks.getValue()) - 1

//         await addStake(api, netuid, convertPublicKeyToSs58(hotkey.publicKey), tao(1), coldkey)

//         const contract = new ethers.Contract(
//             ISTAKING_V2_ADDRESS,
//             IStakingV2ABI,
//             wallet1
//         );

//         const stake = BigInt(
//             await contract.getStake(hotkey.publicKey, coldkey.publicKey, netuid)
//         );

//         // validator returned as bigint now. 
//         const validators =
//             await contract.getAlphaStakedValidators(hotkey.publicKey, netuid)

//         const alpha = BigInt(
//             await contract.getTotalAlphaStaked(hotkey.publicKey, netuid)
//         );
//         assert.ok(stake > 0)
//         assert.equal(validators.length, 1)
//         assert.ok(alpha > 0)

//     })
// })
