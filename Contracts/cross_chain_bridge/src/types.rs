#![allow(dead_code)]

use soroban_sdk::{contracttype, Address, Bytes, BytesN};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageStatus {
    Pending,
    Processed,
    Failed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Validator {
    pub address: Address,
    pub signing_key: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BridgeMessage {
    pub id: BytesN<32>,
    pub source_chain: u32,
    pub sender: Bytes,
    pub receiver: Address,
    pub payload: Bytes,
    pub nonce: u64,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct WrappedAssetInfo {
    pub source_chain: u32,
    pub original_token: Bytes,
    pub stellar_token: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProposalType {
    AddValidator(Validator),
    RemoveValidator(BytesN<32>),
    UpdateThreshold(u32),
    UpdateAdmin(Address),
    PauseBridge,
    UnpauseBridge,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct GovernanceProposal {
    pub id: u64,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub votes_for: u32,
    pub executed: bool,
    pub deadline: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Admin,
    Validators,
    Threshold,
    Paused,
    StellarChainId,
    Message(BytesN<32>),
    WrappedAsset(u32, Bytes),
    ProposalCount,
    Proposal(u64),
    HasVoted(u64, Address),
}
