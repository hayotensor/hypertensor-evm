import { expect } from "chai";
import { step } from "mocha-steps";
import IERC20BalanceTransfer from "../build/contracts/IERC20BalanceTransfer.json";

import { AbiItem } from "web3-utils";

import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, GENESIS_ACCOUNT_BALANCE, EXISTENTIAL_DEPOSIT, TEST_PATH } from "./config";
import { createAndFinalizeBlock, describeWithFrontier, customRequest, hash } from "./util";

// npm run test-script --PATH 'tests/test-staking.ts'

describeWithFrontier("Hypertensor precompiles", (context) => {
  const ERC20_BALANCE_CONTRACT_BYTECODE = IERC20BalanceTransfer.bytecode;
  const ERC20_BALANCE_CONTRACT_ABI = IERC20BalanceTransfer.abi as AbiItem[];
  const ERC20_BALANCE_CONTRACT_ADDRESS = hash(2050);

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

  step("transfer eth (tensor)", async function () {
    const contract = new context.web3.eth.Contract(ERC20_BALANCE_CONTRACT_ABI, ERC20_BALANCE_CONTRACT_ADDRESS, {
        from: GENESIS_ACCOUNT,
        gasPrice: "0x3B9ACA00",
    });

    try {
      await contract.methods.transfer().call();
		} catch (error) {
      console.log("addToDelegateStake err: ", error)
		}

    expect(Number(0)).to.be.lessThan(Number(0));
  });
});
