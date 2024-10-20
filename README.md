# zk-checkpoint

ZK proofs for checkpointing on Polygon PoS to save on settlement gas and faster interop

### Numbers

- Initially out of 2.2M total gas, signature verification took 1.9M (which is 86% of total).
- With ZK proofs, it takes 0.3M gas i.e. 84% improvements from original numbers.
- On-chain verifier call: https://sepolia.etherscan.io/tx/0x3fb166bc84e4c2861a781bfb928042a8bf56a3411157328dc3b2b2d960e75a17

### Generating proofs

1. Make sure your `.env` is updated
2. Choose a checkpoint (find it's id and hash) and choose an L1 block (ideally recentmost).
3. Run the following command to generate proof
```bash
cd operator
RUST_LOG=info cargo run --release --bin prove -- --checkpoint-id A 
    --checkpoint-tx-hash B 
    --l1-block-number C 
    --prove
```
where A: any valid checkpoint id, B: any valid checkpoint tx, C: recentmost L1 block

Note that proof generation will fail if you choose an old checkpoint because it verifies the sequence
against the last submitted checkpoint.

### Deployments

Deploy on sepolia using the command below:
```bash
cd contracts

forge create --rpc-url $RPC_SEPOLIA --constructor-args A B --private-key $PK src/RootChainInfo.sol:RootChainInfo --via-ir

forge create --rpc-url $RPC_SEPOLIA --constructor-args 0x3B6041173B80E77f038f3F2C0f9744f04837185e
0x00b49a3cf8783f4eac77c6b3c26155b4559cb321d88bfd95dc7ff9cb2dbd9d7c --private-key $PK src/PoSVerifier.sol:PoSVerifier --via-ir
```
where the first one is root chain info (which fetches data from L1 contracts) and second is the verifier.
A: root chain proxy (0xbd07D7E1E93c8d4b2a261327F3C28a8EA7167209 on sepolia)
B: stake manager proxy (0x4AE8f648B1Ec892B6cc68C89cc088583964d08bE on sepolia)
C: SP1 verifier (0x3B6041173B80E77f038f3F2C0f9744f04837185e on sepolia)
D: VKey of the program: (0x00ced18987d2fa321ec2a36c2b2ec15d7980cc36fd6ed32ec0e479cf1f5a9f9d)

Recent deployments:
- RootChainInfo: 0xD88656159695770a766C01f6309dD71fE289F970
- PoSVerifier: 0x6e67834E4B98dc8dac5a186eCcdD54C78e9863f2

Deployments on an L1 fork:
- RootChainInfo: 0x065DADD6D5b5dFa9f6a732e1044332126862bb6F
- PoSVerifier: 0x6E0BB58A0F8EB705874A1852c0C5a48926666d97

### Rationale

Polygon PoS being one of the most used chains has a mechanism where it settles to Ethereum (L1) at regular intervals through checkpoints which is helpful for bridging. As a result, the consensus signatures are sent to L1 contract and signature verification for all validators is done on-chain. This incurs a lot of gas on ethereum and **~86%** of total gas used is taken by signature verification for all validators (on mainnet, roughly 105). The costs gas go **>250$** in case of high demand. 

This settlement workflow can be improved using ZK proofs which asserts that majority of validator set (>2/3) voted on a particular checkpoint. This can lead to 2 things:
1. It can drastically improve the L1 gas costs incurred currently on mainnet leading to cheaper on-chain settlement.
2. Because we commit to a specific block hash, other chains and apps on those chains can leverage this info to derive finality and PoS can settle on multiple chains with giving them a strong guarantee of a view of the chain. 