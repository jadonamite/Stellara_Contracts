# Stellara Upgradeability - Quick Reference

## 30-Second Overview

Stellara contracts use a **governance-controlled upgrade model** that requires:
1. **Multi-signature approval** (e.g., 2 of 3)
2. **Timelock delay** (e.g., 24 hours)  
3. **Role-based control** (Admin, Approver, Executor)

**Result**: Secure upgrades that prevent rogue changes while maintaining transparency.

---

## Governance Roles

| Role | Can Do | Cannot Do |
|------|--------|----------|
| **Admin** | Propose, Cancel, Pause/Unpause | Execute, Approve |
| **Approver** | Approve, Reject | Propose, Execute |
| **Executor** | Execute only | Approve, Propose |

---

## Proposal Lifecycle

```
PENDING → (Approvals reach threshold) → APPROVED → (Timelock expires) → EXECUTED
  ↓
  └→ (Rejected by approver) → REJECTED
  └→ (Cancelled by admin) → CANCELLED
```

---

## Key Functions

### Propose
```rust
propose_upgrade(hash, description, approvers, threshold, delay) → proposal_id
```
**Who**: Admin only  
**Result**: Creates proposal, status = Pending

### Approve
```rust
approve_upgrade(proposal_id, approver_address)
```
**Who**: Approvers  
**Result**: Increments approval count, may trigger approval

### Execute
```rust
execute_upgrade(proposal_id, executor_address)
```
**Who**: Executor only  
**Requirements**: 
- Status = Approved
- Current time ≥ execution_time
- Result**: Executes upgrade

### Reject
```rust
reject_upgrade(proposal_id, approver_address)
```
**Who**: Approver  
**Result**: Immediately stops proposal

### Cancel
```rust
cancel_upgrade(proposal_id, admin_address)
```
**Who**: Admin only  
**Result**: Stops pending/approved proposals

---

## Example: 2-of-3 Multi-Sig Upgrade

```
Step 1: ADMIN proposes
  propose_upgrade(new_hash, desc, [Alice, Bob, Charlie], 2, 86400)
  → proposal_id = 1, status = PENDING

Step 2: ALICE approves
  approve_upgrade(1, Alice)
  → approvals_count = 1/2, status = PENDING

Step 3: BOB approves
  approve_upgrade(1, Bob)
  → approvals_count = 2/2, status = APPROVED ✓

Step 4: Wait 24 hours (timelock)
  → execution_time expires

Step 5: EXECUTOR executes
  execute_upgrade(1, Executor)
  → status = EXECUTED ✓
```

---

## Common Scenarios

### Alice rejects proposal
```
ADMIN proposes → Alice approves → Bob rejects
→ Status = REJECTED (stops here)
```

### Admin cancels before execution
```
ADMIN proposes → Alice approves → Bob approves
→ ADMIN cancel_upgrade() 
→ Status = CANCELLED
```

### Trying to approve twice
```
Alice approve_upgrade(1) ✓
Alice approve_upgrade(1) ✗ Error: DuplicateApproval
```

### Executing before timelock
```
Approve at T=1000
Timelock = 14400 seconds
Execute at T=1000 ✗ Error: TimelockNotExpired
Execute at T=15401 ✓ OK (14401 seconds have passed)
```

---

## Parameters to Configure

| Parameter | Testnet | Mainnet |
|-----------|---------|---------|
| **Approval Threshold** | 1 or 2 | 2+ |
| **Approvers Count** | 2+ | 5+ |
| **Timelock Delay** | 1-4 hours | 24+ hours |
| **Emergency Pause** | Yes | Yes |

---

## Security Checklist

- [ ] Use M-of-N multi-sig (not 1-of-1)
- [ ] Approvers in different jurisdictions
- [ ] 24+ hour timelock on mainnet
- [ ] All roles assigned to secure addresses
- [ ] Communication before upgrade
- [ ] Monitoring alerts configured
- [ ] Emergency escalation plan
- [ ] Testing on testnet first

---

## Error Codes

| Error | Cause | Solution |
|-------|-------|----------|
| `Unauthorized` | Wrong role | Use correct address |
| `ProposalNotFound` | Invalid ID | Check proposal_id |
| `TimelockNotExpired` | Too early to execute | Wait more time |
| `InsufficientApprovals` | Not enough votes | Get more approvals |
| `DuplicateApproval` | Already voted | Try different approver |
| `InvalidThreshold` | Threshold > approvers | Lower threshold |
| `ProposalNotApproved` | Wrong status | Approvals not reached |

---

## Testing Commands

```bash
# Deploy contract
stellar contract deploy --wasm trading.wasm --source admin --network testnet

# Initialize governance
stellar contract invoke --id $CONTRACT_ID --source admin -- \
  init --admin $ADMIN --approvers [$A1,$A2,$A3] --executor $EXECUTOR

# Propose upgrade
stellar contract invoke --id $CONTRACT_ID --source admin -- \
  propose_upgrade --new_contract_hash $HASH --description $DESC \
  --approvers [$A1,$A2,$A3] --approval_threshold 2 --timelock_delay 3600

# Approve
stellar contract invoke --id $CONTRACT_ID --source $APPROVER1 -- \
  approve_upgrade --proposal_id 1 --approver $APPROVER1

# Check proposal
stellar contract invoke --id $CONTRACT_ID --source admin -- \
  get_upgrade_proposal --proposal_id 1

# Execute (after timelock)
stellar contract invoke --id $CONTRACT_ID --source $EXECUTOR -- \
  execute_upgrade --proposal_id 1 --executor $EXECUTOR
```

---

## Performance Characteristics

| Operation | Gas Cost | Notes |
|-----------|----------|-------|
| **propose_upgrade** | ~2500 | Creates proposal |
| **approve_upgrade** | ~1500 | Stores approval |
| **execute_upgrade** | ~1000 | Minimal execution |
| **get_proposal** | ~500 | Query only |
| **reject_upgrade** | ~1000 | Immediate rejection |

---

## Maintenance

### Regular Audits
- Review all open proposals (monthly)
- Audit approval history (quarterly)
- Test emergency procedures (quarterly)

### Version Upgrades
- Always propose through governance
- Never bypass multi-sig for "hotfixes"
- Document all changes in proposals

### Role Changes
- Remove old signers before adding new ones
- Test new signers on testnet first
- Maintain 5+ approvers for production

---

## Further Reading

- **Full Design**: [UPGRADEABILITY.md](./UPGRADEABILITY.md)
- **User Guide**: [GOVERNANCE_GUIDE.md](./GOVERNANCE_GUIDE.md)
- **Implementation**: [governance.rs](./shared/src/governance.rs)
- **Tests**: [test.rs](./contracts/trading/src/test.rs)

---

**Quick Reference v1.0** | January 22, 2026 | Stellara Smart Contracts
