#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    token, Address, Env, Map, Symbol, Vec,
};

// ─── Storage Keys ─────────────────────────────────────────────────────────────

const ADMIN: Symbol = symbol_short!("ADMIN");
const CONFIG: Symbol = symbol_short!("CONFIG");
const PAUSED: Symbol = symbol_short!("PAUSED");

// ─── Types ────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct StakingConfig {
    pub staking_token: Address,
    pub min_stake_amount: i128,
    pub emergency_fee_bps: u32,      // fee on emergency withdraw (e.g. 1000 = 10%)
    pub compound_fee_bps: u32,       // fee retained on auto-compound
    pub fee_recipient: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct LockupTier {
    pub duration_ledgers: u64,       // 0 = flexible (no lock)
    pub boost_bps: u32,              // reward multiplier bonus (e.g. 2000 = +20%)
}

#[contracttype]
#[derive(Clone)]
pub struct RewardPool {
    pub token: Address,
    pub reward_rate: i128,           // tokens per ledger (scaled 1e7)
    pub reward_per_token_stored: i128,
    pub last_update_ledger: u64,
    pub total_rewards_deposited: i128,
    pub total_rewards_paid: i128,
    pub period_finish: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct StakePosition {
    pub owner: Address,
    pub amount: i128,
    pub boosted_amount: i128,
    pub lockup_duration: u64,
    pub unlock_ledger: u64,
    pub start_ledger: u64,
    pub auto_compound: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct UserRewardState {
    pub reward_per_token_paid: i128,
    pub accrued: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    TotalStaked,
    TotalBoostedStake,
    StakePosition(Address),
    LockupTiers,
    RewardPool(Address),             // keyed by reward token address
    RewardPools,                     // Vec<Address> of all reward tokens
    UserReward(Address, Address),    // (user, reward_token)
    CompoundRewards(Address),        // queued compoundable rewards per user
}

// ─── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct StakingContract;

#[contractimpl]
impl StakingContract {
    // ── Init ──────────────────────────────────────────────────────────────────

    pub fn initialize(env: Env, admin: Address, config: StakingConfig, tiers: Vec<LockupTier>) {
        admin.require_auth();
        if env.storage().instance().has(&ADMIN) {
            panic!("already initialized");
        }
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&CONFIG, &config);
        env.storage().instance().set(&PAUSED, &false);
        env.storage()
            .persistent()
            .set(&DataKey::LockupTiers, &tiers);
        env.storage()
            .persistent()
            .set(&DataKey::TotalStaked, &0i128);
        env.storage()
            .persistent()
            .set(&DataKey::TotalBoostedStake, &0i128);
        env.storage()
            .persistent()
            .set(&DataKey::RewardPools, &Vec::<Address>::new(&env));
    }

    // ── Admin ─────────────────────────────────────────────────────────────────

    pub fn add_reward_pool(
        env: Env,
        reward_token: Address,
        reward_amount: i128,
        duration_ledgers: u64,
    ) {
        Self::only_admin(&env);
        let config: StakingConfig = env.storage().instance().get(&CONFIG).unwrap();
        let current = env.ledger().sequence() as u64;

        let mut pools: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RewardPools)
            .unwrap();

        let total_boosted: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalBoostedStake)
            .unwrap_or(0);

        let reward_rate = if duration_ledgers == 0 {
            panic!("duration must be > 0");
        } else {
            (reward_amount * 1_0000000) / duration_ledgers as i128
        };

        let pool_key = DataKey::RewardPool(reward_token.clone());
        let pool = if env.storage().persistent().has(&pool_key) {
            let mut existing: RewardPool = env.storage().persistent().get(&pool_key).unwrap();
            Self::_update_reward_per_token(&env, &mut existing, total_boosted);
            // extend or top up existing pool
            let remaining = if current < existing.period_finish {
                (existing.period_finish - current) as i128 * existing.reward_rate
            } else {
                0
            };
            let new_rate = ((remaining + reward_amount) * 1_0000000) / duration_ledgers as i128;
            existing.reward_rate = new_rate;
            existing.period_finish = current + duration_ledgers;
            existing.total_rewards_deposited += reward_amount;
            existing.last_update_ledger = current;
            existing
        } else {
            pools.push_back(reward_token.clone());
            RewardPool {
                token: reward_token.clone(),
                reward_rate,
                reward_per_token_stored: 0,
                last_update_ledger: current,
                total_rewards_deposited: reward_amount,
                total_rewards_paid: 0,
                period_finish: current + duration_ledgers,
            }
        };

