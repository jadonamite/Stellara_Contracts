#![no_std]

mod assets;
mod errors;
mod governance;
mod types;
mod verification;

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Bytes, BytesN, Env, Vec};

use errors::BridgeError;
use types::{BridgeMessage, DataKey, GovernanceProposal, MessageStatus, ProposalType, Validator, WrappedAssetInfo};
use verification::{compute_signing_hash, verify_multi_sig};

#[contract]
pub struct CrossChainBridgeContract;

fn get_admin(env: &Env) -> Result<Address, BridgeError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(BridgeError::NotInitialized)
}

fn require_admin(env: &Env) -> Result<(), BridgeError> {
    let admin = get_admin(env)?;
    admin.require_auth();
    Ok(())
}

fn require_not_paused(env: &Env) -> Result<(), BridgeError> {
    let paused: bool = env.storage().instance().get(&DataKey::Paused).unwrap_or(false);
    if paused {
        Err(BridgeError::Paused)
    } else {
        Ok(())
    }
}

fn get_validators(env: &Env) -> Vec<Validator> {
    env.storage()
        .instance()
        .get(&DataKey::Validators)
        .unwrap_or_else(|| Vec::new(env))
}

fn get_threshold(env: &Env) -> u32 {
    env.storage().instance().get(&DataKey::Threshold).unwrap_or(1)
}

fn require_validator(env: &Env, caller: &Address) -> Result<(), BridgeError> {
    let validators = get_validators(env);
    for v in validators.iter() {
        if v.address == *caller {
            caller.require_auth();
            return Ok(());
        }
    }
    Err(BridgeError::Unauthorized)
}

