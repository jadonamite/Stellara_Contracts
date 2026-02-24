# Stellara Event Schema Documentation

## Overview

This document describes the standardized event schema used across all Stellara smart contracts for consistent off-chain indexing and analytics.

## Event Schema Version

- **Current Version**: 1.0
- **Compatibility**: Backward compatible with legacy events
- **Migration Path**: See Event Schema Versioning section

## Standard Event Structure

All events follow the `StandardEvent` structure:

```rust
pub struct StandardEvent {
    pub event_type: Symbol,           // Event type identifier
    pub contract_address: Address,      // Contract that emitted the event
    pub user_address: Option<Address>,  // User that triggered the event
    pub data: Vec<Symbol>,           // Event data payload
    pub metadata: Map<Symbol, Vec<Symbol>>, // Additional metadata for indexing
    pub timestamp: u64,              // Block timestamp
    pub version: u32,                // Event schema version
}
```

## Event Types

### Core Token Events

#### Transfer Event
- **Event Type**: `transfer`
- **Description**: Emitted when tokens are transferred between addresses
- **Data**: `[amount, token_address]`
- **Metadata**:
  - `amount`: Transfer amount
  - `from`: Sender address
  - `to`: Recipient address
  - `token`: Token contract address

#### Approval Event
- **Event Type**: `approve`
- **Description**: Emitted when an approval is set for token spending
- **Data**: `[amount, token_address]`
- **Metadata**:
  - `amount`: Approved amount
  - `from`: Token owner address
  - `to`: Spender address
  - `token`: Token contract address

#### Mint Event
- **Event Type**: `mint`
- **Description**: Emitted when new tokens are minted
- **Data**: `[amount, token_address, reason?]`
- **Metadata**:
  - `amount`: Minted amount
  - `to`: Recipient address
  - `token`: Token contract address
  - `reason`: Optional reason for minting

#### Burn Event
- **Event Type**: `burn`
- **Description**: Emitted when tokens are burned
- **Data**: `[amount, token_address]`
- **Metadata**:
  - `amount`: Burned amount
  - `from`: Address that burned tokens
  - `token`: Token contract address

### Staking Events

#### Stake Event
- **Event Type**: `stake`
- **Description**: Emitted when tokens are staked
- **Data**: `[amount, lock_period, token_address]`
- **Metadata**:
  - `amount`: Staked amount
  - `lock_period`: Lock period in seconds
  - `token`: Staked token address

#### Unstake Event
- **Event Type**: `unstake`
- **Description**: Emitted when tokens are unstaked
- **Data**: `[amount, rewards, fee, token_address]`
- **Metadata**:
  - `amount`: Unstaked amount
  - `fee`: Early withdrawal fee (if applicable)
  - `token`: Staked token address

#### Rewards Claimed Event
- **Event Type**: `rewards_claimed`
- **Description**: Emitted when staking rewards are claimed
- **Data**: `[base_rewards, bonus_rewards, token_address]`
- **Metadata**:
  - `amount`: Total rewards claimed
  - `token`: Reward token address

#### Pool Updated Event
- **Event Type**: `pool_updated`
- **Description**: Emitted when staking pool parameters are updated
- **Data**: `[reward_rate, bonus_multiplier]`
- **Metadata**:
  - `reward_rate`: New reward rate per second
  - `token`: Pool token address

### Governance Events

#### Vote Event
- **Event Type**: `vote`
- **Description**: Emitted when a vote is cast on a proposal
- **Data**: `[proposal_id, vote_type, voting_power]`
- **Metadata**:
  - `proposal_id`: Unique proposal identifier
  - `vote_type`: "For", "Against", or "Abstain"

#### Proposal Created Event
- **Event Type**: `proposal_created`
- **Description**: Emitted when a new proposal is created
- **Data**: `[proposal_id, title, proposal_type]`
- **Metadata**:
  - `proposal_id`: Unique proposal identifier

#### Proposal Executed Event
- **Event Type**: `proposal_executed`
- **Description**: Emitted when a proposal is executed
- **Data**: `[proposal_id, success]`
- **Metadata**:
  - `proposal_id`: Unique proposal identifier

### Trading Events

#### Trade Executed Event
- **Event Type**: `trade_executed`
- **Description**: Emitted when a trade is executed
- **Data**: `[pair, amount, price, is_buy, fee_amount, fee_token]`
- **Metadata**:
  - `pair`: Trading pair symbol
  - `amount`: Trade amount
  - `price`: Trade price
  - `fee`: Fee amount paid
  - `token`: Fee token address

#### Fee Collected Event
- **Event Type**: `fee_collected`
- **Description**: Emitted when a fee is collected
- **Data**: `[amount, token_address]`
- **Metadata**:
  - `amount`: Fee amount
  - `from`: Payer address
  - `to`: Recipient address
  - `token`: Fee token address

### Admin and Authorization Events

#### Admin Changed Event
- **Event Type**: `admin_changed`
- **Description**: Emitted when contract admin is changed
- **Data**: `[old_admin, new_admin]`
- **Metadata**:
  - `from`: Previous admin address
  - `to`: New admin address

#### Authorization Changed Event
- **Event Type**: `auth_changed`
- **Description**: Emitted when user authorization status changes
- **Data**: `[authorized]`
- **Metadata**:
  - `to`: User address whose authorization changed

## Event Topics

Standard event topic symbols used for consistent indexing:

