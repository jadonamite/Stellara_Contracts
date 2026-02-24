use soroban_sdk::{
    contract, contractimpl, symbol_short, token, Address, Bytes, BytesN, Env, String, Vec,
};

use crate::security::{
    calculate_fee, effective_required_approvals, get_validator_pubkey,
    set_validator_pubkey, validate_threshold, verify_validator_signature,
    VALIDATOR_UPGRADE_TIMELOCK_SECONDS,
};
use crate::storage::{BridgeStorage, chain_to_id};
use crate::types::{
    AssetRegisteredEvent, BridgeCompletedEvent, BridgeDirection, BridgeError,
    BridgeInitiatedEvent, BridgeRejectedEvent, BridgeRequest, BridgeRequestStatus, BridgeStats,
    ChainConfig, EmergencyPauseEvent, ExternalChain, PendingValidatorUpgrade,
    ValidatorAddedEvent, ValidatorRemovedEvent, ValidatorSet, ValidatorSignature,
    ValidatorUpgradeAppliedEvent, ValidatorUpgradeProposedEvent, ValidatorVoteEvent,
    WrappedAsset,
};

#[contract]
pub struct CrossChainBridgeContract;

#[contractimpl]
impl CrossChainBridgeContract {

    // ═══════════════════════════════════════════════════════════════════════════
    // ── Initialization ────────────────────────────────────────────────────────
    // ═══════════════════════════════════════════════════════════════════════════

    /// Deploy and configure the bridge.
    ///
    /// * `admin`      — can register assets, add/remove validators, pause bridge
    /// * `fee_collector` — receives bridge fees
    /// * `validators` — initial validator set (min 3)
    /// * `threshold`  — required approvals (must be > 2/3 of validator count)
    /// * `validator_pubkeys` — parallel list of Ed25519 pubkeys (one per validator)
    pub fn initialize(
        env: Env,
        admin: Address,
        fee_collector: Address,
        validators: Vec<Address>,
        threshold: u32,
        validator_pubkeys: Vec<BytesN<32>>,
    ) -> Result<(), BridgeError> {
        if BridgeStorage::is_initialized(&env) {
            return Err(BridgeError::AlreadyInitialized);
        }

        admin.require_auth();

        // Validate validator set
        let count = validators.len() as u32;
        validate_threshold(threshold, count)?;

        if validator_pubkeys.len() != count {
            return Err(BridgeError::NotValidator); // pubkey count must match
        }

        // Store each validator's signing pubkey
        for i in 0..validators.len() {
            set_validator_pubkey(
                &env,
                &validators.get(i).unwrap(),
                &validator_pubkeys.get(i).unwrap(),
            );
        }

        let validator_set = ValidatorSet {
            validators,
            threshold,
            version: 1,
            updated_at: env.ledger().timestamp(),
        };

        BridgeStorage::set_admin(&env, &admin);
        BridgeStorage::set_fee_collector(&env, &fee_collector);
        BridgeStorage::set_validator_set(&env, &validator_set);
        BridgeStorage::set_initialized(&env);

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ── Asset registration ────────────────────────────────────────────────────
    // ═══════════════════════════════════════════════════════════════════════════

    /// Register a Stellar token as bridgeable to/from an external chain.
    pub fn register_asset(
        env: Env,
        admin: Address,
        stellar_asset: Address,
        external_chain: ExternalChain,
        external_contract: Bytes,
        decimals_stellar: u32,
        decimals_external: u32,
    ) -> Result<(), BridgeError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;
        Self::require_not_paused(&env)?;

        if BridgeStorage::get_wrapped_asset(&env, &stellar_asset, &external_chain).is_some() {
            return Err(BridgeError::AssetAlreadyRegistered);
        }

        Self::require_chain_active(&env, &external_chain)?;

        let asset = WrappedAsset {
            stellar_asset: stellar_asset.clone(),
            external_chain: external_chain.clone(),
            external_contract: external_contract.clone(),
            decimals_stellar,
            decimals_external,
            total_locked: 0,
            total_minted: 0,
            is_active: true,
            backing_ratio_bps: 10_000, // starts at 100%
            registered_at: env.ledger().timestamp(),
            registered_by: admin.clone(),
        };

        BridgeStorage::set_wrapped_asset(&env, &asset);

        env.events().publish(
            (symbol_short!("reg_asset"),),
            AssetRegisteredEvent {
                stellar_asset,
                external_chain,
                external_contract,
                registered_by: admin,
            },
        );

        Ok(())
    }

