// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

interface RootChain {
    function getLastChildBlock() external view returns (uint256);
}
interface StakeManager {
    enum Status {Inactive, Active, Locked, Unstaked}
    function NFTCounter() external view returns (uint256);
    function validators(uint256) external view returns (uint256, uint256, uint256, uint256, uint256, address, address, Status, uint256, uint256, uint256, uint256, uint256);
    function validatorState() external view returns (uint256, uint256);
}

contract RootChainInfo {
    address public rootChain;
    address public stakeManager;

    constructor(address _rootChain, address _stakeManager) {
        rootChain = _rootChain;
        stakeManager = _stakeManager;
    }

    function getLastCheckpointEndBlock() external view returns (uint256) {
        return RootChain(rootChain).getLastChildBlock();
    }

    function getActiveValidatorInfo() public view returns(address[] memory, uint256[] memory, uint256) {
        // Get the total number of validators stored by fetching the NFT count. The count is
        // assigned to the next validator and hence we subtract 1 from it.
        uint256 length = StakeManager(stakeManager).NFTCounter() - 1;

        address[] memory signers = new address[](length);
        uint256[] memory stakes = new uint256[](length);
        bool[] memory isActive = new bool[](length);
        uint256 totalActive = 0;

        // Validator index starts from 1.
        for (uint256 i = 1; i <= length; i++) {
            uint256 selfStake;
            uint256 delegatedStake;
            address signer;
            StakeManager.Status status;
            (selfStake, , , , , signer, , status, , , ,delegatedStake,) = StakeManager(stakeManager).validators(i);
            signers[i-1] = signer;
            stakes[i-1] = selfStake + delegatedStake;
            isActive[i-1] = status == StakeManager.Status.Active;
            if (isActive[i-1]) {
                totalActive += 1;
            }
        }

        address[] memory activeSigners = new address[](totalActive);
        uint256[] memory activeStakes = new uint256[](totalActive);

        uint256 j = 0;
        for (uint256 i = 0; i < length; i++) {
            if (isActive[i]) {
                activeSigners[j] = signers[i];
                activeStakes[j] = stakes[i] / 1e18;
                j++;
            }
        }

        uint256 totalStake;
        (totalStake, ) = StakeManager(stakeManager).validatorState();
        return (activeSigners, activeStakes, totalStake / 1e18);
    }
}