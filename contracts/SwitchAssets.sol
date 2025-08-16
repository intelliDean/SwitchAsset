// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import "./Errors.sol";
import "./ISwitch.sol";

contract SwitchAssets {

    mapping(bytes32 => ISwitch.Asset) private assets;
    mapping(address => ISwitch.Asset[]) private myAssets;

    modifier addressZeroCheck() {

        if (msg.sender == address(0)) {
            revert Errors.ADDRESS_ZERO(msg.sender);
        }
        _;
    }

    function registerAsset(string memory description) public addressZeroCheck {

        bytes32 id = keccak256(abi.encode(msg.sender, block.timestamp));

        if (assets[id].assetOwner != address(0)) {
            revert Errors.ASSET_ALREADY_EXIST(id);
        }

        ISwitch.Asset storage newAsset = assets[id];

        newAsset.assetId = id;
        newAsset.assetOwner = msg.sender;
        newAsset.description = description;
        newAsset.registeredAt = block.timestamp;

        myAssets[msg.sender].push(newAsset);

        emit ISwitch.AssetRegistered(id, msg.sender);
    }

    function getAsset(bytes32 id) public view returns (ISwitch.Asset memory) {
        if (assets[id].assetOwner == address(0)) {
            revert Errors.ASSET_DOES_NOT_EXIST(id);
        }

        return assets[id];
    }


    function getMyAssets() public view returns (ISwitch.Asset[] memory) {

        address caller = msg.sender;

        ISwitch.Asset[] memory myItems = myAssets[caller];

        uint256 validCount = 0;
        for (uint256 i = 0; i < myItems.length; i++) {
            if (assets[myItems[i].assetId].assetOwner == caller) {
                validCount++;
            }
        }

        if (validCount == 0) {
            return new ISwitch.Asset[](0);
        }

        ISwitch.Asset[] memory newList = new ISwitch.Asset[](validCount);

        for (uint256 i = 0; i < myItems.length; i++) {

            if (assets[myItems[i].assetId].assetOwner == caller) {
                newList[validCount - 1] = assets[myItems[i].assetId];
                validCount--;
            }
        }

        return newList;
    }

    function transferAsset(bytes32 id, address newOwner) public addressZeroCheck {

        address caller = msg.sender;

        if (assets[id].assetOwner != caller) {
            revert Errors.ONLY_OWNER(caller);
        }

        if (newOwner == caller) {
            revert Errors.INVALID_TRANSACTION();
        }

        if (assets[id].assetOwner == address(0)) {
            revert Errors.ASSET_DOES_NOT_EXIST(id);
        }

        if (newOwner == address(0)) {
            revert Errors.ADDRESS_ZERO(newOwner);
        }

        address oldOwner = assets[id].assetOwner;

        emit ISwitch.OwnershipTransferred(id, oldOwner, newOwner);
    }
}