use soroban_sdk::{contracttype, contracterror, Address, Bytes, BytesN};

// ═══════════════════════════════════════════════════════════════════════════════
// ── Enumerations ─────────────────────────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

/// Direction of the bridge operation from Stellar's perspective
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BridgeDirection {
    /// User locks tokens on Stellar → they get minted on the external chain
    OutboundToExternal,
    /// User locks tokens on external chain → wrapped tokens minted on Stellar
    InboundFromExternal,
}

/// Lifecycle state of a single bridge request
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BridgeRequestStatus {
    Pending,    // waiting for validator threshold
    Approved,   // threshold reached, executing transfer
    Completed,  // fully settled
    Rejected,   // validators rejected it
    Cancelled,  // initiator cancelled before approval
    Expired,    // not approved within the time window
}

/// Supported external chains — extensible via Custom(u32)
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExternalChain {
    Ethereum,
    Polygon,
    BinanceSmartChain,
    Avalanche,
    Arbitrum,
    Optimism,
    Custom(u32),
}

// ═══════════════════════════════════════════════════════════════════════════════
// ── Core structs ─────────────────────────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

/// A bridge request, created when a user initiates a cross-chain transfer
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BridgeRequest {
    pub request_id: u64,
    pub direction: BridgeDirection,
    pub initiator: Address,

    // Asset
    pub stellar_asset: Address,
    pub amount: i128,
    pub fee_amount: i128,
    pub net_amount: i128,

    // External chain
    pub external_chain: ExternalChain,
    pub external_address: Bytes,        // raw bytes — supports any address format
    pub external_tx_hash: Bytes,        // proof of lock on external chain (inbound)

    // Lifecycle
    pub status: BridgeRequestStatus,
    pub created_at: u64,
    pub expires_at: u64,
    pub completed_at: u64,             // 0 if not completed

    // Multi-sig tally
    pub approval_count: u32,
    pub rejection_count: u32,
    pub required_approvals: u32,
}

/// A wrapped asset — one record per (stellar_asset × external_chain) pair
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WrappedAsset {
    pub stellar_asset: Address,
    pub external_chain: ExternalChain,
    pub external_contract: Bytes,       // contract / program address on external chain
    pub decimals_stellar: u32,
    pub decimals_external: u32,
    pub total_locked: i128,             // locked on Stellar for outbound backing
    pub total_minted: i128,             // wrapped tokens minted on Stellar (inbound)
    pub is_active: bool,
    pub backing_ratio_bps: u32,         // must always be ≥ 10 000 (100%)
    pub registered_at: u64,
    pub registered_by: Address,
}

/// A single validator's vote on a bridge request
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatorSignature {
    pub validator: Address,
    pub request_id: u64,
    pub approved: bool,
    pub signed_at: u64,
    pub signature: BytesN<64>,          // Ed25519 signature over the canonical payload
}

/// Current validator set — version-stamped so old sigs can be rejected
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatorSet {
    pub validators: soroban_sdk::Vec<Address>,
    pub threshold: u32,                 // must be > ⌊2/3 × len⌋
    pub version: u32,
    pub updated_at: u64,
}

/// Per-chain operational configuration
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChainConfig {
    pub chain: ExternalChain,
    pub is_active: bool,
    pub min_confirmations: u32,
    pub max_transfer_amount: i128,      // single-tx circuit breaker
    pub daily_limit: i128,              // rolling 24-hour cap
    pub daily_volume: i128,             // volume in current window
    pub window_start: u64,              // when the current 24 h window opened
    pub fee_bps: u32,                   // bridge fee in basis points
    pub expiry_seconds: u64,            // request lifetime before auto-expiry
}

/// Global bridge statistics — stored in instance storage
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BridgeStats {
    pub total_requests: u64,
    pub total_completed: u64,
    pub total_rejected: u64,
    pub total_volume: i128,
    pub total_fees_collected: i128,
    pub is_paused: bool,
    pub pause_reason: soroban_sdk::String,
}

