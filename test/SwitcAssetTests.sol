// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import "forge-std/Test.sol";
import {SwitchAssets} from "../contracts/SwitchAssets.sol";
import {ISwitch} from "../contracts/ISwitch.sol";
import {Errors} from "../contracts/Errors.sol";

contract SwitchAssetsTest is Test {
    SwitchAssets public switchAssets;

    address public owner = makeAddr("owner");
    address public user1 = makeAddr("user1");
    address public user2 = makeAddr("user2");

    address public zeroAddress = address(0);

    event AssetRegistered(bytes32 indexed assetId, address indexed assetOwner);
    event OwnershipTransferred(
        bytes32 indexed assetId,
        address indexed oldOwner,
        address indexed newOwner
    );

    function setUp() public {
        switchAssets = new SwitchAssets();
    }

    function registerTestAsset(
        address user,
        string memory description
    ) internal returns (bytes32) {
        vm.prank(user);
        bytes32 assetId = keccak256(
            abi.encode(user, block.timestamp, description)
        );
        switchAssets.registerAsset(description);
        return assetId;
    }

    function testRegisterAsset() public {
        string memory description = "testing asset";

        vm.expectEmit(true, true, false, false);
        emit AssetRegistered(
            keccak256(abi.encode(owner, block.timestamp, description)),
            owner
        );

        bytes32 assetId = registerTestAsset(owner, description);

        ISwitch.Asset memory asset = switchAssets.getAsset(assetId);
        assertEq(asset.assetOwner, owner);
        assertEq(asset.description, description);
        assertEq(asset.registeredAt, block.timestamp);
    }

    function testRegisterAssetCaptureEventValues() public {
        string memory description = "testing asset";
        address caller = user1;

        vm.warp(1234);

        vm.recordLogs();

        vm.prank(caller);
        switchAssets.registerAsset(description);

        Vm.Log[] memory entries = vm.getRecordedLogs();

        assertEq(entries.length, 1);

        // topics[0] = event signature
        assertEq(
            entries[0].topics[0],
            keccak256("AssetRegistered(bytes32,address)")
        );

        // compute expected assetId
        bytes32 expectedAssetId = keccak256(
            abi.encode(caller, block.timestamp, description)
        );

        // topics[1] = indexed assetId
        assertEq(entries[0].topics[1], expectedAssetId);

        // topics[2] = indexed assetOwner
        assertEq(entries[0].topics[2], bytes32(uint256(uint160(caller))));

        // data should be empty since both fields are indexed
        assertEq(entries[0].data.length, 0);
    }

    function testRegisterAssetAddressZeroReverts() public {
        vm.expectRevert(
            abi.encodeWithSelector(Errors.ADDRESS_ZERO.selector, zeroAddress)
        );
        vm.prank(zeroAddress);
        switchAssets.registerAsset("should revert");
    }

    function testGetAsset() public {
        string memory description = "testing asset 1";
        bytes32 assetId = registerTestAsset(user1, description);

        ISwitch.Asset memory asset = switchAssets.getAsset(assetId);
        assertEq(asset.assetId, assetId);
        assertEq(asset.assetOwner, user1);
    }

    function testGetAssetNonExistentReverts() public {
        bytes32 nonExistentId = keccak256("non-existent");
        vm.expectRevert(
            abi.encodeWithSelector(
                Errors.ASSET_DOES_NOT_EXIST.selector,
                nonExistentId
            )
        );
        switchAssets.getAsset(nonExistentId);
    }

    function testGetMyAssetsEmpty() public {
        vm.prank(user1);
        ISwitch.Asset[] memory assets = switchAssets.getMyAssets();
        assertEq(assets.length, 0);
    }

    function testGetMyAssetsSingle() public {
        string memory description = "my asset";
        bytes32 assetId = registerTestAsset(user1, description);

        vm.prank(user1);
        ISwitch.Asset[] memory assets = switchAssets.getMyAssets();

        assertEq(assets.length, 1);
        assertEq(assets[0].assetId, assetId);
        assertEq(assets[0].assetOwner, user1);
    }

    function testGetMyAssetsMultiple() public {
        vm.startPrank(user1);

        switchAssets.registerAsset("user1 asset 1");
        switchAssets.registerAsset("user1 asset 2");
        switchAssets.registerAsset("user1 asset 3");

        ISwitch.Asset[] memory assets = switchAssets.getMyAssets();
        vm.stopPrank();

        assertEq(assets.length, 3);

        for (uint i = 0; i < assets.length; i++) {
            assertEq(assets[i].assetOwner, user1);
        }
    }

    function testGetMyAssetsOnlyOwnersAssets() public {
        bytes32 user1Asset = registerTestAsset(user1, "user1 asset");

        bytes32 user2Asset = registerTestAsset(user2, "user2 asset");

        vm.prank(user1);
        ISwitch.Asset[] memory user1Assets = switchAssets.getMyAssets();
        assertEq(user1Assets.length, 1);
        assertEq(user1Assets[0].assetId, user1Asset);

        vm.prank(user2);
        ISwitch.Asset[] memory user2Assets = switchAssets.getMyAssets();
        assertEq(user2Assets.length, 1);
        assertEq(user2Assets[0].assetId, user2Asset);
    }

    function testTransferAsset() public {
        string memory description = "asset to transfer";
        bytes32 assetId = registerTestAsset(user1, description);

        vm.expectEmit(true, true, true, false);
        emit OwnershipTransferred(assetId, user1, user2);

        vm.prank(user1);
        switchAssets.transferAsset(assetId, user2);

        ISwitch.Asset memory asset = switchAssets.getAsset(assetId);
        assertEq(asset.assetOwner, user2);
    }

    function testTransferAssetCaptureEventValues() public {
        address caller = user1;
        address newOwner = user2;

        vm.warp(1234);

        vm.startPrank(caller);
        switchAssets.registerAsset("my asset");
        ISwitch.Asset[] memory myAssets = switchAssets.getMyAssets();
        bytes32 assetId = myAssets[0].assetId;
        vm.stopPrank();

        vm.recordLogs();

        vm.prank(caller);
        switchAssets.transferAsset(assetId, newOwner);

        Vm.Log[] memory entries = vm.getRecordedLogs();

        assertEq(entries.length, 1);

        assertEq(
            entries[0].topics[0],
            keccak256("OwnershipTransferred(bytes32,address,address)")
        );

        assertEq(entries[0].topics[1], assetId);

        //topics[2] = oldOwner
        assertEq(entries[0].topics[2], bytes32(uint256(uint160(caller))));

        //topics[3] = newOwner
        assertEq(entries[0].topics[3], bytes32(uint256(uint160(newOwner))));

        //data should be empty (all params are indexed)
        assertEq(entries[0].data.length, 0);
    }

    function testTransferAssetNonOwnerReverts() public {
        bytes32 assetId = registerTestAsset(user1, "testing asset");

        vm.expectRevert(
            abi.encodeWithSelector(Errors.ONLY_OWNER.selector, user2)
        );
        vm.prank(user2);
        switchAssets.transferAsset(assetId, user2);
    }

    function testTransferAssetToSelfReverts() public {
        bytes32 assetId = registerTestAsset(user1, "testing asset");

        vm.expectRevert(Errors.INVALID_TRANSACTION.selector);
        vm.prank(user1);
        switchAssets.transferAsset(assetId, user1);
    }

    function testTransferAssetNonExistentReverts() public {
        bytes32 nonExistentId = keccak256("non-existent");

        vm.expectRevert(
            abi.encodeWithSelector(
                Errors.ASSET_DOES_NOT_EXIST.selector,
                nonExistentId
            )
        );
        vm.prank(user1);
        switchAssets.transferAsset(nonExistentId, user2);
    }

    function testTransferAssetOnlyOwnerReverts() public {
        bytes32 assetId = registerTestAsset(user1, "testing asset");

        vm.expectRevert(
            abi.encodeWithSelector(Errors.ONLY_OWNER.selector, user2)
        );
        vm.prank(user2);
        switchAssets.transferAsset(assetId, user2);
    }

    function testTransferAssetToZeroAddressReverts() public {
        bytes32 assetId = registerTestAsset(user1, "testing asset");

        vm.expectRevert(
            abi.encodeWithSelector(Errors.ADDRESS_ZERO.selector, zeroAddress)
        );
        vm.prank(user1);
        switchAssets.transferAsset(assetId, zeroAddress);
    }

    function testAssetDescriptionMaxLength() public {
        string
            memory longDescription = "i just make this string to be long but i no really get anything wey i want talk so i am just gonna put anythig wey come my mind like Chelsea is wining the EPL, UCL, Carling cup, F.A Cup, Super cup and Community shield this season. i hope this dscrption is long enough?";

        bytes32 assetId = registerTestAsset(user1, longDescription);

        ISwitch.Asset memory asset = switchAssets.getAsset(assetId);
        assertEq(asset.description, longDescription);
    }

    function testMultipleUsersMultipleAssets() public {
        bytes32 user1Asset1 = registerTestAsset(user1, "User1 Asset 1");
        registerTestAsset(user1, "user1 asset 2");
        registerTestAsset(user1, "user1 asset 3");
        registerTestAsset(user1, "user1 asset 4");

        registerTestAsset(user2, "user1 asset 1");

        vm.prank(user1);
        assertEq(switchAssets.getMyAssets().length, 4);

        vm.prank(user2);
        assertEq(switchAssets.getMyAssets().length, 1);

        vm.prank(user1);
        switchAssets.transferAsset(user1Asset1, user2);

        vm.prank(user1);
        assertEq(switchAssets.getMyAssets().length, 3);

        vm.prank(user2);
        assertEq(switchAssets.getMyAssets().length, 2);
    }
}
