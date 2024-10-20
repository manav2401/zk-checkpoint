// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/// @title SP1 Verifier Interface
/// @author Succinct Labs
/// @notice This contract is the interface for the SP1 Verifier.
interface ISP1Verifier {
    /// @notice Verifies a proof with given public values and vkey.
    /// @dev It is expected that the first 4 bytes of proofBytes must match the first 4 bytes of
    /// target verifier's VERIFIER_HASH.
    /// @param programVKey The verification key for the RISC-V program.
    /// @param publicValues The public values encoded as bytes.
    /// @param proofBytes The proof of the program execution the SP1 zkVM encoded as bytes.
    function verifyProof(
        bytes32 programVKey,
        bytes calldata publicValues,
        bytes calldata proofBytes
    ) external view;
}

interface ISP1VerifierWithHash is ISP1Verifier {
    /// @notice Returns the hash of the verifier.
    function VERIFIER_HASH() external pure returns (bytes32);
}

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