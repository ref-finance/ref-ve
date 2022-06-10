import { NEAR } from 'near-workspaces-ava';
import { initWorkSpace, assertFailure } from './helper';

const workspace = initWorkSpace();

workspace.test('set_owner', async (test, { ref_ve, ft, mft, owner, alice, bob }) => {
  let md = await ref_ve.view('get_metadata');
  test.log(md);

  test.deepEqual(md, {
    version: '0.0.1',
    owner_id: 'ref_owner.test.near',
    operators: [],
    whitelisted_accounts: [],
    lptoken_contract_id: 'lpt.test.near',
    lptoken_id: ':0',
    lptoken_decimals: 24,
    account_count: '0',
    proposal_count: '0',
    cur_total_ve_lpt: '0',
    cur_lock_lpt: '0',
    lostfound: '0',
  });

  await owner.call(ref_ve, 'set_owner', { owner_id: alice }, { attachedDeposit: NEAR.from("1") });

  test.is(
    (await ref_ve.view('get_metadata') as any).owner_id,
    alice.accountId,
  );

  await alice.call(ref_ve, 'set_owner', { owner_id: bob }, { attachedDeposit: NEAR.from("1") });

  test.is(
    (await ref_ve.view('get_metadata') as any).owner_id,
    bob.accountId,
  );

  await bob.call(ref_ve, 'set_owner', { owner_id: owner }, { attachedDeposit: NEAR.from("1") });

  test.is(
    (await ref_ve.view('get_metadata') as any).owner_id,
    owner.accountId,
  );
});

workspace.test('manage_operators', async (test, { ref_ve, ft, mft, owner, alice, bob }) => {

  await owner.call(ref_ve, 'extend_operators', { operators: [alice, bob] }, { attachedDeposit: NEAR.from("1") });
  test.deepEqual(
    (await ref_ve.view('get_metadata') as any).operators,
    [alice.accountId, bob.accountId],
  );

  await owner.call(ref_ve, 'remove_operators', { operators: [bob] }, { attachedDeposit: NEAR.from("1") });
  test.deepEqual(
    (await ref_ve.view('get_metadata') as any).operators,
    [alice.accountId],
  );
});
