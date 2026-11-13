# Auspice: programmable fair launches

## Build, testnet, demo, and submission plan

**Track:** Memecoins With Mechanics (`memecoins`)

**Status:** implementation blueprint; testnet only

**Working product name:** Auspice

**One-line pitch:** A permissionless NEAR launch platform where any creator can publish a NEP-141 coin with a human-gated Genesis and later supply released only by immutable, TEE-verified real-world Signal Doors.

**Core promise:** Supply follows a public rule. Bots do not receive privileged allocations. The contract, not a backend, settles every allocation and burn.

**Product positioning:** A protocol for memecoins with a credible birth and a non-discretionary future.

**Public framing:** Auspice gives every supported memecoin two immutable ceremonies: a human-gated Genesis, then real-world Signal Doors for its future supply. The creator cannot silently allocate more tokens after launch; unused Door allocation becomes permanently unmintable.

**Short lines to test in the demo and README:**

- No insiders at birth. No discretion after.
- Born to humans. Released by reality.
- Every token must earn its way into circulation.

### Platform decision: source templates, not arbitrary post-launch rules

Auspice is a platform, not a weather coin. A creator selects one or more supported Signal Door templates while launching a coin. Version one enables GitHub releases end-to-end; USGS earthquake reports, official sports scores, and approved public-account posts are explicitly modeled future adapters, not unverified launch options. Each Door freezes its source adapter, exact source identity, predicate parameters, TEE WASM URL/hash/build target, allocation reserve, claim cap, eligibility rule, and deadline.

Community members may submit an **evaluation request** for an already configured Door. They may not create a new source, predicate, reserve, or Door after the coin is launched. The UI records those requests in a public Submitted Signals queue (`pending`, `failed`, `verified`, `open`, `closed/burned`). This preserves permissionless participation without turning future supply into a discretionary vote or an arbitrary-URL oracle.

---

## 0. Read this first

This document is a build specification, not legal, investment, or production-security advice.

The deliverable is a NEAR testnet project.

Do not use real funds for the hackathon demo.

Do not present testnet behavior as audited, production-ready, or Sybil-proof.

Do not imply that a CAPTCHA proves one human equals one person.

Do not promise that a TEE proves an external data source is truthful.

The TEE can prove which pinned program fetched which response at a time.

The contract can prove it accepted only an attested result matching its frozen rule.

The team must plainly disclose the remaining risks in the README and demo.

The project must remain public and reproducible for judging.

The human team—not an agent—must publish the X demo post and approve the final submission.

---

## 1. Problem statement

### 1.1 The user problem

Memecoin launches often claim to be fair while early supply is captured by bots, insiders, or wallets that know the launch mechanics first.

Even a launch that starts fairly usually gives future supply control to a team, an opaque treasury, or arbitrary emissions.

Buyers cannot answer two simple questions:

1. Did everyone have the same chance at the first allocation?
2. Why and how will future supply enter circulation?

### 1.2 Why this matters for a memecoin

A memecoin is social money.

Its community forms around a shared story, a shared rule, and a visible ritual.

If the only rule is “buy early and hope,” the community has no credible shared contract.

Auspice makes the coin’s origin and future issuance part of the meme.

### 1.3 The project answer

Auspice gives every supported coin two immutable ceremonies:

1. **Genesis Ceremony:** an equal-price, capped allocation for wallet-bound, human-verified participants.
2. **Signal Doors:** later allocation windows triggered only by an immutable source-template rule verified through an attested TEE result.

The coin’s supply is not discretionary.

The future mint schedule is neither a founder button nor a backend cron job.

### 1.4 The meme

> Supply follows reality, not insiders.

Alternative public copy:

> The world pulls the trigger. Humans get the mint.

Alternative short copy:

> Born fair. Released by signal.

---

## 2. Track fit

The hackathon track asks for memecoins where the meme does something.

It specifically names supply that reacts to the community, treasuries with rules, and on-chain rituals/challenges.

Auspice fits all three in a constrained, verifiable way.

### 2.1 Supply reacts to a public rule

The configured max supply is split into a Genesis allocation and Signal Door reserves.

Signal reserve units remain unminted until the corresponding attested event occurs.

If an event never occurs, its supply never enters circulation.

If a Signal Door opens and participants do not claim, the unclaimed allocation is permanently burned from the mintable cap.

### 2.2 The community matters

The coin does not mint merely because an API changed.

An event opens a chance for new verified participants to claim a capped allocation.

The supply therefore requires both public reality and real participation.

### 2.3 Ritual

Each configured Signal Door becomes a public event:

- watch the signal;
- inspect the attestation;
- open the Door on-chain;
- complete the human pass;
- claim the fixed allocation before the timer ends;
- inspect the mint or burn receipt.

### 2.4 No chart dependency

The demo must never claim price appreciation as a product outcome.

The success metric is whether the rules correctly produce a fair, inspectable allocation and burn.

---

## 3. Product definition

### 3.1 User personas

**Coin creator** creates a Auspice launch configuration.

**Participant** completes a human pass and submits a capped allocation request.

**Signal submitter** requests evaluation of an already configured Door and pays the attached OutLayer execution deposit. The submitter has no mint authority.

**Verifier** inspects the contract configuration, events, and TEE evidence.

**Trader** may transfer or trade the token after it becomes live.

**Judge** follows one end-to-end testnet path without trusting the UI.

### 3.2 Main user stories

As a creator, I can configure a coin’s name, symbol, fixed supply, initial price, Genesis policy, and supported-template Signal Doors before launch.

As a participant, I can see exactly when allocation opens and what I can receive.

As a participant, I can prove my pass is bound to my wallet and cannot be reused.

As a signal submitter, I can inspect the configured source and trigger condition, submit an evaluation request, and follow its public status without gaining allocation privileges.

As a verifier, I can reproduce the contract’s decision from immutable configuration and public receipts.

As a token holder, I can use normal NEP-141 transfers after the coin becomes live.

### 3.3 Non-goals

Auspice is not a trading terminal.

Auspice is not a bonding curve.

Auspice is not a DEX.

Auspice is not an AI agent platform.

Auspice is not a generic CAPTCHA product.

Auspice is not a guarantee of one-human-one-wallet identity.

Auspice is not an oracle claiming objective truth about arbitrary websites or an arbitrary-URL fetch service.

Auspice is not mainnet-ready after the hackathon.

---

## 4. Platform proof coin

The platform supports multiple immutable source templates. For the hackathon, ship one polished, live testnet proof coin that demonstrates the platform without making rain its identity.

### 4.1 Demo identity

**Working name:** Ship Signal

**Working ticker:** SHIP

**Meme:** “Ship it, then let the next humans in.”

**Demo signal:** a specified public GitHub repository publishes a release matching a frozen tag/predicate.

**Reason:** This demonstrates the general platform with a public structured API, no weather overlap, and an event the team can intentionally reproduce on testnet. Sports, earthquake, and approved-public-account templates remain supported platform types, not the flagship identity.

### 4.2 Demo supply configuration

Use deliberately small testnet values.

Maximum mintable supply: `1,000,000 SHIP`.

Genesis reserve: `700,000 SHIP` (70%).

Signal Door reserve: `300,000 SHIP` (30%).

Number of Signal Doors: `3`.

Per-door mintable amount: `100,000 SHIP`.

Genesis participant cap: `10,000 SHIP`.

Signal Door participant cap: `5,000 SHIP`.

Genesis duration: `15 minutes` for the recorded demo.

Signal Door claim duration: `10 minutes` for the recorded demo.

