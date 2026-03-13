#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec,
};

/// Proposal lifecycle states.
/// TODO issue #1: implement full state machine transitions with timing logic.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalState {
    Pending,
    Active,
    Defeated,
    Succeeded,
    Queued,
    Executed,
    Cancelled,
}

/// A governance proposal.
#[contracttype]
#[derive(Clone)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub description: String,
    pub start_ledger: u32,
    pub end_ledger: u32,
    pub votes_for: i128,
    pub votes_against: i128,
    pub votes_abstain: i128,
    pub executed: bool,
    pub cancelled: bool,
}

/// Vote support options.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum VoteSupport {
    Against,
    For,
    Abstain,
}

/// Storage keys.
#[contracttype]
pub enum DataKey {
    Proposal(u64),
    ProposalCount,
    VotingDelay,
    VotingPeriod,
    QuorumNumerator,
    ProposalThreshold,
    Timelock,
    VotesToken,
    Admin,
    HasVoted(u64, Address),
}

#[contract]
pub struct GovernorContract;

#[contractimpl]
impl GovernorContract {
    /// Initialize the governor with configuration.
    pub fn initialize(
        env: Env,
        admin: Address,
        votes_token: Address,
        timelock: Address,
        voting_delay: u32,
        voting_period: u32,
        quorum_numerator: u32,
        proposal_threshold: i128,
    ) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::VotesToken, &votes_token);
        env.storage().instance().set(&DataKey::Timelock, &timelock);
        env.storage().instance().set(&DataKey::VotingDelay, &voting_delay);
        env.storage().instance().set(&DataKey::VotingPeriod, &voting_period);
        env.storage().instance().set(&DataKey::QuorumNumerator, &quorum_numerator);
        env.storage().instance().set(&DataKey::ProposalThreshold, &proposal_threshold);
        env.storage().instance().set(&DataKey::ProposalCount, &0u64);
    }

    /// Create a new governance proposal.
    /// TODO issue #2: add calldata encoding, threshold check, and event emission.
    pub fn propose(
        env: Env,
        proposer: Address,
        description: String,
    ) -> u64 {
        proposer.require_auth();

        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0);
        let proposal_id = count + 1;

        let voting_delay: u32 = env
            .storage()
            .instance()
            .get(&DataKey::VotingDelay)
            .unwrap_or(100);
        let voting_period: u32 = env
            .storage()
            .instance()
            .get(&DataKey::VotingPeriod)
            .unwrap_or(1000);

        let current = env.ledger().sequence();
        let proposal = Proposal {
            id: proposal_id,
            proposer: proposer.clone(),
            description,
            start_ledger: current + voting_delay,
            end_ledger: current + voting_delay + voting_period,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            executed: false,
            cancelled: false,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage()
            .instance()
            .set(&DataKey::ProposalCount, &proposal_id);

        env.events().publish(
            (symbol_short!("propose"), proposer),
            proposal_id,
        );

        proposal_id
    }

    /// Cast a vote on an active proposal.
    /// TODO issue #3: add deduplication check, voting power lookup, and event.
    pub fn cast_vote(
        env: Env,
        voter: Address,
        proposal_id: u64,
        support: VoteSupport,
    ) {
        voter.require_auth();

        let voted: bool = env
            .storage()
            .persistent()
            .get(&DataKey::HasVoted(proposal_id, voter.clone()))
            .unwrap_or(false);
        assert!(!voted, "already voted");

        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        // TODO: fetch actual voting power from token_votes contract.
        let weight: i128 = 1;

        match support {
            VoteSupport::For => proposal.votes_for += weight,
            VoteSupport::Against => proposal.votes_against += weight,
            VoteSupport::Abstain => proposal.votes_abstain += weight,
        }

        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.storage()
            .persistent()
            .set(&DataKey::HasVoted(proposal_id, voter.clone()), &true);

        env.events().publish(
            (symbol_short!("vote"), voter),
            (proposal_id, support),
        );
    }

    /// Cast a vote with an on-chain reason string.
    /// TODO issue #4: store reason in persistent storage and emit event.
    pub fn cast_vote_with_reason(
        env: Env,
        voter: Address,
        proposal_id: u64,
        support: VoteSupport,
        _reason: String,
    ) {
        Self::cast_vote(env, voter, proposal_id, support);
    }

    /// Queue a succeeded proposal for execution via timelock.
    /// TODO issue #5: integrate timelock contract cross-contract call.
    pub fn queue(env: Env, proposal_id: u64) {
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");
        assert!(!proposal.executed && !proposal.cancelled, "invalid state");
        // TODO: verify state == Succeeded, then call timelock.schedule().
        proposal.executed = false; // placeholder
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.events()
            .publish((symbol_short!("queue"),), proposal_id);
    }

    /// Execute a queued proposal.
    /// TODO issue #6: call timelock.execute() with stored calldata.
    pub fn execute(env: Env, proposal_id: u64) {
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");
        assert!(!proposal.executed && !proposal.cancelled, "invalid state");
        proposal.executed = true;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.events()
            .publish((symbol_short!("execute"),), proposal_id);
    }

    /// Cancel a proposal. Only proposer or admin can cancel.
    /// TODO issue #7: enforce cancellation rules, emit event.
    pub fn cancel(env: Env, caller: Address, proposal_id: u64) {
        caller.require_auth();
        let mut proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("admin not set");
        assert!(
            caller == proposal.proposer || caller == admin,
            "not authorized"
        );
        proposal.cancelled = true;
        env.storage()
            .persistent()
            .set(&DataKey::Proposal(proposal_id), &proposal);
        env.events()
            .publish((symbol_short!("cancel"),), proposal_id);
    }

    /// Get the current state of a proposal.
    /// TODO issue #1: implement full timing-aware state transitions.
    pub fn state(env: Env, proposal_id: u64) -> ProposalState {
        let proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");

        if proposal.cancelled {
            return ProposalState::Cancelled;
        }
        if proposal.executed {
            return ProposalState::Executed;
        }

        let current = env.ledger().sequence();
        if current < proposal.start_ledger {
            ProposalState::Pending
        } else if current <= proposal.end_ledger {
            ProposalState::Active
        } else {
            // TODO: check quorum and votes_for > votes_against
            ProposalState::Defeated
        }
    }

    /// Get vote counts for a proposal.
    pub fn proposal_votes(env: Env, proposal_id: u64) -> (i128, i128, i128) {
        let proposal: Proposal = env
            .storage()
            .persistent()
            .get(&DataKey::Proposal(proposal_id))
            .expect("proposal not found");
        (proposal.votes_for, proposal.votes_against, proposal.votes_abstain)
    }

    /// Get governor configuration.
    pub fn voting_delay(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::VotingDelay).unwrap_or(100)
    }

    pub fn voting_period(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::VotingPeriod).unwrap_or(1000)
    }

    pub fn proposal_threshold(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::ProposalThreshold).unwrap_or(0)
    }

    /// Get total proposal count.
    pub fn proposal_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::ProposalCount).unwrap_or(0)
    }
}
