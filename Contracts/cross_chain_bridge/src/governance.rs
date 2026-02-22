use soroban_sdk::{Address, Env, Vec};

use crate::errors::BridgeError;
use crate::types::{DataKey, GovernanceProposal, ProposalType, Validator};

pub fn create(
    env: &Env,
    proposer: Address,
    proposal_type: ProposalType,
    voting_period: u32,
) -> u64 {
    let count: u64 = env.storage().instance().get(&DataKey::ProposalCount).unwrap_or(0);
    let id = count + 1;

    let proposal = GovernanceProposal {
        id,
        proposer,
        proposal_type,
        votes_for: 0,
        executed: false,
        deadline: env.ledger().sequence() + voting_period,
    };

    env.storage().persistent().set(&DataKey::Proposal(id), &proposal);
    env.storage().instance().set(&DataKey::ProposalCount, &id);
    id
}

pub fn vote(env: &Env, proposal_id: u64, voter: Address) -> Result<(), BridgeError> {
    let mut proposal: GovernanceProposal = env
        .storage()
        .persistent()
        .get(&DataKey::Proposal(proposal_id))
        .ok_or(BridgeError::ProposalNotFound)?;

    if proposal.executed {
        return Err(BridgeError::ProposalAlreadyExecuted);
    }
    if env.ledger().sequence() > proposal.deadline {
        return Err(BridgeError::ProposalExpired);
    }
    if env
        .storage()
        .persistent()
        .get::<DataKey, bool>(&DataKey::HasVoted(proposal_id, voter.clone()))
        .unwrap_or(false)
    {
        return Err(BridgeError::AlreadyVoted);
    }

    proposal.votes_for += 1;
    env.storage().persistent().set(&DataKey::Proposal(proposal_id), &proposal);
    env.storage()
        .persistent()
        .set(&DataKey::HasVoted(proposal_id, voter), &true);
    Ok(())
}

pub fn execute(env: &Env, proposal_id: u64, threshold: u32) -> Result<(), BridgeError> {
    let mut proposal: GovernanceProposal = env
        .storage()
        .persistent()
        .get(&DataKey::Proposal(proposal_id))
        .ok_or(BridgeError::ProposalNotFound)?;

    if proposal.executed {
        return Err(BridgeError::ProposalAlreadyExecuted);
    }
    if env.ledger().sequence() <= proposal.deadline {
        return Err(BridgeError::VotingStillActive);
    }
    if proposal.votes_for < threshold {
        return Err(BridgeError::ProposalNotApproved);
    }

    apply(env, &proposal.proposal_type);

    proposal.executed = true;
    env.storage().persistent().set(&DataKey::Proposal(proposal_id), &proposal);
    Ok(())
}

fn apply(env: &Env, proposal_type: &ProposalType) {
    match proposal_type {
        ProposalType::AddValidator(validator) => {
            let mut validators: Vec<Validator> = env
                .storage()
                .instance()
                .get(&DataKey::Validators)
                .unwrap_or_else(|| Vec::new(env));
            validators.push_back(validator.clone());
            env.storage().instance().set(&DataKey::Validators, &validators);
        }
        ProposalType::RemoveValidator(signing_key) => {
            let validators: Vec<Validator> = env
                .storage()
                .instance()
                .get(&DataKey::Validators)
                .unwrap_or_else(|| Vec::new(env));
            let mut updated: Vec<Validator> = Vec::new(env);
            for v in validators.iter() {
                if v.signing_key != *signing_key {
                    updated.push_back(v);
                }
            }
            env.storage().instance().set(&DataKey::Validators, &updated);
        }
        ProposalType::UpdateThreshold(new_threshold) => {
            env.storage().instance().set(&DataKey::Threshold, new_threshold);
        }
        ProposalType::UpdateAdmin(new_admin) => {
            env.storage().instance().set(&DataKey::Admin, new_admin);
        }
        ProposalType::PauseBridge => {
            env.storage().instance().set(&DataKey::Paused, &true);
        }
        ProposalType::UnpauseBridge => {
            env.storage().instance().set(&DataKey::Paused, &false);
        }
    }
}
