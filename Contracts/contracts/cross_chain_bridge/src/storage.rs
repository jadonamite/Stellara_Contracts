use soroban_sdk::{contracttype, Address, Bytes, Env, Vec};
use crate::types::{
    BridgeRequest, BridgeStats, ChainConfig, ExternalChain,
    PendingValidatorUpgrade, ValidatorSet, ValidatorSignature, WrappedAsset,
};

// ═══════════════════════════════════════════════════════════════════════════════
// ── Storage keys ─────────────────────────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    // ── Instance (cheap, static config) ──────────────────────────────────────
    Admin,
    FeeCollector,
    ValidatorSet,
    BridgeStats,
    PendingValidatorUpgrade,
    RequestCounter,
    Initialized,

    // ── Persistent (per-object) ───────────────────────────────────────────────
    BridgeRequest(u64),                          // request_id → BridgeRequest
    WrappedAsset(Address, u32),                  // (stellar_asset, chain_id) → WrappedAsset
    ChainConfig(u32),                            // chain_id → ChainConfig
    ValidatorVote(u64, Address),                 // (request_id, validator) → bool (voted?)
    ProcessedExternalTx(Bytes),                  // external_tx_hash → bool (replay guard)
    ValidatorSlash(Address),                     // validator → slash_count
    UserNonce(Address),                          // user → nonce (replay protection)
}

// ═══════════════════════════════════════════════════════════════════════════════
// ── chain_id helper ──────────────────────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

