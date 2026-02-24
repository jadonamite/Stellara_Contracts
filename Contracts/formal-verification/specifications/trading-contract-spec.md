# Trading Contract Formal Specifications

This document defines the formal specifications and invariants for the Stellara Trading contract. Verification targets critical trading execution functions, state transitions, and fund safety.

## Security Invariants

### 1. Trade Statistics Invariant
```
Let S = (total_trades, total_volume, last_trade_id).
∀ state S after any sequence of trades:
    total_trades ≥ 0
    last_trade_id ≥ 0
    total_trades = last_trade_id  (each trade increments both by 1)
    total_volume = Σ amount_i for all executed trades (non-negative)
```

### 2. Fund Safety (Fees)
```
∀ batch_trade result R:
    total_fees_collected = Σ request.fee_amount for each request in successful_trades
    total_fees_collected ≥ 0
    No fee is collected for a trade that is not in successful_trades.
```

### 3. State Transition Invariant
```
∀ single trade execution:
    pre.last_trade_id + 1 = post.last_trade_id
    pre.total_trades + 1 = post.total_trades
    pre.total_volume + amount = post.total_volume   (when amount > 0 and no overflow)
```

### 4. Batch Consistency
```
∀ batch_trade(requests):
    len(successful_trades) + len(failed_trades) = len(requests)
    ∀ i: failed_trades[i].success ⟺ trade was attempted and failed
    total_fees_collected = Σ fee_amount over indices where request succeeded
```

### 5. Authorization
```
trade(trader, ...)  requires trader.require_auth()
batch_trade: each request.trader.require_auth() before process_single_trade
pause/unpause/set_oracle_config: admin role required
```

---

## Critical Function Specifications

### 1. trade (single trade execution)

**Signature**: `trade(env, trader, pair, amount, price, is_buy, fee_token, fee_amount, fee_recipient) -> Result<u64, FeeError>`

**Preconditions**:
- `trader` is authorized
- Contract is not paused
- `FeeManager::collect_fee` succeeds (trader has sufficient balance for fee)
- Implicit: `amount` is used only for recording; business logic may enforce amount > 0 elsewhere

**Postconditions** (on success):
- New trade id = `old_stats.last_trade_id + 1`
- `stats.total_trades = old_stats.total_trades + 1`
- `stats.total_volume = old_stats.total_volume + amount`
- `stats.last_trade_id = new_trade_id`
- A trade record exists with id = new_trade_id, amount, price, is_buy, trader, pair, timestamp

**Safety properties**:
- No overflow: `old_stats.total_volume + amount` does not overflow (i128)
- No overflow: `old_stats.last_trade_id + 1` does not overflow (u64)
- Fee is collected exactly once per successful trade
- Fund safety: fee_amount is transferred from trader to fee_recipient before trade is recorded

---

### 2. process_single_trade (batch item)

**Signature**: `process_single_trade(env, request, stats, batch_index) -> Result<u64, TradeError>`

**Preconditions**:
- `request.amount > 0` (enforced in function)
- Fee collection succeeds
- `stats` is the current in-memory stats for this batch

**Postconditions** (on success):
- `stats.total_trades` increased by 1
- `stats.total_volume` increased by `request.amount`
- `stats.last_trade_id` = `old_last_trade_id + 1`
- Trade stored with id = `stats.last_trade_id`

**Safety properties**:
- Overflow: `stats.total_volume + request.amount` must not overflow (i128)
- Overflow: `stats.last_trade_id + 1` must not overflow (u64)
- Invalid amount (≤ 0) returns `TradeError::InvalidAmount`

---

### 3. batch_trade

**Signature**: `batch_trade(env, requests) -> Result<BatchTradeOperation, TradeError>`

**Preconditions**:
- `requests.len() ≤ MAX_BATCH_SIZE` (50)
- Contract is not paused
- Each request is processed after `request.trader.require_auth()`

**Postconditions** (on success):
- `len(operation.successful_trades) + len(operation.failed_trades) = len(requests)` (current code pushes every result to failed_trades; see implementation)
- `operation.total_fees_collected = Σ request.fee_amount` for each request that succeeded
- `operation.gas_saved = 1000 * (number of successful trades)`
- Stats in storage updated to reflect all successful trades

**Safety properties**:
- No overflow in aggregate total_fees_collected (i128)
- No overflow in stats (total_volume, last_trade_id, total_trades) across the batch
- Fund safety: total_fees_collected equals sum of fees of successful trades only

---

### 4. Storage: increment_trade_stats

**Signature**: `increment_trade_stats(env, amount) -> u64`

**Preconditions**:
- Stats are initialized
- Caller ensures amount is valid for the trade

**Postconditions**:
- Returns `old_stats.last_trade_id + 1`
- New stats: `total_trades = old + 1`, `total_volume = old_volume + amount`, `last_trade_id = old + 1`

**Safety properties**:
- Overflow: `total_volume.checked_add(amount).is_some()`
- Overflow: `last_trade_id.checked_add(1).is_some()`

---

## Verification Properties (Checklist)

### Arithmetic safety
- [ ] All `total_volume += amount` use checked add or are provably bounded
- [ ] All `last_trade_id += 1` / `last_trade_id + 1` use checked add or are provably bounded
- [ ] `total_fees_collected += request.fee_amount` does not overflow in batch

### State invariants
- [ ] After any trade, `total_trades == last_trade_id`
- [ ] After any trade, `total_volume >= 0` and equals sum of executed trade amounts
- [ ] Paused state prevents trade and batch_trade from modifying state

### Fund safety
- [ ] Fee is collected only once per successful trade
- [ ] total_fees_collected in BatchTradeOperation equals sum of fee_amount for successful trades
- [ ] No trade is recorded without prior successful fee collection (in the same call path)

### Counterexamples
- Verification must complete with no counterexamples for the above properties.

---

## Verification Reports

- All proofs shall be run via the formal verification pipeline.
- Reports shall be generated under `formal-verification/reports/` including:
  - Summary (passed/failed, success rate)
  - Per-proof logs
  - JSON report with timestamp, proof names, and environment (kani/rust version).
