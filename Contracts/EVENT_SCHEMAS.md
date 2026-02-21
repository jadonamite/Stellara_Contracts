# Event Schema Documentation

## Overview

This document provides comprehensive schema documentation for all standardized events emitted by Stellara Network contracts. These events are designed for consistent off-chain indexing and analytics.

## Event Structure Standards

All events follow these standards:
- Use `#[contracttype]` derive macro for Soroban compatibility
- Include `timestamp: u64` field for block timestamp
- Use snake_case field naming
- Include relevant addresses for contract identification
- Provide sufficient context for off-chain processing

## Event Topics

All events use standardized topics from `shared::events::topics`:

| Topic | Symbol | Description |
|-------|--------|-------------|
| `transfer` | `"transfer"` | Token transfers |
| `approve` | `"approve"` | Token approvals |
| `mint` | `"mint"` | Token minting |
| `burn` | `"burn"` | Token burning |
| `stake` | `"stake"` | Token staking |
| `unstake` | `"unstake"` | Token unstaking |
| `claimed` | `"claimed"` | Reward claiming |
| `slashed` | `"slashed"` | Token slashing |
| `grant` | `"grant"` | Vesting grant creation |
| `claim` | `"claim"` | Vesting claims |
| `revoke` | `"revoke"` | Grant revocation |
| `shield` | `"shield"` | Privacy shielding |
| `unshield` | `"unshield"` | Privacy unshielding |
| `private_transfer` | `"private_transfer"` | Private transfers |
| `liquidity_added` | `"liquidity_added"` | Liquidity provision |
| `liquidity_removed` | `"liquidity_removed"` | Liquidity removal |
| `harvest` | `"harvest"` | Farming reward harvesting |
| `trade` | `"trade"` | Trading operations |
| `fee` | `"fee"` | Fee collection |
| `propose` | `"propose"` | Proposal creation |
| `vote` | `"vote"` | Vote casting |
| `execute` | `"execute"` | Proposal execution |
| `emergency` | `"emergency"` | Emergency actions |
| `emergency_pause` | `"emergency_pause"` | Emergency pauses |
| `recovery_started` | `"recovery_started"` | Recovery initiation |
| `recovery_completed` | `"recovery_completed"` | Recovery completion |

---

## Token Events

### TransferEvent

Emitted when tokens are transferred between addresses.

```rust
pub struct TransferEvent {
    pub from: Address,        // Address sending the tokens
    pub to: Address,          // Address receiving the tokens
    pub amount: i128,         // Amount of tokens transferred
    pub token: Address,       // Token contract address
    pub timestamp: u64,       // Block timestamp
}
```

**Indexing Notes:**
- Track balance changes for both addresses
- Update token holder lists
- Calculate transfer volume metrics
- Monitor for suspicious activity

### ApprovalEvent

Emitted when an approval is set or modified.

```rust
pub struct ApprovalEvent {
    pub owner: Address,           // Owner of the tokens
    pub spender: Address,         // Address approved to spend
    pub amount: i128,             // Amount approved
    pub token: Address,           // Token contract address
    pub expiration_ledger: u32,   // Expiration ledger number
    pub timestamp: u64,          // Block timestamp
}
```

**Indexing Notes:**
- Track allowance changes
- Monitor for large approvals
- Update allowance expiration tracking

### MintEvent

Emitted when new tokens are minted.

```rust
pub struct MintEvent {
    pub to: Address,          // Address receiving minted tokens
    pub amount: i128,         // Amount of tokens minted
    pub token: Address,       // Token contract address
    pub total_supply: i128,   // Total supply after minting
    pub timestamp: u64,       // Block timestamp
}
```

**Indexing Notes:**
- Track token supply changes
- Monitor minting patterns
- Calculate inflation metrics

### BurnEvent

Emitted when tokens are burned.

```rust
pub struct BurnEvent {
    pub from: Address,        // Address whose tokens are burned
    pub amount: i128,         // Amount of tokens burned
    pub token: Address,       // Token contract address
    pub total_supply: i128,   // Total supply after burning
    pub timestamp: u64,       // Block timestamp
}
```

**Indexing Notes:**
- Track deflationary patterns
- Monitor burn events for supply analysis
- Update circulating supply metrics

---

## Staking Events

### StakeEvent

Emitted when tokens are staked.

