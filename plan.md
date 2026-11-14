# Auspice: programmable fair launches

**Status:** source-complete for the current testnet milestone; deployment is deferred.

## Product

Auspice is a permissionless NEAR launch platform. A creator publishes a
keyless NEP-141 coin with two immutable allocation ceremonies:

1. **Genesis:** 70% of supply is allocated through Cloudflare Turnstile
   verification performed by a hash-pinned OutLayer TEE worker.
2. **Signal Doors:** the remaining supply is reserved for defined real-world
   events. A separate pinned TEE verifier evaluates the frozen event rule;
   eligible, freshly human-verified wallets claim a capped allocation.

Supply follows public rules rather than a founder mint button. Unused Genesis
or Door reserve becomes permanently unmintable.

### Daily Streak template

The factory also supports an entirely on-chain ritual template. After Genesis,
holders may check in once in each immutable daily window. If the fixed target
is met, every checked-in holder receives the fixed launch-time mint. If it is
missed, the fixed launch-time amount is permanently removed from the ritual
reserve and the streak resets. The contract stores current streak, all-time
high score, daily progress, mint totals, and burn totals. Settlement is
permissionless and the participant cap bounds its work.

`node scripts/simulate-streak.mjs` provides a deterministic 60-day rehearsal
for three participation scenarios before a creator chooses the launch values.

### Proofbound Launch template

Proofbound is a no-oracle public-commitment launch. The creator freezes a
statement, evidence requirement, evidence deadline, voting period, quorum,
pass threshold, creator reward, and voter reward at launch. The creator may
submit one HTTPS evidence URL plus its 32-byte hash; after the evidence
deadline anyone can open a blank-evidence vote. Each settled Genesis recipient
needs a fresh Turnstile-in-TEE approval to cast exactly one equal vote. On a
pass, the creator can claim its locked slice and yes-voters can individually
claim their fixed pro-rata reward. On a failure or no quorum, both slices become
permanently unmintable. This protects fixed token rules, not DEX liquidity.

## Trust model

- The NEAR contract stores immutable configuration, validates callbacks, owns
  NEP-141 balances, and is the only component that mints or removes supply.
- The frontend only reads state and composes wallet transactions.
- The human-verifier runs inside OutLayer with `TURNSTILE_SECRET` supplied by
  OutLayer protected secrets. It returns a wallet-bound, short-lived approval.
- The event-verifier runs inside OutLayer with a frozen source URL and WASM
  hash. It returns a request-bound event result.
- CAPTCHA is bot resistance, not one-person identity. A TEE attests program
  execution, not the objective truth of an external source.

## Enabled source templates

### GitHub Release

The Door pins a repository, its `releases/latest` API URL, and optional release
tag. The verifier accepts only that allowlisted endpoint.

### Open-Meteo Rainfall

The Door pins latitude, longitude, an Open-Meteo `current=rain` URL, and a
minimum rainfall threshold in tenths of a millimetre. The verifier hashes the
fetched response and compares the current rain reading to that threshold.

USGS earthquake, official sports score, and public-account-post enum variants
remain deliberately disabled until each has its own reviewed source adapter.

## Immutable launch configuration

Every launched coin freezes:

- maximum supply, Genesis reserve, Door reserve, price, user caps, and dates;
- event template, source identity/URL, structured predicate, verifier URL/hash,
  build target, and Door deadline;
- human-verifier URL/hash, secret profile/account, hostname, actions, and
  approval lifetime.

There are no setters for any of these rules. The factory owner can change the
global code hash only for coins created in the future; created coin accounts
have no access keys.

## Allocation flow

```text
wallet → Turnstile → request_human_check
       → OutLayer human TEE → private contract callback
       → stored one-use approval → Genesis request or Door claim

wallet → request_signal_check → OutLayer event TEE
       → private contract callback → Door opens
       → fresh human verification → capped Door claim
```

Genesis requests settle permissionlessly after close. Any unused reserve is
removed from the mintable cap at finalization or Door close.

## Frontend

- `/create`: choose Normal, Fortress, Work-to-Mint, Locked Market, GitHub
  Release, Rainfall, Daily Streak, or Proofbound; preset links make an
  inspectable configuration easy to share before launch.
- `/coins`: factory-wide explorer with lifecycle, supply, and Door summaries.
- `/coins/[coinId]`: Genesis verification/allocation, settlement/finalization,
  signal submission, Door verification/claim, closure/burn, and public history.
- `/arena`: targets a configured public demo coin and makes real testnet
  protection probes plus deliberately failing liquidity/rule-rewrite attempts.
- `/verify`: explains how to inspect immutable state and receipts.

### Locked Market template

The Locked Market template assigns the post-Genesis reserve to an immutable
curve. `buy_curve` accepts NEAR and mints from its fixed inventory;
`sell_curve` burns tokens and pays only from the contract's locked NEAR
reserve. There is no creator, factory, or admin withdrawal path. The explicit
withdrawal and configuration-change methods deliberately panic with named
errors for the Demo Arena. This is contract-held curve liquidity, not an
external AMM LP position.

## Current development verification

- `cargo test --workspace`
- `scripts/build-contracts.sh`
- WASI P2 builds for both TEE workers
- `cd web && npm run lint && npm run build`

The offline lifecycle test covers TEE-approved Genesis, settlement,
finalization, TEE-opened Door, fresh TEE-approved Door claim, and unused
reserve removal.

## Deployment gate

No testnet deployment occurs until explicitly authorized. Before deployment:

1. Upload the current event verifier to OutLayer and record its FastFS hash.
2. Confirm `TURNSTILE_SECRET` access for the human verifier’s pinned build.
3. Upload the rebuilt global coin code and point the factory at that successful
   global hash.
4. Set public Vercel environment values for the two verifier artifacts.
5. Create fresh test coins and run the full live GitHub, Rainfall, and Daily
   Streak flows.

Exact artifacts and deployment order are recorded in
[docs/DEPLOYMENT.md](./docs/DEPLOYMENT.md).
