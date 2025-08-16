// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;


interface ISwitch {

    struct Asset {
        bytes32 assetId;
        address assetOwner;
        string description;
        uint256 registeredAt;
    }

    event AssetRegistered(bytes32 indexed assetId, address indexed assetOwner);
    event OwnershipTransferred(bytes32 indexed assetId, address indexed oldOwner, address indexed newOwner);
}