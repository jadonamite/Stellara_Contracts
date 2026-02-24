# Formal Verification Integration

This directory contains formal verification tools and specifications for the Stellara Contracts to mathematically prove correctness and eliminate vulnerabilities.

## 🎯 Objectives

- **Mathematically prove** correctness of critical contract functions
- **Eliminate entire classes** of vulnerabilities through formal methods
- **Integrate with CI/CD** for automated verification
- **Generate comprehensive reports** for review

## 📁 Directory Structure

```
formal-verification/
├── specifications/     # Formal specifications and invariants
├── proofs/            # Proof scripts and verification results
├── tools/             # Tool configurations and scripts
├── benchmarks/        # Performance and correctness benchmarks
├── ci/                # CI/CD integration files
└── README.md          # This documentation
```

## 🔧 Tools Used

### Primary Tools
- **Kani Rust Verifier** - Bounded model checking for Rust code
- **Stellar Laboratory** - Contract testing and simulation
- **Soroban SDK Verification** - Contract-specific verification

### Secondary Tools
- **Cargo-hack** - For systematic testing
- **Proptest** - Property-based testing
- **Mirai** - Static analysis (future integration)

## 🎯 Critical Functions for Verification

### Token Contract (`contracts/token`)
1. **`transfer`** - Core fund movement function
2. **`approve`** - Allowance management
3. **`transfer_from`** - Delegated transfers
4. **`mint`** - Token creation (admin only)
5. **`burn`** - Token destruction
6. **`clawback`** - Admin fund recovery
7. **`set_authorized`** - Authorization control

### Trading Contract (`contracts/trading`) — formally verified
1. **`trade`** - Single trade execution with fee collection
2. **`batch_trade`** - Batch trade execution
3. **`process_single_trade`** - Per-request processing and stats update
4. **Storage** - `increment_trade_stats` (overflow-safe)

**Trading verification crate**: `verification/` (Kani proof harnesses for overflow, state invariants, fund safety). Run from repo root:
```bash
cd formal-verification/verification && cargo kani
```

### Key Properties to Verify
- **Conservation of tokens** - Total supply invariants
- **Authorization enforcement** - Access control correctness
- **Overflow/underflow prevention** - Arithmetic safety (trading: `total_volume`, `last_trade_id`, `total_fees_collected`)
- **State invariants** - Trading: `total_trades == last_trade_id`, `total_volume` consistency
- **Fund safety** - Trading: fees non-negative, no double-spend, fee aggregation overflow-safe
- **Reentrancy protection** - State consistency
- **Allowance correctness** - Delegation integrity

## 🚀 Getting Started

### 1. Install Required Tools

```bash
# Install Kani verifier
cargo install --locked kani-verifier

# Install additional tools
cargo install cargo-hack
```

### 2. Run Verification

```bash
# Verify token contract
cd contracts/token
cargo kani

# Run specific proof
cargo kani --proof-name transfer_safety

# Run all proofs
cargo kani --all-features
```

### 3. Generate Reports

```bash
# Generate verification report
cargo kani --output-format json > verification-report.json

# Run with coverage
cargo kani --coverage
```

## 📊 CI/CD Integration

Verification is integrated into the CI pipeline via:
- **GitHub Actions** - Automated verification on PRs
- **Verification reports** - Generated for each build
- **Failure blocking** - Critical violations block merges
- **Performance monitoring** - Track verification time

## 📈 Current Status

- [x] Directory structure created
- [x] Tool installation documented
- [x] Formal specifications written (token + **trading**)
- [x] Proof harnesses implemented (token + **trading** in `verification/`)
- [ ] CI integration configured
- [x] Verification reports generated (run script + `reports/`)

## 📚 Next Steps

1. **Write formal specifications** for critical functions
2. **Create Kani proof harnesses** for each property
3. **Configure CI pipeline** for automated verification
4. **Document verification results** and reports
5. **Set up monitoring** for verification performance

## 🛡️ Security Benefits

This formal verification approach provides:
- **Mathematical guarantees** of correctness
- **Elimination** of entire vulnerability classes
- **Automated regression testing** for security properties
- **Comprehensive coverage** analysis
- **Audit-ready documentation** and reports