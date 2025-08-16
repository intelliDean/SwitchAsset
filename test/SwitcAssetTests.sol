// SPDX-License-Identifier: MIT
pragma solidity ^0.8.29;

import "forge-std/Test.sol";
import {SwitchAssets} from "../contracts/SwitchAssets.sol";
import {ISwitch} from "../contracts/ISwitch.sol";
import {Errors} from "../contracts/Errors.sol";

contract SwitchAssetsTest is Test {

    SwitchAssets switchAssets;

    address owner = address(0x123);
    address user1 = address(0x456);
    address user2 = address(0x789);

    function setUp() public {

        switchAssets = new SwitchAssets();

        vm.deal(owner, 10 ether);
        vm.deal(user1, 10 ether);
        vm.deal(user2, 10 ether);
    }

    function testRegisterAsset() public {

        vm.prank(owner);

        vm.expectEmit(true, true, false, true);

        bytes32 expectedId = keccak256(abi.encode(owner, block.timestamp));

        emit ISwitch.AssetRegistered(expectedId, owner);

        switchAssets.registerAsset("This is a test asset");

        ISwitch.Asset memory asset = switchAssets.getAsset(expectedId);

        assertEq(asset.assetId, expectedId, "Asset ID should match");
        assertEq(asset.assetOwner, owner, "Owner should be correct");
        assertEq(asset.description, "This is a test asset", "Description should match");

        // assertEq(asset.registeredAt, block.timestamp, "Timestamp should match");

        ISwitch.Asset[] memory myAssets = switchAssets.getMyAssets();
        assertEq(myAssets.length, 1, "Owner should have one asset");
        assertEq(
            myAssets[0].assetId,
            expectedId,
            "MyAssets should contain the asset"
        );
    }

    function testCannotRegisterAssetZeroAddress() public {
        vm.prank(address(0));
        vm.expectRevert(abi.encodeWithSelector(Errors.ADDRESS_ZERO.selector, address(0)));
        switchAssets.registerAsset("This is a test asset");
    }


    function testTransferAsset() public {
        vm.prank(owner);
        bytes32 id = keccak256(abi.encode(owner, block.timestamp));
        switchAssets.registerAsset("Test Asset");

        vm.prank(owner);
        vm.expectEmit(true, true, true, true);
        emit ISwitch.OwnershipTransferred(id, owner, user1);

        switchAssets.transferAsset(id, user1);

        ISwitch.Asset memory asset = switchAssets.getAsset(id);
        assertEq(asset.assetOwner, user1, "New owner should be user1");

        vm.prank(owner);
        ISwitch.Asset[] memory ownerAssets = switchAssets.getMyAssets();
        assertEq(ownerAssets.length, 0, "Owner should have no assets");

        vm.prank(user1);
        ISwitch.Asset[] memory user1Assets = switchAssets.getMyAssets();
        assertEq(user1Assets.length, 1, "User1 should have one asset");
        assertEq(
            user1Assets[0].assetId,
            id,
            "User1 should own the transferred asset"
        );
    }

    function testCannotTransferNonExistentAsset() public {
        vm.prank(owner);
        bytes32 fakeId = bytes32(uint256(999));
         vm.expectRevert(abi.encodeWithSelector(Errors.ASSET_DOES_NOT_EXIST.selector, fakeId));
        switchAssets.transferAsset(fakeId, user1);
    }

    function testCannotTransferByNonOwner() public {
        vm.prank(owner);
        bytes32 id = keccak256(abi.encode(owner, block.timestamp));
        switchAssets.registerAsset("Test Asset");

        vm.prank(user1);
        vm.expectRevert(abi.encodeWithSelector(Errors.ONLY_OWNER.selector, user1));
        switchAssets.transferAsset(id, user2);
    }

    function testCannotTransferToZeroAddress() public {
        vm.prank(owner);
        bytes32 id = keccak256(abi.encode(owner, block.timestamp));
        switchAssets.registerAsset("Test Asset");

        vm.prank(owner);

        vm.expectRevert(abi.encodeWithSelector(Errors.ADDRESS_ZERO.selector, address(0)));
        switchAssets.transferAsset(id, address(0));
    }

    function testCannotTransferToSameOwner() public {
        vm.prank(owner);
        bytes32 id = keccak256(abi.encode(owner, block.timestamp));
        switchAssets.registerAsset("Test Asset");

        vm.prank(owner);

        vm.expectRevert(abi.encodeWithSelector(Errors.INVALID_TRANSACTION.selector, owner));
        switchAssets.transferAsset(id, owner);
    }
}



