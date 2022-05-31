mod setup;
use crate::setup::*;
use std::collections::HashMap;


#[test]
fn test_claim_and_withdraw_all() {
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();

    e.create_proposal(&e.dao, ProposalKind::Poll { descriptions: vec!["topic1".to_string(), "topic2".to_string()] }, e.current_time() + DAY_TS, DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, Some((tokens.nref.account_id(), IncentiveType::Evenly)), 0).assert_success();
    e.skip_time(DAY_SEC);
    e.action_proposal(&users.alice, 0, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.ft_mint(&tokens.nref, &users.alice, to_yocto("100"));
    
    assert_eq!(0, e.get_proposal(0).unwrap().incentive.unwrap().incentive_amount);
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("50"), 0).assert_success();
    assert_eq!(to_yocto("50"), e.get_proposal(0).unwrap().incentive.unwrap().incentive_amount);
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("50"), 0).assert_success();
    assert_eq!(to_yocto("100"), e.get_proposal(0).unwrap().incentive.unwrap().incentive_amount);

    assert_eq!(HashMap::from([(0, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_yocto("200")
    })]), e.get_vote_detail(&users.alice));
    assert_eq!(HashMap::new(), e.get_vote_detail_history(&users.alice));
    assert_eq!(HashMap::new(), e.get_unclaimed_proposal(&users.alice));

    e.skip_time(DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC);
    assert_eq!(HashMap::new(), e.get_vote_detail(&users.alice));
    assert_eq!(HashMap::from([(0, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_yocto("200")
    })]), e.get_vote_detail_history(&users.alice));
    assert_eq!(HashMap::from([(0, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_yocto("200")
    })]), e.get_unclaimed_proposal(&users.alice));
    assert_eq!(0, e.get_proposal(0).unwrap().incentive.unwrap().claimed_amount);
    
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.alice), 0);
    e.claim_and_withdraw_all(&users.alice).assert_success();
    e.claim_and_withdraw_all(&users.alice).assert_success();
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.alice), to_yocto("100"));
    assert_eq!(to_yocto("100"), e.get_proposal(0).unwrap().incentive.unwrap().claimed_amount);
    assert_eq!(HashMap::new(), e.get_vote_detail(&users.alice));
    assert_eq!(HashMap::from([(0, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_yocto("200")
    })]), e.get_vote_detail_history(&users.alice));
    assert_eq!(HashMap::new(), e.get_unclaimed_proposal(&users.alice));
    assert_eq!(format!("{:?}", e.get_proposal(0).unwrap()), format!("{:?}", e.list_proposals(None, None)[0]));
}

#[test]
fn test_claim_reward() {
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();

    e.create_proposal(&e.dao, ProposalKind::Poll { descriptions: vec!["topic1".to_string(), "topic2".to_string()] }, e.current_time() + DAY_TS, DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, Some((tokens.nref.account_id(), IncentiveType::Evenly)), 0).assert_success();
    e.create_proposal(&e.dao, ProposalKind::Poll { descriptions: vec!["topic11".to_string(), "topic22".to_string()] }, e.current_time() + DAY_TS, DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, Some((tokens.nref.account_id(), IncentiveType::Evenly)), 0).assert_success();
    e.create_proposal(&e.dao, ProposalKind::Poll { descriptions: vec!["topic111".to_string(), "topic222".to_string()] }, e.current_time() + DAY_TS, DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, Some((tokens.nref.account_id(), IncentiveType::Evenly)), 0).assert_success();
    e.create_proposal(&e.dao, ProposalKind::Poll { descriptions: vec!["topic111".to_string(), "topic222".to_string()] }, e.current_time() + DAY_TS, DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, None, 0).assert_success();
    e.skip_time(DAY_SEC);
    e.action_proposal(&users.alice, 0, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.action_proposal(&users.alice, 1, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.action_proposal(&users.alice, 2, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.action_proposal(&users.alice, 3, Action::VotePoll { poll_id: 0 }, None).assert_success();
    assert_eq!(HashMap::from([(0, VoteDetail {
        action: Action::VotePoll { poll_id: 0 }, amount: to_yocto("200")
    }), (1, VoteDetail {
        action: Action::VotePoll { poll_id: 0 }, amount: to_yocto("200")
    }), (2, VoteDetail {
        action: Action::VotePoll { poll_id: 0 }, amount: to_yocto("200")
    }), (3, VoteDetail {
        action: Action::VotePoll { poll_id: 0 }, amount: to_yocto("200")
    })]), e.get_vote_detail(&users.alice));
    e.ft_mint(&tokens.nref, &users.alice, to_yocto("200"));
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 0).assert_success();
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 1).assert_success();

    assert_eq!(HashMap::new(), e.get_unclaimed_proposal(&users.alice));
    e.skip_time(DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC);
    assert_eq!(HashMap::from([(0, VoteDetail {
        action: Action::VotePoll { poll_id: 0 }, amount: to_yocto("200")
    }), (1, VoteDetail {
        action: Action::VotePoll { poll_id: 0 }, amount: to_yocto("200")
    }), (2, VoteDetail {
        action: Action::VotePoll { poll_id: 0 }, amount: to_yocto("200")
    })]), e.get_unclaimed_proposal(&users.alice));

    assert_eq!(HashMap::new(), e.get_account_info(&users.alice).unwrap().rewards);
    e.claim_reward(&users.alice, 0);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("100"))]), e.get_account_info(&users.alice).unwrap().rewards);
    e.claim_reward(&users.alice, 1);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("200"))]), e.get_account_info(&users.alice).unwrap().rewards);
    e.claim_reward(&users.alice, 2);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("200"))]), e.get_account_info(&users.alice).unwrap().rewards);
    e.claim_reward(&users.alice, 3);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("200"))]), e.get_account_info(&users.alice).unwrap().rewards);
}

#[test]
fn test_withdraw_reward() {
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();

    e.create_proposal(&e.dao, ProposalKind::Poll { descriptions: vec!["topic1".to_string(), "topic2".to_string()] }, e.current_time() + DAY_TS, DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, Some((tokens.nref.account_id(), IncentiveType::Evenly)), 0).assert_success();
    e.skip_time(DAY_SEC);
    e.action_proposal(&users.alice, 0, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.ft_mint(&tokens.nref, &users.alice, to_yocto("100"));
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 0).assert_success();

    e.skip_time(DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC);
    assert_eq!(HashMap::new(), e.get_account_info(&users.alice).unwrap().rewards);
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("100"))]), e.get_account_info(&users.alice).unwrap().rewards);

    assert_err!(e.storage_unregister(&users.alice, 1), E103_STILL_HAS_REWARD);

    assert_eq!(e.ft_balance_of(&tokens.nref, &users.alice), 0);
    e.withdraw_reward(&users.alice, &tokens.nref, Some(to_yocto("50")));
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("50"))]), e.get_account_info(&users.alice).unwrap().rewards);
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.alice), to_yocto("50"));

    e.withdraw_reward(&users.alice, &tokens.nref, None);
    assert_eq!(HashMap::new(), e.get_account_info(&users.alice).unwrap().rewards);
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.alice), to_yocto("100"));
}