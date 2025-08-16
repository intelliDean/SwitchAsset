const hre = require("hardhat");
require("dotenv").config();

async function main() {


    console.log("🚀 Deploying contracts...OwnershipLib");
    // Step 1: Deploy OwnershipLib
    const ownershipLibFactory = await hre.ethers.getContractFactory("OwnershipLib");
    const ownershipLib = await ownershipLibFactory.deploy();
    console.log(`📚 OwnershipLib deployed at: ${ownershipLib.target}`);

    console.log("🚀 Deploying contracts...Ownership Contract");


    // Step 2: Deploy Ownership using OwnershipLib
    const ownershipContract = await hre.ethers.getContractFactory("Ownership", {
        libraries: {
            // OwnershipLib: ownershipLib.target,
            OwnershipLib: ownershipLib.target,
        },
    });
    const ownership = await ownershipContract.deploy(OWNER);
    console.log(`📦 Ownership deployed at: ${ownership.target}`);

    console.log("🚀 Deploying contracts...SwitchAssets.sol");
    // Step 3: Deploy SwitchAssets.sol with Ownership address
    const AuthenticityFactory = await hre.ethers.getContractFactory("SwitchAssets.sol");

    const authenticity = await AuthenticityFactory.deploy(
        ownership.target,
        CERTIFICATE,
        SIGNING_DOMAIN,
        SIGNATURE_VERSION
    );
    console.log(`🧾 Authenticity deployed at: ${authenticity.target}`);

    console.log("✅ Deployment complete.");
}

main().catch((error) => {
    console.error("❌ Deployment failed:", error);
    process.exitCode = 1;
});
//BASE
// 📚 OwnershipLib deployed at: 0x3fB3BCdd95271Fc8b7ebFF48d4d7541b982355ef
// https://sepolia.basescan.org/address/0x3fB3BCdd95271Fc8b7ebFF48d4d7541b982355ef#code

// 📦 Ownership deployed at: 0x49e8207450dd0204Bb6a89A9edf7CE151cE58BBc
// https://sepolia.basescan.org/address/0x49e8207450dd0204Bb6a89A9edf7CE151cE58BBc#code

// 🧾 SwitchAssets.sol deployed at: 0x98BC72046616b528D4Bc5bbcC7d99f82237A8B55
// https://sepolia.basescan.org/address/0x98BC72046616b528D4Bc5bbcC7d99f82237A8B55#code

// to verify a contract, you need the contract address and also the constructor parameters
// npx hardhat verify --network base 0xf36f55D6Df2f9d5C7829ed5751d7E88FD3E82c2E 0xF2E7E2f51D7C9eEa9B0313C2eCa12f8e43bd1855 0x527caBd4bb83F94f1Fc1888D0691EF95e86795A1


//AVALANCHE
// 📚 OwnershipLib deployed at: 0xF8459F087f8583f57c8Ad313c3805ECE79D127DE
// https://testnet.snowtrace.io/address/0xF8459F087f8583f57c8Ad313c3805ECE79D127DE#code

// 📦 Ownership deployed at: 0xE83BA3F5Ac6bCD62715B8Da620d014b17acA4319
// https://testnet.snowtrace.io/address/0xE83BA3F5Ac6bCD62715B8Da620d014b17acA4319#code

// 🧾 SwitchAssets.sol deployed at: 0x63616b20f7A12f9Ba67BDF957f1400CDbF725fF8
// https://testnet.snowtrace.io/address/0x63616b20f7A12f9Ba67BDF957f1400CDbF725fF8#code

//VERY MAINNET
// 📚 OwnershipLib deployed at: 0x442576ef8EA93B6aA30cb7C779b8cC1e402bca5e

// 📦 Ownership deployed at: 0xBbcD22fd30EFA3c859f3C30a7224aB257D20b112

// 🧾 SwitchAssets.sol deployed at: 0x97D9bcE273974455Bfc3A51E8Fd956D4209066A3