// import {IEri} from "../contracts/ISwitch.sol";
// import {Errors} from "../contracts/Errors.sol";
// import {ECDSA} from "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";
// import {Ownership} from "../contracts/Ownership.sol";

// Mock ISwitch interface
// interface ISwitch {

//     struct Asset {
//         bytes32 assetId;
//         address assetOwner;
//         string description;
//         uint256 registeredAt;
//     }

//     event AssetRegistered(bytes32 indexed assetId, address indexed assetOwner);
//     event OwnershipTransferred(
//         bytes32 indexed assetId,
//         address indexed oldOwner,
//         address indexed newOwner
//     );
// }

// contract Errors {
//     error ADDRESS_ZERO(address);
//     error ASSET_ALREADY_EXIST(bytes32);
//     error ASSET_DOES_NOT_EXIST(bytes32);
//     error ONLY_OWNER(address);
//     error INVALID_TRANSACTIO();
// }

// contract SwitchAssets {
//     mapping(bytes32 => ISwitch.Asset) private assets;
//     mapping(address => ISwitch.Asset[]) private myAssets;

//     modifier addressZeroCheck() {
//         if (msg.sender == address(0)) {
//             revert Errors.ADDRESS_ZERO(msg.sender);
//         }
//         _;
//     }

//     function registerAsset(string memory description) public addressZeroCheck {
//         bytes32 id = keccak256(abi.encode(msg.sender, block.timestamp));

//         if (assets[id].assetOwner != address(0)) {
//             revert Errors.ASSET_ALREADY_EXIST(id);
//         }

//         ISwitch.Asset storage newAsset = assets[id];
//         newAsset.assetId = id;
//         newAsset.assetOwner = msg.sender;
//         newAsset.description = description;
//         newAsset.registeredAt = block.timestamp;

//         myAssets[msg.sender].push(newAsset);

//         emit ISwitch.AssetRegistered(id, msg.sender);
//     }

//     function getAsset(bytes32 id) public view returns (ISwitch.Asset memory) {
//         if (assets[id].assetOwner == address(0)) {
//             revert Errors.ASSET_DOES_NOT_EXIST(id);
//         }
//         return assets[id];
//     }

//     function getMyAssets() public view returns (ISwitch.Asset[] memory) {
//         address caller = msg.sender;
//         ISwitch.Asset[] memory myItems = myAssets[caller];
//         uint256 validCount = 0;

//         for (uint256 i = 0; i < myItems.length; i++) {
//             if (assets[myItems[i].assetId].assetOwner == caller) {
//                 validCount++;
//             }
//         }

//         if (validCount == 0) {
//             return new ISwitch.Asset[](0);
//         }

//         ISwitch.Asset[] memory newList = new ISwitch.Asset[](validCount);
//         for (uint256 i = 0; i < myItems.length; i++) {
//             if (assets[myItems[i].assetId].assetOwner == caller) {
//                 newList[validCount - 1] = assets[myItems[i].assetId];
//                 validCount--;
//             }
//         }
//         return newList;
//     }

//     function transferAsset(
//         bytes32 id,
//         address newOwner
//     ) public addressZeroCheck {
//         address caller = msg.sender;

//         if (assets[id].assetOwner != caller) {
//             revert Errors.ONLY_OWNER(caller);
//         }

//         if (newOwner == caller) {
//             revert Errors.INVALID_TRANSACTIO();
//         }

//         if (assets[id].assetOwner == address(0)) {
//             revert Errors.ASSET_DOES_NOT_EXIST(id);
//         }

//         if (newOwner == address(0)) {
//             revert Errors.ADDRESS_ZERO(newOwner);
//         }

//         address oldOwner = assets[id].assetOwner;
//         assets[id].assetOwner = newOwner;
//         myAssets[newOwner].push(assets[id]);

//         // Remove asset from old owner's myAssets
//         ISwitch.Asset[] storage oldOwnerAssets = myAssets[oldOwner];
//         for (uint256 i = 0; i < oldOwnerAssets.length; i++) {
//             if (oldOwnerAssets[i].assetId == id) {
//                 oldOwnerAssets[i] = oldOwnerAssets[oldOwnerAssets.length - 1];
//                 oldOwnerAssets.pop();
//                 break;
//             }
//         }

//         emit ISwitch.OwnershipTransferred(id, oldOwner, newOwner);
//     }
// }
