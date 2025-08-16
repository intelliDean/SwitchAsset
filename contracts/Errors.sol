// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

contract Errors {
    error ONLY_OWNER(address);
    error ADDRESS_ZERO(address);
    error ASSET_ALREADY_EXIST(bytes32 id);
    error ASSET_DOES_NOT_EXIST(bytes32 id);
    error INVALID_TRANSACTION();
}