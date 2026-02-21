use soroban_sdk::{
    Address, Env, Map, Symbol, Val, Vec, IntoVal,
};

<<<<<<< Updated upstream
/// Standardized event emitter utility
=======
use soroban_sdk::{contracttype, Address, Symbol};

// =============================================================================
// Event Topics (standardized event names)
// =============================================================================

/// Standard event topic names for consistent indexing
pub mod topics {
    use soroban_sdk::{symbol_short, Symbol};

    // Trading events
    pub const TRADE_EXECUTED: Symbol = symbol_short!("trade");
    pub const CONTRACT_PAUSED: Symbol = symbol_short!("paused");
    pub const CONTRACT_UNPAUSED: Symbol = symbol_short!("unpause");
    pub const FEE_COLLECTED: Symbol = symbol_short!("fee");

    // Token events
    pub const TRANSFER: Symbol = symbol_short!("transfer");
    pub const APPROVAL: Symbol = symbol_short!("approve");
    pub const MINT: Symbol = symbol_short!("mint");
    pub const BURN: Symbol = symbol_short!("burn");

    // Governance events
    pub const PROPOSAL_CREATED: Symbol = symbol_short!("propose");
    pub const PROPOSAL_APPROVED: Symbol = symbol_short!("approve");
    pub const PROPOSAL_REJECTED: Symbol = symbol_short!("reject");
    pub const PROPOSAL_EXECUTED: Symbol = symbol_short!("execute");
    pub const PROPOSAL_CANCELLED: Symbol = symbol_short!("cancel");
    pub const VOTE_CAST: Symbol = symbol_short!("vote");
    pub const EMERGENCY_ACTION: Symbol = symbol_short!("emergency");

    // Staking events
    pub const STAKE: Symbol = symbol_short!("stake");
    pub const UNSTAKE: Symbol = symbol_short!("unstake");
    pub const REWARD_CLAIMED: Symbol = symbol_short!("claimed");
    pub const SLASHED: Symbol = symbol_short!("slashed");

    // Social rewards events
    pub const REWARD_ADDED: Symbol = symbol_short!("reward");
    pub const REWARD_CLAIMED_SOCIAL: Symbol = symbol_short!("claimed");

    // Vesting/Academy events
    pub const GRANT_CREATED: Symbol = symbol_short!("grant");
    pub const GRANT_CLAIMED: Symbol = symbol_short!("claim");
    pub const GRANT_REVOKED: Symbol = symbol_short!("revoke");

    // Privacy events
    pub const SHIELD: Symbol = symbol_short!("shield");
    pub const UNSHIELD: Symbol = symbol_short!("unshield");
    pub const PRIVATE_TRANSFER: Symbol = symbol_short!("private_transfer");

    // Yield farming events
    pub const LIQUIDITY_ADDED: Symbol = symbol_short!("liquidity_added");
    pub const LIQUIDITY_REMOVED: Symbol = symbol_short!("liquidity_removed");
    pub const HARVEST: Symbol = symbol_short!("harvest");

    // Disaster recovery events
    pub const EMERGENCY_PAUSE: Symbol = symbol_short!("emergency_pause");
    pub const RECOVERY_INITIATED: Symbol = symbol_short!("recovery_started");
    pub const RECOVERY_COMPLETED: Symbol = symbol_short!("recovery_completed");
}

// =============================================================================
// Token Events
// =============================================================================