Unclaimed reserve action: permanently remove it from the mintable cap and emit a burn event.

### 4.3 Production-like configuration principles

Every rule must be stored in contract state at creation.

Every rule must be readable by a view method.

Every rule must be shown in the UI before a user participates.

No rule may be mutable after creation.

No privileged method may mint, change the source, extend a deadline, or reopen a closed Door.

---

## 5. NEP-141: required foundation

### 5.1 What NEP-141 is

NEP-141 is NEAR’s fungible-token standard.

It gives a token standard methods that wallets, explorers, DEXs, and contracts can understand.

The official specification is the source of truth.

Resource: https://github.com/near/NEPs/blob/master/neps/nep-0141.md

### 5.2 Required interfaces

Implement the fungible token core.

Implement `ft_transfer`.

Implement `ft_transfer_call`.

Implement `ft_total_supply`.

Implement `ft_balance_of`.

Implement the resolver required for transfer-call refunds.

Implement NEP-145 storage management.

Implement NEP-148 fungible-token metadata.

Expose `ft_metadata` with `name`, `symbol`, `icon`, `reference`, `reference_hash`, and `decimals` as appropriate.

### 5.3 Why it is essential

Without NEP-141, this is an isolated points system rather than a credible NEAR token.

With NEP-141, holders can use standard transfers after launch.

With NEP-141, a RHEA pool can trade the token if liquidity is seeded.

With `ft_transfer_call`, later contracts can accept the token atomically.

With storage management, recipients explicitly cover the state needed to hold balances.

### 5.4 What NEP-141 does not do

NEP-141 does not decide fair allocation.

NEP-141 does not verify a CAPTCHA.

NEP-141 does not itself verify an external real-world event.

NEP-141 does not create liquidity.

NEP-141 does not prevent Sybil identities.

Auspice’s custom state machine supplies those mechanics.

### 5.5 Standard behavior after launch

After a coin reaches `Live`, its transfers must behave like an ordinary NEP-141 token.

Do not add hidden transfer taxes.

Do not add an admin freeze function.

Do not add a post-launch privileged mint function.

Do not make a user depend on the frontend to transfer their token.

---

## 6. Auspice mechanism specification

### 6.1 Lifecycle states

Each coin has one lifecycle state.

`Draft` means configuration exists but the public ceremony has not started.

`GenesisOpen` means verified users may reserve capped allocation.

`GenesisSettling` means allocation window is closed and settlement is available.

`Live` means Genesis is finalized and normal NEP-141 transfers are enabled.

`DoorOpen` means one Signal Door is live and eligible new humans may claim.

`Finished` means every Signal Door is settled or expired and no more supply may mint.

`Cancelled` is not allowed after public Genesis creation.

### 6.2 Genesis allocation rule

Genesis opens at a fixed `genesis_open_at` timestamp.

Genesis closes at a fixed `genesis_close_at` timestamp.

A participant must present a valid human pass.

A participant must have a registered NEP-141 storage balance before allocation can mint.

A participant must not exceed the immutable cap.

All valid users see the same stated unit price.

The demo may use equal fixed allocations instead of a full auction.

If fixed-price demand exceeds the Genesis reserve, use deterministic pro-rata allocation.

Any excess attached NEAR must be credited to a refundable balance.

Settlement must be permissionless after the deadline.

### 6.3 Signal Door rule

Every Door includes a fixed event source and predicate.

Every Door includes a unique `door_id`.

Every Door includes a fixed mintable reserve.

Every Door includes a fixed claim-window duration.

Every Door includes a fixed allocation cap.

A valid attestation may open a Door only once.

The contract stores the hash of each accepted attestation response and rejects that response hash if it is submitted again, including against another Door.

Opening a Door must be permissionless.

Claiming from a Door requires a fresh eligible human pass.

A Genesis participant is not eligible for the same Door if the chosen rule is “new humans only.”

Closing a Door must be permissionless after its deadline.

Closing a Door removes unclaimed units from mintability and emits a burn receipt.

After a Door closes, another configured locked Door may open. The lifecycle reaches `Finished` only after no configured Door remains locked or open.

### 6.4 Signal predicate format

Keep the MVP predicate narrow and deterministic.

Do not build an arbitrary-code execution language inside the contract.

For the proof coin, use a GitHub-release predicate with a strict schema.

Example configuration:

```json
{
  "kind": "github_release",
  "source_id": "open_meteo_mumbai",
  "latitude_e6": 19076000,
  "longitude_e6": 72877700,
  "weekday_utc": 4,
  "minimum_precipitation_mm_x100": 1,
  "valid_from": "2026-11-01T00:00:00Z",
  "valid_to": "2026-12-31T23:59:59Z"
}
```

The exact source URL belongs in an immutable source registry keyed by `source_id`.

The data parser must reject an unexpected schema.

The TEE output must contain the normalized value, date, source URL, response hash, and program version hash.

### 6.5 Human-pass format

The pass must bind a user to one specific action.

Suggested fields:

```json
{
  "version": 1,
  "kind": "genesis" ,
  "coin_id": "ship.auspice-yagnesh.testnet",
  "door_id": null,
  "account_id": "alice.testnet",
  "nonce": "hex-encoded-random-nonce",
  "issued_at_ns": "...",
  "expires_at_ns": "...",
  "max_allocation": "10000000000000000000000",
  "verifier_wasm_hash": "sha256:...",
  "signature": "base64..."
}
```

For a Signal Door claim, `kind` becomes `signal_door` and `door_id` is set.

The contract stores a hash of each consumed pass nonce.

The contract rejects an expired pass.

The contract rejects a pass issued for another wallet.

The contract rejects a pass issued for another coin or Door.

The contract rejects replayed nonces.

### 6.6 Reality and limitations

CAPTCHA reduces ordinary automation; it does not eliminate sophisticated Sybil actors.

A person can potentially acquire several accounts and solve several challenges.

Device fingerprinting should not be claimed unless actually implemented and privacy-reviewed.

The TEE proves execution of pinned code; it does not prove an upstream source is universally correct.

The source response, source URL, response hash, fetch time, and normalized output must be displayed.

---

## 7. Architecture

### 7.1 System diagram

```text
Browser / Next.js application
        |
        | wallet transactions and view calls
        v
Auspice Factory on NEAR testnet
        |
        | creates pinned token instances / records config
        v
Auspice Coin Contract (per coin)
        |
        |-- NEP-141 balances and transfers
        |-- Genesis allocation state
        |-- Signal Door state
        |-- pass consumption state
        |-- immutable Birth Certificate
        |
        +---- request execution ----> OutLayer testnet TEE
                                      |
                                      | HTTPS fetch through pinned WASM
                                      v
                                 Signed event result / attestation
                                      |
                                      +---- submitted to Coin Contract

Optional after Live:
Coin NEP-141 token <---- liquidity pool ----> wNEAR on RHEA
```

### 7.2 Contract count

MVP has two custom contracts.

Contract one is `auspice_factory`.

Contract two is one `auspice_coin` instance per launched coin.

OutLayer is an external testnet integration, not a custom contract owned by this repository.

Do not build a custom AMM contract.

Do not build a custom wallet.

### 7.3 Why per-coin contracts

Each coin gets its own account, state, NEP-141 balance map, metadata, and certificate.

Each contract has a readable independent code hash and immutable configuration.

Judges can inspect one coin directly.

The design parallels strong submitted projects without copying their mechanisms.

### 7.4 Global contract option

If factory deployment time becomes a serious blocker, one shared contract can store a `coin_id` namespace.