```rust
pub struct StakeEvent {
    pub user: Address,              // Address staking tokens
    pub amount: i128,               // Amount of tokens staked
    pub staking_contract: Address,  // Staking contract address
    pub lock_period: u64,           // Lock period in seconds
    pub reward_multiplier: u32,     // Reward multiplier
    pub total_staked: i128,         // Total staked amount after this stake
    pub timestamp: u64,             // Block timestamp
}
```

**Indexing Notes:**
- Track staking participation
- Calculate staking ratios
- Monitor lock period distributions
- Track reward multiplier usage

### UnstakeEvent

Emitted when tokens are unstaked.

```rust
pub struct UnstakeEvent {
    pub user: Address,              // Address unstaking tokens
    pub amount: i128,               // Amount of tokens unstaked
    pub staking_contract: Address,  // Staking contract address
    pub rewards_earned: i128,       // Rewards earned
    pub penalty: i128,              // Penalty applied (if any)
    pub total_staked: i128,         // Total staked amount after this unstake
    pub timestamp: u64,             // Block timestamp
}
```

**Indexing Notes:**
- Track unstaking patterns
- Calculate reward yields
- Monitor penalty frequency
- Update staking participation metrics

### StakingRewardClaimedEvent

Emitted when staking rewards are claimed.

```rust
pub struct StakingRewardClaimedEvent {
    pub user: Address,              // Address claiming rewards
    pub amount: i128,               // Amount of rewards claimed
    pub staking_contract: Address,  // Staking contract address
    pub reward_token: Address,      // Reward token address
    pub timestamp: u64,             // Block timestamp
}
```

**Indexing Notes:**
- Track reward claiming patterns
- Calculate reward distribution metrics
- Monitor reward token flows

### SlashedEvent

Emitted when staked tokens are slashed.

```rust
pub struct SlashedEvent {
    pub user: Address,              // Address whose tokens are slashed
    pub amount: i128,               // Amount of tokens slashed
    pub staking_contract: Address,  // Staking contract address
    pub reason: Symbol,             // Reason for slashing
    pub slashed_by: Address,        // Admin who initiated slashing
    pub timestamp: u64,             // Block timestamp
}
```

**Indexing Notes:**
- Track slashing events
- Monitor slashing reasons
- Calculate slash rates
- Track governance actions

---

## Vesting/Academy Events

### GrantCreatedEvent

Emitted when a vesting grant is created.

```rust
pub struct GrantCreatedEvent {
    pub grant_id: u64,         // Unique grant identifier
    pub beneficiary: Address,   // Beneficiary address
    pub amount: i128,          // Total amount of tokens in grant
    pub start_time: u64,       // Start time of vesting
    pub cliff: u64,            // Cliff period (seconds)
    pub duration: u64,         // Total vesting duration (seconds)
    pub created_by: Address,   // Admin who created the grant
    pub timestamp: u64,        // Block timestamp
}
```

**Indexing Notes:**
- Track vesting grant creation
- Calculate vesting schedules
- Monitor grant distributions
- Track beneficiary allocations

### VestingClaimedEvent

Emitted when vesting is claimed.

```rust
pub struct VestingClaimedEvent {
    pub grant_id: u64,         // Grant identifier
    pub beneficiary: Address,   // Beneficiary address
    pub amount: i128,          // Amount claimed
    pub total_claimed: i128,   // Total amount claimed so far
    pub remaining_amount: i128, // Remaining amount to claim
    pub timestamp: u64,        // Block timestamp
}
```

**Indexing Notes:**
- Track vesting claim patterns
- Calculate vesting progress
- Monitor claim frequency
- Update remaining vesting amounts

### GrantRevokedEvent

Emitted when a vesting grant is revoked.

```rust
pub struct GrantRevokedEvent {
    pub grant_id: u64,         // Grant identifier
    pub beneficiary: Address,   // Beneficiary address
    pub amount_revoked: i128,   // Amount revoked (returned to admin)
    pub amount_claimed: i128,   // Amount already claimed by beneficiary
    pub revoked_by: Address,   // Admin who revoked the grant
    pub reason: Symbol,         // Reason for revocation
    pub timestamp: u64,        // Block timestamp
}
```

**Indexing Notes:**
- Track grant revocations
- Monitor revocation reasons
- Calculate revoked amounts
- Update active grant counts

---

## Privacy Events

### ShieldEvent

Emitted when tokens are shielded (made private).

