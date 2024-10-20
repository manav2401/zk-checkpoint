// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

contract Verifier {
    // SP1 related
    address public verifier;
    bytes32 public consensusProofVKey;

    // Last verified bor block details
    uint256 public lastVerifiedBorBlockNumber;
    bytes32 public lastVerifiedBorBlockHash;

    constructor(address _verifier, bytes32 _consensusProofVKey) {
        verifier = _verifier;
        consensusProofVKey = _consensusProofVKey;
    }

    function verifyCheckpointSignatures(
        bytes calldata _proofBytes,
        bytes32 _l1BlockHash,
        bytes32 _borBlockNumber,
        bytes32 _borBlockHash
    ) public view  {
        // Call verifier..
    }

}