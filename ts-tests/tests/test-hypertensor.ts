import { expect } from "chai";
import { step } from "mocha-steps";

import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, GENESIS_ACCOUNT_BALANCE, EXISTENTIAL_DEPOSIT } from "./config";
import { createAndFinalizeBlock, describeWithFrontier, customRequest } from "./util";

describeWithFrontier("Hypertensor precompiles", (context) => {
  const TEST_ACCOUNT = "0xdd33Af49c851553841E94066B54Fd28612522901";
  const TEST_ACCOUNT_PRIVATE_KEY = "0x4ca933bffe83185dda76e7913fc96e5c97cdb7ca1fbfcc085d6376e6f564ef71";
  const TRANFER_VALUE = "0x200"; // 512, must be higher than ExistentialDeposit
  const GAS_PRICE = "0x3B9ACA00"; // 1000000000
  var nonce = 0;

  step("register subnet", async function () {
  });

  step("activate subnet", async function () {
  });
});
