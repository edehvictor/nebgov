use crate::*;
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address, Bytes, Env, String, Symbol};

/// Shared helper: initialize the governor with standard test parameters.
fn setup() -> (Env, GovernorContractClient<'static>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(GovernorContract, ());
    let client = GovernorContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let votes_token = Address::generate(&env);
    let timelock = Address::generate(&env);
    let proposer = Address::generate(&env);
    let voter = Address::generate(&env);

    // voting_delay=10, voting_period=100, quorum_numerator=0, proposal_threshold=0
    client.initialize(&admin, &votes_token, &timelock, &10, &100, &0, &0);

    (env, client, admin, proposer, voter)
}

/// Shared helper: create a new proposal and return its id.
fn make_proposal(env: &Env, client: &GovernorContractClient, proposer: &Address) -> u64 {
    let target = Address::generate(env);
    let fn_name = Symbol::new(env, "noop");
    let calldata = Bytes::new(env);
    let description = String::from_str(env, "Test proposal");
    client.propose(proposer, &description, &target, &fn_name, &calldata)
}

#[test]
/// Verifies that a proposal's initial state is Pending before the voting delay has passed.
fn test_pending_state_before_start_ledger() {
    let (env, client, _, proposer, _) = setup();
    let proposal_id = make_proposal(&env, &client, &proposer);

    assert_eq!(client.state(&proposal_id), ProposalState::Pending);
}

#[test]
/// Verifies that a proposal becomes Active exactly at the start_ledger.
fn test_active_state_at_start_ledger() {
    let (env, client, _, proposer, _) = setup();
    let proposal_id = make_proposal(&env, &client, &proposer);

    // start_ledger = current (0) + voting_delay (10) = 10
    env.ledger().set_sequence_number(10);
    assert_eq!(client.state(&proposal_id), ProposalState::Active);
}

#[test]
/// Verifies that a proposal is Defeated if no votes are cast by the end of the voting period.
fn test_defeated_when_no_votes() {
    let (env, client, _, proposer, _) = setup();
    let proposal_id = make_proposal(&env, &client, &proposer);

    // end_ledger = 10 + 100 = 110. Advance to 111.
    env.ledger().set_sequence_number(111);
    assert_eq!(client.state(&proposal_id), ProposalState::Defeated);
}

#[test]
/// Verifies that a proposal is Defeated if Against votes exceed or equal For votes.
fn test_defeated_when_against_wins() {
    let (env, client, _, proposer, voter) = setup();
    let proposal_id = make_proposal(&env, &client, &proposer);

    env.ledger().set_sequence_number(10); // Active
    client.cast_vote(&voter, &proposal_id, &VoteSupport::Against);

    env.ledger().set_sequence_number(111); // Past end
    assert_eq!(client.state(&proposal_id), ProposalState::Defeated);
}

#[test]
/// Verifies that a proposal is Succeeded if it has at least one For vote and matches majority.
fn test_succeeded_with_majority() {
    let (env, client, _, proposer, voter1) = setup();
    let voter2 = Address::generate(&env);
    let voter3 = Address::generate(&env);
    let proposal_id = make_proposal(&env, &client, &proposer);

    env.ledger().set_sequence_number(10); // Active
    client.cast_vote(&voter1, &proposal_id, &VoteSupport::For);
    client.cast_vote(&voter2, &proposal_id, &VoteSupport::For);
    client.cast_vote(&voter3, &proposal_id, &VoteSupport::Against);

    env.ledger().set_sequence_number(111); // Past end
    assert_eq!(client.state(&proposal_id), ProposalState::Succeeded);
}

#[test]
/// Verifies that the proposer can cancel a proposal, moving it to the Cancelled state.
fn test_cancelled_by_proposer() {
    let (env, client, _, proposer, _) = setup();
    let proposal_id = make_proposal(&env, &client, &proposer);

    client.cancel(&proposer, &proposal_id);
    assert_eq!(client.state(&proposal_id), ProposalState::Cancelled);
}

#[test]
/// Verifies that votes can be cast even in Pending state, documenting current contract behavior.
fn test_vote_state_is_pending_not_active() {
    let (env, client, _, proposer, voter) = setup();
    let proposal_id = make_proposal(&env, &client, &proposer);

    // Current ledger is 0, start_ledger is 10. State is Pending.
    client.cast_vote(&voter, &proposal_id, &VoteSupport::For);
    assert_eq!(client.state(&proposal_id), ProposalState::Pending);
}

#[test]
#[should_panic(expected = "already voted")]
/// Verifies that a voter cannot cast more than one vote on the same proposal.
fn test_cannot_vote_twice() {
    let (env, client, _, proposer, voter) = setup();
    let proposal_id = make_proposal(&env, &client, &proposer);

    env.ledger().set_sequence_number(10); // Active
    client.cast_vote(&voter, &proposal_id, &VoteSupport::For);
    client.cast_vote(&voter, &proposal_id, &VoteSupport::Against);
}
