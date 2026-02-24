#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, Map, String, Symbol, Vec,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

const ADMIN: Symbol = symbol_short!("ADMIN");
const CONFIG: Symbol = symbol_short!("CONFIG");
const PROP_COUNT: Symbol = symbol_short!("PROP_CNT");

// ─── Types ───────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone)]
pub struct GovernanceConfig {
    pub voting_delay: u64,        // ledgers before voting starts
    pub voting_period: u64,       // ledgers voting is open
    pub timelock_delay: u64,      // ledgers before execution after passing
    pub quorum_bps: u32,          // basis points (e.g. 400 = 4%)
    pub threshold_bps: u32,       // basis points majority needed (e.g. 5100 = 51%)
    pub proposal_threshold: i128, // min tokens to create proposal
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ProposalState {
    Pending,
    Active,
    Succeeded,
    Defeated,
    Queued,
    Executed,
    Cancelled,
    Expired,
}

#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub title: String,
    pub description: String,
    pub calldata: Vec<String>,    // encoded action strings
    pub start_ledger: u64,
    pub end_ledger: u64,
    pub eta_ledger: u64,          // earliest execution ledger (after timelock)
    pub for_votes: i128,
    pub against_votes: i128,
    pub abstain_votes: i128,
    pub state: ProposalState,
    pub cancelled: bool,
    pub executed: bool,
}

#[contracttype]
#[derive(Clone)]
pub struct VoteReceipt {
    pub has_voted: bool,
    pub support: u8,              // 0=against, 1=for, 2=abstain
    pub votes: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Proposal(u64),
    VoteReceipt(u64, Address),
    Delegate(Address),
    VotingPower(Address),
    TotalSupply,
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    // ── Init ────────────────────────────────────────────────────────────────