```rust
pub struct ShieldEvent {
    pub user: Address,              // Address shielding tokens
    pub amount: i128,               // Amount of tokens shielded
    pub public_token: Address,      // Public token contract address
    pub privacy_contract: Address,  // Privacy contract address
    pub note_id: Symbol,            // Shielded note identifier
    pub timestamp: u64,             // Block timestamp
}
```

**Indexing Notes:**
- Track privacy adoption
- Calculate shielded token amounts
- Monitor privacy contract usage
- Track note creation patterns

### UnshieldEvent

Emitted when tokens are unshielded (made public).

```rust
pub struct UnshieldEvent {
    pub user: Address,              // Address receiving unshielded tokens
    pub amount: i128,               // Amount of tokens unshielded
    pub public_token: Address,      // Public token contract address
    pub privacy_contract: Address,  // Privacy contract address
    pub note_id: Symbol,            // Shielded note identifier consumed
    pub timestamp: u64,             // Block timestamp
}
```

**Indexing Notes:**
- Track unshielding patterns
- Calculate privacy duration
- Monitor note consumption
- Update privacy metrics

### PrivateTransferEvent

Emitted when a private transfer occurs.

```rust
pub struct PrivateTransferEvent {
    pub from: Address,              // Address sending private tokens
    pub to: Address,                // Address receiving private tokens (encrypted)
    pub amount: i128,               // Amount transferred
    pub privacy_contract: Address,  // Privacy contract address
    pub input_notes: Vec<Symbol>,   // Input note identifiers consumed
    pub output_notes: Vec<Symbol>,  // Output note identifiers created
    pub timestamp: u64,             // Block timestamp
}
```

**Indexing Notes:**
- Track private transfer volume
- Monitor note creation/consumption
- Calculate privacy metrics
- Track transfer patterns (aggregate)

---

## Yield Farming Events

### LiquidityAddedEvent

Emitted when liquidity is added to a farming pool.

```rust
pub struct LiquidityAddedEvent {
    pub user: Address,              // Address providing liquidity
    pub pool_id: u64,               // Pool identifier
    pub amount_a: i128,             // Amount of token A provided
    pub amount_b: i128,             // Amount of token B provided
    pub lp_tokens: i128,            // LP tokens received
    pub farming_contract: Address,  // Farming contract address
    pub timestamp: u64,             // Block timestamp
}
```

**Indexing Notes:**
- Track liquidity provision
- Calculate pool depths
- Monitor LP token distribution
- Track liquidity provider participation

### LiquidityRemovedEvent

Emitted when liquidity is removed from a farming pool.

```rust
pub struct LiquidityRemovedEvent {
    pub user: Address,              // Address removing liquidity
    pub pool_id: u64,               // Pool identifier
    pub amount_a: i128,             // Amount of token A received
    pub amount_b: i128,             // Amount of token B received
    pub lp_tokens: i128,            // LP tokens burned
    pub farming_contract: Address,  // Farming contract address
    pub timestamp: u64,             // Block timestamp
}
```

**Indexing Notes:**
- Track liquidity removal patterns
- Calculate pool depth changes
- Monitor impermanent loss
- Update liquidity provider metrics

### HarvestEvent

Emitted when farming rewards are harvested.

```rust
pub struct HarvestEvent {
    pub user: Address,              // Address harvesting rewards
    pub pool_id: u64,               // Pool identifier
    pub reward_amount: i128,        // Amount of rewards claimed
    pub reward_token: Address,      // Reward token address
    pub farming_contract: Address,  // Farming contract address
    pub timestamp: u64,             // Block timestamp
}
```

**Indexing Notes:**
- Track reward harvesting
- Calculate farming yields
- Monitor reward distribution
- Track farming participation

---

## Governance Events

### VoteCastEvent

Emitted when a vote is cast on a proposal.

```rust
pub struct VoteCastEvent {
    pub proposal_id: u64,      // Proposal identifier
    pub voter: Address,        // Address casting the vote
    pub vote_type: Symbol,     // Vote type (for, against, abstain)
    pub voting_power: u128,    // Voting power used
    pub reason: Symbol,        // Reason for vote (optional)
    pub timestamp: u64,       // Block timestamp
}
```

**Indexing Notes:**
- Track voting participation
- Calculate voting power distribution
- Monitor voting patterns
- Track proposal progress

### EmergencyActionEvent

Emitted when emergency action is taken.

