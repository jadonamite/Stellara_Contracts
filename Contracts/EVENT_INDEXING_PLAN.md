# Event Indexing and Off-chain Data Processing Implementation Plan

## Overview
This document outlines the implementation plan for issue #101: Event Indexing and Off-chain Data Processing. The goal is to standardize event emission patterns and create infrastructure for off-chain indexing of contract events.

## Current State Analysis

### ✅ Already Implemented
- **Shared Events Module**: `shared/src/events.rs` provides standardized event structures
- **EventEmitter Helper**: Centralized event emission with consistent patterns
- **Standard Topics**: Pre-defined event topics for consistent indexing
- **Partial Adoption**: Some contracts already use standardized events:
  - `social_rewards` - Uses RewardAddedEvent, RewardClaimedEvent
  - `trading` - Uses TradeExecutedEvent, FeeCollectedEvent

### ❌ Missing Components
- **Incomplete Adoption**: Many contracts still use custom event patterns
- **Missing Event Types**: Several contract types lack standardized events
- **No Schema Documentation**: Event schemas not documented for indexers
- **No Indexing Guide**: No infrastructure recommendations for off-chain processing

## Implementation Tasks

### Phase 1: Complete Event Standardization

#### 1.1 Add Missing Event Types
Create standardized events for contracts that don't have them yet:

**Token Contract Events** (already partially implemented):
- `TransferEvent` - Standardize token transfers
- `ApprovalEvent` - Standardize token approvals
- `MintEvent` - Token minting
- `BurnEvent` - Token burning

**Staking Contract Events**:
- `StakeEvent` - When tokens are staked
- `UnstakeEvent` - When tokens are unstaked
- `RewardClaimedEvent` - When staking rewards are claimed
- `SlashedEvent` - When staked tokens are slashed

**Governance Token Events**:
- `ProposalCreatedEvent` - Already exists
- `VotedEvent` - When a vote is cast
- `ProposalExecutedEvent` - Already exists
- `EmergencyActionEvent` - Emergency council actions

**Academy/Vesting Events**:
- `GrantCreatedEvent` - When vesting grant is created
- `GrantClaimedEvent` - When vesting is claimed
- `GrantRevokedEvent` - When vesting is revoked

**Privacy Token Events**:
- `ShieldEvent` - When tokens are shielded
- `UnshieldEvent` - When tokens are unshielded
- `PrivateTransferEvent` - Private transfer events

**Yield Farming Events**:
- `LiquidityAddedEvent` - Adding liquidity to pools
- `LiquidityRemovedEvent` - Removing liquidity from pools
- `HarvestEvent` - Claiming farming rewards

**Disaster Recovery Events**:
- `EmergencyPauseEvent` - Emergency pause triggered
- `RecoveryInitiatedEvent` - Recovery process started
- `RecoveryCompletedEvent` - Recovery process completed

#### 1.2 Update Contracts to Use Standardized Events
Update all contracts to use the EventEmitter helper and standardized event structures:

**Priority Order**:
1. Token contract (highest usage)
2. Staking contract
3. Governance token contract
4. Academy/Vesting contracts
5. Privacy contracts
6. Yield farming contracts
7. Disaster recovery contracts
8. Reward distributor contracts

### Phase 2: Event Schema Documentation

#### 2.1 Create Event Schema Registry
Create comprehensive documentation of all event schemas:

**File**: `EVENT_SCHEMAS.md`
- Event type definitions
- Field descriptions and types
- Event topics and their meanings
- Versioning strategy for event schemas
- Data encoding information

#### 2.2 Create Indexer Developer Guide
**File**: `INDEXING_GUIDE.md`
- How to set up event indexing
- Event filtering strategies
- Data transformation examples
- Performance optimization tips
- Common indexing patterns

### Phase 3: Indexing Infrastructure Recommendations

#### 3.1 Reference Indexer Implementation
Create a reference implementation in the `indexer/` directory:

**Components**:
- Event parser for Soroban events
- Database schema for storing indexed events
- API endpoints for accessing indexed data
- Real-time event streaming setup
- Historical data backfilling tools

#### 3.2 Infrastructure Recommendations
Document recommended infrastructure:

**Technology Stack**:
- Event streaming: Apache Kafka / NATS
- Database: PostgreSQL + TimescaleDB
- API: GraphQL + REST
- Monitoring: Prometheus + Grafana
- Deployment: Docker + Kubernetes

## Acceptance Criteria Checklist

### ✅ Events Follow Consistent Structure and Naming
- [ ] All events use `#[contracttype]` derive macro
- [ ] All events have consistent field naming (snake_case)
- [ ] All events include timestamp field
- [ ] All events use standardized topics from `shared::events::topics`
- [ ] All contracts use `EventEmitter` helper for event emission

### ✅ All State Changes Emit Appropriate Events
- [ ] Token transfers emit TransferEvent
- [ ] Staking operations emit staking events
- [ ] Governance actions emit governance events
- [ ] All contract state changes have corresponding events
- [ ] Events include all relevant state change data

### ✅ Event Schemas are Documented and Versioned
- [ ] Complete event schema documentation
- [ ] Field type definitions
- [ ] Event versioning strategy
- [ ] Backward compatibility guidelines
- [ ] Schema validation examples

### ✅ Indexing Recommendations are Provided
- [ ] Reference indexer implementation
- [ ] Infrastructure setup guide
- [ ] Performance optimization guide
- [ ] Monitoring and alerting setup
- [ ] Deployment best practices

## Implementation Timeline

**Week 1**: Phase 1.1 - Add missing event types to shared module
**Week 2**: Phase 1.2 - Update high-priority contracts (token, staking, governance)
**Week 3**: Phase 2 - Create comprehensive documentation
**Week 4**: Phase 3 - Develop indexing infrastructure and recommendations

## Testing Strategy

### Unit Tests
- Test event emission in all contracts
- Validate event structure and data
- Test EventEmitter helper functions

### Integration Tests
- Test end-to-end event flow
- Test indexer consumption of events
- Test event schema validation

### Performance Tests
- Test event emission overhead
- Test indexer throughput
- Test database query performance

## Success Metrics

- **100%** of contracts use standardized events
- **0** custom event patterns remaining
- **Complete** event schema documentation
- **Reference** indexer implementation available
- **Performance** benchmarks meeting targets (<5ms event emission overhead)

## Risks and Mitigations

### Risk: Breaking Changes
- **Mitigation**: Maintain backward compatibility during transition
- **Plan**: Support both old and new event formats temporarily

### Risk: Performance Impact
- **Mitigation**: Benchmark event emission overhead
- **Plan**: Optimize event structures and emission patterns

### Risk: Indexer Complexity
- **Mitigation**: Provide simple reference implementation
- **Plan**: Create comprehensive documentation and examples

## Next Steps

1. **Immediate**: Start implementing missing event types in shared module
2. **Week 1**: Update token contract to use standardized events
3. **Week 2**: Create event schema documentation
4. **Week 3**: Develop reference indexer implementation

---

*This plan will be updated as implementation progresses and new requirements are identified.*