Do not switch architecture late unless tests prove factory deployment is the blocker.

Per-coin subaccounts are preferred for the final demo because the token is independently inspectable.

---

## 8. Repository structure

```text
auspice/
├── README.md
├── LICENSE
├── plan.md
├── .env.example
├── package.json
├── pnpm-workspace.yaml
├── Cargo.toml
├── rust-toolchain.toml
├── contracts/
│   ├── factory/
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs
│   │   └── tests/
│   ├── coin/
│   │   ├── Cargo.toml
│   │   ├── src/lib.rs
│   │   ├── src/token.rs
│   │   ├── src/genesis.rs
│   │   ├── src/signal_doors.rs
│   │   ├── src/attestation.rs
│   │   ├── src/views.rs
│   │   ├── src/events.rs
│   │   └── tests/
│   └── common/
│       ├── Cargo.toml
│       └── src/lib.rs
├── tee/
│   ├── auspice-verifier/
│   │   ├── Cargo.toml
│   │   ├── src/main.rs
│   │   ├── README.md
│   │   └── fixtures/
│   └── human-pass/
│       ├── README.md
│       └── src/
├── web/
│   ├── package.json
│   ├── next.config.ts
│   ├── app/
│   ├── components/
│   ├── lib/
│   ├── public/
│   └── tests/
├── scripts/
│   ├── build-contracts.sh
│   ├── deploy-testnet.sh
│   ├── create-demo-coin.sh
│   ├── seed-demo-users.sh
│   ├── run-signal-fixture.sh
│   ├── verify-testnet.sh
│   └── generate-demo-report.ts
├── integration-tests/
│   ├── README.md
│   ├── factory.test.ts
│   ├── genesis.test.ts
│   ├── signal-door.test.ts
│   ├── pass-replay.test.ts
│   ├── burn.test.ts
│   └── nep141.test.ts
├── docs/
│   ├── ARCHITECTURE.md
│   ├── THREAT_MODEL.md
│   ├── TESTNET.md
│   ├── DEMO.md
│   ├── SUBMISSION.md
│   ├── TEE_ATTESTATION.md
│   └── DESIGN_DECISIONS.md
└── assets/
    ├── screenshots/
    └── demo/
```

### 8.1 Files that must not enter Git

Never commit `.env.local`.

Never commit wallet private keys.

Never commit a production API key.

Never commit CAPTCHA secrets.

Never commit testnet account seed phrases.

Never commit user emails or final-submission payloads.

---

## 9. Smart-contract implementation

### 9.1 Language and framework

Use Rust.

Use the current compatible `near-sdk` and `near-contract-standards` versions selected during implementation.

Pin versions in `Cargo.lock`.

Use the official NEAR fungible-token standard helpers rather than reimplementing balance math.

### 9.2 Contract data model

The coin contract stores metadata.

The coin contract stores immutable launch configuration.

The coin contract stores lifecycle state.

The coin contract stores total minted supply.

The coin contract stores each account allocation request.

The coin contract stores each account’s Genesis minted amount.

The coin contract stores each Door’s current state.

The coin contract stores each consumed human-pass nonce hash.

The coin contract stores each accepted signal proof hash.

The coin contract stores refundable NEAR credit by account.

The coin contract stores an append-only event sequence number.

### 9.3 Suggested Rust structures

```rust
pub struct Coin {
    pub ft: FungibleToken,
    pub metadata: FungibleTokenMetadata,
    pub config: LaunchConfig,
    pub lifecycle: Lifecycle,
    pub genesis_requests: LookupMap<AccountId, Balance>,
    pub genesis_total_requested: Balance,
    pub genesis_minted: LookupMap<AccountId, Balance>,
    pub refundable_near: LookupMap<AccountId, Balance>,
    pub doors: Vector<SignalDoor>,
    pub consumed_passes: UnorderedSet<CryptoHash>,
    pub accepted_signal_proofs: UnorderedSet<CryptoHash>,
    pub minted_supply: Balance,
    pub permanently_unmintable: Balance,
}
```

### 9.4 LaunchConfig fields

```rust
pub struct LaunchConfig {
    pub creator_id: AccountId,
    pub maximum_mintable_supply: Balance,
    pub genesis_reserve: Balance,
    pub genesis_user_cap: Balance,
    pub genesis_price_yocto_near_per_token: Balance,
    pub genesis_open_at_ns: u64,
    pub genesis_close_at_ns: u64,
    pub doors: Vec<DoorConfig>,
    pub human_pass_public_key: PublicKey,
    pub outlayer_account_id: AccountId,
    pub max_signal_age_ns: u64,
    pub no_creator_genesis_allocation: bool,
    pub version: u16,
}
```

### 9.5 DoorConfig fields

```rust
pub struct DoorConfig {
    pub door_id: u16,
    pub template: SignalTemplate,
    pub source_id: String,
    pub source_url: String,
    pub predicate: SignalPredicate,
    pub verifier_wasm_url: String,
    pub verifier_wasm_hash: CryptoHash,
    pub verifier_build_target: String,
    pub reserve: Balance,
    pub user_cap: Balance,
    pub claim_window_ns: u64,
    pub eligible_only_if_not_genesis_participant: bool,
}
```

`SignalTemplate` is a closed enum: `GithubRelease`, `UsgsEarthquake`, `SportsScore`, or `PublicAccountPost`. The v1 verifier and frontend enable `GithubRelease`; the remaining variants fail closed until each receives an audited adapter and separately published verifier artifact. `SignalPredicate` is template-specific structured data, never arbitrary JavaScript, arbitrary HTTP URLs, or a post-launch user-supplied expression. A Door may be evaluated only against its own pinned `WasmUrl { url, hash, build_target }`; using an OutLayer project is optional discovery metadata, never the mutable active-version authority.

### 9.6 DoorState fields

```rust
pub enum DoorStatus {
    Locked,
    Open,
    ClosedClaimed,
    ClosedBurned,
}

pub struct SignalDoor {
    pub config: DoorConfig,
    pub status: DoorStatus,
    pub opened_at_ns: Option<u64>,
    pub closes_at_ns: Option<u64>,
    pub accepted_proof_hash: Option<CryptoHash>,
    pub minted: Balance,
    pub unminted_burned: Balance,
}
```

### 9.7 Public methods

Factory public methods:

- `create_coin(config, metadata)`;
- `get_coin(coin_id)`;
- `list_coins(from_index, limit)`;
- `get_code_hash()`.

Factory creation behavior:

- Any account may call `create_coin`; no allowlist or creator role is required.
- The `creator_id` stored in `config` must equal the transaction signer.
- The factory derives the coin account as `<lowercase-symbol>.<factory-account>` and rejects an existing or pending account.
- The attached deposit funds that new account's state. If the batched account creation or initialization fails, the factory refunds the creator.
- The factory creates the coin account without an access key, applies the configured global contract hash, and calls the coin's immutable initializer.
- The factory owner may update the global code hash only for future coin creation. A created keyless coin is unaffected.

Coin public change methods:

- `storage_deposit(account_id, registration_only)`;
- `request_genesis_allocation(pass, requested_amount)`;
- `settle_genesis()`;
- `finalize_genesis()`;
- `withdraw_refund()`;
- `request_signal_check(door_id)`;
- `get_signal_request(request_id)`;
- `list_signal_requests(from_index, limit)`;
- `claim_signal_door(door_id, human_pass)`;
- `close_signal_door(door_id)`;
- `ft_transfer(receiver_id, amount, memo)`;
- `ft_transfer_call(receiver_id, amount, memo, msg)`;
- `ft_resolve_transfer(...)`.

