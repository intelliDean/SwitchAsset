// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;


interface ISwitch {

    struct Asset {
        bytes32 assetId;
        address assetOwner;
        string description;
        uint256 registeredAt;
    }

    event AssetRegistered(bytes32 indexed id, address indexed owner);
    event OwnershipTransferred(bytes32 indexed id, address indexed oldOwner, address indexed newOwner);
}