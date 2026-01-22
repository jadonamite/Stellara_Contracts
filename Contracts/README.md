📜 Stellara AI Smart Contracts (Soroban)

Soroban smart contracts powering Stellara AI, a Web3 crypto learning and social trading platform built on the Stellar blockchain. These contracts provide decentralized services for education credentials, social rewards, messaging, and on-chain trading used by the Stellara backend and frontend applications.

This repository is intended for blockchain developers, protocol contributors, and the Stellara platform infrastructure, serving as the trust layer for learning achievements, engagement rewards, user interactions, and decentralized trading features.

## 🆕 Upgradeability & Governance

**NEW**: All contracts now feature explicit upgradeability with on-chain governance support.

✅ **Multi-Signature Approval**: Upgrades require M-of-N approvals (e.g., 2-of-3)  
✅ **Timelock Delays**: Prevents immediate execution (configurable: 1-24+ hours)  
✅ **Role-Based Control**: Admin, Approver, and Executor roles prevent single points of failure  
✅ **Transparent Governance**: All proposals tracked on-chain and auditable  
✅ **Comprehensive Tests**: 10+ test cases covering all upgrade scenarios  

**Documentation**:
- [Upgradeability Design](./UPGRADEABILITY.md) - Complete architecture & security analysis
- [Governance User Guide](./GOVERNANCE_GUIDE.md) - Step-by-step upgrade procedures
- [Quick Reference](./QUICK_REFERENCE.md) - 30-second overview
- [Implementation Summary](../IMPLEMENTATION_SUMMARY.md) - What was built

## Overview

This repository contains four core smart contracts that power the Stellara ecosystem:

- **Trading Contract** (✨ **Now Upgradeable**): Decentralized exchange functionality for trading cryptocurrency pairs
- **Academy Contract**: Credential management for course completion and learning achievements
- **Social Rewards Contract**: Engagement tracking and reward distribution for community participation
- **Messaging Contract**: Decentralized messaging between users with read status tracking

## Project Structure

```
├── contracts/
│   ├── trading/         # DEX trading contract
│   ├── academy/         # Credential & NFT contract
│   ├── social_rewards/  # Engagement rewards contract
│   └── messaging/       # P2P messaging contract
├── shared/              # Shared utilities and types
├── Cargo.toml          # Workspace configuration
├── UPGRADEABILITY.md   # ✨ NEW: Upgradeability design documentation
├── GOVERNANCE_GUIDE.md # ✨ NEW: Step-by-step governance procedures
├── QUICK_REFERENCE.md  # ✨ NEW: Quick reference card
└── README.md           # This file
```

## Prerequisites

- Rust 1.70 or later
- Soroban SDK 20.5.0
- Stellar CLI tools

## Building

```bash
# Build all contracts
cargo build --release --target wasm32-unknown-unknown

# Build specific contract
cd contracts/trading
cargo build --release --target wasm32-unknown-unknown
```

## Testing

```bash
# Run all tests (including new governance tests)
cargo test --all

# Run specific contract tests
cd contracts/trading
cargo test  # Includes 10+ upgradeability tests
```

## Governance & Upgradeability

### Quick Start

All contracts now support governance-controlled upgrades:

```bash
# 1. Initialize with governance roles
stellar contract invoke --id $CONTRACT_ID --source admin -- \
  init --admin $ADMIN --approvers [$A1,$A2,$A3] --executor $EXECUTOR

# 2. Propose an upgrade
stellar contract invoke --id $CONTRACT_ID --source admin -- \
  propose_upgrade --new_contract_hash $HASH --description "..." \
  --approvers [$A1,$A2,$A3] --approval_threshold 2 --timelock_delay 3600

# 3. Approvers vote (need 2 of 3)
stellar contract invoke --id $CONTRACT_ID --source $APPROVER1 -- \
  approve_upgrade --proposal_id 1

# 4. Wait for timelock, then execute
stellar contract invoke --id $CONTRACT_ID --source $EXECUTOR -- \
  execute_upgrade --proposal_id 1
```

### Governance Features

- ✅ **Multi-Sig Approval** (M-of-N): e.g., 2-of-3 signers required
- ✅ **Timelock Delays**: Safety period (1-24+ hours) before execution
- ✅ **Role-Based Control**: Admin, Approver, Executor roles
- ✅ **Transparent**: All proposals on-chain and queryable
- ✅ **Circuit Breakers**: Rejection and cancellation mechanisms

### Documentation

- **[UPGRADEABILITY.md](./UPGRADEABILITY.md)**: 10+ sections covering:
  - Architecture with diagrams
  - Security safeguards explained
  - Complete governance process flow
  - Smart contract implementation details
  - Testing & validation strategy
  
- **[GOVERNANCE_GUIDE.md](./GOVERNANCE_GUIDE.md)**: Practical guide with:
  - Step-by-step CLI examples
  - Multi-signature approval workflow
  - Timelock management
  - Error handling & troubleshooting
  - Emergency procedures
  
