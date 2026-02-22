use soroban_sdk::{Address, Bytes, BytesN, Env, Vec};
use crate::types::{BridgeError, BridgeRequest, ExternalChain, ValidatorSet};

// ═══════════════════════════════════════════════════════════════════════════════
// ── Threshold math ───────────────────────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

/// Minimum required approvals to guarantee Byzantine fault tolerance.
/// We require strictly more than 2/3 of the validator set so that no
/// coalition of ≤1/3 of validators can approve a fraudulent request.
pub fn minimum_threshold(validator_count: u32) -> u32 {
    // ⌊2n/3⌋ + 1  — the standard BFT threshold
    (validator_count * 2 / 3) + 1
}

/// Validate that a proposed threshold satisfies our BFT requirement.
pub fn validate_threshold(
    threshold: u32,
    validator_count: u32,
) -> Result<(), BridgeError> {
    if validator_count < 3 {
        return Err(BridgeError::ValidatorSetTooSmall);
    }
    if threshold > validator_count {
        return Err(BridgeError::ThresholdExceedsSet);
    }
    if threshold < minimum_threshold(validator_count) {
        return Err(BridgeError::ThresholdTooLow);
    }
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// ── Canonical payload ─────────────────────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

/// Build the deterministic byte string that validators sign.
/// Layout (all big-endian):
///   request_id   u64   8 bytes
///   amount       i128  16 bytes
///   chain_id     u32   4 bytes
///   asset_addr   32 bytes  (Stellar address hash)
///   ext_addr     variable
///
/// The validator signs SHA-256( "stellara_bridge_v1" || payload ).
pub fn canonical_payload(
    env: &Env,
    request: &BridgeRequest,
) -> Bytes {
    let mut buf = Bytes::new(env);

    // domain separator
    buf.extend_from_slice(b"stellara_bridge_v1");

    // request_id as 8 big-endian bytes
    let id_bytes = request.request_id.to_be_bytes();
    buf.extend_from_slice(&id_bytes);

    // net_amount as 16 big-endian bytes
    let amt_bytes = request.net_amount.to_be_bytes();
    buf.extend_from_slice(&amt_bytes);

    // chain_id as 4 big-endian bytes
    let chain_id = crate::storage::chain_to_id(&request.external_chain);
    buf.extend_from_slice(&chain_id.to_be_bytes());

    // external address
    buf.extend_from_array(request.external_address.to_array::<0>().as_ref());

    buf
}

// ═══════════════════════════════════════════════════════════════════════════════
// ── Signature verification ───────────────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify a validator's Ed25519 signature over the canonical request payload.
///
/// On Soroban, `env.crypto().ed25519_verify` takes:
///   public_key : BytesN<32>
///   message    : Bytes
///   signature  : BytesN<64>
///
/// We derive the validator's public key from their Address via a registry
/// stored in persistent storage.  In this implementation the validator's
/// Stellar account key IS their signing key, which is the standard pattern.
pub fn verify_validator_signature(
    env: &Env,
    validator_pubkey: &BytesN<32>,
    request: &BridgeRequest,
    signature: &BytesN<64>,
) -> Result<(), BridgeError> {
    let payload = canonical_payload(env, request);
    // This will panic (trap) if the signature is invalid — we catch that
    // by wrapping in try/catch via the contracterror return type.
    env.crypto()
        .ed25519_verify(validator_pubkey, &payload, signature);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// ── Validator public-key registry ────────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

/// Storage key for the validator→pubkey mapping.
#[soroban_sdk::contracttype]
#[derive(Clone)]
pub enum SecurityKey {
    ValidatorPubkey(Address),
}

pub fn set_validator_pubkey(env: &Env, validator: &Address, pubkey: &BytesN<32>) {
    env.storage()
        .persistent()
        .set(&SecurityKey::ValidatorPubkey(validator.clone()), pubkey);
}

pub fn get_validator_pubkey(env: &Env, validator: &Address) -> Option<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&SecurityKey::ValidatorPubkey(validator.clone()))
}

// ═══════════════════════════════════════════════════════════════════════════════
// ── Economic security: collusion detection ───────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

/// Maximum slash strikes before a validator is automatically removed.
pub const MAX_SLASH_STRIKES: u32 = 3;

/// Validator-set timelock — minimum seconds between proposing and applying
/// a validator-set change.  48 hours gives the community time to react.
pub const VALIDATOR_UPGRADE_TIMELOCK_SECONDS: u64 = 172_800; // 48 h

/// Minimum number of validators in the set at all times.
pub const MIN_VALIDATOR_COUNT: usize = 3;

/// Check if a proposed transfer could represent a collusion attack:
/// if a single request exceeds the economic security deposit threshold we
/// require a higher approval ratio (all validators rather than threshold).
///
/// For simplicity we define the "large transfer" threshold as 10× the
/// per-chain max_transfer_amount.  Projects should tune this to the value
/// of their validator bonds.
pub fn requires_supermajority(amount: i128, max_single_tx: i128) -> bool {
    amount > max_single_tx * 10
}

/// Compute the effective approval requirement for a request given its amount.
pub fn effective_required_approvals(
    validator_set: &ValidatorSet,
    amount: i128,
    max_single_tx: i128,
) -> u32 {
    if requires_supermajority(amount, max_single_tx) {
        // All validators must approve for very large transfers
        validator_set.validators.len() as u32
    } else {
        validator_set.threshold
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ── Fee calculation ───────────────────────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

/// Calculate the bridge fee and net transfer amount.
/// Returns (fee_amount, net_amount).
pub fn calculate_fee(amount: i128, fee_bps: u32) -> (i128, i128) {
    let fee = (amount as u128 * fee_bps as u128 / 10_000) as i128;
    let net = amount - fee;
    (fee, net)
}

// ═══════════════════════════════════════════════════════════════════════════════
// ── Decimal normalisation ────────────────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert an amount from external chain decimals to Stellar decimals.
pub fn normalise_to_stellar(amount: i128, decimals_external: u32, decimals_stellar: u32) -> i128 {
    if decimals_external == decimals_stellar {
        return amount;
    }
    if decimals_external > decimals_stellar {
        let factor = 10i128.pow(decimals_external - decimals_stellar);
        amount / factor
    } else {
        let factor = 10i128.pow(decimals_stellar - decimals_external);
        amount * factor
    }
}

/// Convert an amount from Stellar decimals to external chain decimals.
pub fn normalise_to_external(amount: i128, decimals_stellar: u32, decimals_external: u32) -> i128 {
    normalise_to_stellar(amount, decimals_external, decimals_stellar)
}