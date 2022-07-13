mod setup;
use crate::setup::*;
use std::collections::HashMap;

#[test]
fn test_create_proposal(){
    let e = init_env();
    let users = Users::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));

    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);

    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    
    e.extend_whitelisted_accounts(&e.owner, vec![users.alice.account_id(), users.dude.account_id()]).assert_success();
    
    // error scene 
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(e.create_proposal(&users.dude, ProposalKind::FarmingReward { farm_list: vec![], total_reward: 2 }, "FarmingReward".to_string(), to_sec(e.current_time()), DEFAULT_MIN_VOTING_DURATION_SEC, 1), E100_ACC_NOT_REGISTERED);
    
    // 2 : E002_NOT_ALLOWED just whitelisted accounts can create proposal 
    e.storage_deposit(&users.bob, &users.bob, to_yocto("1"));
    assert_err!(e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec![], total_reward: 2 }, "FarmingReward".to_string(), to_sec(e.current_time()), DEFAULT_MIN_VOTING_DURATION_SEC, 1), E002_NOT_ALLOWED);

    // 3 : E402_INVALID_START_TIME 
    assert_err!(e.create_proposal(&users.alice, ProposalKind::FarmingReward { farm_list: vec![], total_reward: 2 }, "FarmingReward".to_string(), to_sec(e.current_time()), DEFAULT_MIN_VOTING_DURATION_SEC, 1), E402_INVALID_START_TIME);
    assert_err!(e.create_proposal(&users.alice, ProposalKind::Poll { options: vec![] }, "Poll".to_string(), to_sec(e.current_time()), DEFAULT_MIN_VOTING_DURATION_SEC, 1), E402_INVALID_START_TIME);
    assert_err!(e.create_proposal(&users.alice, ProposalKind::Common, "Common".to_string(), to_sec(e.current_time()), DEFAULT_MIN_VOTING_DURATION_SEC, 1), E402_INVALID_START_TIME);

    // 4 : E208_DESCRIPTION_TOO_LONG
    assert_err!(e.create_proposal(&users.alice, ProposalKind::Common, "a".repeat(2049), to_sec(e.current_time()), DEFAULT_MIN_VOTING_DURATION_SEC, 1), E208_DESCRIPTION_TOO_LONG);

    // 5 : E302_INVALID_DURATION
    assert_err!(e.create_proposal(&users.alice, ProposalKind::Common, "Common".to_string(), to_sec(e.current_time()), DEFAULT_MIN_VOTING_DURATION_SEC - 1, 1), E302_INVALID_DURATION);
    assert_err!(e.create_proposal(&users.alice, ProposalKind::Common, "Common".to_string(), to_sec(e.current_time()), DEFAULT_MAX_VOTING_DURATION_SEC + 1, 1), E302_INVALID_DURATION);

    let mut before = e.get_metadata();
    e.create_proposal(&users.alice, ProposalKind::FarmingReward { farm_list: vec![], total_reward: 2 }, "FarmingReward".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();
    e.create_proposal(&users.alice, ProposalKind::Poll { options: vec![] }, "a".repeat(2048), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();
    e.create_proposal(&users.alice, ProposalKind::Common, "Common".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();
    before.proposal_count = 3.into();
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));
}