Coin public view methods:

- `ft_metadata()`;
- `ft_total_supply()`;
- `ft_balance_of(account_id)`;
- `storage_balance_of(account_id)`;
- `get_launch_config()`;
- `get_lifecycle()`;
- `get_genesis_status()`;
- `get_door(door_id)`;
- `list_doors()`;
- `get_birth_certificate()`;
- `get_refundable_near(account_id)`;
- `is_pass_consumed(pass_hash)`;
- `get_supply_status()`;
- `get_verification_bundle()`.

Genesis settlement behavior:

- After the Genesis close timestamp, any account may call settlement for a specific requested participant account; settlement does not require that participant's private key.
- A participant allocation can settle only once.
- `finalize_genesis()` is permissionless after the close timestamp. It enables standard transfers and permanently removes the unused portion of the Genesis reserve from mintability.
- A valid Genesis request remains settleable after finalization, so a participant cannot lose a valid allocation because a third party enabled transfers first.

### 9.8 Internal methods

- `assert_live_for_transfers()`;
- `assert_storage_registered(account_id)`;
- `verify_human_pass(pass, expected_kind, expected_door)`;
- `consume_nonce(pass_hash)`;
- `mint_to(account_id, amount, memo)`;
- `mark_unmintable(amount, reason)`;
- `emit_auspice_event(event)`;
- `compute_genesis_allocation(account_id)`;
- `assert_no_privileged_mint_caller()`.
- `on_signal_result(request_id)`; private promise callback only. It decodes the configured OutLayer execution result and rejects every direct caller or result that does not bind to the immutable request, coin, Door, source, predicate, verifier hash, freshness policy, and one-time Door state.

### 9.9 Invariants

Minted supply plus permanently unmintable supply must never exceed maximum mintable supply.

No Genesis account may mint more than its cap.

No Door claimant may mint more than its Door cap.

No pass nonce may be consumed twice.

No Door may open twice.

No Door may mint after its close time.

No normal transfer may occur before lifecycle `Live`.

No method may modify `LaunchConfig` after initialization.

No method may replace public keys, source templates, source IDs/URLs, predicates, verifier URLs/hashes/build targets, amounts, or dates.

No method may mint outside `settle_genesis` or `claim_signal_door`.

No method may send attached allocation NEAR to a creator account automatically.

Unused Genesis reserve must be added to permanently unmintable supply when Genesis is finalized.

### 9.10 Important implementation decision

The token’s normal NEP-141 methods are standard rails.

The allocation methods are custom methods.

Do not overload `ft_transfer` to run the oracle, CAPTCHA, or claim logic.

Do not make users send the token to a backend account.

---

## 10. TEE and OutLayer integration

### 10.1 Why use OutLayer

OutLayer is appropriate because it can execute pinned WASM in an Intel TDX environment and produce an attestation binding code, input, and output.

Official docs: https://app.outlayer.ai/docs/getting-started

TEE attestation docs: https://app.outlayer.ai/docs/tee-attestation

NEAR integration docs: https://app.outlayer.ai/docs/near-integration

### 10.1.1 What NEAR provides, and what it does not

NEAR smart contracts provide deterministic, publicly verifiable settlement. They do **not** provide a general-purpose TEE runtime as a base-protocol primitive. A contract cannot safely make an arbitrary HTTPS request, solve a CAPTCHA, or execute private API-key logic by itself.

OutLayer is the ecosystem service used for that off-chain confidential/verifiable-compute role. Its NEAR integration lets a contract request execution of a hash-pinned WASM program and receive a callback/result. Each run has an Intel TDX attestation that binds the worker environment, exact WASM hash, input hash, and output hash.

For Auspice, this division is non-negotiable:

- **NEAR contract:** immutable parameters, Door state, accepted program/key identifiers, pass expiry and replay checks, claim caps, token mint/burn accounting, and final settlement.
- **OutLayer TEE:** executes the published signal-verifier or human-pass verifier program, can keep provider secrets out of the browser, and returns inspectable evidence of what that program processed.
- **Frontend/backend relay:** only requests an execution or submits its result. It has no authority to open a Door, mint tokens, alter rules, or accept an expired/replayed pass.

The attestation proves that a specified program ran in the approved Intel TDX environment with specified inputs and produced the specified output. It does **not** prove that an upstream API source is objectively true, that the verifier program has no bug, or that a CAPTCHA means one unique real human. The verifier and contract must therefore enforce source provenance, freshness, wallet binding, nonce use, expiry, caps, and transparent limitations.

### 10.2 Testnet endpoints

Use the OutLayer testnet path and testnet worker configuration during development.

Do not assume production endpoint names or credentials.

Verify current addresses and API requirements in OutLayer documentation before deployment.

The OutLayer docs identify `worker.outlayer.testnet` as the testnet worker registration contract.

### 10.3 Generic event-verifier WASM responsibilities

The WASM binary must accept deterministic input.

The input includes the request ID/nonce, coin and Door IDs, the immutable template, source identity, source URL, and template-specific predicate parameters.

The program fetches only the configured allowlisted HTTPS endpoint for its source template.

The program parses only the expected JSON fields for that template.

The program normalizes timestamps to UTC and rejects a response outside its bounded freshness window.

The program determines whether the template-specific predicate passes.

The program produces canonical JSON output.

The output includes template name, source ID/URL, source-response hash, fetched time, normalized event facts, predicate result, request ID, coin ID, Door ID, and nonce.

The output must not include secrets.

### 10.4 Canonical output example

```json
{
  "version": 1,
  "template": "GithubRelease",
  "source_id": "github_release:owner/repository",
  "source_url": "https://api.github.com/repos/owner/repository/releases/latest",
  "fetched_at_utc": "2026-11-13T12:00:00Z",
  "event_facts": { "tag_name": "v1.0.0", "published_at_utc": "2026-11-12T12:00:00Z" },
  "predicate_passed": true,
  "request_nonce": "...",
  "response_sha256": "...",
  "wasm_sha256": "..."
}
```

### 10.5 Human-pass service

For the hackathon, use a clearly labeled testnet-only human-pass adapter.

The adapter may use a CAPTCHA provider or a mock test challenge depending on available credentials.

The adapter must never be described as a complete Sybil-resistance system.

The adapter must issue a pass only after the browser sends a valid wallet challenge signature.

The adapter must bind the pass to that account ID, coin ID, Door ID, nonce, expiration, and cap.

The pass-signing key must be attested or transparently labeled as a testnet development key.

The implemented testnet adapter uses a one-use NEP-413 wallet challenge bound to the exact coin, account, optional Door, recipient, and expiry. It verifies the returned key is currently a full-access key for the signing account before issuing a deterministic, wallet-bound pass. It is enabled only with `TESTNET_DEMO_MODE=true`, and requires server-only `HUMAN_PASS_CHALLENGE_SECRET`, `HUMAN_PASS_SIGNING_KEY_PEM`, and `HUMAN_PASS_MAX_ALLOCATION` variables.

### 10.6 Preferred production-like approach

Build a small pinned WASM verifier for the human-pass policy.

Run that verifier in OutLayer’s TEE.

Keep any CAPTCHA provider secret inside OutLayer protected secrets.

Return an attested, wallet-bound decision.

Have the coin contract verify the designated worker’s signature and expected program hash.

### 10.6.1 Implemented target: TEE-verified Turnstile allocation approval (V2 launches)