/// Pending validator-set upgrade proposed by admin (time-locked)
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingValidatorUpgrade {
    pub proposed_validators: soroban_sdk::Vec<Address>,
    pub proposed_threshold: u32,
    pub proposed_at: u64,
    pub effective_at: u64,              // earliest time it can be applied
    pub proposer: Address,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ── Events ───────────────────────────────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

#[contracttype]
#[derive(Clone, Debug)]
pub struct BridgeInitiatedEvent {
    pub request_id: u64,
    pub direction: BridgeDirection,
    pub initiator: Address,
    pub stellar_asset: Address,
    pub amount: i128,
    pub fee_amount: i128,
    pub external_chain: ExternalChain,
    pub external_address: Bytes,
    pub expires_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidatorVoteEvent {
    pub request_id: u64,
    pub validator: Address,
    pub approved: bool,
    pub approval_count: u32,
    pub rejection_count: u32,
    pub required_approvals: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BridgeCompletedEvent {
    pub request_id: u64,
    pub direction: BridgeDirection,
    pub initiator: Address,
    pub stellar_asset: Address,
    pub net_amount: i128,
    pub fee_amount: i128,
    pub completed_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct BridgeRejectedEvent {
    pub request_id: u64,
    pub rejection_count: u32,
    pub rejected_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AssetRegisteredEvent {
    pub stellar_asset: Address,
    pub external_chain: ExternalChain,
    pub external_contract: Bytes,
    pub registered_by: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidatorAddedEvent {
    pub validator: Address,
    pub new_threshold: u32,
    pub validator_set_version: u32,
    pub added_by: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidatorRemovedEvent {
    pub validator: Address,
    pub new_threshold: u32,
    pub validator_set_version: u32,
    pub removed_by: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct EmergencyPauseEvent {
    pub paused: bool,
    pub reason: soroban_sdk::String,
    pub triggered_by: Address,
    pub triggered_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidatorUpgradeProposedEvent {
    pub proposed_by: Address,
    pub effective_at: u64,
    pub validator_count: u32,
    pub threshold: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidatorUpgradeAppliedEvent {
    pub applied_by: Address,
    pub new_version: u32,
    pub validator_count: u32,
    pub threshold: u32,
    pub applied_at: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ── Errors ───────────────────────────────────────────────────────────────────
// ═══════════════════════════════════════════════════════════════════════════════

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum BridgeError {
    // Auth
    Unauthorized                = 5001,
    NotValidator                = 5002,
    NotAdmin                    = 5003,

    // Initialisation
    AlreadyInitialized          = 5010,
    NotInitialized              = 5011,

    // Request lifecycle
    RequestNotFound             = 5020,
    RequestAlreadyProcessed     = 5021,
    RequestExpired              = 5022,
    RequestNotPending           = 5023,
    AlreadyVoted                = 5024,

    // Asset
    AssetNotRegistered          = 5030,
    AssetAlreadyRegistered      = 5031,
    AssetInactive               = 5032,
    BackingRatioBroken          = 5033,

    // Chain
    ChainNotSupported           = 5040,
    ChainInactive               = 5041,

    // Amounts & limits
    AmountTooSmall              = 5050,
    AmountExceedsMax            = 5051,
    DailyLimitExceeded          = 5052,
    InsufficientBalance         = 5053,
    InvalidFee                  = 5054,

    // Validator set
    ValidatorAlreadyExists      = 5060,
    ValidatorNotFound           = 5061,
    ThresholdTooLow             = 5062,      // must be > 2/3 of set
    ThresholdExceedsSet         = 5063,
    ValidatorSetTooSmall        = 5064,      // minimum 3 validators
    NoPendingUpgrade            = 5065,
    UpgradeTimelockActive       = 5066,

    // Bridge state
    BridgePaused                = 5070,
    InvalidSignature            = 5071,
    DuplicateExternalTx         = 5072,
    InvalidNonce                = 5073,
}