#[test]
fn test_remove_proposal(){
    let e = init_env();
    let users = Users::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    
    e.extend_whitelisted_accounts(&e.owner, vec![users.alice.account_id(), users.dude.account_id()]).assert_success();

    e.storage_deposit(&users.dude, &users.dude, to_yocto("1"));
    e.create_proposal(&users.dude, ProposalKind::Common, "Common".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();
    e.create_proposal(&users.alice, ProposalKind::Common, "Common".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();
    
    // error scene 
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(e.remove_proposal(&users.alice, 0), E002_NOT_ALLOWED);
    assert_err!(e.remove_proposal(&users.dude, 1), E002_NOT_ALLOWED);

    // success
    let mut before = e.get_metadata();
    assert_eq!(e.remove_proposal(&users.alice, 1).unwrap_json::<bool>(), true);
    e.skip_time(DAY_SEC);
    assert_eq!(e.remove_proposal(&users.dude, 0).unwrap_json::<bool>(), false);
    before.proposal_count.0 -= 1;
    assert_eq!(format!("{:?}", before), format!("{:?}", e.get_metadata()));
}

#[test]
fn test_action_proposal(){
    let e = init_env();
    let users = Users::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.storage_deposit(&users.bob, &users.bob, to_yocto("1"));
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();

    e.extend_whitelisted_accounts(&e.owner, vec![users.bob.account_id()]).assert_success();
    
    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["noct.near|nref.near&2657".to_string(), "nusdt.near|nusdc.near|ndai.near&1910".to_string()], total_reward: 2 }, "FarmingReward".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();
    e.create_proposal(&users.bob, ProposalKind::Poll { options: vec!["topic1".to_string(), "topic2".to_string()] }, "Poll".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();
    e.create_proposal(&users.bob, ProposalKind::Common, "Common".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();

    // error scene 
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(e.action_proposal(&users.charlie, 0, Action::VoteFarm { farm_id: 0 }, None), E100_ACC_NOT_REGISTERED);
   
    // 2 : E404_PROPOSAL_NOT_EXIST
    assert_err!(e.action_proposal(&users.alice, 5, Action::VoteFarm { farm_id: 0 }, None), E404_PROPOSAL_NOT_EXIST);
    
    // 3 : E205_NOT_VOTABLE
    assert_err!(e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None), E205_NOT_VOTABLE);
    
    e.skip_time(DAY_SEC);

    // 4 : E201_INVALID_VOTE
    assert_err!(e.action_proposal(&users.alice, 2, Action::VoteNonsense, None), E201_INVALID_VOTE);
    assert_err!(e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 11 }, None), E201_INVALID_VOTE);
    assert_err!(e.action_proposal(&users.alice, 0, Action::VotePoll { poll_id: 11 }, None), E201_INVALID_VOTE);

    // success 
    e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    
    // 5 : E200_ALREADY_VOTED
    assert_err!(e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None), E200_ALREADY_VOTED);
    e.action_proposal(&users.alice, 1, Action::VotePoll { poll_id: 1 }, None).assert_success();
    e.action_proposal(&users.alice, 2, Action::VoteReject, None).assert_success();

    assert_eq!(vec![VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }], e.get_proposal(0).unwrap().votes);
    assert_eq!(to_ve_token("200"), e.get_proposal(0).unwrap().ve_amount_at_last_action);

    assert_eq!(vec![VoteInfo{
        total_ballots: 0,
        participants: 0
    }, VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }], e.get_proposal(1).unwrap().votes);
    assert_eq!(to_ve_token("200"), e.get_proposal(1).unwrap().ve_amount_at_last_action);

    assert_eq!(vec![VoteInfo{
        total_ballots: 0,
        participants: 0
    }, VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }], e.get_proposal(2).unwrap().votes);
    assert_eq!(to_ve_token("200"), e.get_proposal(2).unwrap().ve_amount_at_last_action);

    assert_eq!(HashMap::from([(0, VoteDetail{
        action: Action::VoteFarm { farm_id: 0 },
        amount: to_ve_token("200"),
    }), (1, VoteDetail{
        action: Action::VotePoll { poll_id: 1 },
        amount: to_ve_token("200"),
    }), (2, VoteDetail{
        action: Action::VoteReject,
        amount: to_ve_token("200"),
    })]), e.get_vote_detail(&users.alice));

    // append
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    assert_eq!(vec![VoteInfo{
        total_ballots: to_ve_token("400"),
        participants: 1
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }], e.get_proposal(0).unwrap().votes);
    assert_eq!(to_ve_token("400"), e.get_proposal(0).unwrap().ve_amount_at_last_action);

    assert_eq!(vec![VoteInfo{
        total_ballots: 0,
        participants: 0
    }, VoteInfo{
        total_ballots: to_ve_token("400"),
        participants: 1
    }], e.get_proposal(1).unwrap().votes);
    assert_eq!(to_ve_token("400"), e.get_proposal(1).unwrap().ve_amount_at_last_action);

    assert_eq!(vec![VoteInfo{
        total_ballots: 0,
        participants: 0
    }, VoteInfo{
        total_ballots: to_ve_token("400"),
        participants: 1
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }], e.get_proposal(2).unwrap().votes);
    assert_eq!(to_ve_token("400"), e.get_proposal(2).unwrap().ve_amount_at_last_action);

    assert_eq!(1, e.get_proposal(0).unwrap().participants);
    assert_eq!(1, e.get_proposal(1).unwrap().participants);
    assert_eq!(1, e.get_proposal(2).unwrap().participants);

    assert_eq!(HashMap::from([(0, VoteDetail{
        action: Action::VoteFarm { farm_id: 0 },
        amount: to_ve_token("400"),
    }), (1, VoteDetail{
        action: Action::VotePoll { poll_id: 1 },
        amount: to_ve_token("400"),
    }), (2, VoteDetail{
        action: Action::VoteReject,
        amount: to_ve_token("400"),
    })]), e.get_vote_detail(&users.alice));

    assert_eq!(HashMap::new(), e.get_unclaimed_proposal(&users.alice));

    e.skip_time(DAY_SEC + DEFAULT_MAX_LOCKING_DURATION_SEC);
    println!("{:?}", e.get_proposal(0));
    assert_eq!(false, e.get_proposal(2).unwrap().is_nonsense.unwrap());
    println!("{:?}", e.get_account_info(&users.alice));
    let alice = e.get_account_info(&users.alice).unwrap();
    assert_eq!(to_yocto("200"), alice.lpt_amount);
    assert_eq!(to_ve_token("400"), alice.ve_lpt_amount);
    assert_eq!(HashMap::from([(0, VoteDetail{
        action: Action::VoteFarm { farm_id: 0 },
        amount: alice.ve_lpt_amount,
    }), (1, VoteDetail{
        action: Action::VotePoll { poll_id: 1 },
        amount: alice.ve_lpt_amount,
    }), (2, VoteDetail{
        action: Action::VoteReject,
        amount: alice.ve_lpt_amount,
    })]), e.get_vote_detail_history(&users.alice));
}