        token::Client::new(&env, &reward_token).transfer_from(
            &env.current_contract_address(),
            &env.current_contract_address(),
            &env.current_contract_address(),
            &reward_amount,
        );

        env.storage().persistent().set(&pool_key, &pool);
        env.storage()
            .persistent()
            .set(&DataKey::RewardPools, &pools);
    }

    pub fn set_paused(env: Env, paused: bool) {
        Self::only_admin(&env);
        env.storage().instance().set(&PAUSED, &paused);
    }

    pub fn update_config(env: Env, new_config: StakingConfig) {
        Self::only_admin(&env);
        env.storage().instance().set(&CONFIG, &new_config);
    }

    pub fn update_tiers(env: Env, tiers: Vec<LockupTier>) {
        Self::only_admin(&env);
        env.storage()
            .persistent()
            .set(&DataKey::LockupTiers, &tiers);
    }

    pub fn transfer_admin(env: Env, new_admin: Address) {
        Self::only_admin(&env);
        env.storage().instance().set(&ADMIN, &new_admin);
    }

    // ── Staking ───────────────────────────────────────────────────────────────

    pub fn stake(
        env: Env,
        user: Address,
        amount: i128,
        lockup_duration: u64,
        auto_compound: bool,
    ) {
        user.require_auth();
        Self::not_paused(&env);

        let config: StakingConfig = env.storage().instance().get(&CONFIG).unwrap();
        if amount < config.min_stake_amount {
            panic!("below minimum stake");
        }

        let tiers: Vec<LockupTier> = env
            .storage()
            .persistent()
            .get(&DataKey::LockupTiers)
            .unwrap();

        let boost_bps = Self::get_boost_bps(&tiers, lockup_duration);
        let boosted = Self::apply_boost(amount, boost_bps);
        let current = env.ledger().sequence() as u64;

        Self::_update_all_rewards(&env, &user);

        let pos_key = DataKey::StakePosition(user.clone());
        let mut total_staked: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalStaked)
            .unwrap_or(0);
        let mut total_boosted: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalBoostedStake)
            .unwrap_or(0);

        if env.storage().persistent().has(&pos_key) {
            // compound existing before adding
            let mut pos: StakePosition = env.storage().persistent().get(&pos_key).unwrap();
            total_staked += amount;
            total_boosted = total_boosted - pos.boosted_amount + pos.boosted_amount + boosted - Self::apply_boost(pos.amount, Self::get_boost_bps(&tiers, pos.lockup_duration));
            pos.amount += amount;
            pos.boosted_amount += boosted;
            env.storage().persistent().set(&pos_key, &pos);
        } else {
            let pos = StakePosition {
                owner: user.clone(),
                amount,
                boosted_amount: boosted,
                lockup_duration,
                unlock_ledger: if lockup_duration == 0 { 0 } else { current + lockup_duration },
                start_ledger: current,
                auto_compound,
            };
            total_staked += amount;
            total_boosted += boosted;
            env.storage().persistent().set(&pos_key, &pos);
        }

        env.storage()
            .persistent()
            .set(&DataKey::TotalStaked, &total_staked);
        env.storage()
            .persistent()
            .set(&DataKey::TotalBoostedStake, &total_boosted);

        token::Client::new(&env, &config.staking_token).transfer(
            &user,
            &env.current_contract_address(),
            &amount,
        );

        env.events()
            .publish((symbol_short!("staked"), user), (amount, lockup_duration));
    }

    pub fn unstake(env: Env, user: Address, amount: i128) {
        user.require_auth();
        Self::not_paused(&env);

        let config: StakingConfig = env.storage().instance().get(&CONFIG).unwrap();
        let pos_key = DataKey::StakePosition(user.clone());
        let mut pos: StakePosition = env
            .storage()
            .persistent()
            .get(&pos_key)
            .expect("no stake found");

        if amount > pos.amount {
            panic!("insufficient staked balance");
        }

        let current = env.ledger().sequence() as u64;
        if pos.unlock_ledger > 0 && current < pos.unlock_ledger {
            panic!("stake is locked");
        }

        Self::_update_all_rewards(&env, &user);

        let tiers: Vec<LockupTier> = env
            .storage()
            .persistent()
            .get(&DataKey::LockupTiers)
            .unwrap();
        let boost_bps = Self::get_boost_bps(&tiers, pos.lockup_duration);
        let boosted_reduction = Self::apply_boost(amount, boost_bps);

        let mut total_staked: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalStaked)
            .unwrap_or(0);
        let mut total_boosted: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalBoostedStake)
            .unwrap_or(0);

        pos.amount -= amount;
        pos.boosted_amount -= boosted_reduction;
        total_staked -= amount;
        total_boosted -= boosted_reduction;

        if pos.amount == 0 {
            env.storage().persistent().remove(&pos_key);
        } else {
            env.storage().persistent().set(&pos_key, &pos);
        }

        env.storage()
            .persistent()
            .set(&DataKey::TotalStaked, &total_staked);
        env.storage()
            .persistent()
            .set(&DataKey::TotalBoostedStake, &total_boosted);

        token::Client::new(&env, &config.staking_token).transfer(
            &env.current_contract_address(),
            &user,
            &amount,
        );

        env.events()
            .publish((symbol_short!("unstaked"), user), amount);
    }

    // ── Emergency Withdraw ────────────────────────────────────────────────────

    pub fn emergency_withdraw(env: Env, user: Address) {
        user.require_auth();

        let config: StakingConfig = env.storage().instance().get(&CONFIG).unwrap();
        let pos_key = DataKey::StakePosition(user.clone());
        let pos: StakePosition = env
            .storage()
            .persistent()
            .get(&pos_key)
            .expect("no stake found");

        let tiers: Vec<LockupTier> = env
            .storage()
            .persistent()
            .get(&DataKey::LockupTiers)
            .unwrap();
        let boost_bps = Self::get_boost_bps(&tiers, pos.lockup_duration);

        let mut total_staked: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalStaked)
            .unwrap_or(0);
        let mut total_boosted: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalBoostedStake)
            .unwrap_or(0);

        total_staked -= pos.amount;
        total_boosted -= pos.boosted_amount;

        // forfeit all pending rewards
        let pools: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RewardPools)
            .unwrap_or_else(|| Vec::new(&env));
        for pool_addr in pools.iter() {
            let reward_key = DataKey::UserReward(user.clone(), pool_addr.clone());
            env.storage().persistent().remove(&reward_key);
        }

        // apply penalty fee
        let fee = (pos.amount * config.emergency_fee_bps as i128) / 10_000;
        let payout = pos.amount - fee;

        env.storage().persistent().remove(&pos_key);
        env.storage()
            .persistent()
            .set(&DataKey::TotalStaked, &total_staked);
        env.storage()
            .persistent()
            .set(&DataKey::TotalBoostedStake, &total_boosted);

        if fee > 0 {
            token::Client::new(&env, &config.staking_token).transfer(
                &env.current_contract_address(),
                &config.fee_recipient,
                &fee,
            );
        }

        token::Client::new(&env, &config.staking_token).transfer(
            &env.current_contract_address(),
            &user,
            &payout,
        );

        env.events().publish(
            (symbol_short!("emr_exit"), user),
            (payout, fee),
        );
    }

    // ── Rewards ───────────────────────────────────────────────────────────────

    pub fn claim_rewards(env: Env, user: Address) {
        user.require_auth();
        Self::not_paused(&env);
        Self::_update_all_rewards(&env, &user);
        Self::_pay_rewards(&env, &user);
    }

    pub fn compound(env: Env, user: Address) {
        user.require_auth();
        Self::not_paused(&env);

        let config: StakingConfig = env.storage().instance().get(&CONFIG).unwrap();
        let pos_key = DataKey::StakePosition(user.clone());

        if !env.storage().persistent().has(&pos_key) {
            panic!("no active stake");
        }

        let mut pos: StakePosition = env.storage().persistent().get(&pos_key).unwrap();
        if !pos.auto_compound {
            panic!("auto-compound not enabled for this position");
        }

        Self::_update_all_rewards(&env, &user);

        let pools: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RewardPools)
            .unwrap_or_else(|| Vec::new(&env));

        let staking_token = config.staking_token.clone();
        let mut compounded = 0i128;

        for pool_addr in pools.iter() {
            // only compound if reward token == staking token
            if pool_addr == staking_token {
                let reward_key = DataKey::UserReward(user.clone(), pool_addr.clone());
                let mut state: UserRewardState = env
                    .storage()
                    .persistent()
                    .get(&reward_key)
                    .unwrap_or(UserRewardState {
                        reward_per_token_paid: 0,
                        accrued: 0,
                    });

                let fee = (state.accrued * config.compound_fee_bps as i128) / 10_000;
                let net = state.accrued - fee;

                if net > 0 {
                    compounded += net;

                    if fee > 0 {
                        token::Client::new(&env, &pool_addr).transfer(
                            &env.current_contract_address(),
                            &config.fee_recipient,
                            &fee,
                        );
                    }

                    state.accrued = 0;
                    env.storage().persistent().set(&reward_key, &state);
                }
            }
        }

        if compounded == 0 {
            panic!("nothing to compound");
        }

        let tiers: Vec<LockupTier> = env
            .storage()
            .persistent()
            .get(&DataKey::LockupTiers)
            .unwrap();
        let boost_bps = Self::get_boost_bps(&tiers, pos.lockup_duration);
        let extra_boosted = Self::apply_boost(compounded, boost_bps);

        let mut total_staked: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalStaked)
            .unwrap_or(0);
        let mut total_boosted: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalBoostedStake)
            .unwrap_or(0);

        pos.amount += compounded;
        pos.boosted_amount += extra_boosted;
        total_staked += compounded;
        total_boosted += extra_boosted;

        env.storage().persistent().set(&pos_key, &pos);
        env.storage()
            .persistent()
            .set(&DataKey::TotalStaked, &total_staked);
        env.storage()
            .persistent()
            .set(&DataKey::TotalBoostedStake, &total_boosted);

        env.events().publish(
            (symbol_short!("compound"), user),
            compounded,
        );
    }

    // ── Views ─────────────────────────────────────────────────────────────────

    pub fn get_stake(env: Env, user: Address) -> StakePosition {
        env.storage()
            .persistent()
            .get(&DataKey::StakePosition(user))
            .expect("no stake found")
    }

    pub fn get_reward_pool(env: Env, reward_token: Address) -> RewardPool {
        env.storage()
            .persistent()
            .get(&DataKey::RewardPool(reward_token))
            .expect("pool not found")
    }

    pub fn get_reward_pools(env: Env) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::RewardPools)
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn pending_rewards(env: Env, user: Address, reward_token: Address) -> i128 {
        let total_boosted: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalBoostedStake)
            .unwrap_or(0);

        let pos: StakePosition = match env
            .storage()
            .persistent()
            .get(&DataKey::StakePosition(user.clone()))
        {
            Some(p) => p,
            None => return 0,
        };

        let pool: RewardPool = match env
            .storage()
            .persistent()
            .get(&DataKey::RewardPool(reward_token.clone()))
        {
            Some(p) => p,
            None => return 0,
        };

        let rpt = Self::calc_reward_per_token(&env, &pool, total_boosted);
        let user_state: UserRewardState = env
            .storage()
            .persistent()
            .get(&DataKey::UserReward(user, reward_token))
            .unwrap_or(UserRewardState {
                reward_per_token_paid: 0,
                accrued: 0,
            });

        user_state.accrued
            + (pos.boosted_amount * (rpt - user_state.reward_per_token_paid)) / 1_0000000
    }

    pub fn total_staked(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TotalStaked)
            .unwrap_or(0)
    }

    pub fn total_boosted_stake(env: Env) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::TotalBoostedStake)
            .unwrap_or(0)
    }

    pub fn get_lockup_tiers(env: Env) -> Vec<LockupTier> {
        env.storage()
            .persistent()
            .get(&DataKey::LockupTiers)
            .unwrap_or_else(|| Vec::new(&env))
    }

    pub fn get_config(env: Env) -> StakingConfig {
        env.storage().instance().get(&CONFIG).unwrap()
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn _update_all_rewards(env: &Env, user: &Address) {
        let total_boosted: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalBoostedStake)
            .unwrap_or(0);

        let pools: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RewardPools)
            .unwrap_or_else(|| Vec::new(env));

        let user_boosted: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::StakePosition(user.clone()))
            .map(|p: StakePosition| p.boosted_amount)
            .unwrap_or(0);

        for pool_addr in pools.iter() {
            let pool_key = DataKey::RewardPool(pool_addr.clone());
            let mut pool: RewardPool = env.storage().persistent().get(&pool_key).unwrap();

            let rpt = Self::calc_reward_per_token(env, &pool, total_boosted);
            Self::_update_reward_per_token(env, &mut pool, total_boosted);

            let reward_key = DataKey::UserReward(user.clone(), pool_addr.clone());
            let mut state: UserRewardState = env
                .storage()
                .persistent()
                .get(&reward_key)
                .unwrap_or(UserRewardState {
                    reward_per_token_paid: 0,
                    accrued: 0,
                });

            state.accrued +=
                (user_boosted * (rpt - state.reward_per_token_paid)) / 1_0000000;
            state.reward_per_token_paid = pool.reward_per_token_stored;

            env.storage().persistent().set(&reward_key, &state);
            env.storage().persistent().set(&pool_key, &pool);
        }
    }

    fn _update_reward_per_token(env: &Env, pool: &mut RewardPool, total_boosted: i128) {
        let rpt = Self::calc_reward_per_token(env, pool, total_boosted);
        pool.reward_per_token_stored = rpt;
        let current = env.ledger().sequence() as u64;
        pool.last_update_ledger = current.min(pool.period_finish);
    }

    fn calc_reward_per_token(env: &Env, pool: &RewardPool, total_boosted: i128) -> i128 {
        if total_boosted == 0 {
            return pool.reward_per_token_stored;
        }
        let current = env.ledger().sequence() as u64;
        let last = pool.last_update_ledger;
        let finish = pool.period_finish;
        let applicable = current.min(finish);
        if applicable <= last {
            return pool.reward_per_token_stored;
        }
        let elapsed = (applicable - last) as i128;
        pool.reward_per_token_stored
            + (elapsed * pool.reward_rate) / total_boosted
    }

    fn _pay_rewards(env: &Env, user: &Address) {
        let pools: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::RewardPools)
            .unwrap_or_else(|| Vec::new(env));

        for pool_addr in pools.iter() {
            let reward_key = DataKey::UserReward(user.clone(), pool_addr.clone());
            let mut state: UserRewardState = env
                .storage()
                .persistent()
                .get(&reward_key)
                .unwrap_or(UserRewardState {
                    reward_per_token_paid: 0,
                    accrued: 0,
                });

            if state.accrued > 0 {
                let payout = state.accrued;

                let mut pool: RewardPool = env
                    .storage()
                    .persistent()
                    .get(&DataKey::RewardPool(pool_addr.clone()))
                    .unwrap();
                pool.total_rewards_paid += payout;
                env.storage()
                    .persistent()
                    .set(&DataKey::RewardPool(pool_addr.clone()), &pool);

                state.accrued = 0;
                env.storage().persistent().set(&reward_key, &state);

                token::Client::new(env, &pool_addr).transfer(
                    &env.current_contract_address(),
                    user,
                    &payout,
                );

                env.events().publish(
                    (symbol_short!("rewarded"), user.clone()),
                    (pool_addr, payout),
                );
            }
        }
    }

    fn get_boost_bps(tiers: &Vec<LockupTier>, duration: u64) -> u32 {
        let mut best = 0u32;
        for tier in tiers.iter() {
            if duration >= tier.duration_ledgers && tier.boost_bps > best {
                best = tier.boost_bps;
            }
        }
        best
    }

    fn apply_boost(amount: i128, boost_bps: u32) -> i128 {
        amount + (amount * boost_bps as i128) / 10_000
    }

    fn only_admin(env: &Env) {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();
    }

    fn not_paused(env: &Env) {
        let paused: bool = env.storage().instance().get(&PAUSED).unwrap_or(false);
        if paused {
            panic!("contract is paused");
        }
    }
}
