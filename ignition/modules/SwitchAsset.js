// This setup uses Hardhat Ignition to manage smart contract deployments.
// Learn more about it at https://hardhat.org/ignition

const {buildModule} = require("@nomicfoundation/hardhat-ignition/modules");

module.exports = buildModule("SwitchAssetsModule", (m) => {

    const switchAssets = m.contract("SwitchAssets");

    return {switchAssets};
});


// SwitchAssets Contract Address: 0x3897196da6a4f2219ED4F183AFA3A10C8C227f23;
// https://sepolia.basescan.org/address/0x3897196da6a4f2219ED4F183AFA3A10C8C227f23#code

// npx hardhat verify --network base 0xf36f55D6Df2f9d5C7829ed5751d7E88FD3E82c2E 0xF2E7E2f51D7C9eEa9B0313C2eCa12f8e43bd1855 0x527caBd4bb83F94f1Fc1888D0691EF95e86795A1