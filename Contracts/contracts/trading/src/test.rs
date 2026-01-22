#![cfg(test)]

use super::*;
use soroban_sdk::{Env, testutils::Address as _, token};

// We need to import the social rewards contract for testing
// In a workspace, we can register the contract by its WASM, but here we can just register the struct if it's available.
// Since it's a separate crate, we can't easily import the struct without adding it to dependencies.
// However, for unit testing within `trading` crate, we can Mock the reward contract or use `register_contract_wasm` if we had the wasm.
// 
// Alternatively, we can define a mock contract HERE in the test module.

#[contract]
pub struct MockRewardContract;

#[contractimpl]
impl MockRewardContract {
    pub fn add_reward(env: Env, user: Address, amount: i128) {
        if amount <= 0 {
            panic!("Invalid reward amount");
        }
        // Success
    }
}

#[test]
fn test_trade_fee_collection() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TradingContract);
    let client = TradingContractClient::new(&env, &contract_id);

    // Create a token for fees
    let issuer = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract(issuer);
    let token_client = token::Client::new(&env, &token_contract_id);
    let token_admin_client = token::StellarAssetClient::new(&env, &token_contract_id);

    let trader = Address::generate(&env);
    let recipient = Address::generate(&env);
    let fee_amount = 100;

    // Mint tokens to trader
    token_admin_client.mint(&trader, &1000);

    // Perform trade with sufficient balance
    let res = client.trade(&trader, &token_contract_id, &fee_amount, &recipient);
    assert!(res.is_ok());

    // Verify fee transfer
    assert_eq!(token_client.balance(&trader), 900);
    assert_eq!(token_client.balance(&recipient), 100);
}

#[test]
fn test_trade_insufficient_fee() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, TradingContract);
    let client = TradingContractClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract(issuer);
    // No minting, balance is 0

    let trader = Address::generate(&env);
    let recipient = Address::generate(&env);
    let fee_amount = 100;

    // Perform trade with insufficient balance
    // use try_trade to catch error
    let res = client.try_trade(&trader, &token_contract_id, &fee_amount, &recipient);
    
    assert!(res.is_err());
    
    match res {
        Err(Ok(err)) => {
            // Check if it is the expected contract error
            assert_eq!(err, FeeError::InsufficientBalance);
        },
        _ => panic!("Expected contract error"),
    }
}

#[test]
fn test_trade_and_reward_success() {
    let env = Env::default();
    env.mock_all_auths();

    let trading_id = env.register_contract(None, TradingContract);
    let trading_client = TradingContractClient::new(&env, &trading_id);

    let reward_id = env.register_contract(None, MockRewardContract);
    
    // Setup Tokens
    let issuer = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(issuer);
    let token_client = token::Client::new(&env, &token_id);
    let token_admin = token::StellarAssetClient::new(&env, &token_id);

    let trader = Address::generate(&env);
    let recipient = Address::generate(&env);
    let fee = 100;

    token_admin.mint(&trader, &1000);

    // Run trade_and_reward
    let res = trading_client.trade_and_reward(
        &trader, 
        &token_id, 
        &fee, 
        &recipient, 
        &reward_id, 
        &50 // valid reward
    );

    assert!(res.is_ok());

    // Verify fee was paid
    assert_eq!(token_client.balance(&trader), 900);
    assert_eq!(token_client.balance(&recipient), 100);
}

#[test]
fn test_trade_and_reward_atomic_rollback() {
    let env = Env::default();
    env.mock_all_auths();

    let trading_id = env.register_contract(None, TradingContract);
    let trading_client = TradingContractClient::new(&env, &trading_id);

    let reward_id = env.register_contract(None, MockRewardContract);
    
    // Setup Tokens
    let issuer = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract(issuer);
    let token_client = token::Client::new(&env, &token_id);
    let token_admin = token::StellarAssetClient::new(&env, &token_id);

    let trader = Address::generate(&env);
    let recipient = Address::generate(&env);
    let fee = 100;

    token_admin.mint(&trader, &1000);

    // Run trade_and_reward with INVALID reward amount (0)
    // This will cause MockRewardContract to panic.
    // TradingContract::trade_and_reward uses safe_invoke, which catches the panic
    // and returns SafeCallErrors::CALL_FAILED (mapped to u32).
    // The TradingContract then returns Err.
    // The ENV should revert all changes, including the fee payment.
    
    // Note: We use try_trade_and_reward to inspect the error result
    let res = trading_client.try_trade_and_reward(
        &trader, 
        &token_id, 
        &fee, 
        &recipient, 
        &reward_id, 
        &0 // Invalid reward amount -> panic
    );

    // The result should be an Err
    assert!(res.is_err());
    
    // Check error code
    match res {
        Err(Ok(code)) => {
             // We expect CALL_FAILED (2001)
             assert_eq!(code, SafeCallErrors::CALL_FAILED);
        },
        _ => panic!("Expected contract error code"),
    }

    // CRITICAL: Verify ATOMICITY
    // The fee transfer (which happened before the cross-call) must be rolled back.
    // Trader balance should still be 1000.
    assert_eq!(token_client.balance(&trader), 1000);
    assert_eq!(token_client.balance(&recipient), 0);
}
