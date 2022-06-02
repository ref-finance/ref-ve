mod setup;
use crate::setup::*;
use std::collections::HashMap;

#[test]
fn test_create_proposal(){
    let e = init_env();
    let users = Users::init(&e);
    let tokens = Tokens::init(&e);

    e.mft_mint(&lpt_inner_id(), &users.alice, to_yocto("200"));

    e.mft_storage_deposit(&lpt_id(), &e.ve_contract.user_account);

    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    
    e.extend_whitelisted_accounts(&e.owner, vec![users.alice.account_id(), users.dude.account_id()]).assert_success();
    
    // error scene 
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(e.create_proposal(&users.dude, ProposalKind::FarmingReward { farm_list: vec![], num_portions: 2 }, e.current_time(), 1000, None, 0), E100_ACC_NOT_REGISTERED);
    
    // 2 : E002_NOT_ALLOWED just whitelisted accounts can create proposal 
    e.storage_deposit(&users.bob, &users.bob, to_yocto("1"));
    assert_err!(e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec![], num_portions: 2 }, e.current_time(), 1000, None, 0), E002_NOT_ALLOWED);

    // 3 : E402_INVALID_START_TIME 
    assert_err!(e.create_proposal(&users.alice, ProposalKind::FarmingReward { farm_list: vec![], num_portions: 2 }, e.current_time(), 1000, None, 0), E402_INVALID_START_TIME);
    assert_err!(e.create_proposal(&users.alice, ProposalKind::Poll { descriptions: vec![] }, e.current_time(), 1000, None, 0), E402_INVALID_START_TIME);
    assert_err!(e.create_proposal(&users.alice, ProposalKind::Common { description: "Common Proposal".to_string() }, e.current_time(), 1000, None, to_yocto("1")), E402_INVALID_START_TIME);

    // 4 : E405_PROPOSAL_NOT_SUPPORT_INCENTIVE
    assert_err!(e.create_proposal(&users.alice, ProposalKind::FarmingReward { farm_list: vec![], num_portions: 2 }, e.current_time() + DAY_TS, 1000, Some((tokens.nref.account_id(), IncentiveType::Evenly)), 0), E405_PROPOSAL_NOT_SUPPORT_INCENTIVE);
    assert_err!(e.create_proposal(&users.alice, ProposalKind::Common { description: "Common Proposal".to_string() }, e.current_time() + DAY_TS, 1000, Some((tokens.nref.account_id(), IncentiveType::Evenly)), 0), E405_PROPOSAL_NOT_SUPPORT_INCENTIVE);

    let mut before = e.get_metadata();
    e.create_proposal(&users.alice, ProposalKind::FarmingReward { farm_list: vec![], num_portions: 2 }, e.current_time() + DAY_TS, 1000, None, 0).assert_success();
    e.create_proposal(&users.alice, ProposalKind::Poll { descriptions: vec![] }, e.current_time() + DAY_TS, 1000, None, 0).assert_success();
    e.create_proposal(&users.alice, ProposalKind::Common { description: "Common Proposal".to_string() }, e.current_time() + DAY_TS, DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, None, to_yocto("1")).assert_success();
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
    e.create_proposal(&users.dude, ProposalKind::Common { description: "Common Proposal1".to_string() }, e.current_time() + DAY_TS, 1000, None, 0).assert_success();
    e.create_proposal(&users.alice, ProposalKind::Common { description: "Common Proposal2".to_string() }, e.current_time() + DAY_TS, DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, None, to_yocto("1")).assert_success();
    
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
    
    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["ref<>celo".to_string(), "usn<>usdt".to_string()], num_portions: 2 }, e.current_time() + DAY_TS, 1000, None, 0).assert_success();
    e.create_proposal(&users.bob, ProposalKind::Poll { descriptions: vec!["topic1".to_string(), "topic2".to_string()] }, e.current_time() + DAY_TS, 1000, None, 0).assert_success();
    e.create_proposal(&users.bob, ProposalKind::Common { description: "Common Proposal".to_string() }, e.current_time() + DAY_TS, DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, None, to_yocto("1")).assert_success();

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

    assert_eq!(vec![to_yocto("200"), 0], e.get_proposal(0).unwrap().votes);
    assert_eq!(vec![0, to_yocto("200")], e.get_proposal(1).unwrap().votes);
    assert_eq!(vec![0, to_yocto("200"), 0, to_yocto("200")], e.get_proposal(2).unwrap().votes);

    assert_eq!(HashMap::from([(0, VoteDetail{
        action: Action::VoteFarm { farm_id: 0 },
        amount: to_yocto("200"),
    }), (1, VoteDetail{
        action: Action::VotePoll { poll_id: 1 },
        amount: to_yocto("200"),
    }), (2, VoteDetail{
        action: Action::VoteReject,
        amount: to_yocto("200"),
    })]), e.get_vote_detail(&users.alice));

    // append
    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    assert_eq!(vec![to_yocto("400"), 0], e.get_proposal(0).unwrap().votes);
    assert_eq!(vec![0, to_yocto("400")], e.get_proposal(1).unwrap().votes);
    assert_eq!(vec![0, to_yocto("400"), 0, to_yocto("400")], e.get_proposal(2).unwrap().votes);

    assert_eq!(1, e.get_proposal(0).unwrap().participants);
    assert_eq!(1, e.get_proposal(1).unwrap().participants);
    assert_eq!(1, e.get_proposal(2).unwrap().participants);

    assert_eq!(HashMap::from([(0, VoteDetail{
        action: Action::VoteFarm { farm_id: 0 },
        amount: to_yocto("400"),
    }), (1, VoteDetail{
        action: Action::VotePoll { poll_id: 1 },
        amount: to_yocto("400"),
    }), (2, VoteDetail{
        action: Action::VoteReject,
        amount: to_yocto("400"),
    })]), e.get_vote_detail(&users.alice));

    assert_eq!(HashMap::new(), e.get_unclaimed_proposal(&users.alice));

    e.skip_time(DAY_SEC + DEFAULT_MAX_LOCKING_DURATION_SEC);
    println!("{:?}", e.get_proposal(0));
    assert_eq!(false, e.get_proposal(2).unwrap().is_nonsense.unwrap());
    println!("{:?}", e.get_account_info(&users.alice));
    let alice = e.get_account_info(&users.alice).unwrap();
    assert_eq!(to_yocto("200"), alice.lpt_amount);
    assert_eq!(to_yocto("400"), alice.ve_lpt_amount);
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

    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["ref<>celo".to_string(), "usn<>usdt".to_string()], num_portions: 2 }, e.current_time() + DAY_TS, 1000, None, 0).assert_success();

    e.skip_time(DAY_SEC);

    e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    e.action_proposal(&users.bob, 0, Action::VoteFarm { farm_id: 1 }, None).assert_success();
    
    assert_eq!(vec![to_yocto("200"), to_yocto("200")], e.get_proposal(0).unwrap().votes);
    assert_eq!(2, e.get_proposal(0).unwrap().participants);

    e.skip_time(DAY_SEC + DEFAULT_MAX_LOCKING_DURATION_SEC);
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

    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["ref<>celo".to_string(), "usn<>usdt".to_string()], num_portions: 2 }, e.current_time() + DAY_TS, 1000, None, 0).assert_success();

    e.skip_time(DAY_SEC);

    e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    e.action_proposal(&users.bob, 0, Action::VoteFarm { farm_id: 1 }, None).assert_success();
    
    assert_eq!(vec![to_yocto("400"), to_yocto("200")], e.get_proposal(0).unwrap().votes);
    assert_eq!(2, e.get_proposal(0).unwrap().participants);

    e.skip_time(DAY_SEC + DEFAULT_MAX_LOCKING_DURATION_SEC);
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

    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["ref<>celo".to_string(), "usn<>usdt".to_string(), "ref<>aurora".to_string()], num_portions: 3 }, e.current_time() + DAY_TS, 1000, None, 0).assert_success();

    e.skip_time(DAY_SEC);

    e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    e.action_proposal(&users.bob, 0, Action::VoteFarm { farm_id: 1 }, None).assert_success();
    e.action_proposal(&users.charlie, 0, Action::VoteFarm { farm_id: 2 }, None).assert_success();
    
    assert_eq!(vec![to_yocto("400"), to_yocto("200"), to_yocto("200")], e.get_proposal(0).unwrap().votes);
    assert_eq!(3, e.get_proposal(0).unwrap().participants);

    e.skip_time(DAY_SEC + DEFAULT_MAX_LOCKING_DURATION_SEC);
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

    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["ref<>celo".to_string(), "usn<>usdt".to_string(), "ref<>aurora".to_string()], num_portions: 3 }, e.current_time() + DAY_TS, 1000, None, 0).assert_success();

    e.skip_time(DAY_SEC);

    e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    e.action_proposal(&users.bob, 0, Action::VoteFarm { farm_id: 1 }, None).assert_success();
    e.action_proposal(&users.charlie, 0, Action::VoteFarm { farm_id: 2 }, None).assert_success();
    
    assert_eq!(vec![to_yocto("400"), to_yocto("100"), to_yocto("200")], e.get_proposal(0).unwrap().votes);
    assert_eq!(3, e.get_proposal(0).unwrap().participants);

    e.skip_time(DAY_SEC + DEFAULT_MAX_LOCKING_DURATION_SEC);
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

    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["ref<>celo".to_string(), "usn<>usdt".to_string(), "ref<>aurora".to_string()], num_portions: 10 }, e.current_time() + DAY_TS, 1000, None, 0).assert_success();

    e.skip_time(DAY_SEC);

    e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    e.action_proposal(&users.bob, 0, Action::VoteFarm { farm_id: 1 }, None).assert_success();
    e.action_proposal(&users.charlie, 0, Action::VoteFarm { farm_id: 2 }, None).assert_success();
    
    assert_eq!(vec![to_yocto("400"), to_yocto("100"), to_yocto("200")], e.get_proposal(0).unwrap().votes);
    assert_eq!(3, e.get_proposal(0).unwrap().participants);

    e.skip_time(DAY_SEC + DEFAULT_MAX_LOCKING_DURATION_SEC);
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

    e.create_proposal(&users.bob, ProposalKind::FarmingReward { farm_list: vec!["ref<>celo".to_string(), "usn<>usdt".to_string()], num_portions: 2 }, e.current_time() + DAY_TS, 1000, None, 0).assert_success();
    e.create_proposal(&users.bob, ProposalKind::Poll { descriptions: vec!["topic1".to_string(), "topic2".to_string()] }, e.current_time() + DAY_TS, 1000, None, 0).assert_success();
    e.create_proposal(&users.bob, ProposalKind::Common { description: "Common Proposal".to_string() }, e.current_time() + DAY_TS, DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC, None, to_yocto("1")).assert_success();

    e.skip_time(DAY_SEC);

    e.action_proposal(&users.alice, 0, Action::VoteFarm { farm_id: 0 }, None).assert_success();
    e.action_proposal(&users.alice, 1, Action::VotePoll { poll_id: 1 }, None).assert_success();
    e.action_proposal(&users.alice, 2, Action::VoteReject, None).assert_success();

    assert_eq!(vec![to_yocto("200"), 0], e.get_proposal(0).unwrap().votes);
    assert_eq!(vec![0, to_yocto("200")], e.get_proposal(1).unwrap().votes);
    assert_eq!(vec![0, to_yocto("200"), 0, to_yocto("200")], e.get_proposal(2).unwrap().votes);

    let alice = e.get_account_info(&users.alice).unwrap();
    assert_eq!(to_yocto("100"), alice.lpt_amount);
    assert_eq!(to_yocto("200"), alice.ve_lpt_amount);
    assert_eq!(HashMap::from([(0, VoteDetail {
        action: Action::VoteFarm { farm_id: 0 }, amount: to_yocto("200")
    }), (1, VoteDetail {
        action: Action::VotePoll { poll_id: 1 }, amount: to_yocto("200")
    }), (2, VoteDetail {
        action: Action::VoteReject, amount: to_yocto("200")
    })]), e.get_vote_detail(&users.alice));

    // error scene 
    // 1 : E100_ACC_NOT_REGISTERED
    assert_err!(e.action_cancel(&users.dude, 0), E100_ACC_NOT_REGISTERED);

    // 2 : E206_NO_VOTED
    assert_err!(e.action_cancel(&users.alice, 5), E206_NO_VOTED);

    // success
    e.action_cancel(&users.alice, 0).assert_success();
    assert_eq!(vec![0, 0], e.get_proposal(0).unwrap().votes);
    assert_eq!(HashMap::from([(1, VoteDetail {
        action: Action::VotePoll { poll_id: 1 }, amount: to_yocto("200")
    }), (2, VoteDetail {
        action: Action::VoteReject, amount: to_yocto("200")
    })]), e.get_vote_detail(&users.alice));

    // 3 : E206_NO_VOTED
    assert_err!(e.action_cancel(&users.alice, 0), E206_NO_VOTED);

    e.action_cancel(&users.alice, 1).assert_success();
    assert_eq!(vec![0, 0], e.get_proposal(1).unwrap().votes);
    assert_eq!(HashMap::from([(2, VoteDetail {
        action: Action::VoteReject, amount: to_yocto("200")
    })]), e.get_vote_detail(&users.alice));

    // 4 : E204_VOTE_CAN_NOT_CANCEL
    e.skip_time(DEFAULT_MIN_PROPOSAL_VOTING_PERIOD_SEC);
    assert_err!(e.action_cancel(&users.alice, 2), E204_VOTE_CAN_NOT_CANCEL);
    let alice = e.get_account_info(&users.alice).unwrap();
    assert_eq!(to_yocto("100"), alice.lpt_amount);
    assert_eq!(to_yocto("200"), alice.ve_lpt_amount);
    assert_eq!(HashMap::new(), e.get_vote_detail(&users.alice));
    assert_eq!(HashMap::from([(2, VoteDetail {
        action: Action::VoteReject, amount: to_yocto("200")
    })]), e.get_vote_detail_history(&users.alice));

    e.lock_lpt(&users.alice, to_yocto("100"), DEFAULT_MAX_LOCKING_DURATION_SEC).assert_success();
    let alice = e.get_account_info(&users.alice).unwrap();
    assert_eq!(to_yocto("200"), alice.lpt_amount);
    assert_eq!(to_yocto("400"), alice.ve_lpt_amount);
    assert_eq!(HashMap::new(), e.get_vote_detail(&users.alice));
    assert_eq!(HashMap::from([(2, VoteDetail {
        action: Action::VoteReject, amount: to_yocto("200")
    })]), e.get_vote_detail_history(&users.alice));
}