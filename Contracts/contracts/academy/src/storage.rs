//! Optimized storage module for Academy Vesting Contract
//!
//! Storage optimizations:
//! - Instance storage for admin, token, governance, and counter (static data)
//! - Persistent storage for individual vesting schedules with indexed access
//! - User vesting index for efficient lookups
//! - Optimized key patterns using enum-based keys

use soroban_sdk::{contracttype, Address, Env, Symbol, Vec, symbol_short};

/// Contract version for migration tracking
const CONTRACT_VERSION: u32 = 2;

/// Storage keys using enum for type safety and efficiency
#[contracttype]
#[derive(Clone, Debug)]
pub enum AcademyDataKey {
    // ── V1 keys (unchanged) ────────────────────────────────────────────────
    Init,
    Admin,
    Token,
    Governance,
    Counter,
    Schedule(u64),                    // Individual V1 schedule by ID
    UserScheduleIds(Address),         // List of schedule IDs for a user (shared by V1 + V2)
    ActiveSchedules,                  // Index of active (non-claimed, non-revoked) schedules

    // ── V2 keys (issue #184) ───────────────────────────────────────────────
    ScheduleV2(u64),                  // Individual V2 schedule by ID (with performance triggers)
    UserTransferRestriction(Address), // Global transfer restriction flag per account
}

/// Storage manager for academy vesting contract
pub struct AcademyStorage;

impl AcademyStorage {
    // =========================================================================
    // Initialization
    // =========================================================================

    pub fn is_initialized(env: &Env) -> bool {
        env.storage().instance().has(&AcademyDataKey::Init)
    }

    pub fn set_initialized(env: &Env) {
        env.storage().instance().set(&AcademyDataKey::Init, &true);
        env.storage().instance().set(&AcademyDataKey::Counter, &0u64);
    }

    // =========================================================================
    // Version Management
    // =========================================================================

