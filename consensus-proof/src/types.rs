use prost::Message;
use std::{io::Cursor, ops::Sub};

// Include the `types` module, which is generated from types.proto.
pub mod heimdall_types {
    include!(concat!(env!("OUT_DIR"), "/types.rs"));
}

// Function to pad bytes to 32 bytes as in Go's convertTo32
pub fn pad_to_32_bytes(input: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let input_len = input.len();

    // Copy the input bytes to the end of the 32-byte array, padding with leading zeros
    output[32 - input_len..].copy_from_slice(input);

    output
}

pub fn checkpoint_to_bytes(checkpoint: &heimdall_types::CheckpointMsg) -> Vec<u8> {
    let mut result = Vec::new();

    // proposer
    result.extend_from_slice(pad_to_32_bytes(&checkpoint.proposer).as_slice());

    // start_block
    result.extend_from_slice(pad_to_32_bytes(&checkpoint.start_block.to_be_bytes()).as_slice());

    // end_block
    result.extend_from_slice(pad_to_32_bytes(&checkpoint.end_block.to_be_bytes()).as_slice());

    // root_hash
    result.extend_from_slice(pad_to_32_bytes(&checkpoint.root_hash).as_slice());

    // account_root_hash
    result.extend_from_slice(pad_to_32_bytes(&checkpoint.account_root_hash).as_slice());

    // bor_chain_id
    let temp: u64 = checkpoint.bor_chain_id.parse().unwrap();
    result.extend_from_slice(pad_to_32_bytes(&temp.to_be_bytes()).as_slice());

    result
}

pub fn bytes_to_checkpoint(buf: Vec<u8>) -> heimdall_types::CheckpointMsg {
    let proposer: [u8; 20] = buf[12..32].try_into().unwrap();
    let start_block: u64 = u64::from_be_bytes(buf[56..64].try_into().unwrap());
    let end_block: u64 = u64::from_be_bytes(buf[88..96].try_into().unwrap());
    let root_hash: [u8; 32] = buf[96..128].try_into().unwrap();
    let account_root_hash: [u8; 32] = buf[128..160].try_into().unwrap();
    let bor_chain_id_u64: u64 = u64::from_be_bytes(buf[184..192].try_into().unwrap());
    let bor_chain_id: String = bor_chain_id_u64.to_string();
    heimdall_types::CheckpointMsg {
        proposer: proposer.to_vec(),
        start_block,
        end_block,
        root_hash: root_hash.to_vec(),
        account_root_hash: account_root_hash.to_vec(),
        bor_chain_id,
    }
}

/// Serialize the checkpoint message
pub fn serialize_checkpoint_tx(m: &heimdall_types::StdTx) -> Vec<u8> {
    let mut buf = Vec::with_capacity(m.encoded_len());

    // Unwrap is safe, since we have reserved sufficient capacity in the vector.
    m.encode_length_delimited(&mut buf).unwrap();
    buf
}