/// Event emitted when tokens are transferred
#[contracttype]
#[derive(Clone, Debug)]
pub struct TransferEvent {
    /// Address sending the tokens
    pub from: Address,
    /// Address receiving the tokens
    pub to: Address,
    /// Amount of tokens transferred
    pub amount: i128,
    /// Token contract address
    pub token: Address,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when an approval is set
#[contracttype]
#[derive(Clone, Debug)]
pub struct ApprovalEvent {
    /// Owner of the tokens
    pub owner: Address,
    /// Address approved to spend
    pub spender: Address,
    /// Amount approved
    pub amount: i128,
    /// Token contract address
    pub token: Address,
    /// Expiration ledger number
    pub expiration_ledger: u32,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when tokens are minted
#[contracttype]
#[derive(Clone, Debug)]
pub struct MintEvent {
    /// Address receiving minted tokens
    pub to: Address,
    /// Amount of tokens minted
    pub amount: i128,
    /// Token contract address
    pub token: Address,
    /// Total supply after minting
    pub total_supply: i128,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when tokens are burned
#[contracttype]
#[derive(Clone, Debug)]
pub struct BurnEvent {
    /// Address whose tokens are burned
    pub from: Address,
    /// Amount of tokens burned
    pub amount: i128,
    /// Token contract address
    pub token: Address,
    /// Total supply after burning
    pub total_supply: i128,
    /// Block timestamp
    pub timestamp: u64,
}

// =============================================================================
// Staking Events
// =============================================================================

/// Event emitted when tokens are staked
#[contracttype]
#[derive(Clone, Debug)]
pub struct StakeEvent {
    /// Address staking tokens
    pub user: Address,
    /// Amount of tokens staked
    pub amount: i128,
    /// Staking contract address
    pub staking_contract: Address,
    /// Lock period in seconds
    pub lock_period: u64,
    /// Reward multiplier
    pub reward_multiplier: u32,
    /// Total staked amount after this stake
    pub total_staked: i128,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when tokens are unstaked
#[contracttype]
#[derive(Clone, Debug)]
pub struct UnstakeEvent {
    /// Address unstaking tokens
    pub user: Address,
    /// Amount of tokens unstaked
    pub amount: i128,
    /// Staking contract address
    pub staking_contract: Address,
    /// Rewards earned
    pub rewards_earned: i128,
    /// Penalty applied (if any)
    pub penalty: i128,
    /// Total staked amount after this unstake
    pub total_staked: i128,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when staking rewards are claimed
#[contracttype]
#[derive(Clone, Debug)]
pub struct StakingRewardClaimedEvent {
    /// Address claiming rewards
    pub user: Address,
    /// Amount of rewards claimed
    pub amount: i128,
    /// Staking contract address
    pub staking_contract: Address,
    /// Reward token address
    pub reward_token: Address,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when staked tokens are slashed
#[contracttype]
#[derive(Clone, Debug)]
pub struct SlashedEvent {
    /// Address whose tokens are slashed
    pub user: Address,
    /// Amount of tokens slashed
    pub amount: i128,
    /// Staking contract address
    pub staking_contract: Address,
    /// Reason for slashing
    pub reason: Symbol,
    /// Admin who initiated slashing
    pub slashed_by: Address,
    /// Block timestamp
    pub timestamp: u64,
}

// =============================================================================
// Vesting/Academy Events
// =============================================================================

/// Event emitted when a vesting grant is created
#[contracttype]
#[derive(Clone, Debug)]
pub struct GrantCreatedEvent {
    /// Unique grant identifier
    pub grant_id: u64,
    /// Beneficiary address
    pub beneficiary: Address,
    /// Total amount of tokens in grant
    pub amount: i128,
    /// Start time of vesting
    pub start_time: u64,
    /// Cliff period (seconds before any tokens unlock)
    pub cliff: u64,
    /// Total vesting duration (seconds)
    pub duration: u64,
    /// Admin who created the grant
    pub created_by: Address,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when vesting is claimed
#[contracttype]
#[derive(Clone, Debug)]
pub struct VestingClaimedEvent {
    /// Grant identifier
    pub grant_id: u64,
    /// Beneficiary address
    pub beneficiary: Address,
    /// Amount claimed
    pub amount: i128,
    /// Total amount claimed so far
    pub total_claimed: i128,
    /// Remaining amount to claim
    pub remaining_amount: i128,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when a vesting grant is revoked
#[contracttype]
#[derive(Clone, Debug)]
pub struct GrantRevokedEvent {
    /// Grant identifier
    pub grant_id: u64,
    /// Beneficiary address
    pub beneficiary: Address,
    /// Amount revoked (returned to admin)
    pub amount_revoked: i128,
    /// Amount already claimed by beneficiary
    pub amount_claimed: i128,
    /// Admin who revoked the grant
    pub revoked_by: Address,
    /// Reason for revocation
    pub reason: Symbol,
    /// Block timestamp
    pub timestamp: u64,
}

// =============================================================================
// Privacy Events
// =============================================================================

/// Event emitted when tokens are shielded (made private)
#[contracttype]
#[derive(Clone, Debug)]
pub struct ShieldEvent {
    /// Address shielding tokens
    pub user: Address,
    /// Amount of tokens shielded
    pub amount: i128,
    /// Public token contract address
    pub public_token: Address,
    /// Privacy contract address
    pub privacy_contract: Address,
    /// Shielded note identifier
    pub note_id: Symbol,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when tokens are unshielded (made public)
#[contracttype]
#[derive(Clone, Debug)]
pub struct UnshieldEvent {
    /// Address receiving unshielded tokens
    pub user: Address,
    /// Amount of tokens unshielded
    pub amount: i128,
    /// Public token contract address
    pub public_token: Address,
    /// Privacy contract address
    pub privacy_contract: Address,
    /// Shielded note identifier that was consumed
    pub note_id: Symbol,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when a private transfer occurs
#[contracttype]
#[derive(Clone, Debug)]
pub struct PrivateTransferEvent {
    /// Address sending private tokens
    pub from: Address,
    /// Address receiving private tokens (encrypted)
    pub to: Address, // In practice, this might be encrypted
    /// Amount transferred
    pub amount: i128,
    /// Privacy contract address
    pub privacy_contract: Address,
    /// Input note identifiers consumed
    pub input_notes: Vec<Symbol>,
    /// Output note identifiers created
    pub output_notes: Vec<Symbol>,
    /// Block timestamp
    pub timestamp: u64,
}

// =============================================================================
// Yield Farming Events
// =============================================================================

/// Event emitted when liquidity is added to a farming pool
#[contracttype]
#[derive(Clone, Debug)]
pub struct LiquidityAddedEvent {
    /// Address providing liquidity
    pub user: Address,
    /// Pool identifier
    pub pool_id: u64,
    /// Amount of token A provided
    pub amount_a: i128,
    /// Amount of token B provided
    pub amount_b: i128,
    /// LP tokens received
    pub lp_tokens: i128,
    /// Farming contract address
    pub farming_contract: Address,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when liquidity is removed from a farming pool
#[contracttype]
#[derive(Clone, Debug)]
pub struct LiquidityRemovedEvent {
    /// Address removing liquidity
    pub user: Address,
    /// Pool identifier
    pub pool_id: u64,
    /// Amount of token A received
    pub amount_a: i128,
    /// Amount of token B received
    pub amount_b: i128,
    /// LP tokens burned
    pub lp_tokens: i128,
    /// Farming contract address
    pub farming_contract: Address,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when farming rewards are harvested
#[contracttype]
#[derive(Clone, Debug)]
pub struct HarvestEvent {
    /// Address harvesting rewards
    pub user: Address,
    /// Pool identifier
    pub pool_id: u64,
    /// Amount of rewards claimed
    pub reward_amount: i128,
    /// Reward token address
    pub reward_token: Address,
    /// Farming contract address
    pub farming_contract: Address,
    /// Block timestamp
    pub timestamp: u64,
}

// =============================================================================
// Disaster Recovery Events
// =============================================================================

/// Event emitted when emergency pause is triggered
#[contracttype]
#[derive(Clone, Debug)]
pub struct EmergencyPauseEvent {
    /// Contract being paused
    pub contract_address: Address,
    /// Authority who triggered the pause
    pub paused_by: Address,
    /// Reason for emergency pause
    pub reason: Symbol,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when recovery process is initiated
#[contracttype]
#[derive(Clone, Debug)]
pub struct RecoveryInitiatedEvent {
    /// Contract being recovered
    pub contract_address: Address,
    /// Recovery identifier
    pub recovery_id: u64,
    /// Authority who initiated recovery
    pub initiated_by: Address,
    /// Type of recovery (backup, migration, etc.)
    pub recovery_type: Symbol,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when recovery process is completed
#[contracttype]
#[derive(Clone, Debug)]
pub struct RecoveryCompletedEvent {
    /// Contract that was recovered
    pub contract_address: Address,
    /// Recovery identifier
    pub recovery_id: u64,
    /// Authority who completed recovery
    pub completed_by: Address,
    /// Success status
    pub success: bool,
    /// New contract state hash (if applicable)
    pub new_state_hash: Symbol,
    /// Block timestamp
    pub timestamp: u64,
}

// =============================================================================
// Additional Governance Events
// =============================================================================

/// Event emitted when a vote is cast
#[contracttype]
#[derive(Clone, Debug)]
pub struct VoteCastEvent {
    /// Proposal identifier
    pub proposal_id: u64,
    /// Address casting the vote
    pub voter: Address,
    /// Vote type (for, against, abstain)
    pub vote_type: Symbol,
    /// Voting power used
    pub voting_power: u128,
    /// Reason for vote (optional)
    pub reason: Symbol,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when emergency action is taken
#[contracttype]
#[derive(Clone, Debug)]
pub struct EmergencyActionEvent {
    /// Unique action identifier
    pub action_id: u64,
    /// Emergency council member who took action
    pub council_member: Address,
    /// Type of emergency action
    pub action_type: Symbol,
    /// Target contract address
    pub target_contract: Address,
    /// Action description
    pub description: Symbol,
    /// Block timestamp
    pub timestamp: u64,
}

// =============================================================================
// Trading Events
// =============================================================================

/// Event emitted when a trade is executed
#[contracttype]
#[derive(Clone, Debug)]
pub struct TradeExecutedEvent {
    /// Unique trade identifier
    pub trade_id: u64,
    /// Address of the trader
    pub trader: Address,
    /// Trading pair symbol (e.g., "XLMUSDC")
    pub pair: Symbol,
    /// Trade amount
    pub amount: i128,
    /// Trade price
    pub price: i128,
    /// Whether this is a buy (true) or sell (false)
    pub is_buy: bool,
    /// Fee amount collected
    pub fee_amount: i128,
    /// Token used for fee payment
    pub fee_token: Address,
    /// Block timestamp when trade occurred
    pub timestamp: u64,
}

/// Event emitted when contract is paused
#[contracttype]
#[derive(Clone, Debug)]
pub struct ContractPausedEvent {
    /// Admin who paused the contract
    pub paused_by: Address,
    /// Block timestamp when paused
    pub timestamp: u64,
}

/// Event emitted when contract is unpaused
#[contracttype]
#[derive(Clone, Debug)]
pub struct ContractUnpausedEvent {
    /// Admin who unpaused the contract
    pub unpaused_by: Address,
    /// Block timestamp when unpaused
    pub timestamp: u64,
}

/// Event emitted when a fee is collected
#[contracttype]
#[derive(Clone, Debug)]
pub struct FeeCollectedEvent {
    /// Address paying the fee
    pub payer: Address,
    /// Address receiving the fee
    pub recipient: Address,
    /// Fee amount
    pub amount: i128,
    /// Token used for payment
    pub token: Address,
    /// Block timestamp
    pub timestamp: u64,
}

// =============================================================================
// Governance Events
// =============================================================================

/// Event emitted when an upgrade proposal is created
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProposalCreatedEvent {
    /// Unique proposal identifier
    pub proposal_id: u64,
    /// Address that created the proposal
    pub proposer: Address,
    /// Hash of the new contract to upgrade to
    pub new_contract_hash: Symbol,
    /// Contract being upgraded
    pub target_contract: Address,
    /// Description of the proposal
    pub description: Symbol,
    /// Required approvals for execution
    pub approval_threshold: u32,
    /// Timelock delay before execution (seconds)
    pub timelock_delay: u64,
    /// Block timestamp when created
    pub timestamp: u64,
}

/// Event emitted when a proposal is approved
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProposalApprovedEvent {
    /// Proposal identifier
    pub proposal_id: u64,
    /// Address that approved
    pub approver: Address,
    /// Current approval count after this approval
    pub current_approvals: u32,
    /// Required approvals for execution
    pub threshold: u32,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when a proposal is rejected
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProposalRejectedEvent {
    /// Proposal identifier
    pub proposal_id: u64,
    /// Address that rejected
    pub rejector: Address,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when a proposal is executed
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProposalExecutedEvent {
    /// Proposal identifier
    pub proposal_id: u64,
    /// Address that executed
    pub executor: Address,
    /// New contract hash that was deployed
    pub new_contract_hash: Symbol,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when a proposal is cancelled
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProposalCancelledEvent {
    /// Proposal identifier
    pub proposal_id: u64,
    /// Admin who cancelled
    pub cancelled_by: Address,
    /// Block timestamp
    pub timestamp: u64,
}

// =============================================================================
// Social Rewards Events
// =============================================================================

/// Event emitted when a reward is added/granted to a user
#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardAddedEvent {
    /// Unique reward identifier
    pub reward_id: u64,
    /// User receiving the reward
    pub user: Address,
    /// Reward amount
    pub amount: i128,
    /// Type of reward (e.g., "referral", "engagement", "achievement")
    pub reward_type: Symbol,
    /// Optional metadata/reason for the reward
    pub reason: Symbol,
    /// Admin who granted the reward
    pub granted_by: Address,
    /// Block timestamp
    pub timestamp: u64,
}

/// Event emitted when a reward is claimed
#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardClaimedEvent {
    /// Reward identifier
    pub reward_id: u64,
    /// User who claimed
    pub user: Address,
    /// Amount claimed
    pub amount: i128,
    /// Block timestamp
    pub timestamp: u64,
}

// =============================================================================
// Event Emission Helpers
// =============================================================================

use soroban_sdk::Env;

/// Helper trait for emitting standardized events
>>>>>>> Stashed changes
pub struct EventEmitter;

impl EventEmitter {
    pub const CURRENT_VERSION: u32 = 1;

    /// Emit a standardized event
    pub fn emit_standard(
        env: &Env,
        event_type: Symbol,
        user_address: Address,
        data: Vec<Val>,
        metadata: Map<Symbol, Vec<Val>>,
    ) {
        // Publish standardized event with structured data
        env.events().publish(
            (Symbol::new(env, "stellara_event"), event_type.clone()),
            (
                env.current_contract_address(),
                user_address,
                data,
                metadata,
                env.ledger().timestamp(),
                Self::CURRENT_VERSION,
            ),
        );
    }

    /// Emit a transfer event using standardized format
    pub fn transfer(env: &Env, from: Address, to: Address, amount: i128, token: Address) {
        let mut data = Vec::new(env);
        data.push_back(amount.into_val(env));
        data.push_back(token.into_val(env));

        let mut metadata = Map::new(env);
        metadata.set(Symbol::new(env, "amount"), Vec::from_array(env, [amount.into_val(env)]));
        metadata.set(Symbol::new(env, "from"), Vec::from_array(env, [from.into_val(env)]));
        metadata.set(Symbol::new(env, "to"), Vec::from_array(env, [to.into_val(env)]));
        metadata.set(Symbol::new(env, "token"), Vec::from_array(env, [token.into_val(env)]));

        Self::emit_standard(env, Symbol::new(env, "transfer"), from.clone(), data, metadata);
        
        // Also emit legacy event for backward compatibility
        env.events().publish(
            (Symbol::new(env, "transfer"), from.clone(), to),
            amount,
        );
    }

    /// Emit an approval event using standardized format
    pub fn approve(env: &Env, from: Address, spender: Address, amount: i128, token: Address) {
        let mut data = Vec::new(env);
        data.push_back(amount.into_val(env));
        data.push_back(token.into_val(env));

        let mut metadata = Map::new(env);
        metadata.set(Symbol::new(env, "amount"), Vec::from_array(env, [amount.into_val(env)]));
        metadata.set(Symbol::new(env, "from"), Vec::from_array(env, [from.into_val(env)]));
        metadata.set(Symbol::new(env, "to"), Vec::from_array(env, [spender.into_val(env)]));
        metadata.set(Symbol::new(env, "token"), Vec::from_array(env, [token.into_val(env)]));

        Self::emit_standard(env, Symbol::new(env, "approve"), from.clone(), data, metadata);
        
        // Also emit legacy event for backward compatibility
        env.events().publish(
            (Symbol::new(env, "approve"), from.clone(), spender),
            amount,
        );
    }

    /// Emit a mint event using standardized format
    pub fn mint(env: &Env, to: Address, amount: i128, token: Address, reason: Option<String>) {
        let mut data = Vec::new(env);
        data.push_back(amount.into_val(env));
        data.push_back(token.into_val(env));

        let mut metadata = Map::new(env);
        metadata.set(Symbol::new(env, "amount"), Vec::from_array(env, [amount.into_val(env)]));
        metadata.set(Symbol::new(env, "to"), Vec::from_array(env, [to.into_val(env)]));
        metadata.set(Symbol::new(env, "token"), Vec::from_array(env, [token.into_val(env)]));
        
        if let Some(r) = reason {
            metadata.set(Symbol::new(env, "reason"), Vec::from_array(env, [r.clone().into_val(env)]));
        }

        Self::emit_standard(env, Symbol::new(env, "mint"), to.clone(), data, metadata);
        
        // Also emit legacy event for backward compatibility
        env.events().publish(
            (Symbol::new(env, "mint"), to.clone()),
            amount,
        );
    }

    /// Emit a burn event using standardized format
    pub fn burn(env: &Env, from: Address, amount: i128, token: Address) {
        let mut data = Vec::new(env);
        data.push_back(amount.into_val(env));
        data.push_back(token.into_val(env));

        let mut metadata = Map::new(env);
        metadata.set(Symbol::new(env, "amount"), Vec::from_array(env, [amount.into_val(env)]));
        metadata.set(Symbol::new(env, "from"), Vec::from_array(env, [from.into_val(env)]));
        metadata.set(Symbol::new(env, "token"), Vec::from_array(env, [token.into_val(env)]));

        Self::emit_standard(env, Symbol::new(env, "burn"), from.clone(), data, metadata);
        
        // Also emit legacy event for backward compatibility
        env.events().publish(
            (Symbol::new(env, "burn"), from.clone()),
            amount,
        );
    }

    /// Emit an admin change event using standardized format
    pub fn admin_changed(env: &Env, old_admin: Address, new_admin: Address) {
        let mut data = Vec::new(env);
        data.push_back(new_admin.into_val(env));

        let mut metadata = Map::new(env);
        metadata.set(Symbol::new(env, "from"), Vec::from_array(env, [old_admin.into_val(env)]));
        metadata.set(Symbol::new(env, "to"), Vec::from_array(env, [new_admin.into_val(env)]));

        Self::emit_standard(env, Symbol::new(env, "admin_changed"), old_admin.clone(), data, metadata);
        
        // Also emit legacy event for backward compatibility
        env.events().publish(
            (Symbol::new(env, "admin_changed"), old_admin.clone()),
            new_admin,
        );
    }

    /// Emit an authorization change event using standardized format
    pub fn authorization_changed(env: &Env, user: Address, authorized: bool) {
        let mut data = Vec::new(env);
        data.push_back(authorized.into_val(env));

        let mut metadata = Map::new(env);
        metadata.set(Symbol::new(env, "to"), Vec::from_array(env, [user.into_val(env)]));

        Self::emit_standard(env, Symbol::new(env, "auth_changed"), user.clone(), data, metadata);
        
        // Also emit legacy event for backward compatibility
        env.events().publish(
            (Symbol::new(env, "auth_changed"), user.clone()),
            authorized,
        );
    }

    /// Emit a proposal created event using standardized format
    pub fn proposal_created(env: &Env, proposer: Address, proposal_id: u64, title: String, proposal_type: Symbol) {
        let mut data = Vec::new(env);
        data.push_back(proposal_id.into_val(env));
        data.push_back(title.clone().into_val(env));
        data.push_back(proposal_type.clone().into_val(env));

        let mut metadata = Map::new(env);
        metadata.set(Symbol::new(env, "proposal_id"), Vec::from_array(env, [proposal_id.into_val(env)]));

        Self::emit_standard(env, Symbol::new(env, "propose"), proposer.clone(), data, metadata);
        
        // Also emit legacy event for backward compatibility
        env.events().publish(
            (Symbol::new(env, "propose"), proposer.clone()),
            (proposal_id, title, proposal_type, env.ledger().timestamp()),
        );
    }

    /// Emit a proposal executed event using standardized format
    pub fn proposal_executed(env: &Env, executor: Address, proposal_id: u64, success: bool) {
        let mut data = Vec::new(env);
        data.push_back(proposal_id.into_val(env));
        data.push_back(success.into_val(env));

        let mut metadata = Map::new(env);
        metadata.set(Symbol::new(env, "proposal_id"), Vec::from_array(env, [proposal_id.into_val(env)]));

        Self::emit_standard(env, Symbol::new(env, "execute"), executor.clone(), data, metadata);
        
        // Also emit legacy event for backward compatibility
        env.events().publish(
            (Symbol::new(env, "execute"), executor.clone()),
            (proposal_id, success, env.ledger().timestamp()),
        );
    }
}

/// Event schema versioning utilities
pub struct EventSchema;

impl EventSchema {
    /// Get current schema version
    pub fn current_version() -> u32 {
        EventEmitter::CURRENT_VERSION
    }

    /// Validate event schema compatibility
    pub fn is_compatible(version: u32) -> bool {
        version <= Self::current_version()
    }

    // =============================================================================
    // Token Event Helpers
    // =============================================================================

    /// Emit a transfer event
    pub fn transfer(env: &Env, event: TransferEvent) {
        env.events().publish((topics::TRANSFER,), event);
    }

    /// Emit an approval event
    pub fn approval(env: &Env, event: ApprovalEvent) {
        env.events().publish((topics::APPROVAL,), event);
    }

    /// Emit a mint event
    pub fn mint(env: &Env, event: MintEvent) {
        env.events().publish((topics::MINT,), event);
    }

    /// Emit a burn event
    pub fn burn(env: &Env, event: BurnEvent) {
        env.events().publish((topics::BURN,), event);
    }

    // =============================================================================
    // Staking Event Helpers
    // =============================================================================

    /// Emit a stake event
    pub fn stake(env: &Env, event: StakeEvent) {
        env.events().publish((topics::STAKE,), event);
    }

    /// Emit an unstake event
    pub fn unstake(env: &Env, event: UnstakeEvent) {
        env.events().publish((topics::UNSTAKE,), event);
    }

    /// Emit a staking reward claimed event
    pub fn staking_reward_claimed(env: &Env, event: StakingRewardClaimedEvent) {
        env.events().publish((topics::REWARD_CLAIMED,), event);
    }

    /// Emit a slashed event
    pub fn slashed(env: &Env, event: SlashedEvent) {
        env.events().publish((topics::SLASHED,), event);
    }

    // =============================================================================
    // Vesting/Academy Event Helpers
    // =============================================================================

    /// Emit a grant created event
    pub fn grant_created(env: &Env, event: GrantCreatedEvent) {
        env.events().publish((topics::GRANT_CREATED,), event);
    }

    /// Emit a vesting claimed event
    pub fn vesting_claimed(env: &Env, event: VestingClaimedEvent) {
        env.events().publish((topics::GRANT_CLAIMED,), event);
    }

    /// Emit a grant revoked event
    pub fn grant_revoked(env: &Env, event: GrantRevokedEvent) {
        env.events().publish((topics::GRANT_REVOKED,), event);
    }

    // =============================================================================
    // Privacy Event Helpers
    // =============================================================================

    /// Emit a shield event
    pub fn shield(env: &Env, event: ShieldEvent) {
        env.events().publish((topics::SHIELD,), event);
    }

    /// Emit an unshield event
    pub fn unshield(env: &Env, event: UnshieldEvent) {
        env.events().publish((topics::UNSHIELD,), event);
    }

    /// Emit a private transfer event
    pub fn private_transfer(env: &Env, event: PrivateTransferEvent) {
        env.events().publish((topics::PRIVATE_TRANSFER,), event);
    }

    // =============================================================================
    // Yield Farming Event Helpers
    // =============================================================================

    /// Emit a liquidity added event
    pub fn liquidity_added(env: &Env, event: LiquidityAddedEvent) {
        env.events().publish((topics::LIQUIDITY_ADDED,), event);
    }

    /// Emit a liquidity removed event
    pub fn liquidity_removed(env: &Env, event: LiquidityRemovedEvent) {
        env.events().publish((topics::LIQUIDITY_REMOVED,), event);
    }

    /// Emit a harvest event
    pub fn harvest(env: &Env, event: HarvestEvent) {
        env.events().publish((topics::HARVEST,), event);
    }

    // =============================================================================
    // Disaster Recovery Event Helpers
    // =============================================================================

    /// Emit an emergency pause event
    pub fn emergency_pause(env: &Env, event: EmergencyPauseEvent) {
        env.events().publish((topics::EMERGENCY_PAUSE,), event);
    }

    /// Emit a recovery initiated event
    pub fn recovery_initiated(env: &Env, event: RecoveryInitiatedEvent) {
        env.events().publish((topics::RECOVERY_INITIATED,), event);
    }

    /// Emit a recovery completed event
    pub fn recovery_completed(env: &Env, event: RecoveryCompletedEvent) {
        env.events().publish((topics::RECOVERY_COMPLETED,), event);
    }

    // =============================================================================
    // Additional Governance Event Helpers
    // =============================================================================

    /// Emit a vote cast event
    pub fn vote_cast(env: &Env, event: VoteCastEvent) {
        env.events().publish((topics::VOTE_CAST,), event);
    }

    /// Emit an emergency action event
    pub fn emergency_action(env: &Env, event: EmergencyActionEvent) {
        env.events().publish((topics::EMERGENCY_ACTION,), event);
    }
}
