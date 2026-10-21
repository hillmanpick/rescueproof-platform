// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @notice Stores record-integrity roots; does NOT certify food safety or delivery.
contract RescueProofAnchor {
    address public owner;
    address public pendingOwner;
    mapping(bytes32 => uint256) public roots;
    event RootAnchored(bytes32 indexed root, uint256 timestamp);
    event OwnershipTransferStarted(address indexed nextOwner);
    event OwnershipTransferred(address indexed previousOwner, address indexed newOwner);

    constructor(address initialOwner) {
        require(initialOwner != address(0), "zero owner");
        owner = initialOwner;
    }
    modifier onlyOwner() { require(msg.sender == owner, "owner only"); _; }
    function anchor(bytes32 root) external onlyOwner {
        require(root != bytes32(0), "empty root");
        if (roots[root] == 0) {
            roots[root] = block.timestamp;
            emit RootAnchored(root, block.timestamp);
        }
    }
    function transferOwnership(address nextOwner) external onlyOwner {
        require(nextOwner != address(0), "zero owner");
        pendingOwner = nextOwner;
        emit OwnershipTransferStarted(nextOwner);
    }
    function acceptOwnership() external {
        require(msg.sender == pendingOwner, "pending owner only");
        address previous = owner;
        owner = pendingOwner;
        pendingOwner = address(0);
        emit OwnershipTransferred(previous, owner);
    }
}