/// Deserialize the checkpoint message to extract checkpoint info
pub fn deserialize_checkpoint_tx(
    buf: &mut Vec<u8>,
) -> Result<heimdall_types::StdTx, prost::DecodeError> {
    // Hack for handling interface
    // 184, 1, 240, 98, 93, 238, 10, 111, 215, 168, 164, 169, 10, 20
    // 176, 1, 10, 107, 10, 20
    //
    // 184, 1, 240, 98, 93, 238, 10, 111, 215, 168, 164, 169, 10, 20
    // 176, 1, 10, 107, 10, 20
    let old_prefix: Vec<u8> = vec![1, 240, 98, 93, 238, 10, 111, 215, 168, 164, 169];
    let mut new_prefix: Vec<u8> = vec![1, 10, 107];

    let matches = buf.len() > old_prefix.len()
        && old_prefix[..].iter().enumerate().all(|(i, &b)| {
            if i == 6 {
                new_prefix[2] = buf[i + 1].sub(4);
                true
            } else {
                b == buf[i + 1]
            }
        });

    if matches {
        buf.drain(1..1 + old_prefix.len());
        buf.splice(1..1, new_prefix.iter().cloned());
        buf[0] = buf[0].sub(8);
    } else {
        return Err(prost::DecodeError::new("Invalid prefix"));
    }

    heimdall_types::StdTx::decode_length_delimited(&mut Cursor::new(buf))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use alloy_primitives::{hex, keccak256, Address};
    use base64::{prelude::BASE64_STANDARD, Engine};
    use reth_primitives::recover_signer_unchecked;

    // #[test]
    fn test_deserialize_checkpoint_msg() {
        let a = "uAHwYl3uCm/XqKSpChRtwt1U8kl57CYhJ5THGv7+1yIoDBDz7LAGGPLwsAYiIG13yje6CCcTwisX8k0naX249I92JIpsCbcU/f5Pnp+xKiBLa5lLmdJONehiavQZoIfseEmNl2Jl5YedfCK5JBw7mDIFODAwMDISQX5H4v7pEORvrXwVu2+pyUKQJXkvyP8pVb5a7V3KDStwW6AwgsQnh/MKlPe+y/YEKxbVH8J6XqILlTOmiQhnSi8A".to_string();
        let mut decoded_tx_data = BASE64_STANDARD.decode(a).expect("tx_data decoding failed");
        let decoded_message = deserialize_checkpoint_tx(&mut decoded_tx_data).unwrap();
        println!("decoded_tx_data: {:?}", decoded_tx_data);

        let m = heimdall_types::CheckpointMsg {
            proposer: hex::decode("6dc2dd54f24979ec26212794c71afefed722280c")
                .unwrap()
                .to_vec(),
            start_block: 13383283,
            end_block: 13383794,
            root_hash: hex::decode(
                "6d77ca37ba082713c22b17f24d27697db8f48f76248a6c09b714fdfe4f9e9fb1",
            )
            .unwrap()
            .to_vec(),
            account_root_hash: hex::decode(
                "4b6b994b99d24e35e8626af419a087ec78498d976265e5879d7c22b9241c3b98",
            )
            .unwrap()
            .to_vec(),
            bor_chain_id: "80002".to_string(),
        };
        let msg = heimdall_types::StdTx {
            msg: Some(m),
            signature: hex::decode("7e47e2fee910e46fad7c15bb6fa9c9429025792fc8ff2955be5aed5dca0d2b705ba03082c42787f30a94f7becbf6042b16d51fc27a5ea20b9533a68908674a2f00").unwrap().to_vec(),
            memo: "".to_string(),
        };

        assert_eq!(decoded_message, msg);
    }

    #[test]
    fn test_sidetx() {
        let signature = "FC1Sp9LFVzWEDv9Q9oRs7sDUJZLKGmG9KVQwJ4VxreZzZUG7lTmNlKqISt5Pso/G8WZJzYUWHFzjJkj2uJwjTAE=".to_string();
        let decoded_signature = BASE64_STANDARD
            .decode(signature)
            .expect("unable to decode signature");

        let mut sig = [0u8; 65];
        sig.copy_from_slice(decoded_signature.as_slice());

        let hash = "Gjn/w2EdD2nl9bC91e5Jj3upa77GamV7OUOCXeRwA14=".to_string();
        let hash_bytes = BASE64_STANDARD.decode(hash).expect("unable to decode hash");

        let m = heimdall_types::CheckpointMsg {
            proposer: hex::decode("6dc2dd54f24979ec26212794c71afefed722280c")
                .unwrap()
                .to_vec(),
            start_block: 13383283,
            end_block: 13383794,
            root_hash: hex::decode(
                "6d77ca37ba082713c22b17f24d27697db8f48f76248a6c09b714fdfe4f9e9fb1",
            )
            .unwrap()
            .to_vec(),
            account_root_hash: hex::decode(
                "4b6b994b99d24e35e8626af419a087ec78498d976265e5879d7c22b9241c3b98",
            )
            .unwrap()
            .to_vec(),
            bor_chain_id: "80002".to_string(),
        };

        let data = checkpoint_to_bytes(&m);
        let derived = bytes_to_checkpoint(data.clone());
        assert_eq!(m, derived);

        let mut result = vec![1];
        result.extend_from_slice(data.as_slice());
        println!("result: {:?}", result);
        let message_hash = keccak256(result);
        // println!("message_hash: {:?}", message_hash.clone().bytes());

        let recovered_signer = recover_signer_unchecked(&sig, &message_hash).unwrap_or_default();
        let expected = Address::from_str("392E41C8044B783aA9e305840645F2D2D7D51757").unwrap();
        println!("expected: {:?}", expected);
        println!("recovered_signer: {:?}", recovered_signer);
        assert_eq!(expected, recovered_signer);
    }
}