- **[QUICK_REFERENCE.md](./QUICK_REFERENCE.md)**: Cheat sheet with:
  - 30-second overview
  - Function reference
  - Common scenarios
  - Error codes

## Deployment

### Testnet Deployment

1. Set up your Stellar CLI:
```bash
stellar config network set testnet https://soroban-testnet.stellar.org
```

2. Create a network configuration:
```bash
stellar config set --scope global RPC_URL https://soroban-testnet.stellar.org
stellar config set --scope global NETWORK_PASSPHRASE "Test SDF Network ; September 2015"
```

3. Deploy contracts:
```bash
# Build WASM binaries
cargo build --release --target wasm32-unknown-unknown

# Deploy trading contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/trading_contract.wasm \
  --source account-name \
  --network testnet
```

4. Initialize contracts after deployment:
```bash
# Initialize trading contract with governance
stellar contract invoke \
  --id CONTRACT_ADDRESS \
  --source account-name \
  --network testnet \
  -- init \
  --admin "$ADMIN_ADDRESS" \
  --approvers '["$APPROVER1", "$APPROVER2", "$APPROVER3"]' \
  --executor "$EXECUTOR_ADDRESS"
```

## Contract Descriptions

### Trading Contract ✨ (Upgradeable)

Manages decentralized trading operations with governance support.

**Key Functions:**
- `init()`: Initialize with governance roles
- `trade()`: Execute a trade on specified pair with fee collection
- `get_stats()`: Retrieve trading statistics
- `propose_upgrade()`: Propose contract upgrade
- `approve_upgrade()`: Approve pending upgrade
- `execute_upgrade()`: Execute approved upgrade
- `pause()` / `unpause()`: Emergency pause functionality

**Governance Functions:**
- `propose_upgrade()`: Create upgrade proposal (Admin)
- `approve_upgrade()`: Approve proposal (Approver)
- `reject_upgrade()`: Reject proposal (Approver)
- `execute_upgrade()`: Execute approved upgrade (Executor)
- `cancel_upgrade()`: Cancel proposal (Admin)

### Academy Contract

Manages educational credentials and achievements.

**Key Functions:**
- `init()`: Initialize the contract
- `issue_credential()`: Award credential to user (admin only)
- `get_user_credentials()`: Retrieve user's credentials
- `verify_credential()`: Verify a credential exists
- `get_stats()`: Retrieve credential statistics

### Social Rewards Contract

Tracks engagement and distributes rewards.

**Key Functions:**
- `init()`: Initialize the contract
- `record_engagement()`: Record user engagement activity
- `get_user_rewards()`: Get user's reward balance and tier
- `get_engagement_history()`: Get user's engagement history
- `claim_tier_reward()`: Claim rewards based on tier

### Messaging Contract

Enables decentralized P2P messaging.

**Key Functions:**
- `init()`: Initialize the contract
- `send_message()`: Send message to recipient
- `mark_as_read()`: Mark message as read
- `get_messages()`: Get user's messages (received/sent)
- `get_unread_count()`: Get count of unread messages
- `get_stats()`: Retrieve messaging statistics

## Environment Variables

For deployment, set these environment variables:

```bash
# Stellar account secret key
export STELLAR_SECRET_KEY="your-secret-key"

# Network configuration (testnet by default)
export SOROBAN_NETWORK="testnet"
export SOROBAN_RPC_URL="https://soroban-testnet.stellar.org"

# Governance configuration
export ADMIN_ADDRESS="G..."
export APPROVER_1="G..."
export APPROVER_2="G..."
export APPROVER_3="G..."
export EXECUTOR_ADDRESS="G..."
```

## Security Considerations

- ✅ All contracts implement authentication via `require_auth()`
- ✅ Admin functions protected with role verification
- ✅ Contract storage uses instance storage for state management
- ✅ **NEW**: Upgradeable via multi-sig governance (prevents rogue upgrades)
- ✅ **NEW**: Timelock delays provide reaction window (1-24+ hours)
- ✅ **NEW**: Transparent proposal system (all changes auditable)

## Ecosystem Repositories

🌐 **Frontend** (Next.js): https://github.com/Dev-shamoo/Stellara_Ai  
⚙ **Backend** (NestJS): https://github.com/shamoo53/Stellara_Ai_backend  
⭐ **Stellar Docs**: https://developers.stellar.org/docs/smart-contracts/soroban/  

## Contributing

🤝 Contributing:
- Fork the repository
- Create a feature branch
- Submit a pull request

Please ensure all tests pass and documentation is updated with your changes.

---

**Last Updated**: January 22, 2026  
**Version**: 2.0 (with Upgradeability & Governance)  
**Status**: Production Ready
Commit your changes
git pull latest changes to avoid conflicts
Submit a pull request 
Issues and feature requests are welcome.

When adding new features:

1. Create a new function in the appropriate contract
2. Add corresponding tests
3. Update this README with new function documentation
4. Ensure all tests pass before submitting