New launches must use a separate, hash-pinned `auspice-human-verifier` WASI
program. The browser renders a Cloudflare Turnstile widget and submits its
short-lived token with a wallet-bound human-check request. The coin contract
forwards that request to OutLayer. The verifier reads `TURNSTILE_SECRET` only
from OutLayer encrypted secrets, calls Cloudflare Siteverify, and returns a
canonical approval bound to the coin, caller account, action, request nonce,
host, and expiry. A private contract callback validates those bindings and
records one consumable approval. The later Genesis/Signal-Door allocation
transaction consumes that on-chain approval and settles the purchase/mint in
the NEP-141 contract.

The frontend and any relay are untrusted transport only: neither can approve a
human, forge a callback, mint, or reuse an approval. The proof token is
short-lived and is supplied to the TEE execution; it must never be logged or
persisted by the frontend.

This is CAPTCHA-based bot resistance, not proof of unique personhood. A future
World-ID/nullifier adapter can add one-person uniqueness without changing the
core separation of TEE approval and on-chain settlement.

The deployed `SHIP` proof coin remains a historical V1/testnet demonstration
with its clearly labelled signed test pass. Its contract has no access keys and
cannot be changed. The Turnstile path applies to new V2 launches only.

### 10.7 Testnet shortcut policy

If a real CAPTCHA integration risks the core demo, use a mock provider only behind `TESTNET_DEMO_MODE=true`.

The UI, README, and demo must visibly label the mock.

The contract must still verify signature, binding, expiry, nonce, and replay protection.

Never fake a TEE proof.

Never claim an unsigned response is attested.

### 10.8 Oracle security checklist

- Pin the expected WASM hash in the coin configuration.
- Pin the expected signal public key in the coin configuration.
- Bind output to `coin_id` and `door_id`.
- Bind output to a unique request nonce.
- Bind the signed proof message to the source response hash as well as the coin, Door, source ID, and expiry.
- Store a maximum accepted signal age in immutable launch configuration and reject a result whose `fetched_at_ns` is older than that age.
- Include an expiration time.
- Reject stale proofs.
- Reject mismatched source IDs.
- Reject mismatched predicate versions.
- Reject duplicate proof hashes.
- Record the accepted output hash on-chain.
- Show the full verification bundle in the frontend.

### 10.9 Contract-enforced OutLayer flow (no trusted backend)

The TEE is external compute, but it must never be the authority that changes token supply. The same `auspice_coin` NEP-141 contract that owns balances and mint logic also owns the full Signal Door state machine.

1. **Immutable Door configuration.** At initialization, the contract stores each Door’s closed source template, source identity/URL, predicate parameters, exact OutLayer `WasmUrl` (`url`, SHA-256 hash, build target), caps/durations, and human-pass policy. These fields have no admin update method.
2. **Permissionless submission.** Any wallet calls `request_signal_check(door_id)`. The contract rejects an already open/closed Door or a still-live request for that Door, creates a unique request ID and nonce, records it in the Submitted Signals queue, and requests the Door’s exact hash-pinned WASM from OutLayer testnet.
3. **Attested execution.** OutLayer runs the exact pinned verifier WASM in its TEE. The program fetches/parses the fixed template source and returns canonical output containing `request_id`, nonce, coin/Door IDs, template, source/result hashes, fetch time, predicate result, and verifier WASM hash.
4. **Authenticated callback.** The asynchronous `on_signal_result(request_id, result)` callback is private. It accepts only the configured OutLayer callback route and only an outstanding request created by this coin contract. A browser, creator, relayer, or arbitrary wallet cannot call it successfully.
5. **On-chain validation.** Before opening a Door, the contract matches the response against its own immutable state: request ID and nonce, coin/Door IDs, source template/identity, exact verifier URL/hash/build target, predicate version, freshness deadline, successful predicate, and one-time Door status. It records the accepted output/proof hash for inspection.
6. **Only the contract opens the Door.** On passing every check, the contract changes the Door from `Locked` to `Open`; it does not mint a discretionary creator allocation.
7. **Only the contract mints.** A claimant calls `claim_signal_door` with a fresh wallet-bound human pass. The contract verifies the configured signature/policy, coin/Door/account binding, expiry, cap, and unused nonce. Only then does its custom mint logic credit NEP-141 balance.
8. **Permissionless close/burn.** After the immutable deadline, anyone may call `close_signal_door`. The contract marks any unused Door reserve permanently unmintable and records a burn event.

The frontend or a relay may help a user initiate a request, but it is not trusted: it cannot open a Door, change a rule, add a new event, bypass a human-pass check, mint, or stop a different user from calling the permissionless contract methods.

Do not submit a bare `attestation_hash` from the frontend and treat that hash as proof. A hash is only a commitment. The contract must bind it to an expected request, callback origin, frozen verifier policy, nonce, freshness check, and one-time Door transition.

Per-call Intel TDX quote verification is not the intended on-chain path because it is prohibitively expensive. The contract relies on the configured OutLayer integration/callback for settlement and saves the resulting hashes/identifiers; the UI exposes the full attestation so judges and users can independently verify the TDX evidence.

### 10.10 On-chain event history

The coin keeps an append-only on-chain event list for Genesis requests and settlement, Genesis finalization, Door opening, Door claims, and Door closure/burn. It emits a matching `EVENT_JSON` log for each recorded event. The `get_verification_bundle` view returns the configured TEE identifiers, Door states, and recent event history for independent inspection.

---

## 11. Frontend plan

### 11.1 Stack

Use Next.js with TypeScript.

Use React.

Use a NEAR wallet selector compatible with testnet.

Use a lightweight component system.

Use a serverless API route only for non-authoritative convenience calls.

No backend route may settle allocations, mint tokens, or modify contract truth.

### 11.2 Design goal

The UI should make the rule understandable before explaining the technology.

Avoid dashboards that look like a generic DeFi terminal.

Use a “birth certificate” visual language.

Use a live Signal Door visual that feels like an event certificate and public queue, not a trading chart.

### 11.3 Main routes

`/` is the project landing page.

`/launches` lists testnet Auspice coins.

`/coin/[coinId]` is the public coin page.

`/coin/[coinId]/genesis` runs the initial allocation flow.

`/coin/[coinId]/doors/[doorId]` runs a Signal Door claim.

`/coin/[coinId]/certificate` shows the Birth Certificate.

`/coin/[coinId]/verify` shows raw verification data and explorer links.

`/create` is optional and only enabled after the demo coin works.

`/docs` mirrors the key project documentation.

### 11.4 Landing page sections

Hero: “Supply follows reality, not insiders.”

Problem: “Fair launches are often speed races disguised as equal access.”

How it works: three-step visual for Birth, Signal, Door.

Live demo: Ship Signal card with current lifecycle state.

Proof: code hash, contract account, TEE WASM hash, and event count.

Limitations: clear panel on CAPTCHA and oracle limits.

CTA: “Inspect the Birth Certificate.”

### 11.5 Coin page sections

Header contains name, ticker, testnet badge, and account ID.

Lifecycle banner displays current state.

Supply timeline displays Genesis, Door reserve, minted, and permanently unmintable amount.

Rules panel lists immutable parameters.

Signal panel lists source and predicate.

Door panel lists state, countdown, reserve, cap, and eligibility.

Claim panel handles wallet connection and pass flow.

Certificate panel links to raw receipts.

Transfer panel explains normal NEP-141 behavior after live state.

Trading panel links to a testnet pool only if one exists.

### 11.6 Birth Certificate contents

Coin account ID.

Factory account ID.

Contract code hash.

Metadata hash.

Maximum mintable supply.

Genesis reserve.

Genesis cap.

