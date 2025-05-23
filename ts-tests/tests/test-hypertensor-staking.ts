// import { expect } from "chai";
// import { step } from "mocha-steps";
// import Staking from "../build/contracts/Staking.json";
// import Subnet from "../build/contracts/Subnet.json";

// import { AbiItem } from "web3-utils";

// import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, GENESIS_ACCOUNT_BALANCE, EXISTENTIAL_DEPOSIT, SEED_PATH, ALITH_ACCOUNT, ETH_BLOCK_GAS_LIMIT } from "./config";
// import { createAndFinalizeBlock, describeWithFrontier, customRequest, hash } from "./util";
// import { getRandomSubstrateKeypair, getSubstrateApi } from "../helpers/substrate";
// import { forceSetBalanceToSs58Address } from "../helpers/balance";
// import { convertPublicKeyToSs58, ss58ToEthAddress } from "../helpers/address-utils";
// import { TypedApi } from "polkadot-api";
// import { dev } from "@polkadot-api/descriptors";

// // npm run test-script --PATH 'tests/test-hypertensor-staking.ts'

// describeWithFrontier("Hypertensor staking", (context) => {
//   const STAKING_CONTRACT_BYTECODE = Staking.bytecode;
//   const STAKING_CONTRACT_ABI = Staking.abi as AbiItem[];
//   const STAKING_CONTRACT_ADDRESS = hash(2048);

//   const SUBNET_CONTRACT_BYTECODE = Subnet.bytecode;
//   const SUBNET_CONTRACT_ABI = Subnet.abi as AbiItem[];
//   const SUBNET_CONTRACT_ADDRESS = hash(2049);

//   const TEST_ACCOUNT = "0xdd33Af49c851553841E94066B54Fd28612522901";
//   const TEST_ACCOUNT_PRIVATE_KEY = "0x4ca933bffe83185dda76e7913fc96e5c97cdb7ca1fbfcc085d6376e6f564ef71";
//   const GAS_PRICE = "0x3B9ACA00"; // 1000000000
//   var nonce = 0;

//   let web3;
//   let api: TypedApi<typeof dev>

//   before(async () => {
//     web3 = context.web3;
//     api = await getSubstrateApi()
//     await createAndFinalizeBlock(context.web3);
//     web3.eth.accounts.wallet.add(GENESIS_ACCOUNT_PRIVATE_KEY);
//     web3.eth.defaultAccount = web3.eth.accounts.wallet[0].address;
//   });

//   step("add to delegate stake balance", async function () {
//     console.log("starting staking tests")

//     // const subnetContract = new context.web3.eth.Contract(SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS, {
//     //     from: GENESIS_ACCOUNT,
//     //     gasPrice: "0x3B9ACA00",
//     // });
//     const subnetContract = new context.web3.eth.Contract(SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS);

//     // console.log("staking subnetContract.methods:       ", subnetContract.methods)

//     const subnetId = await subnetContract.methods.getSubnetId(SEED_PATH).call();
//     console.log("staking subnetId:       ", subnetId)

//     const apiSubnetData = await api.query.Network.SubnetsData.getValue(subnetId)

//     expect(Number(subnetId)).to.be.equal(Number(apiSubnetData.id));
//     expect(Number(subnetId)).to.not.be.equal(0);

//     // const stakingContract = new context.web3.eth.Contract(STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, {
//     //     from: ALITH_ACCOUNT,
//     // });
//     const stakingContract = new context.web3.eth.Contract(STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, {
//       from: GENESIS_ACCOUNT,
//       // gasPrice: "0x3B9ACA00",
//       // gas: 5242880,
//     });
//     // console.log("staking stakingContract.methods:       ", stakingContract.methods)

//     const alithBalance = await context.web3.eth.getBalance(ALITH_ACCOUNT);
//     console.log("staking alithBalance:     ", alithBalance)

//     const sharesBefore = await stakingContract.methods.accountSubnetDelegateStakeShares(GENESIS_ACCOUNT, subnetId).call();
//     console.log("staking sharesBefore:  ", sharesBefore)

//     await createAndFinalizeBlock(context.web3);

//     let nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);

//     let tx = stakingContract.methods.addToDelegateStake(subnetId, "100000000000000000000").send({
//       gasLimit: ETH_BLOCK_GAS_LIMIT-10000000,
//       nonce: nonce ++
//     })
//       .on('transactionHash', async function(hash: string){
//         console.log("hash", hash)
//         let receipt0 = await context.web3.eth.getTransactionReceipt(hash);
//         console.log("receipt0", receipt0)
//       })
//       .on('confirmation', function(confirmationNumber, receipt){
//         console.log("confirmationNumber", confirmationNumber)
//         console.log("receipt", receipt)
//       })
//       .on('receipt', function(receipt){
//           console.log("receipt", receipt)
//       })
//       .on('error', function(error, receipt) {
//           console.log("error", error)
//           console.log("receipt", receipt)
//       });

//     tx = await tx
//     console.log("tx", tx)

//     // await createAndFinalizeBlock(context.web3);

//     // const sharesAfter = await stakingContract.methods.accountSubnetDelegateStakeShares(GENESIS_ACCOUNT, subnetId).call();
//     // console.log("staking sharesAfter:   ", sharesAfter)


//     // const contract = new context.web3.eth.Contract(STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, {
//     //     from: GENESIS_ACCOUNT,
//     //     gasPrice: "0x3B9ACA00",
//     //     // gas: 1,
//     //     // gasPrice: "21000",
//     // });
//     // const contract = new context.web3.eth.Contract(STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS);