    pub fn get_version(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&Symbol::new(env, "version"))
            .unwrap_or(0)
    }

    pub fn set_version(env: &Env, version: u32) {
        env.storage()
            .instance()
            .set(&Symbol::new(env, "version"), &version);
    }

    // =========================================================================
    // Admin, Token & Governance
    // =========================================================================

    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&AcademyDataKey::Admin, admin);
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&AcademyDataKey::Admin)
    }

    pub fn set_token(env: &Env, token: &Address) {
        env.storage().instance().set(&AcademyDataKey::Token, token);
    }

    pub fn get_token(env: &Env) -> Option<Address> {
        env.storage().instance().get(&AcademyDataKey::Token)
    }

    pub fn set_governance(env: &Env, governance: &Address) {
        env.storage()
            .instance()
            .set(&AcademyDataKey::Governance, governance);
    }

    pub fn get_governance(env: &Env) -> Option<Address> {
        env.storage().instance().get(&AcademyDataKey::Governance)
    }

    // =========================================================================
    // Counter
    // =========================================================================

    pub fn get_counter(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&AcademyDataKey::Counter)
            .unwrap_or(0)
    }

    /// Atomically increment and return the next schedule ID.
    /// Used by both V1 and V2 grants so IDs are globally unique.
    pub fn increment_counter(env: &Env) -> u64 {
        let current = Self::get_counter(env);
        let next = current + 1;
        env.storage()
            .instance()
            .set(&AcademyDataKey::Counter, &next);
        next
    }

    // =========================================================================
    // V1 Schedule Storage
    // =========================================================================

    /// Store individual V1 vesting schedule
    pub fn set_schedule<T: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(
        env: &Env,
        schedule_id: u64,
        schedule: &T,
    ) {
        env.storage()
            .persistent()
            .set(&AcademyDataKey::Schedule(schedule_id), schedule);
    }

    /// Get V1 vesting schedule by ID
    pub fn get_schedule<T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val> + Clone>(
        env: &Env,
        schedule_id: u64,
    ) -> Option<T> {
        env.storage()
            .persistent()
            .get(&AcademyDataKey::Schedule(schedule_id))
    }

    /// Check if a V1 schedule exists
    pub fn has_schedule(env: &Env, schedule_id: u64) -> bool {
        env.storage()
            .persistent()
            .has(&AcademyDataKey::Schedule(schedule_id))
    }

    // =========================================================================
    // V2 Schedule Storage (issue #184)
    // =========================================================================

    /// Store individual V2 vesting schedule (with performance triggers)
    pub fn set_schedule_v2<T: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(
        env: &Env,
        schedule_id: u64,
        schedule: &T,
    ) {
        env.storage()
            .persistent()
            .set(&AcademyDataKey::ScheduleV2(schedule_id), schedule);
    }

    /// Get V2 vesting schedule by ID
    pub fn get_schedule_v2<T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val> + Clone>(
        env: &Env,
        schedule_id: u64,
    ) -> Option<T> {
        env.storage()
            .persistent()
            .get(&AcademyDataKey::ScheduleV2(schedule_id))
    }

    /// Check if a V2 schedule exists
    pub fn has_schedule_v2(env: &Env, schedule_id: u64) -> bool {
        env.storage()
            .persistent()
            .has(&AcademyDataKey::ScheduleV2(schedule_id))
    }

    // =========================================================================
    // Transfer Restrictions (issue #184)
    // =========================================================================

    /// Set a global transfer restriction for an account.
    /// When restricted, the account cannot call claim or claim_v2.
    pub fn set_user_transfer_restriction(env: &Env, user: &Address, restricted: bool) {
        env.storage().persistent().set(
            &AcademyDataKey::UserTransferRestriction(user.clone()),
            &restricted,
        );
    }

    /// Returns true if the account is globally transfer-restricted.
    pub fn is_transfer_restricted(env: &Env, user: &Address) -> bool {
        env.storage()
            .persistent()
            .get(&AcademyDataKey::UserTransferRestriction(user.clone()))
            .unwrap_or(false)
    }

    // =========================================================================
    // User Schedule Index (shared by V1 and V2)
    // =========================================================================

    /// Add a schedule ID to a user's index (called for both V1 and V2 grants).
    pub fn add_schedule_to_user_index(env: &Env, user: &Address, schedule_id: u64) {
        let key = AcademyDataKey::UserScheduleIds(user.clone());
        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));
        ids.push_back(schedule_id);
        env.storage().persistent().set(&key, &ids);
    }

    /// Get all schedule IDs belonging to a user (V1 + V2 combined).
    pub fn get_user_schedule_ids(env: &Env, user: &Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&AcademyDataKey::UserScheduleIds(user.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Lazy-load V1 schedules for a user.
    pub fn get_user_schedules<T>(env: &Env, user: &Address) -> Vec<T>
    where
        T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>
            + Clone
            + soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
    {
        let ids = Self::get_user_schedule_ids(env, user);
        let mut schedules = Vec::new(env);
        for id in ids.iter() {
            if let Some(s) = Self::get_schedule::<T>(env, id) {
                schedules.push_back(s);
            }
        }
        schedules
    }

    /// Lazy-load V2 schedules for a user.
    pub fn get_user_schedules_v2<T>(env: &Env, user: &Address) -> Vec<T>
    where
        T: soroban_sdk::TryFromVal<Env, soroban_sdk::Val>
            + Clone
            + soroban_sdk::IntoVal<Env, soroban_sdk::Val>,
    {
        let ids = Self::get_user_schedule_ids(env, user);
        let mut schedules = Vec::new(env);
        for id in ids.iter() {
            if let Some(s) = Self::get_schedule_v2::<T>(env, id) {
                schedules.push_back(s);
            }
        }
        schedules
    }

    // =========================================================================
    // Active Schedules Index
    // =========================================================================

    /// Add a schedule ID to the active index.
    pub fn add_to_active_index(env: &Env, schedule_id: u64) {
        let mut active: Vec<u64> = env
            .storage()
            .persistent()
            .get(&AcademyDataKey::ActiveSchedules)
            .unwrap_or_else(|| Vec::new(env));
        active.push_back(schedule_id);
        env.storage()
            .persistent()
            .set(&AcademyDataKey::ActiveSchedules, &active);
    }

    /// Remove a schedule ID from the active index (called on claim or revoke).
    pub fn remove_from_active_index(env: &Env, schedule_id: u64) {
        let active: Vec<u64> = env
            .storage()
            .persistent()
            .get(&AcademyDataKey::ActiveSchedules)
            .unwrap_or_else(|| Vec::new(env));

        let mut new_active = Vec::new(env);
        for id in active.iter() {
            if id != schedule_id {
                new_active.push_back(id);
            }
        }
        env.storage()
            .persistent()
            .set(&AcademyDataKey::ActiveSchedules, &new_active);
    }

    /// Get all currently active schedule IDs (V1 + V2 combined).
    pub fn get_active_schedule_ids(env: &Env) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&AcademyDataKey::ActiveSchedules)
            .unwrap_or_else(|| Vec::new(env))
    }

    // =========================================================================
    // Migration Support
    // =========================================================================

    /// Returns true if the stored version is behind the current contract version.
    pub fn needs_migration(env: &Env) -> bool {
        Self::get_version(env) < CONTRACT_VERSION
    }

    /// Run storage migration and return the number of records migrated.
    pub fn migrate_storage(env: &Env) -> u64 {
        let current_version = Self::get_version(env);

        if current_version == 0 {
            Self::set_version(env, CONTRACT_VERSION);
            0
        } else if current_version == 1 {
            let migrated = Self::migrate_from_legacy(env);
            Self::set_version(env, CONTRACT_VERSION);
            migrated
        } else {
            0
        }
    }

    fn migrate_from_legacy(env: &Env) -> u64 {
        let mut migrated = 0u64;

        let legacy_admin_key = symbol_short!("admin");
        if let Some(admin) = env
            .storage()
            .persistent()
            .get::<_, Address>(&legacy_admin_key)
        {
            Self::set_admin(env, &admin);
            migrated += 1;
        }

        let legacy_token_key = symbol_short!("token");
        if let Some(token) = env
            .storage()
            .persistent()
            .get::<_, Address>(&legacy_token_key)
        {
            Self::set_token(env, &token);
            migrated += 1;
        }

        let legacy_gov_key = symbol_short!("gov");
        if let Some(gov) = env
            .storage()
            .persistent()
            .get::<_, Address>(&legacy_gov_key)
        {
            Self::set_governance(env, &gov);
            migrated += 1;
        }

        let legacy_counter_key = symbol_short!("cnt");
        if let Some(counter) = env
            .storage()
            .persistent()
            .get::<_, u64>(&legacy_counter_key)
        {
            env.storage()
                .instance()
                .set(&AcademyDataKey::Counter, &counter);
            migrated += 1;
        }

        migrated
    }

    /// Returns true if any legacy key exists in persistent storage.
    pub fn has_legacy_data(env: &Env) -> bool {
        env.storage()
            .persistent()
            .has(&symbol_short!("admin"))
            || env.storage().persistent().has(&symbol_short!("token"))
            || env.storage().persistent().has(&symbol_short!("gov"))
            || env.storage().persistent().has(&symbol_short!("cnt"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_key_variants() {
        let key1 = AcademyDataKey::Init;
        let key2 = AcademyDataKey::Counter;
        let key3 = AcademyDataKey::Schedule(1);
        let key4 = AcademyDataKey::ScheduleV2(42);

        match key1 {
            AcademyDataKey::Init => (),
            _ => panic!("Expected Init"),
        }
        match key2 {
            AcademyDataKey::Counter => (),
            _ => panic!("Expected Counter"),
        }
        match key3 {
            AcademyDataKey::Schedule(id) => assert_eq!(id, 1),
            _ => panic!("Expected Schedule"),
        }
        match key4 {
            AcademyDataKey::ScheduleV2(id) => assert_eq!(id, 42),
            _ => panic!("Expected ScheduleV2"),
        }
    }
}