Genesis start and end timestamps.

Creator allocation policy.

TEE WASM hash.

TEE worker identity or configured key.

Signal source and predicate.

Genesis accepted-pass count.

Genesis allocation count.

Genesis minted supply.

Door state and proof hashes.

Mint and burn event receipts.

All explorer links.

### 11.7 Frontend safety rules

Do not show “guaranteed fair” without the limitations beside it.

Do not hide the testnet badge.

Do not default a user to a transaction they have not reviewed.

Do not ask a wallet to sign a blank message.

Do not store private keys in local storage.

Do not expose CAPTCHA secret keys to the client.

---

## 12. Testnet environment

### 12.0 Current deployment record

The permissionless factory is deployed on NEAR testnet at `auspice-yagnesh.testnet`; its owner is `auspice-yourname.testnet`. The live V1 proof coin is `ship.auspice-yagnesh.testnet` and retains its original immutable testnet pass policy. The V2 human verifier is published at `busymuseum5448.testnet/auspice-human-verifier` with hash-pinned WASM `350b17d09fa390a28fbfdae49579206b8777ed48f07daef9db14a959782564ec` and its FastFS URL recorded in the deployment environment. Its `TURNSTILE_SECRET` must be stored only in OutLayer encrypted secrets before any V2 launch can use it.

### 12.1 Network choice

Use NEAR testnet for every custom contract and token demo.

The hackathon guide explicitly allows testnet-only projects.

Do not deploy to mainnet for this project unless the human separately authorizes it.

### 12.2 Accounts

Create a dedicated testnet factory account.

Create a dedicated testnet deployer account.

Create testnet accounts for Alice, Bob, Carol, Bot, and Watcher.

Document all public testnet account IDs in `docs/TESTNET.md`.

Never document seed phrases.

### 12.3 Suggested account names

`auspice.testnet` for the factory.

`ship.auspice-yagnesh.testnet` for the demo coin.

`alice.auspice.testnet` for a participant fixture.

`bob.auspice.testnet` for a participant fixture.

`bot.auspice.testnet` for an invalid-pass fixture.

`watcher.auspice.testnet` for a permissionless Signal Door opener.

### 12.4 Funding

Fund testnet accounts through an approved testnet faucet or current NEAR CLI flow.

Use only enough testnet NEAR for storage, gas, and demo transactions.

Record approximate funding needs in `docs/TESTNET.md`.

### 12.5 Deployment checklist

- Build reproducible WASM artifacts.
- Record SHA-256 of every contract artifact.
- Deploy factory to testnet.
- Verify factory views.
- Create demo coin from factory.
- Verify generated coin account.
- Register fixture accounts for storage.
- Run Genesis fixture.
- Run invalid-pass fixture.
- Run valid Signal Door fixture.
- Run expiry/burn fixture.
- Capture transaction IDs and screenshots.

---

## 13. Test plan

### 13.1 Unit-test categories

Metadata tests verify NEP-148 fields.

Storage tests verify registration is required where expected.

Transfer tests verify standard behavior after live state.

Pre-live tests verify normal transfer rejects before `Live`.

Genesis tests verify timing and cap rules.

Pass tests verify binding, expiry, signature, and replay rejection.

Signal tests verify predicate, source, nonce, expiry, and duplicate-proof rejection.

Door tests verify one open, capped claims, deadline behavior, and burn behavior.

Supply tests verify conservation.

Access tests verify no privileged mint or config mutation path exists.

### 13.2 Mandatory happy-path test

Create a coin with a small Genesis reserve and one Door.

Register Alice and Bob for token storage.

Issue valid Genesis passes for Alice and Bob.

Submit their allocation requests.

Advance test time beyond Genesis close.

Call `settle_genesis` from a third-party account.

Verify correct NEP-141 balances.

Verify normal `ft_transfer` now succeeds.

Submit a valid attested signal proof.

Verify the Door opens.

Issue a valid Door pass for Carol.

Claim Door allocation.

Advance time past Door close.

Call `close_signal_door` from a third-party account.

Verify remaining reserve is permanently unmintable.

### 13.3 Mandatory negative tests

Reject Genesis allocation before start time.

Reject Genesis allocation after close time.

Reject unregistered recipient mint.

Reject pass for another account.

Reject pass for another coin.

Reject pass for another Door.

Reject expired pass.

Reject reused pass nonce.

Reject invalid signature.

Reject proof with unexpected WASM hash.

Reject proof with unexpected source ID.

Reject proof with failed predicate.

Reject signal Door open twice.

Reject Signal Door claim after close.

Reject Signal Door claim by Genesis participant when rule forbids it.

Reject configuration mutation after creation.

Reject a privileged post-launch mint attempt.

Reject normal token transfer before live state.

### 13.4 Supply-conservation equation

At every point:

```text
minted_supply
+ permanently_unmintable_supply
+ remaining_genesis_reserve
+ sum(remaining_door_reserves)
= maximum_mintable_supply
```

Write a property test that generates random valid and invalid action sequences.

Assert the equation after every successful state transition.

### 13.5 End-to-end tests

Use testnet or a local sandbox for the core full flow.

E2E tests must use real wallet-like signed transactions where practical.

E2E tests must not depend on an uncommitted secret.

E2E tests should produce a compact report suitable for README evidence.

---

## 14. Security and threat model

### 14.1 Assets to protect

Mint authority.

User allocation funds.

Refund balances.

Signal Door reserves.

Human-pass integrity.

Signal-proof integrity.

Contract configuration immutability.

### 14.2 Threat: creator mints extra supply

Mitigation: no external mint method exists.

Mitigation: internal mint is callable only from deterministic settlement and claim paths.

Mitigation: code and configuration are public.

Mitigation: use zero access keys or a deliberately finalized immutable deployment design after setup.

### 14.3 Threat: creator changes the signal rule

Mitigation: signal source ID, predicate, public key, WASM hash, cap, reserve, and schedule are immutable initialization fields.

Mitigation: expose them in `get_launch_config`.

### 14.4 Threat: pass replay

Mitigation: pass includes a unique nonce.

Mitigation: contract stores consumed nonce hashes.

Mitigation: pass binds coin, action kind, Door ID, wallet, and expiry.

### 14.5 Threat: stale source data

Mitigation: attested output includes fetched time and source-reported time when available.

Mitigation: contract enforces a maximum proof age.

Mitigation: the TEE program rejects stale source payloads according to documented rules.

### 14.6 Threat: fake signal proof

Mitigation: verify signature against immutable key.

Mitigation: verify expected program/WASM hash in proof.

Mitigation: verify proof fields against immutable Door configuration.

Mitigation: reject duplicate proof hashes and Door opening after the first accepted proof.

### 14.7 Threat: TEE code is malicious

Mitigation: publish source for the WASM code.

Mitigation: publish reproducible build instructions.

Mitigation: show WASM hash in configuration and attestation view.

Mitigation: clearly state that a TEE proves execution, not business-logic correctness.

### 14.8 Threat: Sybil users solve CAPTCHA repeatedly

Mitigation: cap allocation per pass and wallet.

Mitigation: bind pass to signed wallet challenge.

Mitigation: use short pass expiry and one-time nonces.

Mitigation: disclose this is mitigation, not proof of unique personhood.

### 14.9 Threat: storage registration failures

Mitigation: frontend checks `storage_balance_of` before actions.

Mitigation: frontend offers `storage_deposit` first.

Mitigation: contract error messages name the required action.

### 14.10 Threat: reentrancy and async transfer behavior

Mitigation: use standard NEP-141 resolver logic.