    /// Enable or disable an already-registered asset.
    pub fn set_asset_active(
        env: Env,
        admin: Address,
        stellar_asset: Address,
        external_chain: ExternalChain,
        active: bool,
    ) -> Result<(), BridgeError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut asset =
            BridgeStorage::get_wrapped_asset(&env, &stellar_asset, &external_chain)
                .ok_or(BridgeError::AssetNotRegistered)?;

        asset.is_active = active;
        BridgeStorage::set_wrapped_asset(&env, &asset);
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ── Chain configuration ───────────────────────────────────────────────────
    // ═══════════════════════════════════════════════════════════════════════════

    /// Add or update a chain's operational configuration.
    pub fn configure_chain(
        env: Env,
        admin: Address,
        chain: ExternalChain,
        min_confirmations: u32,
        max_transfer_amount: i128,
        daily_limit: i128,
        fee_bps: u32,
        expiry_seconds: u64,
    ) -> Result<(), BridgeError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let now = env.ledger().timestamp();
        let existing = BridgeStorage::get_chain_config(&env, &chain);

        let config = ChainConfig {
            chain,
            is_active: true,
            min_confirmations,
            max_transfer_amount,
            daily_limit,
            daily_volume: existing.as_ref().map(|c| c.daily_volume).unwrap_or(0),
            window_start: existing.as_ref().map(|c| c.window_start).unwrap_or(now),
            fee_bps,
            expiry_seconds,
        };