#[contractimpl]
impl CrossChainBridgeContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        validators: Vec<Validator>,
        threshold: u32,
        stellar_chain_id: u32,
    ) -> Result<(), BridgeError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(BridgeError::AlreadyInitialized);
        }
        if threshold == 0 || threshold > validators.len() {
            return Err(BridgeError::InvalidThreshold);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Validators, &validators);
        env.storage().instance().set(&DataKey::Threshold, &threshold);
        env.storage().instance().set(&DataKey::StellarChainId, &stellar_chain_id);
        env.storage().instance().set(&DataKey::Paused, &false);
        Ok(())
    }

    // Receives a cross-chain message with multi-sig proof and mints wrapped assets.
    pub fn receive_message(
        env: Env,
        message: BridgeMessage,
        original_token: Bytes,
        asset_amount: i128,
        signer_indices: Vec<u32>,
        signatures: Vec<BytesN<64>>,
    ) -> Result<(), BridgeError> {
        require_not_paused(&env)?;

        if env
            .storage()
            .persistent()
            .get::<DataKey, MessageStatus>(&DataKey::Message(message.id.clone()))
            .is_some()
        {
            return Err(BridgeError::MessageAlreadyProcessed);
        }

        let stellar_chain_id: u32 = env
            .storage()
            .instance()
            .get(&DataKey::StellarChainId)
            .ok_or(BridgeError::NotInitialized)?;

        let signing_hash = compute_signing_hash(&env, &message.id, stellar_chain_id);
        let validators = get_validators(&env);
        let threshold = get_threshold(&env);

        let mut signing_keys: Vec<BytesN<32>> = Vec::new(&env);
        for v in validators.iter() {
            signing_keys.push_back(v.signing_key);
        }

        verify_multi_sig(
            &env,
            &signing_hash,
            &signing_keys,
            &signer_indices,
            &signatures,
            threshold,
        )?;

        let asset_info = assets::get_asset(&env, message.source_chain, &original_token)?;
        assets::mint_wrapped(&env, &asset_info.stellar_token, &message.receiver, asset_amount);

        env.storage()
            .persistent()
            .set(&DataKey::Message(message.id.clone()), &MessageStatus::Processed);

        env.events().publish(
            (symbol_short!("msg_recv"), message.id.clone()),
            (message.source_chain, message.receiver, asset_amount),
        );

        Ok(())
    }

    // Burns wrapped assets and emits an event for the destination chain to release original tokens.
    pub fn initiate_bridge_out(
        env: Env,
        user: Address,
        stellar_token: Address,
        amount: i128,
        destination_chain: u32,
        destination_address: Bytes,
    ) -> Result<(), BridgeError> {
        require_not_paused(&env)?;
        user.require_auth();

        assets::take_from_user(&env, &stellar_token, &user, amount);
        assets::burn_wrapped(&env, &stellar_token, amount);

        let mut nonce_data = Bytes::new(&env);
        nonce_data.append(&Bytes::from_slice(&env, b"bridge_out:"));
        let addr_bytes: Bytes = stellar_token.clone().into();
        nonce_data.append(&addr_bytes);
        nonce_data.append(&Bytes::from_slice(&env, &amount.to_be_bytes()));
        nonce_data.append(&Bytes::from_slice(&env, &env.ledger().sequence().to_be_bytes()));
        let message_id = env.crypto().sha256(&nonce_data);

        env.events().publish(
            (symbol_short!("bridge_out"), message_id.clone()),
            (destination_chain, destination_address, stellar_token, amount),
        );

        Ok(())
    }

    pub fn register_wrapped_asset(
        env: Env,
        source_chain: u32,
        original_token: Bytes,
        stellar_token: Address,
    ) -> Result<(), BridgeError> {
        require_admin(&env)?;
        assets::register(&env, source_chain, original_token, stellar_token);
        Ok(())
    }

    pub fn create_proposal(
        env: Env,
        proposer: Address,
        proposal_type: ProposalType,
        voting_period: u32,
    ) -> Result<u64, BridgeError> {
        require_not_paused(&env)?;
        require_validator(&env, &proposer)?;
        let id = governance::create(&env, proposer, proposal_type, voting_period);
        env.events().publish((symbol_short!("prop_new"), id), ());
        Ok(id)
    }

    pub fn vote_proposal(env: Env, voter: Address, proposal_id: u64) -> Result<(), BridgeError> {
        require_not_paused(&env)?;
        require_validator(&env, &voter)?;
        governance::vote(&env, proposal_id, voter)?;
        env.events().publish((symbol_short!("prop_vote"), proposal_id), ());
        Ok(())
    }

    pub fn execute_proposal(env: Env, proposal_id: u64) -> Result<(), BridgeError> {
        let threshold = get_threshold(&env);
        governance::execute(&env, proposal_id, threshold)?;
        env.events().publish((symbol_short!("prop_exec"), proposal_id), ());
        Ok(())
    }

    pub fn pause(env: Env) -> Result<(), BridgeError> {
        require_admin(&env)?;
        env.storage().instance().set(&DataKey::Paused, &true);
        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), BridgeError> {
        require_admin(&env)?;
        env.storage().instance().set(&DataKey::Paused, &false);
        Ok(())
    }

    pub fn add_validator(env: Env, validator: Validator) -> Result<(), BridgeError> {
        require_admin(&env)?;
        let mut validators = get_validators(&env);
        validators.push_back(validator);
        env.storage().instance().set(&DataKey::Validators, &validators);
        Ok(())
    }

    pub fn remove_validator(env: Env, signing_key: BytesN<32>) -> Result<(), BridgeError> {
        require_admin(&env)?;
        let validators = get_validators(&env);
        let mut updated: Vec<Validator> = Vec::new(&env);
        let mut found = false;
        for v in validators.iter() {
            if v.signing_key == signing_key {
                found = true;
            } else {
                updated.push_back(v);
            }
        }
        if !found {
            return Err(BridgeError::ValidatorNotFound);
        }
        let threshold = get_threshold(&env);
        if threshold > updated.len() {
            return Err(BridgeError::InvalidThreshold);
        }
        env.storage().instance().set(&DataKey::Validators, &updated);
        Ok(())
    }

    pub fn update_threshold(env: Env, threshold: u32) -> Result<(), BridgeError> {
        require_admin(&env)?;
        let validators = get_validators(&env);
        if threshold == 0 || threshold > validators.len() {
            return Err(BridgeError::InvalidThreshold);
        }
        env.storage().instance().set(&DataKey::Threshold, &threshold);
        Ok(())
    }

    // --- View functions ---

    pub fn get_message_status(env: Env, message_id: BytesN<32>) -> Option<MessageStatus> {
        env.storage().persistent().get(&DataKey::Message(message_id))
    }

    pub fn get_wrapped_asset(
        env: Env,
        source_chain: u32,
        original_token: Bytes,
    ) -> Option<WrappedAssetInfo> {
        env.storage()
            .persistent()
            .get(&DataKey::WrappedAsset(source_chain, original_token))
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<GovernanceProposal> {
        env.storage().persistent().get(&DataKey::Proposal(proposal_id))
    }

    pub fn get_validators(env: Env) -> Vec<Validator> {
        get_validators(&env)
    }

    pub fn get_threshold(env: Env) -> u32 {
        get_threshold(&env)
    }

    pub fn is_paused(env: Env) -> bool {
        env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }
}