Mitigation: keep allocation and refund state separate from `ft_transfer_call` callbacks.

Mitigation: test failed receiver callbacks and refund paths.

---

## 15. Trading and liquidity

### 15.1 What “tradable” means

An NEP-141 token can be transferred once live.

It becomes swap-tradable when a liquidity pool exists.

Creating a token does not automatically create liquidity.

### 15.2 Recommended plan

Do not build an AMM or bonding curve.

Use RHEA/Ref-compatible liquidity only after the Auspice core is fully complete.

RHEA documents permissionless pooling for NEAR tokens.

Resource: https://guide.rhea.finance/getting-started/using-the-platform

### 15.3 Testnet trading demo

If a testnet flow is stable and documented, seed a tiny testnet SHIP/wNEAR pool.

If testnet pool creation becomes fragile, do not make it a required judge step.

Instead, demonstrate that the live token exposes standard NEP-141 transfers and include an explicit “DEX integration planned” note.

Never fake a market chart.

Never claim a token is liquid if the demo only has internal transfers.

### 15.4 meme.cooking relationship

meme.cooking already supports equal-price fair auctions and locked liquidity.

Auspice must not present itself as a clone of meme.cooking.

Auspice’s distinction is post-launch, human-only, real-world-signal-gated supply release.

Potential future integration: a meme.cooking launch could reserve a defined supply slice for Auspice Signal Doors.

Do not claim an integration exists until it exists and is demonstrated.

---

## 16. Milestone plan

### Milestone 0: lock the story

Write the one-sentence problem.

Write the one-sentence mechanism.

Freeze the demo coin and one signal rule.

Write exclusions to stop scope creep.

Success condition: a teammate can explain the project in 20 seconds.

### Milestone 1: contract skeleton

Implement factory initialization.

Implement coin initialization.

Implement NEP-141, NEP-145, and NEP-148 skeleton.

Implement views.

Implement `Live` state transfer tests.

Success condition: a created testnet coin has valid metadata and standard balances.

### Milestone 2: Genesis

Implement pass structure and signature verification.

Implement storage registration flow.

Implement allocation requests.

Implement permissionless settlement.

Implement refunds.

Implement Birth Certificate view.

Success condition: Alice and Bob receive exact testnet allocation after settlement.

### Milestone 3: invalid path proof

Implement invalid-pass rejection.

Implement pass replay rejection.

Implement wrong-wallet rejection.

Implement pre-live transfer rejection.

Success condition: demo can show one human success and one bot failure.

### Milestone 4: Signal Door

Implement Door configuration.

Implement attested proof data structure.

Implement proof signature verification.

Implement Door opening.

Implement Door claim.

Implement Door closure and unmintable supply accounting.

Success condition: a valid fixture proof opens one Door and unclaimed reserve burns.

### Milestone 5: OutLayer proof

Build the generic event-verifier WASM.

Publish the source and reproducible build path.

Run it in the configured OutLayer testnet environment.

Record attestation data.

Connect its output to the actual contract path.

Success condition: a public testnet transaction accepts a genuine, inspectable proof.

### Milestone 6: polished frontend

Build Landing, Coin, Certificate, Verify, and Door views.

Add explorer deep links.

Add clear testnet and limitations labels.

Success condition: a judge needs no source code to understand the flow.

### Milestone 7: verification and evidence

Run all tests.

Run full testnet rehearsal from a clean wallet.

Capture transaction IDs.

Write README exact reproduction steps.

Record demo.

Success condition: a fresh reviewer can follow “do X, observe Y.”

---

## 17. Demo script

Target total video time: 90 to 150 seconds.

### Scene 1: problem, 0 to 10 seconds

Show text: “Most fair launches are fair only for bots.”

Say: “Auspice makes a coin’s future supply follow a public real-world rule, and lets only verified humans claim it.”

### Scene 2: immutable configuration, 10 to 25 seconds

Open the Ship Signal coin page.

Show 70% Genesis and 30% Signal Door reserve.

Show the fixed rule: “A matching GitHub release opens this Door.”

Show code hash, no-admin statement, and testnet badge.

### Scene 3: failed bot action, 25 to 40 seconds

Attempt a Genesis or Door allocation without a valid pass.

Show wallet transaction failure or contract error.

Show the error code in the UI and explorer.

### Scene 4: human allocation, 40 to 60 seconds

Connect Alice testnet account.

Complete the visible testnet pass flow.

Submit allocation request or Door claim.

Show minted SHIP balance.

### Scene 5: actual signal proof, 60 to 90 seconds

Open the OutLayer attestation page.

Show the WASM hash, input/output, and TEE verification.

Submit the matching proof to `open_signal_door`.

Show the Door timer become live.

### Scene 6: burn and proof, 90 to 120 seconds

Close a demo Door after partial claims.

Show unclaimed supply permanently removed from the mintable reserve.

Open Birth Certificate.

Show all receipts and explorer links.

### Scene 7: close, 120 to 135 seconds

Say: “Auspice does not predict a chart. It gives every memecoin a transparent birth and rules for every later mint.”

Show repository URL and testnet demo URL.

---

## 18. README specification

The README must be both human-readable and machine-verifiable.

### 18.1 Required opening section

Start with a single paragraph defining the problem and product.

Include the testnet-only warning.

Include the track name.

Include the live demo URL, repository URL, factory account, demo coin account, and demo video URL once available.

### 18.2 Required sections

- What Auspice solves.
- How Genesis works.
- How Signal Doors work.
- Why NEP-141 matters.
- What runs on-chain.
- What the TEE attests.
- Limitations and threat model.
- Architecture diagram.
- Local setup.
- Test commands.
- Testnet deployment commands.
- Exact judge walkthrough.
- Contract addresses and hashes.
- License.
- Hackathon disclosure.

### 18.3 Machine-checkable walkthrough

Use steps in this form:

1. Open `[testnet URL]`.
2. Connect a NEAR testnet wallet.
3. Open `Ship Signal`.
4. Read the immutable rule in the Rules panel.
5. Call `get_birth_certificate` or use the UI verification link.
6. Submit the supplied valid testnet fixture pass.
7. Observe SHIP balance change from `0` to `[expected amount]`.
8. Submit the same pass again.
9. Observe `PASS_ALREADY_USED` rejection.
10. Open the attested Signal Door proof.
11. Trigger or inspect `open_signal_door`.
12. Observe the Door claim and eventual burn receipt.

### 18.4 README claims to avoid

Do not write “Sybil-proof.”

Do not write “impossible to bot.”

Do not write “source truth guaranteed.”

Do not write “mainnet safe.”

Do not write “investment opportunity.”

Do not write “fairer than every launchpad” without evidence.

---

## 19. Submission narrative

### 19.1 Short submission description

Auspice is a testnet NEP-141 fair-launch platform where supply follows immutable public rules instead of founder discretion. Coins begin with a wallet-bound, human-verified Genesis allocation, then keep part of their supply unminted until an attested real-world Signal Door opens. The NEAR contract verifies the pass/proof, caps allocations, mints eligible balances, and permanently removes unclaimed supply. Our proof coin, SHIP, opens a new human allocation only when an attested GitHub-release rule is satisfied.

### 19.2 How NEAR fits

NEAR hosts the token, allocation state machine, immutable configuration, mint/burn accounting, public event receipts, and the verification path.

NEP-141 provides interoperable fungible-token behavior after launch.

OutLayer attestation provides inspectable evidence that a pinned WASM program fetched and evaluated the external signal.

### 19.3 Why it is original