```rust
// Core token events
pub const TRANSFER: Symbol = symbol_short!("transfer");
pub const APPROVE: Symbol = symbol_short!("approve");
pub const MINT: Symbol = symbol_short!("mint");
pub const BURN: Symbol = symbol_short!("burn");

// Staking events
pub const STAKE: Symbol = symbol_short!("stake");
pub const UNSTAKE: Symbol = symbol_short!("unstake");
pub const REWARDS_CLAIMED: Symbol = symbol_short!("rewards_claimed");
pub const POOL_UPDATED: Symbol = symbol_short!("pool_updated");

// Governance events
pub const VOTE: Symbol = symbol_short!("vote");
pub const PROPOSAL_CREATED: Symbol = symbol_short!("propose");
pub const PROPOSAL_EXECUTED: Symbol = symbol_short!("execute");

// Trading events
pub const TRADE_EXECUTED: Symbol = symbol_short!("trade");
pub const FEE_COLLECTED: Symbol = symbol_short!("fee");

// Admin events
pub const ADMIN_CHANGED: Symbol = symbol_short!("admin_changed");
pub const AUTHORIZATION_CHANGED: Symbol = symbol_short!("auth_changed");
```

## Metadata Keys

Standard metadata keys for consistent indexing:

```rust
pub const AMOUNT_KEY: Symbol = symbol_short!("amount");
pub const FROM_KEY: Symbol = symbol_short!("from");
pub const TO_KEY: Symbol = symbol_short!("to");
pub const TOKEN_KEY: Symbol = symbol_short!("token");
pub const PAIR_KEY: Symbol = symbol_short!("pair");
pub const PRICE_KEY: Symbol = symbol_short!("price");
pub const FEE_KEY: Symbol = symbol_short!("fee");
pub const REASON_KEY: Symbol = symbol_short!("reason");
pub const PROPOSAL_ID_KEY: Symbol = symbol_short!("proposal_id");
pub const VOTE_TYPE_KEY: Symbol = symbol_short!("vote_type");
pub const LOCK_PERIOD_KEY: Symbol = symbol_short!("lock_period");
pub const REWARD_RATE_KEY: Symbol = symbol_short!("reward_rate");
```

## Event Schema Versioning

### Version 1.0 (Current)
- Standardized event structure across all contracts
- Consistent metadata keys
- Backward compatibility with legacy events
- Enhanced indexing capabilities

### Migration Path

The `EventSchema` utility provides migration paths for schema upgrades:

```rust
// Get current version
let current_version = EventSchema::current_version(); // 1

// Check compatibility
let is_compatible = EventSchema::is_compatible(version);

// Get migration steps
let migration_steps = EventSchema::get_migration_path(from_version, to_version);
```

### Future Versions

Planned enhancements for future versions:
- **v2.0**: Add batch operation support
- **v3.0**: Include gas usage metadata
- **v4.0**: Add cross-chain event correlation

## Indexing Recommendations

### 1. Event Processing Pipeline

```
Blockchain Events → Event Parser → Standardizer → Indexer → Database
```

### 2. Database Schema

Recommended database schema for event storage:

```sql
-- Events table
CREATE TABLE events (
    id BIGINT PRIMARY KEY,
    event_type VARCHAR(50) NOT NULL,
    contract_address VARCHAR(66) NOT NULL,
    user_address VARCHAR(66),
    timestamp BIGINT NOT NULL,
    version INT NOT NULL,
    data JSONB,
    metadata JSONB,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_contract ON events(contract_address);
CREATE INDEX idx_events_user ON events(user_address);
CREATE INDEX idx_events_timestamp ON events(timestamp);
CREATE INDEX idx_events_metadata ON events USING GIN(metadata);
```

### 3. Real-time Processing

- Use WebSocket connections for real-time event streaming
- Implement event deduplication using transaction hashes
- Set up event filtering by contract address and event type

### 4. Analytics and Monitoring

- Track event frequency patterns
- Monitor for unusual event volumes
- Generate daily/weekly event statistics
- Set up alerts for critical events

## Implementation Examples

### Emitting Standardized Events

```rust
use shared::events::EventEmitter;

// Transfer event
EventEmitter::transfer(&env, from, to, amount, token_address);

// Staking event
EventEmitter::stake(&env, user, amount, lock_period, token_address);

// Vote event
EventEmitter::vote(&env, voter, proposal_id, vote_type, voting_power);
```

### Custom Event Emission

```rust
use shared::events::EventEmitter;

let mut data = Vec::new(&env);
data.push_back(custom_data.into_val(&env));

let mut metadata = Map::new(&env);
metadata.set(EventEmitter::CUSTOM_KEY, Vec::from_array(&env, [custom_value.into_val(&env)]));

EventEmitter::emit_standard(&env, custom_event_type, Some(user), data, metadata);
```

## Best Practices

1. **Always use standardized event emitters** for consistency
2. **Include relevant metadata** for better indexing
3. **Use appropriate event types** from predefined set
4. **Maintain backward compatibility** when extending events
5. **Document new event types** in this schema
6. **Test event emission** thoroughly before deployment

## Legacy Support

The event system maintains backward compatibility by emitting both:
1. Standardized events using the new schema
2. Legacy events in the original format

This ensures existing indexing systems continue to work while new systems can leverage the enhanced schema.

## Troubleshooting

### Common Issues

1. **Missing Events**: Ensure contracts import and use `EventEmitter`
2. **Incorrect Metadata**: Verify metadata keys match standard constants
3. **Version Mismatches**: Check event schema version compatibility
4. **Indexing Delays**: Monitor event processing pipeline performance

### Debug Information

Enable debug logging to verify event emission:
- Check event topics match expected symbols
- Verify metadata contains required fields
- Confirm event data is properly formatted

For more information, see the [Stellara Contracts Documentation](./README.md) and [Indexing Infrastructure Guide](./INDEXING_GUIDE.md).
