// import { expect } from "chai";
// import { step } from "mocha-steps";
// import Staking from "../build/contracts/Staking.json";
// import Subnet from "../build/contracts/Subnet.json";
// import { AbiItem } from "web3-utils";

// import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, GENESIS_ACCOUNT_BALANCE, EXISTENTIAL_DEPOSIT, ALL_ACCOUNTS, TEST_PATH } from "./config";
// import { createAndFinalizeBlock, describeWithFrontier, customRequest, hash } from "./util";

// describeWithFrontier("Hypertensor precompiles", (context) => {
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

//   // register and activate subnet
//   before(async () => {

//   });

//   step("register subnet", async function () {
//     const contract = new context.web3.eth.Contract(SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS, {
//         from: GENESIS_ACCOUNT,
//         gasPrice: "0x3B9ACA00",
//     });
//     console.log("SUBNET_CONTRACT_ADDRESS:      ", SUBNET_CONTRACT_ADDRESS)

//     // const sharesBefore = await contract.methods.accountSubnetDelegateStakeShares(TEST_PATH);

//     // console.log("subnet sharesBefore: ", sharesBefore)
// 		try {
//       const tx = await contract.methods.registerSubnet(
//         TEST_PATH,
//         16,
//         0,
//         0,
//         1,
//         3,
//         ALL_ACCOUNTS
//       ).call();
// 		} catch (error) {
//       console.log("subnet err: ", error)
// 		}


//     // console.log("subnet registerSubnet: ", tx)

//     // const sharesAfter = await contract.methods.accountSubnetDelegateStakeShares(TEST_PATH);

//     // console.log("subnet sharesAfter: ", sharesAfter)

//     const subnetId = await contract.methods.getSubnetId(TEST_PATH).call();

//     console.log("subnet subnetId: ", subnetId)

// 		expect(subnetId).to.not.be.equal(0);
//   });

// //   step("activate subnet", async function () {
// //   });
// });
