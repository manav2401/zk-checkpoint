// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {ISP1Verifier} from "@sp1-contracts/ISP1Verifier.sol";

contract PoSVerifier {
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
        bytes32 _borBlockHash,
        uint256 _borBlockNumber
    ) public {
        bytes memory publicValues = abi.encodePacked(_l1BlockHash, _borBlockHash, _borBlockNumber);
        ISP1Verifier(verifier).verifyProof(consensusProofVKey, publicValues, _proofBytes);
        lastVerifiedBorBlockHash = _borBlockHash;
        lastVerifiedBorBlockNumber = _borBlockNumber;
    }

}