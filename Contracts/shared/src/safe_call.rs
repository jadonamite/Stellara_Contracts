use soroban_sdk::{Address, Env, Symbol, Val, Vec, Error};

pub mod errors {
    pub const CALL_FAILED: u32 = 2001;
    pub const CONTRACT_NOT_FOUND: u32 = 2002;
}

/// Safely invokes a contract method with error handling checks.
/// 
/// # Arguments
/// * `env` - The environment
/// * `contract` - The address of the contract to call
/// * `func` - The function name to call
/// * `args` - The arguments to pass
/// 
/// # Returns
/// * `Result<Val, u32>` - The return value or an error code
pub fn safe_invoke(
    env: &Env,
    contract: &Address,
    func: &Symbol,
    args: Vec<Val>,
) -> Result<Val, u32> {
    // 1. Defensive Check: Ensure contract exists/is deployed?
    // Soroban doesn't have a direct "exists" check on Address easily accessible without trying to call 
    // or checking ledger entries, but try_call handles non-existence as an error.

    // 2. Try Call
    // invoke_contract_try returns Result<Val, Error>
    // We map generic errors to our specific codes if needed, or propagate.
    
    // Explicitly type the result to see what compiler thinks
    let res: Result<Val, Error> = env.try_invoke_contract(contract, func, args);

    match res {
        Ok(val) => Ok(val),
        Err(e) => {
            // Log the error for debugging
            // env.events().publish((Symbol::new(env, "call_failed"),), e);
            
            // In a real module we might inspect `e` to see if it's a missing contract vs logic error.
            // For now, we wrap it.
            Err(errors::CALL_FAILED)
        }
    }
}

/// Verifies a contract address is valid (basic check).
pub fn verify_target(_env: &Env, _contract: &Address) -> bool {
    // This is a placeholder. In Soroban, an Address is just a handle.
    // We could check if it's a contract address vs account address if needed.
    true
}
