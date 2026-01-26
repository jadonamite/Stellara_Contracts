#![cfg(test)]

use super::*;
use soroban_sdk::{Env, testutils::Address as _, Vec, symbol_short};
use shared::governance::ProposalStatus;

#[test]
fn test_contract_initialization() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);

    let admin = Address::random(&env);
    let approver1 = Address::random(&env);
    let approver2 = Address::random(&env);
    let executor = Address::random(&env);

    let mut approvers = Vec::new(&env);
    approvers.push_back(approver1.clone());
    approvers.push_back(approver2.clone());

    let result = UpgradeableTradingContract::init(
        env.clone(),
        admin.clone(),
        approvers.clone(),
        executor.clone(),
    );

    assert!(result.is_ok());

    // Verify version is set
    let version = UpgradeableTradingContract::get_version(env);
    assert_eq!(version, 1);
}

#[test]
fn test_contract_cannot_be_initialized_twice() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);

    let admin = Address::random(&env);
    let approver = Address::random(&env);
    let executor = Address::random(&env);

    let mut approvers = Vec::new(&env);
    approvers.push_back(approver);

    // First initialization should succeed
    let result1 = UpgradeableTradingContract::init(
        env.clone(),
        admin.clone(),
        approvers.clone(),
        executor.clone(),
    );
    assert!(result1.is_ok());

    // Second initialization should fail
    let result2 = UpgradeableTradingContract::init(env, admin, approvers, executor);
    assert!(result2.is_err());
}

#[test]
fn test_upgrade_proposal_creation() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);

    let admin = Address::random(&env);
    let approver = Address::random(&env);
    let executor = Address::random(&env);

    let mut approvers = Vec::new(&env);
    approvers.push_back(approver.clone());

    // Initialize contract
    let _ = UpgradeableTradingContract::init(
        env.clone(),
        admin.clone(),
        approvers.clone(),
        executor.clone(),
    );

    // Propose an upgrade
    let new_hash = symbol_short!("v2hash");
    let description = symbol_short!("Upgrade");
    let result = UpgradeableTradingContract::propose_upgrade(
        env.clone(),
        admin.clone(),
        new_hash,
        description,
        approvers.clone(),
        1,
        3600, // 1 hour timelock
    );

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1); // First proposal ID

    // Get proposal details
    let proposal = UpgradeableTradingContract::get_upgrade_proposal(env, 1);
    assert!(proposal.is_ok());
    let prop = proposal.unwrap();
    assert_eq!(prop.id, 1);
    assert_eq!(prop.approvals_count, 0);
    assert_eq!(prop.status, ProposalStatus::Pending);
}

#[test]
fn test_upgrade_proposal_approval_flow() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);

    let admin = Address::random(&env);
    let approver1 = Address::random(&env);
    let approver2 = Address::random(&env);
    let executor = Address::random(&env);

    let mut approvers = Vec::new(&env);
    approvers.push_back(approver1.clone());
    approvers.push_back(approver2.clone());

    // Initialize contract
    let _ = UpgradeableTradingContract::init(
        env.clone(),
        admin.clone(),
        approvers.clone(),
        executor.clone(),
    );

    // Propose an upgrade with 2 approvals required
    let new_hash = symbol_short!("v2hash");
    let description = symbol_short!("Upgrade");
    let proposal_id = UpgradeableTradingContract::propose_upgrade(
        env.clone(),
        admin.clone(),
        new_hash,
        description,
        approvers.clone(),
        2, // Need 2 approvals
        3600,
    )
    .unwrap();

    // First approval
    let result1 = UpgradeableTradingContract::approve_upgrade(env.clone(), proposal_id, approver1.clone());
    assert!(result1.is_ok());

    let prop = UpgradeableTradingContract::get_upgrade_proposal(env.clone(), proposal_id).unwrap();
    assert_eq!(prop.approvals_count, 1);
    assert_eq!(prop.status, ProposalStatus::Pending); // Still pending, need one more

    // Second approval
    let result2 = UpgradeableTradingContract::approve_upgrade(env.clone(), proposal_id, approver2.clone());
    assert!(result2.is_ok());

    let prop = UpgradeableTradingContract::get_upgrade_proposal(env.clone(), proposal_id).unwrap();
    assert_eq!(prop.approvals_count, 2);
    assert_eq!(prop.status, ProposalStatus::Approved); // Now approved!
}

#[test]
fn test_upgrade_timelock_enforcement() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);

    let admin = Address::random(&env);
    let approver = Address::random(&env);
    let executor = Address::random(&env);

    let mut approvers = Vec::new(&env);
    approvers.push_back(approver.clone());

    // Initialize contract
    let _ = UpgradeableTradingContract::init(
        env.clone(),
        admin.clone(),
        approvers.clone(),
        executor.clone(),
    );

    // Propose an upgrade with 4-hour timelock
    let proposal_id = UpgradeableTradingContract::propose_upgrade(
        env.clone(),
        admin.clone(),
        symbol_short!("v2hash"),
        symbol_short!("Upgrade"),
        approvers,
        1,
        14400, // 4 hours = 14400 seconds
    )
    .unwrap();

    // Approve the proposal
    let _ = UpgradeableTradingContract::approve_upgrade(env.clone(), proposal_id, approver);

    // Try to execute immediately (should fail)
    let execute_result = UpgradeableTradingContract::execute_upgrade(
        env.clone(),
        proposal_id,
        executor.clone(),
    );
    assert!(execute_result.is_err()); // Should fail - timelock not expired

    // Advance time to after timelock
    env.ledger().set_timestamp(1000 + 14401); // Past the 4-hour mark

    // Now execution should succeed
    let execute_result = UpgradeableTradingContract::execute_upgrade(
        env.clone(),
        proposal_id,
        executor,
    );
    assert!(execute_result.is_ok());

    // Verify proposal is marked as executed
    let prop = UpgradeableTradingContract::get_upgrade_proposal(env, proposal_id).unwrap();
    assert_eq!(prop.status, ProposalStatus::Executed);
    assert!(prop.executed);
}

