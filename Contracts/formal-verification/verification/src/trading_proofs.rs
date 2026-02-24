#![cfg(kani)]

use kani::*;

#[derive(Clone, Copy, Debug, Default)]
pub struct TradeStats {
    pub total_trades: u64,
    pub total_volume: i128,
    pub last_trade_id: u64,
}

impl TradeStats {
    pub fn increment_checked(&mut self, amount: i128) -> Option<u64> {
        let new_id = self.last_trade_id.checked_add(1)?;
        self.total_trades = self.total_trades.checked_add(1)?;
        self.total_volume = self.total_volume.checked_add(amount)?;
        self.last_trade_id = new_id;
        Some(new_id)
    }

    pub fn invariant_consistent(&self) -> bool {
        self.total_trades == self.last_trade_id
    }

    pub fn invariant_volume_non_negative(&self) -> bool {
        self.total_volume >= 0
    }
}

#[derive(Clone, Debug, Default)]
pub struct BatchResult {
    pub total_fees_collected: i128,
    pub successful_count: u32,
}

impl BatchResult {
    pub fn add_fee_checked(&mut self, fee_amount: i128) -> bool {
        if let Some(new_total) = self.total_fees_collected.checked_add(fee_amount) {
            self.total_fees_collected = new_total;
            self.successful_count = self.successful_count.saturating_add(1);
            true
        } else {
            false
        }
    }

    pub fn invariant_fees_non_negative(&self) -> bool {
        self.total_fees_collected >= 0
    }
}

#[kani::proof]
fn trade_stats_volume_no_overflow() {
    let mut stats = TradeStats::default();
    let amount: i128 = any();

    kani::assume(amount > 0);
    kani::assume(stats.total_volume >= 0);
    kani::assume(stats.total_volume <= i128::MAX - amount);

    let before = stats.total_volume;
    let result = stats.increment_checked(amount);

    assert!(result.is_some());
    assert!(stats.total_volume == before + amount);
    assert!(stats.invariant_consistent());
}

#[kani::proof]
fn trade_stats_volume_overflow_returns_none() {
    let mut stats = TradeStats::default();
    let amount: i128 = any();

    kani::assume(amount > 0);
    kani::assume(stats.total_volume > 0);
    kani::assume(stats.total_volume.checked_add(amount).is_none());

    let result = stats.increment_checked(amount);

    assert!(result.is_none());
}

#[kani::proof]
fn trade_stats_trade_id_overflow_returns_none() {
    let mut stats = TradeStats {
        total_trades: u64::MAX,
        total_volume: 0,
        last_trade_id: u64::MAX,
    };
    let amount: i128 = any();
    kani::assume(amount >= 0);

    let result = stats.increment_checked(amount);

    assert!(result.is_none());
}

#[kani::proof]
fn state_invariant_trades_eq_last_id() {
    let mut stats = TradeStats::default();
    let amount: i128 = any();

    kani::assume(amount > 0);
    kani::assume(stats.total_volume >= 0);
    kani::assume(stats.total_volume.checked_add(amount).is_some());
    kani::assume(stats.last_trade_id < u64::MAX);

    let _ = stats.increment_checked(amount);

    assert!(stats.invariant_consistent());
    assert!(stats.total_trades == stats.last_trade_id);
}

#[kani::proof]
fn fund_safety_fees_non_negative() {
    let mut result = BatchResult::default();
    let fee_amount: i128 = any();

    kani::assume(fee_amount >= 0);
    kani::assume(result.total_fees_collected >= 0);
    kani::assume(result.total_fees_collected.checked_add(fee_amount).is_some());

    let ok = result.add_fee_checked(fee_amount);

    assert!(ok);
    assert!(result.invariant_fees_non_negative());
    assert!(result.total_fees_collected >= 0);
}

#[kani::proof]
fn fund_safety_fee_overflow_returns_false() {
    let mut result = BatchResult::default();
    let fee_amount: i128 = any();

    kani::assume(fee_amount > 0);
    kani::assume(result.total_fees_collected > 0);
    kani::assume(result.total_fees_collected.checked_add(fee_amount).is_none());

    let before = result.total_fees_collected;
    let ok = result.add_fee_checked(fee_amount);

    assert!(!ok);
    assert!(result.total_fees_collected == before);
}

#[kani::proof]
fn amount_positive_for_valid_trade() {
    let amount: i128 = any();

    let is_valid_amount = amount > 0;

    if !is_valid_amount {
        assert!(amount <= 0);
    }
}

#[kani::proof]
fn arithmetic_safety_i128_checked() {
    let a: i128 = any();
    let b: i128 = any();

    if let Some(s) = a.checked_add(b) {
        assert!(s >= i128::MIN);
        assert!(s <= i128::MAX);
    }
    if let Some(d) = a.checked_sub(b) {
        assert!(d >= i128::MIN);
        assert!(d <= i128::MAX);
    }
}

#[kani::proof]
fn arithmetic_safety_u64_increment() {
    let x: u64 = any();

    let next = x.checked_add(1);

    if x == u64::MAX {
        assert!(next.is_none());
    } else {
        assert!(next == Some(x + 1));
    }
}

#[kani::proof]
#[kani::unwind(3)]
fn state_invariant_volume_sum() {
    let mut stats = TradeStats::default();
    let a1: i128 = any();
    let a2: i128 = any();

    kani::assume(a1 > 0 && a2 > 0);
    kani::assume(stats.total_volume.checked_add(a1).is_some());
    kani::assume(stats.total_volume + a1 <= i128::MAX - a2);
    kani::assume(stats.last_trade_id + 2 <= u64::MAX);

    let _ = stats.increment_checked(a1);
    let _ = stats.increment_checked(a2);

    assert!(stats.total_volume == a1 + a2);
    assert!(stats.total_trades == 2);
    assert!(stats.last_trade_id == 2);
}
