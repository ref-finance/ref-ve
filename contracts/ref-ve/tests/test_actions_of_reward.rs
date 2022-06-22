mod setup;
use crate::setup::*;
use std::collections::HashMap;


#[test]
fn test_claim_and_withdraw_all() {
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    e.extend_whitelisted_incentive_tokens(&e.owner, vec![tokens.nref.account_id(), tokens.wnear.account_id()]).assert_success();
    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();

    e.extend_whitelisted_accounts(&e.owner, vec![users.alice.account_id()]).assert_success();

    e.create_proposal(&users.alice, ProposalKind::Poll { options: vec!["topic1".to_string(), "topic2".to_string()] }, "Poll".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, 0).assert_success();
    e.create_proposal(&users.alice, ProposalKind::Poll { options: vec!["topic1".to_string(), "topic2".to_string()] }, "Poll".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, 0).assert_success();
    e.create_proposal(&users.alice, ProposalKind::Poll { options: vec!["topic1".to_string(), "topic2".to_string()] }, "Poll".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, 0).assert_success();
    e.skip_time(DAY_SEC);
    e.action_proposal(&users.alice, 0, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.action_proposal(&users.alice, 1, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.action_proposal(&users.alice, 2, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.ft_mint(&tokens.nref, &users.alice, to_yocto("200"));
    e.ft_mint(&tokens.wnear, &users.alice, to_yocto("200"));
    
    assert!(e.get_proposal(0).unwrap().incentive.is_empty());
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("50"), 0, 0).assert_success();
    assert_eq!(to_yocto("50"), e.get_proposal(0).unwrap().incentive.get(&0).unwrap().incentive_amounts[0]);
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("50"), 0, 0).assert_success();
    assert_eq!(to_yocto("100"), e.get_proposal(0).unwrap().incentive.get(&0).unwrap().incentive_amounts[0]);
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 1, 0).assert_success();
    assert_eq!(to_yocto("100"), e.get_proposal(1).unwrap().incentive.get(&0).unwrap().incentive_amounts[0]);
    e.deposit_reward(&tokens.wnear, &users.alice, to_yocto("100"), 2, 0).assert_success();
    assert_eq!(to_yocto("100"), e.get_proposal(1).unwrap().incentive.get(&0).unwrap().incentive_amounts[0]);


    assert_eq!(HashMap::from([(0, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }), (1, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }), (2, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    })]), e.get_vote_detail(&users.alice));
    assert_eq!(HashMap::new(), e.get_vote_detail_history(&users.alice));
    assert_eq!(HashMap::new(), e.get_unclaimed_proposal(&users.alice));

    e.skip_time(DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC);

    assert_eq!(to_yocto("200"), e.get_unclaimed_rewards(&users.alice).get(&tokens.nref.account_id()).unwrap().0);
    assert_eq!(to_yocto("100"), e.get_unclaimed_rewards(&users.alice).get(&tokens.wnear.account_id()).unwrap().0);

    assert_eq!(HashMap::new(), e.get_vote_detail(&users.alice));
    assert_eq!(HashMap::from([(0, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }), (1, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }), (2, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    })]), e.get_vote_detail_history(&users.alice));
    assert_eq!(HashMap::from([(0, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }), (1, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }), (2, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    })]), e.get_unclaimed_proposal(&users.alice));
    assert_eq!(0, e.get_proposal(0).unwrap().incentive.get(&0).unwrap().claimed_amounts[0]);
    
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.alice), 0);
    e.claim_and_withdraw_all(&users.alice).assert_success();
    e.claim_and_withdraw_all(&users.alice).assert_success();
    assert_eq!(e.ft_balance_of(&tokens.nref, &users.alice), to_yocto("200"));
    assert_eq!(to_yocto("100"), e.get_proposal(0).unwrap().incentive.get(&0).unwrap().claimed_amounts[0]);
    assert_eq!(HashMap::new(), e.get_vote_detail(&users.alice));
    assert_eq!(HashMap::from([(0, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }), (1, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }), (2, VoteDetail{
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    })]), e.get_vote_detail_history(&users.alice));
    assert_eq!(HashMap::new(), e.get_unclaimed_proposal(&users.alice));
    assert_eq!(format!("{:?}", e.get_proposal(0).unwrap()), format!("{:?}", e.list_proposals(None, None)[0]));
}

