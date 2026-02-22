use soroban_sdk::{token, Address, Bytes, Env};

use crate::errors::BridgeError;
use crate::types::{DataKey, WrappedAssetInfo};

pub fn register(env: &Env, source_chain: u32, original_token: Bytes, stellar_token: Address) {
    let info = WrappedAssetInfo {
        source_chain,
        original_token: original_token.clone(),
        stellar_token,
    };
    env.storage()
        .persistent()
        .set(&DataKey::WrappedAsset(source_chain, original_token), &info);
}

pub fn get_asset(env: &Env, source_chain: u32, original_token: &Bytes) -> Result<WrappedAssetInfo, BridgeError> {
    env.storage()
        .persistent()
        .get(&DataKey::WrappedAsset(source_chain, original_token.clone()))
        .ok_or(BridgeError::WrappedAssetNotFound)
}

pub fn mint_wrapped(env: &Env, stellar_token: &Address, recipient: &Address, amount: i128) {
    let client = token::StellarAssetClient::new(env, stellar_token);
    client.mint(recipient, &amount);
}

pub fn burn_wrapped(env: &Env, stellar_token: &Address, amount: i128) {
    let bridge = env.current_contract_address();
    let client = token::Client::new(env, stellar_token);
    client.burn(&bridge, &amount);
}

pub fn take_from_user(env: &Env, stellar_token: &Address, user: &Address, amount: i128) {
    let bridge = env.current_contract_address();
    let client = token::Client::new(env, stellar_token);
    client.transfer_from(&bridge, user, &bridge, &amount);
}