    pub fn initialize(
        env: Env,
        admin: Address,
        config: GovernanceConfig,
        initial_supply: i128,
    ) {
        admin.require_auth();
        if env.storage().instance().has(&ADMIN) {
            panic!("already initialized");
        }
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&CONFIG, &config);
        env.storage().instance().set(&PROP_COUNT, &0u64);
        env.storage().persistent().set(&DataKey::TotalSupply, &initial_supply);
    }

    // ── Governance Config ────────────────────────────────────────────────────

    pub fn update_config(env: Env, new_config: GovernanceConfig) {
        Self::only_admin(&env);
        env.storage().instance().set(&CONFIG, &new_config);
    }

    pub fn get_config(env: Env) -> GovernanceConfig {
        env.storage().instance().get(&CONFIG).unwrap()
    }

    // ── Voting Power / Delegation ─────────────────────────────────────────

    pub fn set_voting_power(env: Env, account: Address, amount: i128) {
        Self::only_admin(&env);
        env.storage()
            .persistent()
            .set(&DataKey::VotingPower(account), &amount);
    }

    pub fn delegate(env: Env, delegator: Address, delegatee: Address) {
        delegator.require_auth();
        env.storage()
            .persistent()
            .set(&DataKey::Delegate(delegator.clone()), &delegatee);
        env.events().publish(
            (symbol_short!("delegate"), delegator),
            delegatee,
        );
    }

    pub fn get_delegate(env: Env, account: Address) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Delegate(account.clone()))
            .unwrap_or(account)
    }

    pub fn get_votes(env: Env, account: Address) -> i128 {
        let effective = Self::resolve_delegate(&env, account);
        env.storage()
            .persistent()
            .get(&DataKey::VotingPower(effective))
            .unwrap_or(0i128)
    }

    // ── Proposals ────────────────────────────────────────────────────────────

    pub fn propose(
        env: Env,
        proposer: Address,
        title: String,
        description: String,
        calldata: Vec<String>,
    ) -> u64 {
        proposer.require_auth();

        let config: GovernanceConfig = env.storage().instance().get(&CONFIG).unwrap();
        let votes = Self::get_votes(env.clone(), proposer.clone());

        if votes < config.proposal_threshold {
            panic!("insufficient voting power to propose");
        }

        let current = env.ledger().sequence() as u64;
        let start = current + config.voting_delay;
        let end = start + config.voting_period;

        let mut count: u64 = env.storage().instance().get(&PROP_COUNT).unwrap();
        count += 1;

        let proposal = Proposal {
            id: count,
            proposer: proposer.clone(),
            title,
            description,
            calldata,
            start_ledger: start,
            end_ledger: end,
            eta_ledger: 0,
            for_votes: 0,
            against_votes: 0,
            abstain_votes: 0,
            state: ProposalState::Pending,
            cancelled: false,
            executed: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(count), &proposal);
        env.storage().instance().set(&PROP_COUNT, &count);

        env.events().publish(
            (symbol_short!("proposed"), proposer),
            count,
        );

        count
    }

    pub fn cancel_proposal(env: Env, caller: Address, proposal_id: u64) {
        caller.require_auth();
        let mut proposal = Self::load_proposal(&env, proposal_id);

        if proposal.proposer != caller {
            Self::require_admin(&env);
        }
        if proposal.executed {
            panic!("already executed");
        }

        proposal.cancelled = true;
        proposal.state = ProposalState::Cancelled;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        let mut proposal = Self::load_proposal(&env, proposal_id);
        proposal.state = Self::compute_state(&env, &proposal);
        proposal
    }

    pub fn get_proposal_state(env: Env, proposal_id: u64) -> ProposalState {
        let proposal = Self::load_proposal(&env, proposal_id);
        Self::compute_state(&env, &proposal)
    }

    // ── Voting ────────────────────────────────────────────────────────────────

    pub fn cast_vote(env: Env, voter: Address, proposal_id: u64, support: u8) {
        voter.require_auth();
        Self::_cast_vote(&env, voter, proposal_id, support);
    }

    pub fn cast_vote_with_reason(
        env: Env,
        voter: Address,
        proposal_id: u64,
        support: u8,
        _reason: String,
    ) {
        voter.require_auth();
        Self::_cast_vote(&env, voter, proposal_id, support);
    }

    pub fn get_vote_receipt(env: Env, proposal_id: u64, voter: Address) -> VoteReceipt {
        env.storage()
            .persistent()
            .get(&DataKey::VoteReceipt(proposal_id, voter))
            .unwrap_or(VoteReceipt {
                has_voted: false,
                support: 0,
                votes: 0,
            })
    }

    // ── Timelock / Queue / Execute ────────────────────────────────────────────

    pub fn queue(env: Env, caller: Address, proposal_id: u64) {
        caller.require_auth();
        let config: GovernanceConfig = env.storage().instance().get(&CONFIG).unwrap();
        let mut proposal = Self::load_proposal(&env, proposal_id);
        let state = Self::compute_state(&env, &proposal);

        if state != ProposalState::Succeeded {
            panic!("proposal not succeeded");
        }

        let current = env.ledger().sequence() as u64;
        proposal.eta_ledger = current + config.timelock_delay;
        proposal.state = ProposalState::Queued;

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        env.events().publish(
            (symbol_short!("queued"), proposal_id),
            proposal.eta_ledger,
        );
    }

    pub fn execute(env: Env, caller: Address, proposal_id: u64) {
        caller.require_auth();
        let mut proposal = Self::load_proposal(&env, proposal_id);

        if proposal.state != ProposalState::Queued {
            panic!("proposal not queued");
        }
        if proposal.eta_ledger == 0 {
            panic!("eta not set");
        }

        let current = env.ledger().sequence() as u64;
        if current < proposal.eta_ledger {
            panic!("timelock not elapsed");
        }

        let grace = proposal.eta_ledger + 100_800; // ~7 days grace period
        if current > grace {
            proposal.state = ProposalState::Expired;
            env.storage()
                .persistent()
                .set(&DataKey::Proposal(proposal_id), &proposal);
            panic!("proposal expired");
        }

        proposal.executed = true;
        proposal.state = ProposalState::Executed;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        env.events().publish(
            (symbol_short!("executed"), caller),
            proposal_id,
        );
    }

    // ── Admin ─────────────────────────────────────────────────────────────────

    pub fn transfer_admin(env: Env, new_admin: Address) {
        Self::only_admin(&env);
        env.storage().instance().set(&ADMIN, &new_admin);
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN).unwrap()
    }

    pub fn proposal_count(env: Env) -> u64 {
        env.storage().instance().get(&PROP_COUNT).unwrap_or(0)
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    fn load_proposal(env: &Env, id: u64) -> Proposal {
        env.storage()
            .persistent()
            .get(&DataKey::Proposal(id))
            .expect("proposal not found")
    }

    fn compute_state(env: &Env, proposal: &Proposal) -> ProposalState {
        if proposal.cancelled {
            return ProposalState::Cancelled;
        }
        if proposal.executed {
            return ProposalState::Executed;
        }

        let config: GovernanceConfig = env.storage().instance().get(&CONFIG).unwrap();
        let current = env.ledger().sequence() as u64;
        let total_supply: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalSupply)
            .unwrap_or(1);

        if current < proposal.start_ledger {
            return ProposalState::Pending;
        }
        if current <= proposal.end_ledger {
            return ProposalState::Active;
        }

        // voting ended — check quorum and threshold
        let total_votes = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
        let quorum_needed = (total_supply * config.quorum_bps as i128) / 10_000;

        if total_votes < quorum_needed {
            return ProposalState::Defeated;
        }

        let decisive = proposal.for_votes + proposal.against_votes;
        if decisive == 0 {
            return ProposalState::Defeated;
        }

        let for_bps = (proposal.for_votes * 10_000) / decisive;
        if for_bps >= config.threshold_bps as i128 {
            if proposal.state == ProposalState::Queued {
                return ProposalState::Queued;
            }
            return ProposalState::Succeeded;
        }

        ProposalState::Defeated
    }

    fn _cast_vote(env: &Env, voter: Address, proposal_id: u64, support: u8) {
        let receipt_key = DataKey::VoteReceipt(proposal_id, voter.clone());
        let existing: VoteReceipt = env
            .storage()
            .persistent()
            .get(&receipt_key)
            .unwrap_or(VoteReceipt { has_voted: false, support: 0, votes: 0 });

        if existing.has_voted {
            panic!("already voted");
        }

        let mut proposal = Self::load_proposal(env, proposal_id);
        let state = Self::compute_state(env, &proposal);

        if state != ProposalState::Active {
            panic!("voting not active");
        }

        if support > 2 {
            panic!("invalid support value: 0=against 1=for 2=abstain");
        }

        let effective = Self::resolve_delegate(env, voter.clone());
        let votes: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::VotingPower(effective))
            .unwrap_or(0);

        if votes == 0 {
            panic!("no voting power");
        }

        match support {
            0 => proposal.against_votes += votes,
            1 => proposal.for_votes += votes,
            2 => proposal.abstain_votes += votes,
            _ => panic!("invalid support"),
        }

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);

        env.storage().persistent().set(
            &receipt_key,
            &VoteReceipt { has_voted: true, support, votes },
        );

        env.events().publish(
            (symbol_short!("voted"), voter),
            (proposal_id, support, votes),
        );
    }

    fn resolve_delegate(env: &Env, account: Address) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Delegate(account.clone()))
            .unwrap_or(account)
    }

    fn only_admin(env: &Env) {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();
    }

    fn require_admin(env: &Env) {
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        admin.require_auth();
    }
}
