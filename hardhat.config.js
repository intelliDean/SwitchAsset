require("@nomicfoundation/hardhat-toolbox");
require("@nomicfoundation/hardhat-verify");
const {vars} = require("hardhat/config");

/** @type import('hardhat/config').HardhatUserConfig */


module.exports = {
    solidity: {
        version: "0.8.29",
        settings: {
            optimizer: {
                enabled: true,
                runs: 1000
            }
        }
    },

    networks: {
        base: {
            url: vars.get("BASE_URL"),
            accounts: [`0x${vars.get("PRIVATE_KEY")}`]
        },
        fuji: {
            url: "https://api.avax-test.network/ext/bc/C/rpc",
            // gasPrice: 225000000000,
            // chainId: 43113,
            accounts: [`0x${vars.get("PRIVATE_KEY")}`]
        },
        very: {
            url: "https://rpc.verylabs.io",
            chainId: 4613,
            accounts: [`0x${vars.get("PRIVATE_KEY")}`]
        }
    },
    etherscan: {
        apiKey: vars.get("BASE_SCAN_API")
    },
    paths: {
        sources: "./contracts",
        tests: "./test-hardhat", // i deleted test-hardhat, i am using foundry to test
        cache: "./hh-cache",
        artifacts: "./hh-artifacts"
    }
};