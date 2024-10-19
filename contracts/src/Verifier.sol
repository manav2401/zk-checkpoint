// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface RootChain {
    function getLastChildBlock() external view returns (uint256);
}

contract Verifier {
    address public rootChain;

    constructor(address _rootChain) {
        rootChain = _rootChain;
    }

    function getLastCheckpointEndBlock() external view returns (uint256) {
        return RootChain(rootChain).getLastChildBlock();
    }
}