#[test]
fn test_claim_reward() {
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    e.extend_whitelisted_incentive_tokens(&e.owner, vec![tokens.nref.account_id(), tokens.wnear.account_id()]).assert_success();
    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_mint(&lpt_inner_id(), &users.bob, to_yocto("200"));
    e.mft_mint(&lpt_inner_id(), &users.charlie, to_yocto("200"));
    e.mft_mint(&lpt_inner_id(), &users.eve, to_yocto("200"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.lock_lpt(&users.bob, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.lock_lpt(&users.charlie, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.lock_lpt(&users.eve, to_yocto("200"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();

    e.extend_whitelisted_accounts(&e.owner, vec![users.dude.account_id()]).assert_success();
    e.storage_deposit(&users.dude, &users.dude, to_yocto("1"));

    e.create_proposal(&users.dude, ProposalKind::Poll { options: vec!["topic1".to_string(), "topic2".to_string()] }, "Poll".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, 0).assert_success();
    e.create_proposal(&users.dude, ProposalKind::Poll { options: vec!["topic11".to_string(), "topic22".to_string()] }, "Poll".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, 0).assert_success();
    e.create_proposal(&users.dude, ProposalKind::FarmingReward { farm_list: vec!["noct.near|nskyward.near".to_string(), "nusdt.near|nusdc.near|ndai.near".to_string(), "usn.near|nusdt.near".to_string()], total_reward: 20000 }, "FarmingReward".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, 0).assert_success();
    e.create_proposal(&users.dude, ProposalKind::Poll { options: vec!["topic111".to_string(), "topic222".to_string()] }, "Poll".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, 0).assert_success();
    e.create_proposal(&users.dude, ProposalKind::FarmingReward { farm_list: vec!["noct.near|nskyward.near".to_string(), "nusdt.near|nusdc.near|ndai.near".to_string(), "usn.near|nusdt.near".to_string()], total_reward: 20000 }, "FarmingReward".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, 0).assert_success();
    e.create_proposal(&users.dude, ProposalKind::Poll { options: vec!["topic111".to_string(), "topic222".to_string()] }, "Poll".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, 0).assert_success();
   
    e.skip_time(DAY_SEC);
    e.action_proposal(&users.alice, 0, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.action_proposal(&users.alice, 1, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.action_proposal(&users.bob, 1, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.action_proposal(&users.alice, 2, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    e.action_proposal(&users.bob, 2, Action::VoteFarm { farm_id: 1 }, None).assert_success();
    e.action_proposal(&users.charlie, 2, Action::VoteFarm { farm_id: 2 }, None).assert_success();
    e.action_proposal(&users.eve, 2, Action::VoteFarm { farm_id: 2 }, None).assert_success();
    e.action_proposal(&users.charlie, 3, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.action_proposal(&users.eve, 3, Action::VotePoll { poll_id: 1 }, None).assert_success();
    e.action_proposal(&users.charlie, 4, Action::VoteFarm { farm_id: 1 }, None).assert_success();
    e.action_proposal(&users.eve, 4, Action::VoteFarm { farm_id: 1 }, None).assert_success();
    e.action_proposal(&users.charlie, 5, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.action_proposal(&users.eve, 5, Action::VotePoll { poll_id: 1 }, None).assert_success();

    assert_eq!(HashMap::from([(0, VoteDetail {
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }), (1, VoteDetail {
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }), (2, VoteDetail {
        action: Action::VoteFarm { farm_id: 0 }, amount: to_ve_token("200")
    })]), e.get_vote_detail(&users.alice));
    assert_eq!(HashMap::from([(1, VoteDetail {
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }), (2, VoteDetail {
        action: Action::VoteFarm { farm_id: 1 }, amount: to_ve_token("200")
    })]), e.get_vote_detail(&users.bob));
    e.ft_mint(&tokens.nref, &users.dude, to_yocto("2000"));
    e.ft_mint(&tokens.wnear, &users.dude, to_yocto("2000"));
    e.ft_mint(&tokens.ndai, &users.dude, to_yocto("2000"));
    e.ft_mint(&tokens.noct, &users.dude, to_yocto("2000"));
    e.deposit_reward(&tokens.nref, &users.dude, to_yocto("100"), 0, 0).assert_success();
    e.deposit_reward(&tokens.nref, &users.dude, to_yocto("100"), 1, 0).assert_success();
    e.deposit_reward(&tokens.noct, &users.dude, to_yocto("50"), 2, 0).assert_success();
    e.deposit_reward(&tokens.nref, &users.dude, to_yocto("50"), 2, 0).assert_success();
    e.deposit_reward(&tokens.nref, &users.dude, to_yocto("50"), 2, 1).assert_success();
    e.deposit_reward(&tokens.nref, &users.dude, to_yocto("120"), 2, 2).assert_success();
    e.deposit_reward(&tokens.nref, &users.dude, to_yocto("120"), 3, 0).assert_success();
    e.deposit_reward(&tokens.ndai, &users.dude, to_yocto("210"), 4, 1).assert_success();
    e.deposit_reward(&tokens.wnear, &users.dude, to_yocto("210"), 5, 0).assert_success();

    assert_eq!(HashMap::new(), e.get_unclaimed_proposal(&users.alice));
    e.skip_time(DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC);
    assert_eq!(HashMap::from([(0, VoteDetail {
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }), (1, VoteDetail {
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }),  (2, VoteDetail {
        action: Action::VoteFarm { farm_id: 0 }, amount: to_ve_token("200")
    })]), e.get_unclaimed_proposal(&users.alice));

    assert_eq!(HashMap::from([(1, VoteDetail {
        action: Action::VotePoll { poll_id: 0 }, amount: to_ve_token("200")
    }), (2, VoteDetail {
        action: Action::VoteFarm { farm_id: 1 }, amount: to_ve_token("200")
    })]), e.get_unclaimed_proposal(&users.bob));

    assert_eq!(HashMap::new(), e.get_account_info(&users.alice).unwrap().rewards);
    e.claim_reward(&users.alice, 0);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("100"))]), e.get_account_info(&users.alice).unwrap().rewards);
    e.claim_reward(&users.alice, 1);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("150"))]), e.get_account_info(&users.alice).unwrap().rewards);
    e.claim_reward(&users.alice, 1);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("150"))]), e.get_account_info(&users.alice).unwrap().rewards);
    e.claim_reward(&users.alice, 2);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("200")), (tokens.noct.account_id(), to_yocto("50"))]), e.get_account_info(&users.alice).unwrap().rewards);
    e.claim_reward(&users.alice, 3);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("200")), (tokens.noct.account_id(), to_yocto("50"))]), e.get_account_info(&users.alice).unwrap().rewards);
    e.claim_reward(&users.bob, 1);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("50"))]), e.get_account_info(&users.bob).unwrap().rewards);
    e.claim_reward(&users.bob, 2);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("100"))]), e.get_account_info(&users.bob).unwrap().rewards);
    e.claim_reward(&users.charlie, 2);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("40"))]), e.get_account_info(&users.charlie).unwrap().rewards);
    e.claim_reward(&users.eve, 2);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("80"))]), e.get_account_info(&users.eve).unwrap().rewards);
    e.claim_reward(&users.charlie, 3);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("80"))]), e.get_account_info(&users.charlie).unwrap().rewards);
    e.claim_reward(&users.eve, 3);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("160"))]), e.get_account_info(&users.eve).unwrap().rewards);
    e.claim_reward(&users.charlie, 4);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("80")), (tokens.ndai.account_id(), to_yocto("70"))]), e.get_account_info(&users.charlie).unwrap().rewards);
    e.claim_reward(&users.eve, 4);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("160")), (tokens.ndai.account_id(), to_yocto("140"))]), e.get_account_info(&users.eve).unwrap().rewards);
    e.claim_reward(&users.charlie, 5);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("80")), (tokens.ndai.account_id(), to_yocto("70")), (tokens.wnear.account_id(), to_yocto("70"))]), e.get_account_info(&users.charlie).unwrap().rewards);
    e.claim_reward(&users.eve, 5);
    assert_eq!(HashMap::from([(tokens.nref.account_id(), to_yocto("160")), (tokens.ndai.account_id(), to_yocto("140")), (tokens.wnear.account_id(), to_yocto("140"))]), e.get_account_info(&users.eve).unwrap().rewards);
    assert_eq!(e.get_proposal(0).unwrap().incentive.get(&0).unwrap().incentive_amounts, e.get_proposal(0).unwrap().incentive.get(&0).unwrap().claimed_amounts);
    assert_eq!(e.get_proposal(1).unwrap().incentive.get(&0).unwrap().incentive_amounts, e.get_proposal(1).unwrap().incentive.get(&0).unwrap().claimed_amounts);
    assert_eq!(vec![VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }, VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }, VoteInfo{
        total_ballots: to_ve_token("600"),
        participants: 2
    }], e.get_proposal(2).unwrap().votes);
    assert_eq!(4, e.get_proposal(2).unwrap().participants);
    assert_eq!(to_ve_token("1000"), e.get_proposal(2).unwrap().ve_amount_at_last_action);
}

#[test]
fn test_withdraw_reward() {
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    e.extend_whitelisted_incentive_tokens(&e.owner, vec![tokens.nref.account_id(), tokens.wnear.account_id()]).assert_success();

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();

    e.extend_whitelisted_accounts(&e.owner, vec![users.dude.account_id()]).assert_success();
    e.storage_deposit(&users.dude, &users.dude, to_yocto("1"));

    e.create_proposal(&users.dude, ProposalKind::Poll { options: vec!["topic1".to_string(), "topic2".to_string()] }, "Poll".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, 0).assert_success();
    e.skip_time(DAY_SEC);
    e.action_proposal(&users.alice, 0, Action::VotePoll { poll_id: 0 }, None).assert_success();
    e.ft_mint(&tokens.nref, &users.alice, to_yocto("100"));
    e.deposit_reward(&tokens.nref, &users.alice, to_yocto("100"), 0, 0).assert_success();

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