pub fn chain_to_id(chain: &ExternalChain) -> u32 {
    match chain {
        ExternalChain::Ethereum          => 1,
        ExternalChain::Polygon           => 137,
        ExternalChain::BinanceSmartChain => 56,
        ExternalChain::Avalanche         => 43114,
        ExternalChain::Arbitrum          => 42161,
        ExternalChain::Optimism          => 10,
        ExternalChain::Custom(id)        => *id,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ── BridgeStorage — static helper methods ────────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

pub struct BridgeStorage;

impl BridgeStorage {

    // ─── Initialization ──────────────────────────────────────────────────────

    pub fn is_initialized(env: &Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    pub fn set_initialized(env: &Env) {
        env.storage().instance().set(&DataKey::Initialized, &true);
    }

    // ─── Admin ───────────────────────────────────────────────────────────────

    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&DataKey::Admin, admin);
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    // ─── Fee collector ───────────────────────────────────────────────────────

    pub fn set_fee_collector(env: &Env, collector: &Address) {
        env.storage().instance().set(&DataKey::FeeCollector, collector);
    }

    pub fn get_fee_collector(env: &Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::FeeCollector)
    }

    // ─── Validator set ───────────────────────────────────────────────────────

    pub fn set_validator_set(env: &Env, set: &ValidatorSet) {
        env.storage().instance().set(&DataKey::ValidatorSet, set);
    }

    pub fn get_validator_set(env: &Env) -> Option<ValidatorSet> {
        env.storage().instance().get(&DataKey::ValidatorSet)
    }

    pub fn is_validator(env: &Env, address: &Address) -> bool {
        match Self::get_validator_set(env) {
            None => false,
            Some(set) => set.validators.iter().any(|v| v == *address),
        }
    }

    // ─── Pending validator upgrade (timelock) ─────────────────────────────────

    pub fn set_pending_upgrade(env: &Env, upgrade: &PendingValidatorUpgrade) {
        env.storage()
            .instance()
            .set(&DataKey::PendingValidatorUpgrade, upgrade);
    }

    pub fn get_pending_upgrade(env: &Env) -> Option<PendingValidatorUpgrade> {
        env.storage()
            .instance()
            .get(&DataKey::PendingValidatorUpgrade)
    }

    pub fn clear_pending_upgrade(env: &Env) {
        env.storage()
            .instance()
            .remove(&DataKey::PendingValidatorUpgrade);
    }

    // ─── Bridge stats ────────────────────────────────────────────────────────

    pub fn set_stats(env: &Env, stats: &BridgeStats) {
        env.storage().instance().set(&DataKey::BridgeStats, stats);
    }

    pub fn get_stats(env: &Env) -> BridgeStats {
        env.storage()
            .instance()
            .get(&DataKey::BridgeStats)
            .unwrap_or(BridgeStats {
                total_requests: 0,
                total_completed: 0,
                total_rejected: 0,
                total_volume: 0,
                total_fees_collected: 0,
                is_paused: false,
                pause_reason: soroban_sdk::String::from_str(env, ""),
            })
    }

    pub fn is_paused(env: &Env) -> bool {
        Self::get_stats(env).is_paused
    }

    // ─── Request counter ─────────────────────────────────────────────────────

    pub fn next_request_id(env: &Env) -> u64 {
        let current: u64 = env
            .storage()
            .instance()
            .get(&DataKey::RequestCounter)
            .unwrap_or(0u64);
        let next = current + 1;
        env.storage()
            .instance()
            .set(&DataKey::RequestCounter, &next);
        next
    }

    // ─── Bridge requests ─────────────────────────────────────────────────────

    pub fn set_request(env: &Env, request: &BridgeRequest) {
        env.storage()
            .persistent()
            .set(&DataKey::BridgeRequest(request.request_id), request);
    }

    pub fn get_request(env: &Env, request_id: u64) -> Option<BridgeRequest> {
        env.storage()
            .persistent()
            .get(&DataKey::BridgeRequest(request_id))
    }

    // ─── Wrapped assets ──────────────────────────────────────────────────────

    pub fn set_wrapped_asset(env: &Env, asset: &WrappedAsset) {
        let chain_id = chain_to_id(&asset.external_chain);
        env.storage()
            .persistent()
            .set(&DataKey::WrappedAsset(asset.stellar_asset.clone(), chain_id), asset);
    }

    pub fn get_wrapped_asset(
        env: &Env,
        stellar_asset: &Address,
        chain: &ExternalChain,
    ) -> Option<WrappedAsset> {
        let chain_id = chain_to_id(chain);
        env.storage()
            .persistent()
            .get(&DataKey::WrappedAsset(stellar_asset.clone(), chain_id))
    }

    // ─── Chain config ────────────────────────────────────────────────────────

    pub fn set_chain_config(env: &Env, config: &ChainConfig) {
        let chain_id = chain_to_id(&config.chain);
        env.storage()
            .persistent()
            .set(&DataKey::ChainConfig(chain_id), config);
    }

    pub fn get_chain_config(env: &Env, chain: &ExternalChain) -> Option<ChainConfig> {
        let chain_id = chain_to_id(chain);
        env.storage()
            .persistent()
            .get(&DataKey::ChainConfig(chain_id))
    }

    // ─── Validator votes ─────────────────────────────────────────────────────

    /// Returns true if this validator has already voted on this request
    pub fn has_voted(env: &Env, request_id: u64, validator: &Address) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::ValidatorVote(request_id, validator.clone()))
    }

    pub fn record_vote(
        env: &Env,
        request_id: u64,
        validator: &Address,
        sig: &ValidatorSignature,
    ) {
        env.storage()
            .persistent()
            .set(&DataKey::ValidatorVote(request_id, validator.clone()), sig);
    }

    pub fn get_vote(
        env: &Env,
        request_id: u64,
        validator: &Address,
    ) -> Option<ValidatorSignature> {
        env.storage()
            .persistent()
            .get(&DataKey::ValidatorVote(request_id, validator.clone()))
    }

    // ─── Replay protection ───────────────────────────────────────────────────

    pub fn is_external_tx_processed(env: &Env, tx_hash: &Bytes) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::ProcessedExternalTx(tx_hash.clone()))
    }

    pub fn mark_external_tx_processed(env: &Env, tx_hash: &Bytes) {
        env.storage()
            .persistent()
            .set(&DataKey::ProcessedExternalTx(tx_hash.clone()), &true);
    }

    // ─── Validator slash tracking ─────────────────────────────────────────────

    pub fn get_slash_count(env: &Env, validator: &Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::ValidatorSlash(validator.clone()))
            .unwrap_or(0u32)
    }

    pub fn increment_slash(env: &Env, validator: &Address) -> u32 {
        let count = Self::get_slash_count(env, validator) + 1;
        env.storage()
            .persistent()
            .set(&DataKey::ValidatorSlash(validator.clone()), &count);
        count
    }

    // ─── User nonce ──────────────────────────────────────────────────────────

    pub fn get_nonce(env: &Env, user: &Address) -> u64 {
        env.storage()
            .persistent()
            .get(&DataKey::UserNonce(user.clone()))
            .unwrap_or(0u64)
    }

    pub fn set_nonce(env: &Env, user: &Address, nonce: u64) {
        env.storage()
            .persistent()
            .set(&DataKey::UserNonce(user.clone()), &nonce);
    }

    // ─── Daily volume reset ───────────────────────────────────────────────────

    /// Check and reset the 24-hour volume window if it has expired.
    /// Returns updated config ready to be saved.
    pub fn refresh_daily_window(env: &Env, mut config: ChainConfig) -> ChainConfig {
        let now = env.ledger().timestamp();
        if now >= config.window_start + 86_400 {
            config.daily_volume = 0;
            config.window_start = now;
        }
        config
    }

    // ─── Backing ratio guard ─────────────────────────────────────────────────

    /// Returns true if the wrapped asset's backing ratio is healthy (>= 100%).
    /// locked / minted >= 1 means each wrapped token is backed by a real token.
    pub fn backing_ratio_healthy(asset: &WrappedAsset) -> bool {
        if asset.total_minted == 0 {
            return true; // no wrapped tokens → trivially healthy
        }
        // backing_ratio_bps = (total_locked * 10_000) / total_minted
        // healthy when ratio >= 10_000
        let ratio = (asset.total_locked as u128 * 10_000) / asset.total_minted as u128;
        ratio >= 10_000
    }
}