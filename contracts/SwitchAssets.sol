// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import "./Errors.sol";
import "./ISwitch.sol";

contract SwitchAssets {

    //if i do this, it will reduce storage load but increases computation during get all assets
//    bytes32[] private allAssets;

    //if i do this, it increases the storage load but reduces computation load during get all assets
    ISwitch.Asset[] private allAssets;

    mapping(bytes32 => ISwitch.Asset) private assets;
    mapping(address => bytes32[]) private myAssets;
    

    modifier addressZeroCheck() {

        if (msg.sender == address(0)) {
            revert Errors.ADDRESS_ZERO(msg.sender);
        }
        _;
    }

    function registerAsset(string memory description) public addressZeroCheck {

        address caller = msg.sender;

        bytes32 id = keccak256(abi.encode(caller, block.timestamp, description)); // block.prevrandao

        if (assets[id].assetOwner != address(0)) {
            revert Errors.ASSET_ALREADY_EXIST(id);
        }

        ISwitch.Asset storage newAsset = assets[id]; //asset to storage

        newAsset.assetId = id;
        newAsset.assetOwner = caller;
        newAsset.description = description;
        newAsset.registeredAt = block.timestamp;

        myAssets[caller].push(id); //individual assets
        allAssets.push(newAsset); //all assets

        emit ISwitch.AssetRegistered(id, caller);
    }

    function getAsset(bytes32 id) public view returns (ISwitch.Asset memory) {
        if (assets[id].assetOwner == address(0)) {
            revert Errors.ASSET_DOES_NOT_EXIST(id);
        }

        return assets[id];
    }
    function getAllAssets() public view returns (ISwitch.Asset[] memory) {

//        if (allAssets.length == 0) {
//            return new ISwitch.Asset[](0);
//        }
//
//        ISwitch.Asset[] memory assetList = new ISwitch.Asset[](allAssets.length);
//
//        for (uint256 i = 0; i < allAssets.length; i++) {
//            assetList[i] = assets[allAssets[i]];
//        }

        // return assetList

        return allAssets;
    }


    function getMyAssets() public view returns (ISwitch.Asset[] memory) {

        address caller = msg.sender;

        bytes32[] memory myItems = myAssets[caller];

        uint256 validCount = 0;
        for (uint256 i = 0; i < myItems.length; i++) {
            if (assets[myItems[i]].assetOwner == caller) {
                validCount++;
            }
        }

        if (validCount == 0) {
            return new ISwitch.Asset[](0);
        }

        ISwitch.Asset[] memory newList = new ISwitch.Asset[](validCount);

        for (uint256 i = 0; i < myItems.length; i++) {

            if (assets[myItems[i]].assetOwner == caller) {
                newList[validCount - 1] = assets[myItems[i]];
                validCount--;
            }
        }

        return newList;
    }

    function transferAsset(bytes32 assetId, address newOwner) public addressZeroCheck {

        address caller = msg.sender;

        if (assets[assetId].assetOwner == address(0)) {
            revert Errors.ASSET_DOES_NOT_EXIST(assetId);
        }

        if (assets[assetId].assetOwner != caller) {
            revert Errors.ONLY_OWNER(caller);
        }

        if (newOwner == caller) {
            revert Errors.INVALID_TRANSACTION();
        }        

        if (newOwner == address(0)) {
            revert Errors.ADDRESS_ZERO(newOwner);
        }

        address oldOwner = assets[assetId].assetOwner;

        assets[assetId].assetOwner = newOwner;
        myAssets[newOwner].push(assetId);

        emit ISwitch.OwnershipTransferred(assetId, oldOwner, newOwner);
    }
}

// Part 1 – Smart Contract Development (DONE ✅)
// Develop and deploy an Ethereum-based smart contract (Solidity) that implements a basic asset registry. Each asset should have:
// • Asset ID (unique)
// • Owner address
// • Description
// • Registration timestamp

// Requirements: 
// • Ability to register a new asset ✅
// • Ability to transfer ownership ✅
// • Event emission for both registration and transfer ✅
// • Write at least 5 unit tests using Foundry ✅


// SwitchAssets Contract Address: 0xb91f90fc5c8125226486417db014eaa21f7b27a0;
// https://sepolia.basescan.org/address/0xb91f90fc5c8125226486417db014eaa21f7b27a0#code