# Academy Vesting Contract - Quick Reference

## ⚡ 30-Second Overview

A **secure vesting contract** for academy rewards with:
- ✅ Time-based vesting (cliff + linear)
- ✅ Single-claim semantics (atomic, no double-spend)
- ✅ Governance revocation (with timelock)
- ✅ Event emission (Grant, Claim, Revoke)
- ✅ 18+ comprehensive tests

---

## 🎯 Key Concepts

### Vesting Timeline
```
start_time ──cliff──> start_time+cliff ──linear vesting──> start_time+duration

Before Cliff:    0% vested
At Cliff:        0% vested (vesting starts after)
Midway:          ~50% vested (linear)
Full Duration:   100% vested
```

### Single-Claim Semantics
```
First Claim:
├─ Check claimed flag (false)
├─ Transfer tokens
├─ Set claimed = true
└─ Return OK

Second Claim:
├─ Check claimed flag (true)
└─ Return AlreadyClaimed (PREVENTED)
```

---

## 🔧 Quick Function Guide

| Function | Role | Purpose |
|----------|------|---------|
| `init()` | System | Initialize contract (admin, token, governance) |
| `grant_vesting()` | Admin | Backend creates vesting schedule |
| `claim()` | User | User claims vested tokens (atomic) |
| `revoke()` | Admin | Revoke grant with timelock |
| `get_vesting()` | Public | Query schedule details |
| `get_vested_amount()` | Public | Calculate current vested amount |
| `get_info()` | Public | Get contract info |

---

## 📝 Usage Examples

### Initialize (Backend Setup)
```rust
AcademyVestingContract::init(
    env,
    admin_address,
    token_contract,
    governance_contract,
)?;
```

### Grant Vesting (Backend)
```rust
let grant_id = AcademyVestingContract::grant_vesting(
    env,
    admin,
    user_address,
    5000,                        // Amount
    env.ledger().timestamp(),    // Start now
    86400,                       // 1-day cliff
    31536000,                    // 1-year duration
)?;
// Emits: GrantEvent
```

### Check Vesting Progress (Anytime)
```rust
let vested = AcademyVestingContract::get_vested_amount(env, grant_id)?;
println!("Vested: {} tokens", vested);
```

### Claim Tokens (User - Once)
```rust
let claimed = AcademyVestingContract::claim(env, grant_id, user_address)?;
println!("Claimed: {} tokens", claimed);
// Second attempt returns: AlreadyClaimed error
```

### Revoke Grant (Admin)
```rust
AcademyVestingContract::revoke(
    env,
    grant_id,
    admin,
    3600 * 24,  // 1-day min delay
)?;
// Emits: RevokeEvent
```

---

## 🛡️ Security Features

### Authorization
- **Admin only**: `grant_vesting()`, `revoke()`
- **Beneficiary only**: `claim()` (requires signature)
- **Public**: `get_*()` query functions

### Single-Claim Protection
- Atomic flag prevents double-spend
- Replay attacks impossible
- No refund mechanism (by design)

### Timelock Safety
- Minimum 1-hour revocation delay
- Prevents surprise grant cancellations
- Time must elapse before revocation allowed

### Input Validation
- `amount > 0`
- `cliff ≤ duration`
- Schedule doesn't exist check (GrantNotFound)

---

## ⚠️ Error Codes

| Error | Code | When |
|-------|------|------|
| `Unauthorized` | 4001 | Not admin/beneficiary |
| `NotVested` | 4002 | Cliff not passed |
| `AlreadyClaimed` | 4003 | Already claimed once |
| `InvalidSchedule` | 4004 | Bad parameters |
| `InsufficientBalance` | 4005 | Not enough tokens |
| `GrantNotFound` | 4006 | ID doesn't exist |
| `Revoked` | 4007 | Grant revoked |
| `InvalidTimelock` | 4008 | Delay < 1 hour |
| `NotEnoughTimeForRevoke` | 4009 | Timelock not elapsed |

---

## 🧪 Test Coverage