        BridgeStorage::set_chain_config(&env, &config);
        Ok(())
    }

    pub fn set_chain_active(
        env: Env,
        admin: Address,
        chain: ExternalChain,
        active: bool,
    ) -> Result<(), BridgeError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut config = BridgeStorage::get_chain_config(&env, &chain)
            .ok_or(BridgeError::ChainNotSupported)?;
        config.is_active = active;
        BridgeStorage::set_chain_config(&env, &config);
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ── Outbound: Stellar → External chain ───────────────────────────────────
    // ═══════════════════════════════════════════════════════════════════════════

    /// Lock Stellar tokens to trigger minting on the external chain.
    ///
    /// Flow:
    ///   1. User calls `initiate_outbound`
    ///   2. Contract locks tokens (transfers from user to contract)
    ///   3. Relayer picks up the event
    ///   4. Validators observe the lock on Stellar and sign approval
    ///   5. `submit_validator_vote` is called with each signature
    ///   6. On threshold → `complete_outbound` releases on the external chain
    ///      (this is handled by the off-chain relayer; the Stellar side is done
    ///       once tokens are locked)
    pub fn initiate_outbound(
        env: Env,
        initiator: Address,
        stellar_asset: Address,
        amount: i128,
        external_chain: ExternalChain,
        external_address: Bytes,
    ) -> Result<u64, BridgeError> {
        initiator.require_auth();
        Self::require_not_paused(&env)?;

        if amount <= 0 {
            return Err(BridgeError::AmountTooSmall);
        }

        // Chain checks
        let mut chain_config = BridgeStorage::get_chain_config(&env, &external_chain)
            .ok_or(BridgeError::ChainNotSupported)?;
        if !chain_config.is_active {
            return Err(BridgeError::ChainInactive);
        }
        if amount > chain_config.max_transfer_amount {
            return Err(BridgeError::AmountExceedsMax);
        }

        // Asset checks
        let mut asset =
            BridgeStorage::get_wrapped_asset(&env, &stellar_asset, &external_chain)
                .ok_or(BridgeError::AssetNotRegistered)?;
        if !asset.is_active {
            return Err(BridgeError::AssetInactive);
        }

        // Daily limit
        chain_config = BridgeStorage::refresh_daily_window(&env, chain_config);
        if chain_config.daily_volume + amount > chain_config.daily_limit {
            return Err(BridgeError::DailyLimitExceeded);
        }

        // Fee
        let (fee, net) = calculate_fee(amount, chain_config.fee_bps);

        // Validator set for required approvals
        let validator_set = BridgeStorage::get_validator_set(&env)
            .ok_or(BridgeError::NotInitialized)?;
        let required = effective_required_approvals(
            &validator_set,
            amount,
            chain_config.max_transfer_amount,
        );

        let now = env.ledger().timestamp();
        let request_id = BridgeStorage::next_request_id(&env);

        let request = BridgeRequest {
            request_id,
            direction: BridgeDirection::OutboundToExternal,
            initiator: initiator.clone(),
            stellar_asset: stellar_asset.clone(),
            amount,
            fee_amount: fee,
            net_amount: net,
            external_chain: external_chain.clone(),
            external_address: external_address.clone(),
            external_tx_hash: Bytes::new(&env), // empty for outbound
            status: BridgeRequestStatus::Pending,
            created_at: now,
            expires_at: now + chain_config.expiry_seconds,
            completed_at: 0,
            approval_count: 0,
            rejection_count: 0,
            required_approvals: required,
        };

        // Lock tokens from user
        let token_client = token::Client::new(&env, &stellar_asset);
        let balance = token_client.balance(&initiator);
        if balance < amount {
            return Err(BridgeError::InsufficientBalance);
        }
        token_client.transfer(&initiator, &env.current_contract_address(), &amount);

        // Update state
        asset.total_locked += amount;
        chain_config.daily_volume += amount;
        BridgeStorage::set_wrapped_asset(&env, &asset);
        BridgeStorage::set_chain_config(&env, &chain_config);
        BridgeStorage::set_request(&env, &request);

        let mut stats = BridgeStorage::get_stats(&env);
        stats.total_requests += 1;
        stats.total_volume += amount;
        stats.total_fees_collected += fee;
        BridgeStorage::set_stats(&env, &stats);

        // Transfer fee immediately to collector
        if fee > 0 {
            if let Some(collector) = BridgeStorage::get_fee_collector(&env) {
                token_client.transfer(&env.current_contract_address(), &collector, &fee);
            }
        }

        env.events().publish(
            (symbol_short!("outbound"),),
            BridgeInitiatedEvent {
                request_id,
                direction: BridgeDirection::OutboundToExternal,
                initiator,
                stellar_asset,
                amount,
                fee_amount: fee,
                external_chain,
                external_address,
                expires_at: now + chain_config.expiry_seconds,
            },
        );

        Ok(request_id)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ── Inbound: External chain → Stellar ────────────────────────────────────
    // ═══════════════════════════════════════════════════════════════════════════

    /// Submit proof that tokens were locked on an external chain.
    /// The relayer (or user) calls this with the external TX hash as evidence.
    /// Validators then vote; on threshold the contract mints wrapped tokens.
    pub fn initiate_inbound(
        env: Env,
        initiator: Address,
        stellar_asset: Address,
        amount: i128,
        external_chain: ExternalChain,
        external_address: Bytes,
        external_tx_hash: Bytes,
    ) -> Result<u64, BridgeError> {
        initiator.require_auth();
        Self::require_not_paused(&env)?;

        if amount <= 0 {
            return Err(BridgeError::AmountTooSmall);
        }

        // Replay protection
        if BridgeStorage::is_external_tx_processed(&env, &external_tx_hash) {
            return Err(BridgeError::DuplicateExternalTx);
        }

        // Chain checks
        let mut chain_config = BridgeStorage::get_chain_config(&env, &external_chain)
            .ok_or(BridgeError::ChainNotSupported)?;
        if !chain_config.is_active {
            return Err(BridgeError::ChainInactive);
        }
        if amount > chain_config.max_transfer_amount {
            return Err(BridgeError::AmountExceedsMax);
        }

        // Asset checks
        let asset = BridgeStorage::get_wrapped_asset(&env, &stellar_asset, &external_chain)
            .ok_or(BridgeError::AssetNotRegistered)?;
        if !asset.is_active {
            return Err(BridgeError::AssetInactive);
        }

        // Daily limit
        chain_config = BridgeStorage::refresh_daily_window(&env, chain_config);
        if chain_config.daily_volume + amount > chain_config.daily_limit {
            return Err(BridgeError::DailyLimitExceeded);
        }

        let (fee, net) = calculate_fee(amount, chain_config.fee_bps);

        let validator_set = BridgeStorage::get_validator_set(&env)
            .ok_or(BridgeError::NotInitialized)?;
        let required = effective_required_approvals(
            &validator_set,
            amount,
            chain_config.max_transfer_amount,
        );

        let now = env.ledger().timestamp();
        let request_id = BridgeStorage::next_request_id(&env);

        let request = BridgeRequest {
            request_id,
            direction: BridgeDirection::InboundFromExternal,
            initiator: initiator.clone(),
            stellar_asset: stellar_asset.clone(),
            amount,
            fee_amount: fee,
            net_amount: net,
            external_chain: external_chain.clone(),
            external_address: external_address.clone(),
            external_tx_hash: external_tx_hash.clone(),
            status: BridgeRequestStatus::Pending,
            created_at: now,
            expires_at: now + chain_config.expiry_seconds,
            completed_at: 0,
            approval_count: 0,
            rejection_count: 0,
            required_approvals: required,
        };

        chain_config.daily_volume += amount;
        BridgeStorage::set_chain_config(&env, &chain_config);
        BridgeStorage::set_request(&env, &request);

        let mut stats = BridgeStorage::get_stats(&env);
        stats.total_requests += 1;
        stats.total_volume += amount;
        stats.total_fees_collected += fee;
        BridgeStorage::set_stats(&env, &stats);

        env.events().publish(
            (symbol_short!("inbound"),),
            BridgeInitiatedEvent {
                request_id,
                direction: BridgeDirection::InboundFromExternal,
                initiator,
                stellar_asset,
                amount,
                fee_amount: fee,
                external_chain,
                external_address,
                expires_at: now + chain_config.expiry_seconds,
            },
        );

        Ok(request_id)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ── Validator voting ──────────────────────────────────────────────────────
    // ═══════════════════════════════════════════════════════════════════════════

    /// A validator submits their signed vote on a bridge request.
    ///
    /// When the approval count reaches `required_approvals` the request is
    /// automatically finalised (tokens minted or released).
    /// When the rejection count exceeds (validator_count - threshold) the
    /// request is rejected and outbound tokens are returned.
    pub fn submit_validator_vote(
        env: Env,
        validator: Address,
        request_id: u64,
        approved: bool,
        signature: BytesN<64>,
    ) -> Result<BridgeRequestStatus, BridgeError> {
        validator.require_auth();
        Self::require_not_paused(&env)?;

        // Verify validator membership
        if !BridgeStorage::is_validator(&env, &validator) {
            return Err(BridgeError::NotValidator);
        }

        // Fetch and validate request
        let mut request = BridgeStorage::get_request(&env, request_id)
            .ok_or(BridgeError::RequestNotFound)?;

        if request.status != BridgeRequestStatus::Pending {
            return Err(BridgeError::RequestAlreadyProcessed);
        }

        // Expiry check
        let now = env.ledger().timestamp();
        if now > request.expires_at {
            request.status = BridgeRequestStatus::Expired;
            BridgeStorage::set_request(&env, &request);
            // Return locked tokens for expired outbound requests
            if request.direction == BridgeDirection::OutboundToExternal {
                Self::return_locked_tokens(&env, &request)?;
            }
            return Ok(BridgeRequestStatus::Expired);
        }

        // Duplicate vote check
        if BridgeStorage::has_voted(&env, request_id, &validator) {
            return Err(BridgeError::AlreadyVoted);
        }

        // Signature verification
        let pubkey = get_validator_pubkey(&env, &validator)
            .ok_or(BridgeError::NotValidator)?;
        verify_validator_signature(&env, &pubkey, &request, &signature)?;

        // Record the vote
        let sig = ValidatorSignature {
            validator: validator.clone(),
            request_id,
            approved,
            signed_at: now,
            signature,
        };
        BridgeStorage::record_vote(&env, request_id, &validator, &sig);

        if approved {
            request.approval_count += 1;
        } else {
            request.rejection_count += 1;
        }

        let validator_set = BridgeStorage::get_validator_set(&env)
            .ok_or(BridgeError::NotInitialized)?;
        let validator_count = validator_set.validators.len() as u32;

        env.events().publish(
            (symbol_short!("vote"),),
            ValidatorVoteEvent {
                request_id,
                validator: validator.clone(),
                approved,
                approval_count: request.approval_count,
                rejection_count: request.rejection_count,
                required_approvals: request.required_approvals,
            },
        );

        // ── Threshold reached → approve ───────────────────────────────────────
        if request.approval_count >= request.required_approvals {
            request.status = BridgeRequestStatus::Approved;
            BridgeStorage::set_request(&env, &request);
            let status = Self::finalise_request(&env, &mut request)?;
            return Ok(status);
        }

        // ── Enough rejections → reject ────────────────────────────────────────
        // Reject if remaining possible approvals can never reach threshold.
        let votes_cast = request.approval_count + request.rejection_count;
        let remaining = validator_count.saturating_sub(votes_cast);
        if request.approval_count + remaining < request.required_approvals {
            request.status = BridgeRequestStatus::Rejected;
            BridgeStorage::set_request(&env, &request);

            // Return locked tokens for rejected outbound requests
            if request.direction == BridgeDirection::OutboundToExternal {
                Self::return_locked_tokens(&env, &request)?;
            }

            let mut stats = BridgeStorage::get_stats(&env);
            stats.total_rejected += 1;
            BridgeStorage::set_stats(&env, &stats);

            env.events().publish(
                (symbol_short!("rejected"),),
                BridgeRejectedEvent {
                    request_id,
                    rejection_count: request.rejection_count,
                    rejected_at: now,
                },
            );

            return Ok(BridgeRequestStatus::Rejected);
        }

        BridgeStorage::set_request(&env, &request);
        Ok(BridgeRequestStatus::Pending)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ── Cancel ───────────────────────────────────────────────────────────────
    // ═══════════════════════════════════════════════════════════════════════════

    /// Cancel a pending request and refund the initiator (outbound only).
    pub fn cancel_request(
        env: Env,
        initiator: Address,
        request_id: u64,
    ) -> Result<(), BridgeError> {
        initiator.require_auth();

        let mut request = BridgeStorage::get_request(&env, request_id)
            .ok_or(BridgeError::RequestNotFound)?;

        if request.initiator != initiator {
            return Err(BridgeError::Unauthorized);
        }
        if request.status != BridgeRequestStatus::Pending {
            return Err(BridgeError::RequestNotPending);
        }

        request.status = BridgeRequestStatus::Cancelled;
        BridgeStorage::set_request(&env, &request);

        if request.direction == BridgeDirection::OutboundToExternal {
            Self::return_locked_tokens(&env, &request)?;
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ── Validator management ──────────────────────────────────────────────────
    // ═══════════════════════════════════════════════════════════════════════════

    /// Propose a full validator-set replacement (time-locked 48 h).
    pub fn propose_validator_upgrade(
        env: Env,
        admin: Address,
        new_validators: Vec<Address>,
        new_threshold: u32,
        new_pubkeys: Vec<BytesN<32>>,
    ) -> Result<u64, BridgeError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let count = new_validators.len() as u32;
        validate_threshold(new_threshold, count)?;

        if new_pubkeys.len() != count {
            return Err(BridgeError::NotValidator);
        }

        let now = env.ledger().timestamp();
        let effective_at = now + VALIDATOR_UPGRADE_TIMELOCK_SECONDS;

        let upgrade = PendingValidatorUpgrade {
            proposed_validators: new_validators.clone(),
            proposed_threshold: new_threshold,
            proposed_at: now,
            effective_at,
            proposer: admin.clone(),
        };

        BridgeStorage::set_pending_upgrade(&env, &upgrade);

        // Pre-register pubkeys so they are ready on apply
        for i in 0..new_validators.len() {
            set_validator_pubkey(
                &env,
                &new_validators.get(i).unwrap(),
                &new_pubkeys.get(i).unwrap(),
            );
        }

        env.events().publish(
            (symbol_short!("v_prop"),),
            ValidatorUpgradeProposedEvent {
                proposed_by: admin,
                effective_at,
                validator_count: count,
                threshold: new_threshold,
            },
        );

        Ok(effective_at)
    }

    /// Apply a previously proposed validator-set upgrade after the timelock.
    pub fn apply_validator_upgrade(
        env: Env,
        admin: Address,
    ) -> Result<u32, BridgeError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let upgrade = BridgeStorage::get_pending_upgrade(&env)
            .ok_or(BridgeError::NoPendingUpgrade)?;

        let now = env.ledger().timestamp();
        if now < upgrade.effective_at {
            return Err(BridgeError::UpgradeTimelockActive);
        }

        let mut current = BridgeStorage::get_validator_set(&env)
            .ok_or(BridgeError::NotInitialized)?;

        let new_version = current.version + 1;
        let new_set = ValidatorSet {
            validators: upgrade.proposed_validators.clone(),
            threshold: upgrade.proposed_threshold,
            version: new_version,
            updated_at: now,
        };

        BridgeStorage::set_validator_set(&env, &new_set);
        BridgeStorage::clear_pending_upgrade(&env);

        env.events().publish(
            (symbol_short!("v_apply"),),
            ValidatorUpgradeAppliedEvent {
                applied_by: admin,
                new_version,
                validator_count: upgrade.proposed_validators.len() as u32,
                threshold: upgrade.proposed_threshold,
                applied_at: now,
            },
        );

        Ok(new_version)
    }

    /// Register a single additional validator (admin shortcut — no timelock).
    /// Use `propose_validator_upgrade` for full set replacements.
    pub fn add_validator(
        env: Env,
        admin: Address,
        validator: Address,
        pubkey: BytesN<32>,
        new_threshold: u32,
    ) -> Result<(), BridgeError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut set = BridgeStorage::get_validator_set(&env)
            .ok_or(BridgeError::NotInitialized)?;

        // Duplicate check
        if set.validators.iter().any(|v| v == validator) {
            return Err(BridgeError::ValidatorAlreadyExists);
        }

        set.validators.push_back(validator.clone());
        let count = set.validators.len() as u32;
        validate_threshold(new_threshold, count)?;

        set.threshold = new_threshold;
        set.version += 1;
        set.updated_at = env.ledger().timestamp();

        set_validator_pubkey(&env, &validator, &pubkey);
        BridgeStorage::set_validator_set(&env, &set);

        env.events().publish(
            (symbol_short!("v_add"),),
            ValidatorAddedEvent {
                validator,
                new_threshold,
                validator_set_version: set.version,
                added_by: admin,
            },
        );

        Ok(())
    }

    /// Remove a validator (admin shortcut — keeps remaining set valid).
    pub fn remove_validator(
        env: Env,
        admin: Address,
        validator: Address,
        new_threshold: u32,
    ) -> Result<(), BridgeError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut set = BridgeStorage::get_validator_set(&env)
            .ok_or(BridgeError::NotInitialized)?;

        let before_len = set.validators.len();
        let mut new_validators = Vec::new(&env);
        for v in set.validators.iter() {
            if v != validator {
                new_validators.push_back(v);
            }
        }

        if new_validators.len() == before_len {
            return Err(BridgeError::ValidatorNotFound);
        }

        let count = new_validators.len() as u32;
        validate_threshold(new_threshold, count)?;

        set.validators = new_validators;
        set.threshold = new_threshold;
        set.version += 1;
        set.updated_at = env.ledger().timestamp();

        BridgeStorage::set_validator_set(&env, &set);

        env.events().publish(
            (symbol_short!("v_rm"),),
            ValidatorRemovedEvent {
                validator,
                new_threshold,
                validator_set_version: set.version,
                removed_by: admin,
            },
        );

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ── Emergency controls ────────────────────────────────────────────────────
    // ═══════════════════════════════════════════════════════════════════════════

    /// Pause or unpause all bridge operations (admin only).
    pub fn set_pause(
        env: Env,
        admin: Address,
        paused: bool,
        reason: String,
    ) -> Result<(), BridgeError> {
        admin.require_auth();
        Self::require_admin(&env, &admin)?;

        let mut stats = BridgeStorage::get_stats(&env);
        stats.is_paused = paused;
        stats.pause_reason = reason.clone();
        BridgeStorage::set_stats(&env, &stats);

        env.events().publish(
            (symbol_short!("pause"),),
            EmergencyPauseEvent {
                paused,
                reason,
                triggered_by: admin,
                triggered_at: env.ledger().timestamp(),
            },
        );

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ── Queries ───────────────────────────────────────────────────────────────
    // ═══════════════════════════════════════════════════════════════════════════

    pub fn get_request(env: Env, request_id: u64) -> Result<BridgeRequest, BridgeError> {
        BridgeStorage::get_request(&env, request_id)
            .ok_or(BridgeError::RequestNotFound)
    }

    pub fn get_wrapped_asset(
        env: Env,
        stellar_asset: Address,
        external_chain: ExternalChain,
    ) -> Result<WrappedAsset, BridgeError> {
        BridgeStorage::get_wrapped_asset(&env, &stellar_asset, &external_chain)
            .ok_or(BridgeError::AssetNotRegistered)
    }

    pub fn get_chain_config(
        env: Env,
        chain: ExternalChain,
    ) -> Result<ChainConfig, BridgeError> {
        BridgeStorage::get_chain_config(&env, &chain)
            .ok_or(BridgeError::ChainNotSupported)
    }

    pub fn get_validator_set(env: Env) -> Result<ValidatorSet, BridgeError> {
        BridgeStorage::get_validator_set(&env).ok_or(BridgeError::NotInitialized)
    }

    pub fn get_stats(env: Env) -> BridgeStats {
        BridgeStorage::get_stats(&env)
    }

    pub fn get_nonce(env: Env, user: Address) -> u64 {
        BridgeStorage::get_nonce(&env, &user)
    }

    pub fn get_validator_slash_count(env: Env, validator: Address) -> u32 {
        BridgeStorage::get_slash_count(&env, &validator)
    }

    pub fn has_validator_voted(env: Env, request_id: u64, validator: Address) -> bool {
        BridgeStorage::has_voted(&env, request_id, &validator)
    }

    pub fn is_paused(env: Env) -> bool {
        BridgeStorage::is_paused(&env)
    }

    pub fn check_backing_ratio(
        env: Env,
        stellar_asset: Address,
        external_chain: ExternalChain,
    ) -> Result<u32, BridgeError> {
        let asset =
            BridgeStorage::get_wrapped_asset(&env, &stellar_asset, &external_chain)
                .ok_or(BridgeError::AssetNotRegistered)?;

        if asset.total_minted == 0 {
            return Ok(10_000); // trivially 100%
        }

        let ratio = (asset.total_locked as u128 * 10_000 / asset.total_minted as u128) as u32;
        Ok(ratio)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ── Private helpers ───────────────────────────────────────────────────────
    // ═══════════════════════════════════════════════════════════════════════════

    fn require_admin(env: &Env, caller: &Address) -> Result<(), BridgeError> {
        let admin = BridgeStorage::get_admin(env).ok_or(BridgeError::NotInitialized)?;
        if *caller != admin {
            return Err(BridgeError::NotAdmin);
        }
        Ok(())
    }

    fn require_not_paused(env: &Env) -> Result<(), BridgeError> {
        if BridgeStorage::is_paused(env) {
            return Err(BridgeError::BridgePaused);
        }
        Ok(())
    }

    fn require_chain_active(env: &Env, chain: &ExternalChain) -> Result<(), BridgeError> {
        match BridgeStorage::get_chain_config(env, chain) {
            None => Err(BridgeError::ChainNotSupported),
            Some(c) if !c.is_active => Err(BridgeError::ChainInactive),
            _ => Ok(()),
        }
    }

    /// Finalise an approved request — mint (inbound) or mark complete (outbound).
    fn finalise_request(
        env: &Env,
        request: &mut BridgeRequest,
    ) -> Result<BridgeRequestStatus, BridgeError> {
        let now = env.ledger().timestamp();

        match request.direction {
            BridgeDirection::InboundFromExternal => {
                // Mark external TX as processed (replay guard)
                BridgeStorage::mark_external_tx_processed(env, &request.external_tx_hash);

                // Update asset accounting
                let mut asset = BridgeStorage::get_wrapped_asset(
                    env,
                    &request.stellar_asset,
                    &request.external_chain,
                )
                .ok_or(BridgeError::AssetNotRegistered)?;

                asset.total_minted += request.net_amount;

                // Backing ratio guard — must stay ≥ 100%
                if !BridgeStorage::backing_ratio_healthy(&asset) {
                    return Err(BridgeError::BackingRatioBroken);
                }

                asset.backing_ratio_bps = if asset.total_minted > 0 {
                    (asset.total_locked as u128 * 10_000 / asset.total_minted as u128) as u32
                } else {
                    10_000
                };

                BridgeStorage::set_wrapped_asset(env, &asset);

                // Mint wrapped tokens to the initiator
                let token_client =
                    token::Client::new(env, &request.stellar_asset);
                token_client.transfer(
                    &env.current_contract_address(),
                    &request.initiator,
                    &request.net_amount,
                );
            }

            BridgeDirection::OutboundToExternal => {
                // Tokens were already locked in initiate_outbound.
                // The relayer/validators will release on the external chain.
                // Here we just update accounting — the net_amount stays locked.
                // The fee was already collected in initiate_outbound.
            }
        }

        request.status = BridgeRequestStatus::Completed;
        request.completed_at = now;
        BridgeStorage::set_request(env, request);

        let mut stats = BridgeStorage::get_stats(env);
        stats.total_completed += 1;
        BridgeStorage::set_stats(env, &stats);

        env.events().publish(
            (symbol_short!("complete"),),
            BridgeCompletedEvent {
                request_id: request.request_id,
                direction: request.direction.clone(),
                initiator: request.initiator.clone(),
                stellar_asset: request.stellar_asset.clone(),
                net_amount: request.net_amount,
                fee_amount: request.fee_amount,
                completed_at: now,
            },
        );

        Ok(BridgeRequestStatus::Completed)
    }

    /// Release locked tokens back to the initiator (rejected / cancelled / expired outbound).
    fn return_locked_tokens(env: &Env, request: &BridgeRequest) -> Result<(), BridgeError> {
        let token_client = token::Client::new(env, &request.stellar_asset);

        // Return the full amount minus fee (fee was already paid on initiate)
        let refund = request.net_amount;
        if refund > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &request.initiator,
                &refund,
            );
        }

        // Unwind locked accounting
        if let Some(mut asset) =
            BridgeStorage::get_wrapped_asset(env, &request.stellar_asset, &request.external_chain)
        {
            asset.total_locked = asset.total_locked.saturating_sub(request.amount);
            BridgeStorage::set_wrapped_asset(env, &asset);
        }

        Ok(())
    }
}