The product is not simply “a coin that mints when it rains.”

It combines event-driven issuance with a continued human-only distribution rule.

The signal opens a fair allocation Door rather than directly rewarding whoever runs fastest.

The same rule applies to every later supply release.

### 19.4 Track selection

Primary track: `memecoins`.

Possible secondary track: `open` only if the final product meaningfully expands beyond the memecoin track.

Do not add a secondary track merely to increase coverage.

---

## 20. Presentation strategy

### 20.1 What judges should remember

“Reality can release supply, but bots cannot.”

### 20.2 What not to lead with

Do not lead with Rust.

Do not lead with contract architecture.

Do not lead with a token price chart.

Do not lead with the word CAPTCHA.

### 20.3 What to lead with

Lead with the unfair-launch problem.

Show a failed bot allocation immediately.

Show a live human claim immediately after.

Then explain that the same fairness rule continues after launch through Signal Doors.

### 20.4 Technical credibility moment

Show a direct contract configuration view.

Show the immutable TEE WASM hash.

Show a real testnet receipt.

Show the proof verifier or explorer link.

### 20.5 Design credibility moment

Make the supply timeline beautiful and legible.

Make the Door feel like a ritual, not an admin form.

Use one visual language consistently: certificate, seal, signal queue, countdown, proof.

---

## 21. Scope control

### 21.1 Must ship

One live testnet coin.

NEP-141 core, storage, metadata, resolver.

Genesis human-pass verification.

Genesis settlement and refund path.

One Signal Door.

One actual or transparently mocked TEE proof flow.

Door claim and unclaimed-supply removal.

Birth Certificate UI.

Unit tests and one full integration test.

README and video demo.

### 21.2 Nice to ship

Multi-coin factory UI.

Three signal templates.

RHEA testnet pool.

Cross-chain funding quote.

More than one Door.

On-chain proof viewer with raw decoder.

### 21.3 Explicitly cut if time is short

Custom DEX.

Bonding curve.

Creator fee system.

Staking.

AI agent.

Social feed.

Governance DAO.

NFTs.

Multi-chain settlement.

Production identity system.

---

## 22. Acceptance criteria

### 22.1 Contract acceptance criteria

The demo coin returns correct `ft_metadata`.

The demo coin returns correct `ft_total_supply`.

The demo coin requires registration to receive balances.

The demo coin rejects normal transfers before Live state.

The demo coin accepts normal NEP-141 transfer after Live state.

The demo coin rejects an invalid or replayed pass.

The demo coin settles Genesis permissionlessly.

The demo coin opens a Door only from a valid proof.

The demo coin mints only the configured Door allocation.

The demo coin permanently removes unclaimed allocation after Door close.

The demo coin exposes immutable configuration views.

### 22.2 Frontend acceptance criteria

The site loads without a wallet connected.

The site clearly shows testnet status.

The site explains the mechanic before prompting transactions.

The site shows all fixed rules.

The site links each action to an explorer receipt.

The site shows useful error messages.

The site explains human-proof and oracle limitations.

### 22.3 Demo acceptance criteria

The demo starts from the problem.

The demo shows one rejected bot-like attempt.

The demo shows one successful human allocation.

The demo shows one accepted attested signal.

The demo shows one mint or burn result.

The demo shows NEAR’s concrete role.

---

## 23. Resource list

### 23.1 Hackathon

- Live guide: https://nearhackathon.com/HACKATHON.md
- Hackathon site: https://nearhackathon.com/
- Track prompt examples: https://github.com/nearai/near-hackathon-prompt-examples
- Memecoins track example: https://github.com/nearai/near-hackathon-prompt-examples/blob/main/tracks/11-memecoins.md
- NEAR Builders: https://nearbuilders.org/build
- NEAR Discord: https://discord.gg/nearprotocol
- NEAR builders Telegram: https://t.me/nearbuilderschat

### 23.2 NEAR standards and docs

- NEP-141: https://github.com/near/NEPs/blob/master/neps/nep-0141.md
- NEP index: https://near.github.io/NEPs/
- NEAR docs: https://docs.near.org/
- NEAR RPC providers: https://docs.near.org/api/rpc/providers
- NEAR smart-contract skill: https://github.com/near/agent-skills/tree/main/skills/near-smart-contracts

### 23.3 OutLayer

- Getting started: https://app.outlayer.ai/docs/getting-started
- TEE attestation: https://app.outlayer.ai/docs/tee-attestation
- Trust and verification: https://app.outlayer.ai/docs/trust-verification
- NEAR integration: https://app.outlayer.ai/docs/near-integration
- HTTPS API: https://app.outlayer.ai/docs/https-api
- Playground: https://app.outlayer.ai/playground

### 23.4 Liquidity and ecosystem

- RHEA guide: https://guide.rhea.finance/
- RHEA pooling: https://guide.rhea.finance/near-chain-guides/how-rhea-finance-works/pooling
- RHEA CLI: https://guide.rhea.finance/developers-1/cli-trading
- meme.cooking docs: https://docs.meme.cooking/
- meme.cooking: https://meme.cooking/

### 23.5 Design study

Study public NEAR projects for test coverage, clarity, and publicly inspectable receipts.

Do not copy another project's core mechanics or presentation language.

---

## 24. Final pre-submission checklist

- [ ] Repository is public.
- [ ] Repository has an MIT or chosen license.
- [ ] README opens with the problem and exact demo path.
- [ ] README discloses testnet-only status.
- [ ] README discloses CAPTCHA/Sybil limitations.
- [ ] README discloses external-data/TEE limitations.
- [ ] All contract source is included.
- [ ] All frontend source is included.
- [ ] `.env.example` has placeholders only.
- [ ] No secrets or wallet keys are committed.
- [ ] Tests pass from a clean checkout.
- [ ] Testnet contract IDs are documented.
- [ ] Testnet transaction IDs are documented.
- [ ] Hosted demo works in an incognito browser.
- [ ] Demo video shows a full flow.
- [ ] X post is drafted for the human to publish.
- [ ] X post says it was built at NEAR HACKATHON.
- [ ] X post tags `@NEARProtocol` and `#NEARHackathon`.
- [ ] Submission description is within the required length.
- [ ] Team lead, email, team size, payout wallet, repo, and demo URL are supplied by the human.
- [ ] Human explicitly approves the exact submission payload before it is sent.

---

## 25. Final decision rubric

Proceed with Auspice only if all of the following are true.

The team can demonstrate an actual contract-enforced allocation rule.

The team can demonstrate at least one actual testnet mint and one rejection.

The team can demonstrate at least one attested or clearly labeled mock signal proof.

The team can explain every limitation in plain language.

The team can finish the core flow before adding a DEX, multi-coin factory UI, or extra signal templates.

The team can make the phrase “Supply follows reality, not insiders” true in code.

If any of these are false near deadline, cut scope rather than fake completeness.

---

## 26. Definition of done

Auspice is done for the hackathon when a judge can:

1. Open the public site.
2. Read the immutable Ship Signal rule.
3. Inspect the testnet NEP-141 metadata and supply.
4. See why a bot-like call fails.
5. Receive an allocation through a valid wallet-bound pass.
6. Inspect the TEE signal attestation.
7. Open or inspect a Signal Door.
8. Observe a cap-enforced mint or unclaimed-supply burn.
9. Verify the result through public contract views and testnet receipts.
10. Understand, in one sentence, why NEAR is necessary.

That sentence is:

> NEAR makes Auspice’s token, mint rules, receipt history, and attested Signal Door settlement public and enforceable—not a backend promise.