//     // const balance = await context.web3.eth.getBalance(GENESIS_ACCOUNT);
//     // console.log("staking balance:       ", balance)
//     // // console.log("staking balance:       ", BigInt(1000e18))

//     // const balance2 = await context.web3.eth.getBalance(web3.eth.defaultAccount);
//     // console.log("staking balance 2:     ", balance2)

//     // const balance3 = await context.web3.eth.getBalance(ALITH_ACCOUNT);
//     // console.log("staking balance 3:     ", balance3)

//     // // const hotkey = getRandomSubstrateKeypair();
//     // // const coldkey = getRandomSubstrateKeypair();

//     // // console.log("hotkey:     ", hotkey)
//     // // console.log("coldkey:    ", coldkey)

//     // const stakingContract = new context.web3.eth.Contract(STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, {
//     //     from: ALITH_ACCOUNT,
//     // });

//     // // const alith = keyring.addFromUri('//Alith');
//     // // console.log("alith", alith)
//     // // const sudoAlithAddress = ss58ToEthAddress(alith.address)
//     // // console.log("sudoAlithAddress", sudoAlithAddress)

//     // stakingContract.methods.addToDelegateStake(subnetId, "100000000000000000000").send()
//     //   .on('transactionHash', async function(hash: string){
//     //     console.log("hash", hash)
//     //     let receipt0 = await context.web3.eth.getTransactionReceipt(hash);
//     //     console.log("receipt0", receipt0)
//     //   })
//     //   .on('confirmation', function(confirmationNumber, receipt){
//     //     console.log("confirmationNumber", confirmationNumber)
//     //     console.log("receipt", receipt)
//     //   })
//     //   .on('receipt', function(receipt){
//     //       console.log("receipt", receipt)
//     //   })
//     //   .on('error', function(error, receipt) {
//     //       console.log("error", error)
//     //       console.log("receipt", receipt)
//     //   });


//     // await forceSetBalanceToSs58Address(convertPublicKeyToSs58(hotkey.publicKey));
//     // await forceSetBalanceToSs58Address(convertPublicKeyToSs58(coldkey.publicKey));

//     // // console.log((await context.web3.eth.currentProvider))

//     // const hotkeySs58 = convertPublicKeyToSs58(hotkey.publicKey)
//     // const hotkeyEth = ss58ToEthAddress(hotkeySs58)

//     // const balance3 = await context.web3.eth.getBalance(hotkeyEth);
//     // console.log("staking balance 3:     ", balance2)

//     // const sharesBefore = await contract.methods.accountSubnetDelegateStakeShares(TEST_ACCOUNT, subnetId).call();
//     // console.log("staking sharesBefore:  ", sharesBefore)

//     // await createAndFinalizeBlock(context.web3);

//     // contract.methods.addToDelegateStake(subnetId, "10000000000000000000").send({
//     //   from: GENESIS_ACCOUNT,
//     //   gas: 21404,
//     //   gasPrice: GAS_PRICE,
//     //   value: "10000000000000000000"
//     // })
//     //   .on('transactionHash', async function(hash: string){
//     //     console.log("hash", hash)
//     //     let receipt0 = await context.web3.eth.getTransactionReceipt(hash);
//     //     console.log("receipt0", receipt0)
//     //   })
//     //   .on('confirmation', function(confirmationNumber, receipt){
//     //     console.log("confirmationNumber", confirmationNumber)
//     //     console.log("receipt", receipt)
//     //   })
//     //   .on('receipt', function(receipt){
//     //       console.log("receipt", receipt)
//     //   })
//     //   .on('error', function(error, receipt) {
//     //       console.log("error", error)
//     //       console.log("receipt", receipt)
//     //   });


//     // // try {
//     // //   // const tx = await contract.methods.addToDelegateStake(subnetId, "10000000000000000000").send({from: GENESIS_ACCOUNT});
//     // //   // console.log("yolo", tx)

//     // //   contract.methods.addToDelegateStake(subnetId, "10000000000000000000").send({from: GENESIS_ACCOUNT})
//     // //     .on('transactionHash', function(hash){
//     // //       console.log("hash", hash)
//     // //     })
//     // //     .on('confirmation', function(confirmationNumber, receipt){
//     // //       console.log("confirmationNumber", confirmationNumber)
//     // //       console.log("receipt", receipt)
//     // //     })
//     // //     .on('receipt', function(receipt){
//     // //        console.log("receipt", receipt)
//     // //     })

//     // //   // contract.methods.addToDelegateStake(subnetId, "10000000000000000000").send({from: GENESIS_ACCOUNT})
//     // //   //   .on('receipt', function(tx){
//     // //   //       console.log("yolo", tx)
//     // //   //   });
//     // //   // await contract.methods.addToDelegateStake(subnetId, "1000000000000000000000").send({
//     // //   //   from: GENESIS_ACCOUNT,
//     // //   //   value: "1", // or non-zero if your precompile expects it
//     // //   //   gas: "100",
//     // //   // });

// 		// // } catch (error) {
//     // //   console.log("addToDelegateStake err: ", error)
// 		// // }

//     // const sharesAfter = await contract.methods.accountSubnetDelegateStakeShares(TEST_ACCOUNT, subnetId).call();
//     // console.log("staking sharesAfter:   ", sharesAfter)

//     // expect(Number(sharesBefore)).to.be.lessThan(Number(sharesAfter));
//     // expect(Number(0)).to.be.lessThan(Number(0));
//   });
// });