```rust
pub struct EmergencyActionEvent {
    pub action_id: u64,            // Unique action identifier
    pub council_member: Address,   // Emergency council member who took action
    pub action_type: Symbol,        // Type of emergency action
    pub target_contract: Address,  // Target contract address
    pub description: Symbol,       // Action description
    pub timestamp: u64,            // Block timestamp
}
```

**Indexing Notes:**
- Track emergency actions
- Monitor council activity
- Calculate emergency response times
- Track action effectiveness

---

## Disaster Recovery Events

### EmergencyPauseEvent

Emitted when emergency pause is triggered.

```rust
pub struct EmergencyPauseEvent {
    pub contract_address: Address,  // Contract being paused
    pub paused_by: Address,         // Authority who triggered the pause
    pub reason: Symbol,             // Reason for emergency pause
    pub timestamp: u64,             // Block timestamp
}
```

**Indexing Notes:**
- Track emergency pauses
- Monitor pause reasons
- Calculate pause durations
- Track system stability metrics

### RecoveryInitiatedEvent

Emitted when recovery process is initiated.

```rust
pub struct RecoveryInitiatedEvent {
    pub contract_address: Address,  // Contract being recovered
    pub recovery_id: u64,           // Recovery identifier
    pub initiated_by: Address,       // Authority who initiated recovery
    pub recovery_type: Symbol,      // Type of recovery (backup, migration, etc.)
    pub timestamp: u64,             // Block timestamp
}
```

**Indexing Notes:**
- Track recovery initiations
- Monitor recovery types
- Calculate recovery success rates
- Track system resilience

### RecoveryCompletedEvent

Emitted when recovery process is completed.

```rust
pub struct RecoveryCompletedEvent {
    pub contract_address: Address,  // Contract that was recovered
    pub recovery_id: u64,           // Recovery identifier
    pub completed_by: Address,      // Authority who completed recovery
    pub success: bool,              // Success status
    pub new_state_hash: Symbol,      // New contract state hash (if applicable)
    pub timestamp: u64,             // Block timestamp
}
```

**Indexing Notes:**
- Track recovery completions
- Calculate recovery success rates
- Monitor recovery durations
- Track system restoration

---

## Event Versioning Strategy

### Version Numbering
- Events use semantic versioning based on contract deployment
- Breaking changes require new event versions
- Backward compatibility maintained when possible

### Schema Evolution
- New fields can be added without version change
- Field removal requires version bump
- Field type changes require version bump

### Migration Strategy
- Support multiple event versions during transition
- Provide migration tools for indexers
- Document deprecation timelines

---

## Indexing Best Practices

### Event Filtering
- Filter by topic for relevant events
- Use address filtering for user-specific data
- Implement time-based filtering for efficiency

### Data Validation
- Validate event structure against schema
- Check for required fields
- Verify data types and ranges

### Performance Optimization
- Batch process events when possible
- Use database indexes for frequent queries
- Implement caching for repeated calculations

### Error Handling
- Handle malformed events gracefully
- Implement retry mechanisms for failures
- Log processing errors for debugging

---

## Data Types Reference

| Rust Type | Soroban Type | Description |
|-----------|--------------|-------------|
| `Address` | `Address` | Contract or user address |
| `i128` | `i128` | 128-bit signed integer |
| `u64` | `u64` | 64-bit unsigned integer |
| `u32` | `u32` | 32-bit unsigned integer |
| `u128` | `u128` | 128-bit unsigned integer |
| `Symbol` | `Symbol` | Short string (max 10 bytes) |
| `Vec<T>` | `Vec<T>` | Dynamic array |
| `bool` | `bool` | Boolean value |

---

## Example Event Processing

```javascript
// Example JavaScript event processing
function processTransferEvent(event) {
    const { from, to, amount, token, timestamp } = event.data;
    
    // Update balances
    updateBalance(from, -amount);
    updateBalance(to, amount);
    
    // Track transfer volume
    updateTransferVolume(token, amount, timestamp);
    
    // Check for suspicious activity
    if (amount > SUSPICIOUS_THRESHOLD) {
        flagSuspiciousActivity(event);
    }
}
```

---

## Support and Maintenance

For questions about event schemas or indexing implementation:
- Check the GitHub repository for latest updates
- Review test cases for example event structures
- Contact the development team for clarification

Event schemas are updated as part of contract deployments. Always check the latest version when implementing indexers.