**18+ Tests** covering:
- ✅ Initialization & re-initialization guard
- ✅ Single & multiple grant creation
- ✅ Vesting calculation at all stages
- ✅ Claim authorization & single-semantics
- ✅ Revocation constraints & timelock
- ✅ Query functions & error cases
- ✅ Integration flow (end-to-end)

**Run Tests:**
```bash
cd Contracts/contracts/academy
cargo test --lib
```

---

## 📊 Data Model

```rust
struct VestingSchedule {
    beneficiary: Address,   // Who receives tokens
    amount: i128,          // Total tokens
    start_time: u64,       // When vesting starts
    cliff: u64,            // Delay before unlocking
    duration: u64,         // Total vesting period
    claimed: bool,         // Single-claim flag
    revoked: bool,         // Revocation flag
    revoke_time: u64,      // When revoked
}

struct GrantEvent {
    grant_id: u64,
    beneficiary: Address,
    amount: i128,
    start_time: u64,
    cliff: u64,
    duration: u64,
    granted_at: u64,
    granted_by: Address,
}

struct ClaimEvent {
    grant_id: u64,
    beneficiary: Address,
    amount: i128,
    claimed_at: u64,
}

struct RevokeEvent {
    grant_id: u64,
    beneficiary: Address,
    revoked_at: u64,
    revoked_by: Address,
}
```

---

## 🚀 Deployment Checklist

- [ ] Contract compiles (`cargo build`)
- [ ] All tests pass (`cargo test`)
- [ ] Token contract deployed & working
- [ ] Admin address identified
- [ ] Governance address identified (if needed)
- [ ] Testnet addresses prepared
- [ ] Initial grant prepared
- [ ] Monitoring/indexing setup ready

---

## 🔗 Integration Points

### Backend
- Call `grant_vesting()` when issuing academy rewards
- Emit backend event after grant
- Store grant_id in user profile

### Frontend
- Display vesting schedule from `get_vesting()`
- Show progress with `get_vested_amount()`
- Enable claim button when fully vested
- Track claim status with `claimed` flag

### Indexing
- Subscribe to `GrantEvent` for new vesting
- Subscribe to `ClaimEvent` for claims
- Subscribe to `RevokeEvent` for revocations
- Build user vesting history

### Governance
- Monitor revocation events
- Track admin actions
- Audit grant history

---

## 💡 Best Practices

### For Backend
1. Always verify grant creation before returning to user
2. Store grant_id immediately in user profile
3. Emit application-level event for audit
4. Monitor for failed grants (InsufficientBalance)

### For Users
1. Check vesting progress periodically
2. Only claim when fully vested (NotVested error otherwise)
3. One claim per grant (AlreadyClaimed prevents duplicates)
4. Review grant details before claiming

### For Admins
1. Use appropriate revocation delays (1-24 hours)
2. Document reason for revocations
3. Monitor revocation events
4. Plan maintenance windows around cliffs

---

## 🧠 Common Questions

**Q: Can I claim before the cliff?**
A: No, you'll get `NotVested` error. Wait for cliff to pass.

**Q: Can I claim multiple times?**
A: No, first claim succeeds, subsequent attempts return `AlreadyClaimed`.

**Q: What if I miss my claim?**
A: Tokens remain available indefinitely. Claim whenever ready.

**Q: Can admins revoke after I claim?**
A: No, revoke only works on unclaimed grants. Once claimed, tokens are yours.

**Q: What's the minimum revocation delay?**
A: 1 hour (3600 seconds). Protects users from surprise revocations.

**Q: Are vesting events permanent?**
A: Yes, events immutable on-chain. Perfect for auditing.

---

## 📚 More Information

- **Full Design**: [VESTING_DESIGN.md](./VESTING_DESIGN.md)
- **Code**: [vesting.rs](./src/vesting.rs)
- **Tests**: [test.rs](./src/test.rs)
- **Soroban Docs**: https://developers.stellar.org/docs/learn/soroban

---

## ✅ Acceptance Criteria Met

- [x] Time-based vesting with cliff
- [x] Revocation with timelock
- [x] Single-claim semantics
- [x] Event emission
- [x] Backend grant support
- [x] Comprehensive tests
- [x] Integration test included

