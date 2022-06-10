import { Workspace, NEAR, Gas, NearAccount, BN } from "near-workspaces-ava";

export function initWorkSpace() {
  return Workspace.init(async ({ root }) => {
    const owner = await root.createAccount('ref_owner');
    const alice = await root.createAccount('alice');
    const bob = await root.createAccount('bob');
    const lpt_contract = await root.createAccount('lpt');
    const lptoken_id = ':0';
    const symbol = 'loveRef';
    const lptoken_decimals = 24;

    const ref_ve = await deployContract(root, owner.accountId, symbol, lpt_contract.accountId, lptoken_id, lptoken_decimals);
    const ft = await deployFt(root);
    const mft = await deployMft(root);

    return { ref_ve, ft, mft, owner, alice, bob };
  });
}

export async function deployContract(
  root: NearAccount,
  owner_id: string,
  symbol: string,
  lptoken_contract_id: string,
  lptoken_id: string,
  lptoken_decimals: number,
  contractId = 'ref-ve',
) {
  return root.createAndDeploy(
    contractId,
    'compiled-contracts/ref_ve.wasm',
    {
      method: 'new',
      args: {
        owner_id,
        symbol,
        lptoken_contract_id,
        lptoken_id,
        lptoken_decimals
      }
    }
  )
}

export async function deployFt(
    root: NearAccount,
    contractId = 'mock-ft',
) {
    return root.createAndDeploy(
        contractId,
        'compiled-contracts/mock_ft.wasm',
        {
            method: 'new',
            args: {
                name: "mock-ft",
                symbol: "ft",
                decimals: 24,
            }
        }
    )
}

export async function deployMft(
    root: NearAccount,
    contractId = 'mock-mft',
) {
    return root.createAndDeploy(
        contractId,
        'compiled-contracts/mock_mft.wasm',
        {
            method: 'new',
            args: {
                name: "mock-mft",
                symbol: "mft",
                decimals: 24,
            }
        }
    )
}

export async function assertFailure(
  test: any,
  action: Promise<unknown>,
  errorMessage?: string
) {
  let failed = false;

  try {
    await action;
  } catch (e) {
    if (errorMessage) {
      let errObj = eval('('+ e.message + ')');
      // test.log(errObj);
      // let msg: string = e.kind.ExecutionError;
      let msg: string = errObj.result.status.Failure.ActionError.kind.FunctionCallError.ExecutionError;
      test.truthy(
        msg.includes(errorMessage),
        `Bad error message. expect: "${errorMessage}", actual: "${msg}"`
      );
    }
    failed = true;
  }

  test.is(
    failed,
    true,
    "Action didn't fail"
  );
}

export async function callWithMetrics(
    account: NearAccount,
    contractId: NearAccount | string,
    methodName: string,
    args: Record<string, unknown>,
    options?: {
      gas?: string | BN;
      attachedDeposit?: string | BN;
    }
  ) {
    const txResult = await account.call_raw(contractId, methodName, args, options);
    const successValue = txResult.parseResult();
    const outcome = txResult.result.transaction_outcome.outcome;
    const logs = txResult.logs;
    const gasBurnt = Gas.from(outcome.gas_burnt);
    const tokensBurnt = NEAR.from(outcome.gas_burnt + '000000000');
    return {
      successValue,
      metrics: {
        tokensBurnt,
        gasBurnt,
        logs
      }
    }
}

// This is needed due to some unknown issues of balance accuracy in sandbox
export async function numbersEqual(test: any, a: NEAR, b: NEAR, diff = 0.000001) {
  test.is(
    a.sub(b).abs().lt(NEAR.parse(diff.toString())),
    true
  )
}

// Match considering precision loss
export async function noMoreThanOneYoctoDiff(test: any, a: NEAR, b: NEAR) {
  test.is(
    a.sub(b).abs().lte(NEAR.from("1")),
    true
  )
}

export async function untill(ts: number) {
  let now = Date.now();
  if (now < ts) {
      return new Promise((resolve) => setTimeout(resolve, ts-now));
  } 
}

export function skip(...args: any[]) {
  console.debug(`Skipping test ${args[0]} ...`);
};

export async function registerFungibleTokenUser(ft: NearAccount, user: NearAccount) {
  const storage_balance = await ft.view(
    'storage_balance_bounds',
    {}
  ) as any;
  await user.call(
    ft,
    'storage_deposit',
    { account_id: user },
    { attachedDeposit: storage_balance.min.toString() },
  );
}

export function parseNEAR(a: number): NEAR {
  const yoctoString = a.toLocaleString('fullwide', { useGrouping: false });
  return NEAR.from(yoctoString);
}