#[test]
fn test_upgrade_rejection_flow() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);

    let admin = Address::random(&env);
    let approver = Address::random(&env);
    let executor = Address::random(&env);

    let mut approvers = Vec::new(&env);
    approvers.push_back(approver.clone());

    // Initialize contract
    let _ = UpgradeableTradingContract::init(
        env.clone(),
        admin.clone(),
        approvers.clone(),
        executor.clone(),
    );

    // Propose an upgrade
    let proposal_id = UpgradeableTradingContract::propose_upgrade(
        env.clone(),
        admin.clone(),
        symbol_short!("v2hash"),
        symbol_short!("Upgrade"),
        approvers,
        1,
        3600,
    )
    .unwrap();

    // Reject the proposal
    let result = UpgradeableTradingContract::reject_upgrade(env.clone(), proposal_id, approver);
    assert!(result.is_ok());

    // Verify status is rejected
    let prop = UpgradeableTradingContract::get_upgrade_proposal(env, proposal_id).unwrap();
    assert_eq!(prop.status, ProposalStatus::Rejected);
}

#[test]
fn test_upgrade_cancellation_by_admin() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);

    let admin = Address::random(&env);
    let approver = Address::random(&env);
    let executor = Address::random(&env);

    let mut approvers = Vec::new(&env);
    approvers.push_back(approver.clone());

    // Initialize contract
    let _ = UpgradeableTradingContract::init(
        env.clone(),
        admin.clone(),
        approvers.clone(),
        executor.clone(),
    );

    // Propose an upgrade
    let proposal_id = UpgradeableTradingContract::propose_upgrade(
        env.clone(),
        admin.clone(),
        symbol_short!("v2hash"),
        symbol_short!("Upgrade"),
        approvers,
        1,
        3600,
    )
    .unwrap();

    // Admin can cancel at any time
    let result = UpgradeableTradingContract::cancel_upgrade(env.clone(), proposal_id, admin);
    assert!(result.is_ok());

    // Verify status is cancelled
    let prop = UpgradeableTradingContract::get_upgrade_proposal(env, proposal_id).unwrap();
    assert_eq!(prop.status, ProposalStatus::Cancelled);
}

#[test]
fn test_multi_sig_protection() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);

    let admin = Address::random(&env);
    let approver1 = Address::random(&env);
    let approver2 = Address::random(&env);
    let approver3 = Address::random(&env);
    let executor = Address::random(&env);

    let mut approvers = Vec::new(&env);
    approvers.push_back(approver1.clone());
    approvers.push_back(approver2.clone());
    approvers.push_back(approver3.clone());

    // Initialize contract
    let _ = UpgradeableTradingContract::init(
        env.clone(),
        admin.clone(),
        approvers.clone(),
        executor.clone(),
    );

    // Propose with 2 of 3 multi-sig requirement
    let proposal_id = UpgradeableTradingContract::propose_upgrade(
        env.clone(),
        admin.clone(),
        symbol_short!("v2hash"),
        symbol_short!("Upgrade"),
        approvers,
        2, // 2 of 3 required
        3600,
    )
    .unwrap();

    // Get initial proposal
    let prop = UpgradeableTradingContract::get_upgrade_proposal(env.clone(), proposal_id).unwrap();
    assert_eq!(prop.approval_threshold, 2);

    // First approver approves
    let _ = UpgradeableTradingContract::approve_upgrade(env.clone(), proposal_id, approver1);
    let prop = UpgradeableTradingContract::get_upgrade_proposal(env.clone(), proposal_id).unwrap();
    assert_eq!(prop.approvals_count, 1);
    assert_eq!(prop.status, ProposalStatus::Pending); // Not enough yet

    // Second approver approves
    let _ = UpgradeableTradingContract::approve_upgrade(env.clone(), proposal_id, approver2);
    let prop = UpgradeableTradingContract::get_upgrade_proposal(env.clone(), proposal_id).unwrap();
    assert_eq!(prop.approvals_count, 2);
    assert_eq!(prop.status, ProposalStatus::Approved); // Now approved!

    // Even if third approver wanted to approve, proposal is already approved
    // This demonstrates multi-sig security: distributed decision-making
}

#[test]
fn test_duplicate_approval_prevention() {
    let env = Env::default();
    env.ledger().set_timestamp(1000);

    let admin = Address::random(&env);
    let approver = Address::random(&env);
    let executor = Address::random(&env);

    let mut approvers = Vec::new(&env);
    approvers.push_back(approver.clone());

    // Initialize contract
    let _ = UpgradeableTradingContract::init(
        env.clone(),
        admin.clone(),
        approvers,
        executor,
    );

    // Propose an upgrade
    let proposal_id = UpgradeableTradingContract::propose_upgrade(
        env.clone(),
        admin.clone(),
        symbol_short!("v2hash"),
        symbol_short!("Upgrade"),
        vec![approver.clone()],
        1,
        3600,
    )
    .unwrap();

    // First approval should succeed
    let result1 = UpgradeableTradingContract::approve_upgrade(
        env.clone(),
        proposal_id,
        approver.clone(),
    );
    assert!(result1.is_ok());

    // Second approval from same address should fail
    let result2 = UpgradeableTradingContract::approve_upgrade(
        env,
        proposal_id,
        approver,
    );
    assert!(result2.is_err()); // Cannot approve twice
}