#[test]
fn test_action_proposal_farming_reward_01(){
    let e = init_env();
    let users = Users::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_mint(&lpt_inner_id(), &users.bob, to_yocto("200"));

    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);

    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.lock_lpt(&users.bob, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    
    e.extend_whitelisted_accounts(&e.owner, vec![users.bob.account_id()]).assert_success();

    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["noct.near|nref.near&2657".to_string(), "nusdt.near|nusdc.near|ndai.near&1910".to_string()], total_reward: 2 }, "FarmingReward".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();

    e.skip_time(DAY_SEC);

    e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    e.action_proposal(&users.bob, 0, Action::VoteFarm { farm_id: 1 }, None).assert_success();
    
    assert_eq!(vec![VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }, VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }], e.get_proposal(0).unwrap().votes);
    assert_eq!(2, e.get_proposal(0).unwrap().participants);
    assert_eq!(to_ve_token("400"), e.get_proposal(0).unwrap().ve_amount_at_last_action);
}

#[test]
fn test_action_proposal_farming_reward_02(){
    let e = init_env();
    let users = Users::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_mint(&lpt_inner_id(), &users.bob, to_yocto("200"));

    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);

    e.lock_lpt(&users.alice, to_yocto("200"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.lock_lpt(&users.bob, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    
    e.extend_whitelisted_accounts(&e.owner, vec![users.bob.account_id()]).assert_success();

    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["noct.near|nref.near&2657".to_string(), "nusdt.near|nusdc.near|ndai.near&1910".to_string()], total_reward: 2 }, "FarmingReward".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();

    e.skip_time(DAY_SEC);

    e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    e.action_proposal(&users.bob, 0, Action::VoteFarm { farm_id: 1 }, None).assert_success();
    
    assert_eq!(vec![VoteInfo{
        total_ballots: to_ve_token("400"),
        participants: 1
    }, VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }], e.get_proposal(0).unwrap().votes);
    assert_eq!(2, e.get_proposal(0).unwrap().participants);
    assert_eq!(to_ve_token("600"), e.get_proposal(0).unwrap().ve_amount_at_last_action);
}

#[test]
fn test_action_proposal_farming_reward_03(){
    let e = init_env();
    let users = Users::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_mint(&lpt_inner_id(), &users.bob, to_yocto("200"));
    e.mft_mint(&lpt_inner_id(), &users.charlie, to_yocto("200"));

    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);

    e.lock_lpt(&users.alice, to_yocto("200"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.lock_lpt(&users.bob, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.lock_lpt(&users.charlie, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    
    e.extend_whitelisted_accounts(&e.owner, vec![users.bob.account_id()]).assert_success();

    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["noct.near|nref.near&2657".to_string(), "nusdt.near|nusdc.near|ndai.near&1910".to_string(), "usn.near|nusdt.near&3020".to_string()], total_reward: 3 }, "FarmingReward".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();

    e.skip_time(DAY_SEC);

    e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    e.action_proposal(&users.bob, 0, Action::VoteFarm { farm_id: 1 }, None).assert_success();
    e.action_proposal(&users.charlie, 0, Action::VoteFarm { farm_id: 2 }, None).assert_success();
    
    assert_eq!(vec![VoteInfo{
        total_ballots: to_ve_token("400"),
        participants: 1
    }, VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }, VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }], e.get_proposal(0).unwrap().votes);
    assert_eq!(3, e.get_proposal(0).unwrap().participants);
    assert_eq!(to_ve_token("800"), e.get_proposal(0).unwrap().ve_amount_at_last_action);
}

