import { expect } from "chai";
import { step } from "mocha-steps";
import Staking from "../build/contracts/Staking.json";
import Subnet from "../build/contracts/Subnet.json";

import { AbiItem } from "web3-utils";

import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, GENESIS_ACCOUNT_BALANCE, EXISTENTIAL_DEPOSIT, TEST_PATH } from "./config";
import { createAndFinalizeBlock, describeWithFrontier, customRequest, hash } from "./util";

// npm run test-script --PATH 'tests/test-staking.ts'

describeWithFrontier("Hypertensor precompiles", (context) => {
  const STAKING_CONTRACT_BYTECODE = Staking.bytecode;
  const STAKING_CONTRACT_ABI = Staking.abi as AbiItem[];
  const STAKING_CONTRACT_ADDRESS = hash(2048);

  const SUBNET_CONTRACT_BYTECODE = Subnet.bytecode;
  const SUBNET_CONTRACT_ABI = Subnet.abi as AbiItem[];
  const SUBNET_CONTRACT_ADDRESS = hash(2049);

  const TEST_ACCOUNT = "0xdd33Af49c851553841E94066B54Fd28612522901";
  const TEST_ACCOUNT_PRIVATE_KEY = "0x4ca933bffe83185dda76e7913fc96e5c97cdb7ca1fbfcc085d6376e6f564ef71";
  const GAS_PRICE = "0x3B9ACA00"; // 1000000000
  var nonce = 0;

  let web3;

  before(async () => {
    web3 = context.web3;
    await createAndFinalizeBlock(context.web3);
    web3.eth.accounts.wallet.add(GENESIS_ACCOUNT_PRIVATE_KEY);
    web3.eth.defaultAccount = web3.eth.accounts.wallet[0].address;
  });

  step("add to delegate stake balance", async function () {
    // const subnetContract = new context.web3.eth.Contract(SUBNET_CONTRACT_ABI, SUBNET_CONTRACT_ADDRESS, {
    //     from: GENESIS_ACCOUNT,
    //     gasPrice: "0x3B9ACA00",
    // });

    // const subnetId = await subnetContract.methods.getSubnetId(TEST_PATH).call();
    // console.log("staking subnetId:      ", subnetId)
    const subnetId = 1;

    const contract = new context.web3.eth.Contract(STAKING_CONTRACT_ABI, STAKING_CONTRACT_ADDRESS, {
        from: GENESIS_ACCOUNT,
        gasPrice: "0x3B9ACA00",
    });

    const balance = await context.web3.eth.getBalance(GENESIS_ACCOUNT);
    console.log("staking balance:       ", balance)
    console.log("staking balance:       ", BigInt(1000e18))

    const sharesBefore = await contract.methods.accountSubnetDelegateStakeShares(TEST_ACCOUNT, subnetId).call();
    console.log("staking sharesBefore:  ", sharesBefore)

    try {
      await contract.methods.addToDelegateStake(subnetId, BigInt(1000e18)).call();
		} catch (error) {
      console.log("addToDelegateStake err: ", error)
		}

    await createAndFinalizeBlock(context.web3);

    const sharesAfter = await contract.methods.accountSubnetDelegateStakeShares(TEST_ACCOUNT, subnetId).call();
    console.log("staking sharesAfter:   ", sharesAfter)

    expect(Number(sharesBefore)).to.be.lessThan(Number(sharesAfter));
  });

  step("remove from delegate stake balance", async function () {
  });
});
