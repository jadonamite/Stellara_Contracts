use soroban_sdk::{Bytes, BytesN, Env, Vec};

use crate::errors::BridgeError;

pub fn compute_signing_hash(env: &Env, message_id: &BytesN<32>, stellar_chain_id: u32) -> BytesN<32> {
    let mut data = Bytes::new(env);
    data.append(&Bytes::from_slice(env, b"stellar_bridge:v1:"));
    let id_bytes: Bytes = message_id.clone().into();
    data.append(&id_bytes);
    data.append(&Bytes::from_slice(env, &stellar_chain_id.to_be_bytes()));
    env.crypto().sha256(&data)
}

pub fn verify_multi_sig(
    env: &Env,
    signing_hash: &BytesN<32>,
    signing_keys: &Vec<BytesN<32>>,
    signer_indices: &Vec<u32>,
    signatures: &Vec<BytesN<64>>,
    threshold: u32,
) -> Result<(), BridgeError> {
    let sig_count = signatures.len();

    if sig_count < threshold {
        return Err(BridgeError::ThresholdNotMet);
    }
    if sig_count != signer_indices.len() {
        return Err(BridgeError::InvalidSignatureCount);
    }

    let hash_bytes: Bytes = signing_hash.clone().into();

    for i in 0..sig_count {
        let idx_i = signer_indices.get(i).unwrap();

        for j in (i + 1)..sig_count {
            if signer_indices.get(j).unwrap() == idx_i {
                return Err(BridgeError::DuplicateSigner);
            }
        }

        if idx_i >= signing_keys.len() {
            return Err(BridgeError::InvalidSignerIndex);
        }

        let key = signing_keys.get(idx_i).unwrap();
        let sig = signatures.get(i).unwrap();
        env.crypto().ed25519_verify(&key, &hash_bytes, &sig);
    }

    Ok(())
}
