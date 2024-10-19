use alloy_sol_types::sol;


sol! {
    contract Verifier {
        function getLastCheckpointEndBlock() external view returns (uint256);
    }
}