#[test]
fn test_action_proposal_farming_reward_04(){
    let e = init_env();
    let users = Users::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_mint(&lpt_inner_id(), &users.bob, to_yocto("200"));
    e.mft_mint(&lpt_inner_id(), &users.charlie, to_yocto("200"));

    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);

    e.lock_lpt(&users.alice, to_yocto("200"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.lock_lpt(&users.bob, to_yocto("50"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.lock_lpt(&users.charlie, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    
    e.extend_whitelisted_accounts(&e.owner, vec![users.bob.account_id()]).assert_success();

    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["noct.near|nref.near&2657".to_string(), "nusdt.near|nusdc.near|ndai.near&1910".to_string(), "usn.near|nusdt.near&3020".to_string()], total_reward: 3 }, "FarmingReward".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();

    e.skip_time(DAY_SEC);

    e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    e.action_proposal(&users.bob, 0, Action::VoteFarm { farm_id: 1 }, None).assert_success();
    e.action_proposal(&users.charlie, 0, Action::VoteFarm { farm_id: 2 }, None).assert_success();
    
    assert_eq!(vec![VoteInfo{
        total_ballots: to_ve_token("400"),
        participants: 1
    }, VoteInfo{
        total_ballots: to_ve_token("100"),
        participants: 1
    }, VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }], e.get_proposal(0).unwrap().votes);
    assert_eq!(3, e.get_proposal(0).unwrap().participants);
    assert_eq!(to_ve_token("700"), e.get_proposal(0).unwrap().ve_amount_at_last_action);
}

#[test]
fn test_action_proposal_farming_reward_05(){
    let e = init_env();
    let users = Users::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_mint(&lpt_inner_id(), &users.bob, to_yocto("200"));
    e.mft_mint(&lpt_inner_id(), &users.charlie, to_yocto("200"));

    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);

    e.lock_lpt(&users.alice, to_yocto("200"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.lock_lpt(&users.bob, to_yocto("50"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    e.lock_lpt(&users.charlie, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    
    e.extend_whitelisted_accounts(&e.owner, vec![users.bob.account_id()]).assert_success();

    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["noct.near|nref.near&2657".to_string(), "nusdt.near|nusdc.near|ndai.near&1910".to_string(), "usn.near|nusdt.near&3020".to_string()], total_reward: 10 }, "FarmingReward".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();

    e.skip_time(DAY_SEC);

    e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    e.action_proposal(&users.bob, 0, Action::VoteFarm { farm_id: 1 }, None).assert_success();
    e.action_proposal(&users.charlie, 0, Action::VoteFarm { farm_id: 2 }, None).assert_success();
    
    assert_eq!(vec![VoteInfo{
        total_ballots: to_ve_token("400"),
        participants: 1
    }, VoteInfo{
        total_ballots: to_ve_token("100"),
        participants: 1
    }, VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }], e.get_proposal(0).unwrap().votes);
    assert_eq!(3, e.get_proposal(0).unwrap().participants);
    assert_eq!(to_ve_token("700"), e.get_proposal(0).unwrap().ve_amount_at_last_action);
}

#[test]
fn test_action_cancel(){
    let e = init_env();
    let users = Users::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));
    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);
    e.storage_deposit(&users.bob, &users.bob, to_yocto("1"));
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    
    e.extend_whitelisted_accounts(&e.owner, vec![users.bob.account_id()]).assert_success();

    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["noct.near|nref.near&2657".to_string(), "nusdt.near|nusdc.near|ndai.near&1910".to_string()], total_reward: 2 }, "FarmingReward".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();
    e.create_proposal(&users.bob, ProposalKind::Poll { options: vec!["topic1".to_string(), "topic2".to_string()] }, "Poll".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();
    e.create_proposal(&users.bob, ProposalKind::Common, "Common".to_string(), to_sec(e.current_time() + DAY_TS), DEFAULT_MIN_VOTING_DURATION_SEC, 1).assert_success();

    e.skip_time(DAY_SEC);

    e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    e.action_proposal(&users.alice, 1, Action::VotePoll { poll_id: 1 }, None).assert_success();
    e.action_proposal(&users.alice, 2, Action::VoteReject, None).assert_success();

    assert_eq!(vec![VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }], e.get_proposal(0).unwrap().votes);
    assert_eq!(to_ve_token("200"), e.get_proposal(0).unwrap().ve_amount_at_last_action);
    assert_eq!(vec![VoteInfo{
        total_ballots: 0,
        participants: 0
    }, VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }], e.get_proposal(1).unwrap().votes);
    assert_eq!(to_ve_token("200"), e.get_proposal(1).unwrap().ve_amount_at_last_action);
    assert_eq!(vec![VoteInfo{
        total_ballots: 0,
        participants: 0
    }, VoteInfo{
        total_ballots: to_ve_token("200"),
        participants: 1
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }], e.get_proposal(2).unwrap().votes);
    assert_eq!(to_ve_token("200"), e.get_proposal(2).unwrap().ve_amount_at_last_action);

    let alice = e.get_account_info(&users.alice).unwrap();
    assert_eq!(to_yocto("100"), alice.lpt_amount);
    assert_eq!(to_ve_token("200"), alice.ve_lpt_amount);
    assert_eq!(HashMap::from([(0, VoteDetail {
        action: Action::VoteFarm { farm_id: 0 }, amount: to_ve_token("200")
    }), (1, VoteDetail {
        action: Action::VotePoll { poll_id: 1 }, amount: to_ve_token("200")
    }), (2, VoteDetail {
        action: Action::VoteReject, amount: to_ve_token("200")
    })]), e.get_vote_detail(&users.alice));

    // error scene 
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(e.action_cancel(&users.dude, 0), E100_ACC_NOT_REGISTERED);

    // 2 : E206_NO_VOTED
    assert_err!(e.action_cancel(&users.alice, 5), E206_NO_VOTED);

    // success
    e.action_cancel(&users.alice, 0).assert_success();
    assert_eq!(vec![VoteInfo{
        total_ballots: 0,
        participants: 0
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }], e.get_proposal(0).unwrap().votes);
    assert_eq!(to_ve_token("200"), e.get_proposal(0).unwrap().ve_amount_at_last_action);

    assert_eq!(HashMap::from([(1, VoteDetail {
        action: Action::VotePoll { poll_id: 1 }, amount: to_ve_token("200")
    }), (2, VoteDetail {
        action: Action::VoteReject, amount: to_ve_token("200")
    })]), e.get_vote_detail(&users.alice));

    // 3 : E206_NO_VOTED
    assert_err!(e.action_cancel(&users.alice, 0), E206_NO_VOTED);

    e.action_cancel(&users.alice, 1).assert_success();
    assert_eq!(vec![VoteInfo{
        total_ballots: 0,
        participants: 0
    }, VoteInfo{
        total_ballots: 0,
        participants: 0
    }], e.get_proposal(1).unwrap().votes);
    assert_eq!(to_ve_token("200"), e.get_proposal(1).unwrap().ve_amount_at_last_action);

    assert_eq!(HashMap::from([(2, VoteDetail {
        action: Action::VoteReject, amount: to_ve_token("200")
    })]), e.get_vote_detail(&users.alice));

    // 4 : E204_VOTE_CAN_NOT_CANCEL
    e.skip_time(DEFAULT_MIN_VOTING_DURATION_SEC);
    assert_err!(e.action_cancel(&users.alice, 2), E204_VOTE_CAN_NOT_CANCEL);
    let alice = e.get_account_info(&users.alice).unwrap();
    assert_eq!(to_yocto("100"), alice.lpt_amount);
    assert_eq!(to_ve_token("200"), alice.ve_lpt_amount);
    assert_eq!(HashMap::new(), e.get_vote_detail(&users.alice));
    assert_eq!(HashMap::from([(2, VoteDetail {
        action: Action::VoteReject, amount: to_ve_token("200")
    })]), e.get_vote_detail_history(&users.alice));

    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    let alice = e.get_account_info(&users.alice).unwrap();
    assert_eq!(to_yocto("200"), alice.lpt_amount);
    assert_eq!(to_ve_token("400"), alice.ve_lpt_amount);
    assert_eq!(HashMap::new(), e.get_vote_detail(&users.alice));
    assert_eq!(HashMap::from([(2, VoteDetail {
        action: Action::VoteReject, amount: to_ve_token("200")
    })]), e.get_vote_detail_history(&users.alice));
}