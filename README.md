# Auspice

## Supply follows rules, not insiders.

**Auspice is the fair-launch engine for memes that need to keep their word.**
It lets anyone publish a fixed-supply [NEP-141][nep-141] coin whose Genesis
distribution and future supply are governed by rules that can be inspected,
tested, and never rewritten after launch.

There is no founder mint button, no upgrade authority on a launched coin, and no backend that can rewrite economics after people join.

## The problem

Most token launches ask people to trust a creator, private server, or dashboard.
Those systems can change allocation rules, favour bots, reopen supply, withdraw
liquidity, or quietly alter the deal after launch. Communities are left trying
to audit promises after tokens have already been sold.

Auspice makes the launch rule inspectable and contract-enforced:

- **Fixed supply:** coins mint only from reserves committed at launch; unused reserves become permanently unmintable.
- **Permissionless launches:** anyone may publish a coin through the factory.
- **Immutable birth certificate:** supply splits, dates, caps, source rules, verifier hashes, vesting, and protections are public contract state.
- **No mutable launch settings:** launched coin configuration has no setter.
- **Human-gated Genesis:** Turnstile is checked by a hash-pinned OutLayer TEE before the contract permits an allocation.
- **Receipts, not promises:** mints, burns, streak outcomes, event requests, commitment votes, and protection probes are recorded on-chain.

The meme is not the chart. **The mechanic is the product.**

## How Auspice solves it

Auspice turns a launch into an economic agreement written before the first
token is distributed. A creator chooses an allocation structure and a
mechanic; the factory creates a coin whose contract stores those choices
forever. The frontend makes the rules understandable and lets a wallet sign
actions, but it has no permission to change the rules or issue supply.

In plain terms: **real people enter at Genesis, then the coin earns its next
supply movement through a public rule.** A community can keep a streak alive,
complete work, vote on a proof, wait for a real-world signal, or trade against
a reserve that nobody can withdraw. Every outcome leaves a receipt.

The first distribution is intentionally separated from the later mechanic.
Genesis gives real participants a capped way to enter; the remaining reserve
then follows a rule the community can watch and verify. That rule may be a
daily ritual, completed work, a published commitment, an external event, or a
protected market. It is not a cron job or an operator decision.

For example, a Daily Streak coin has a target, a fixed daily reward, a fixed
miss penalty, a daily window, and a ritual reserve at launch. Holders check in
once. After the window, anyone can settle it: meeting the target mints the
published reward to that day’s participants; missing it permanently removes
the published reserve slice and resets the streak. The current streak, high
score, participation progress, total mints, and total removals remain public
state and event history.

This same design pattern runs through every template: supply can move only by
the rule committed to in the Birth Certificate, and a rule that is not met
cannot be replaced with a creator override.

## What launches can do

Every coin begins with fixed supply rails:

    maximum supply
    ├── Genesis reserve     → human-gated allocation during a fixed window
    ├── mechanism reserve   → the selected immutable mechanic
    └── creator reserve     → optional immutable vesting, never an admin mint

| Template | The immutable rule |
| --- | --- |
| **Normal** | Fixed-supply Genesis launch with optional vesting and circulation decay. |
| **Fortress** | Creator vesting, max wallet, max transaction, transfer cooldown, and a burn fee that can only decrease on its frozen schedule. |
| **Work-to-Mint** | A fixed reserve unlocks only for valid wallet-bound proof-of-work nonces meeting launch-time difficulty and per-wallet cap. The browser mines; the contract validates. |
| **Locked Market** | A contract-held curve: buy mints from fixed inventory against NEAR; sell burns and pays only from the same locked reserve. No creator/factory withdrawal method exists. |
| **GitHub Signal** | A pinned release-source verifier opens a capped Door when its frozen rule is met. |
| **Rainfall Signal** | A pinned Open-Meteo rainfall source and threshold can open a capped Door. |
| **Daily Streak** | Holders check in once per day. Hit target: checkers receive the fixed reward. Miss target: a fixed reserve slice is permanently removed and the streak resets. |
| **Proofbound** | A public statement, evidence deadline, quorum, threshold, and reward slices are frozen. Human-checked settled Genesis participants cast one equal vote; reward slices release or become permanently unmintable. |

Suitable launches can additionally use creator vesting, lazy circulation decay, and public protection-probe receipts.

## Human-gated Genesis flow

    wallet → Turnstile → request human check
           → pinned OutLayer TEE → private NEAR callback
           → short-lived wallet-bound approval
           → request Genesis allocation
           → permissionless settlement and finalization

1. A participant connects a NEAR wallet and completes Turnstile.
2. The wallet signs the human-check request.
3. The coin requests its configured OutLayer human-verifier WASM URL, SHA-256 hash, build target, action, hostname, and protected-secret profile.
4. The verifier calls Cloudflare Siteverify inside the TEE. The secret is injected only from OutLayer protected secrets.
5. Only the coin's **private callback** can store a short-lived approval bound to wallet, request nonce, coin, and action.
6. The wallet signs a capped Genesis allocation request. Neither browser nor Vercel can mint or approve an allocation.
7. After close, anyone settles requests and finalizes Genesis. Unallocated reserve becomes permanently unmintable.

Signal Doors use the same architecture:

    wallet → signal check → pinned event TEE → private callback → Door opens
    wallet → fresh human check → capped Door claim → unused reserve may close/burn

## Architecture

    Next.js frontend
      wallet selector · launch form · explorer · passport · arena
      Turnstile widget · local nonce miner · transaction composer
                             │ signed NEAR transactions
                             ▼
    NEAR factory + keyless per-coin NEP-141 contract
      immutable reserves, caps, dates, mechanics, events, callbacks
                             │ hash-pinned asynchronous request
                             ▼
    OutLayer TEE
      human verifier: protected Turnstile secret + Siteverify
      signal verifier: frozen GitHub/Open-Meteo source + predicate
                             │ private callback result
                             ▼
    coin state: approval / Door result / mint-burn-audit history

### Frontend

- **/create** — Normal, Fortress, Work-to-Mint, Locked Market, GitHub Signal, Rainfall Signal, Daily Streak, and Proofbound configurations with shareable presets.
- **/coins** — factory-wide explorer with lifecycle, supply, Doors, and module summaries.
- **/coins/[coinId]** — Launch Passport, Genesis, settlement/finalization, signal requests, Door claims/closure, Streak, Work-to-Mint, decay, curve, and event history.
- **/arena** — real protection probes plus deliberately failing immutable liquidity/rule-rewrite calls.
- **/verify** — independently inspect birth certificates, hashes, supply state, and NEAR receipts.

The frontend reads state and composes wallet-signed transactions only. It is never an allocation authority. Browser RPC requests use a same-origin API relay to avoid CORS and single-public-RPC rate limits.

### Factory and contracts

The Rust factory creates **keyless per-coin accounts** from its current global coin-code hash and indexes those launches. It can affect only coins created in the future; launched coins have their own state and no configurable upgrade authority.

The shared coin contract implements:

- [NEP-141 fungible tokens][nep-141];
- [NEP-145 storage management][nep-145];
- [NEP-148 metadata][nep-148];
- fixed reserve accounting, permissionless settlement, and permanent unused reserve removal;
- private OutLayer callbacks;
- Fortress, Work-to-Mint, Streak, Proofbound, decay, protection, and locked-curve modules; and
- bounded public events/audit history.

### TEE programs

Two WASI P2 programs are built separately from the NEAR contracts:

1. **Human verifier:** checks a request-bound Turnstile token using the protected TURNSTILE_SECRET; validates hostname/action and returns a scoped approval payload.
2. **Signal verifier:** fetches and evaluates a frozen GitHub Release or Open-Meteo Rainfall source. A launch pins the source URL, predicate, WASM hash, and build target.

## Trust model

| Component | Trusted for | Not trusted for |
| --- | --- | --- |
| Coin contract | balances, supply, immutable rules, callback validation | independently knowing a human/event is true |
| Factory | creating/indexing launches | changing an existing coin |
| Frontend / Vercel | rendering and composing transactions | minting or allocation approval |
| OutLayer TEE | running the pinned verifier and protecting injected secrets | proving one person has one wallet; making an external source objectively true |
| Cloudflare Turnstile | bot-resistance signal | permanent human identity |

## Sybil resistance and limits

This is deliberately **demo-grade bot resistance**, not global personhood:

- Turnstile is verified in the TEE; allocation is settled on-chain.
- An approval is one-use and bound to wallet, request nonce, coin, action, and short expiration.
- An approval cannot be replayed across wallets, actions, or requests.
- A determined attacker can still solve challenges or control multiple wallets.
- A TEE proves the pinned program processed input; it does not prove a public data source is always correct.
- This project is testnet-only and unaudited. Do not use real funds.

## Verify a coin yourself

Reviewers do not need to trust the UI:

1. Open the coin Launch Passport.
2. Read its Birth Certificate and launch configuration.
3. Compare verifier URL/hash and frozen source/predicate to the published artifact.
4. Read supply status; minted plus permanently unmintable supply must never exceed maximum supply.
5. Inspect events, Door requests, Streak status, Work-to-Mint status, commitments, and protection-probe receipts.
6. Inspect the associated transaction and receipt in NEAR Explorer.

Before choosing Streak parameters, run the deterministic 60-day rehearsal:

    node scripts/simulate-streak.mjs

## Repository map

    contracts/factory/              permissionless keyless coin factory
    contracts/coin/                 NEP-141 token and immutable mechanics
    tee/auspice-human-verifier/     Turnstile verifier for OutLayer WASI P2
    tee/auspice-verifier/           GitHub Release and rainfall verifier
    web/                            Next.js app, wallet, explorer, arena
    scripts/                        builds and streak simulation
    docs/                           deployment and testnet notes

## Local development

Prerequisites: Rust with wasm32-wasip2, Node.js/npm, a NEAR testnet wallet, and an OutLayer testnet account for live TEE artifact publication.

    scripts/build-contracts.sh
    cargo test --workspace
    cargo build -p auspice-verifier --target wasm32-wasip2 --release
    cargo build -p auspice-human-verifier --target wasm32-wasip2 --release
    node scripts/simulate-streak.mjs
    cd web && npm install && npm run lint && npm run build && npm run test:e2e

Copy [the example environment file](./.env.example) into local/Vercel configuration. Public environment values describe network and artifact configuration. The Cloudflare secret key belongs only in OutLayer protected secrets as TURNSTILE_SECRET — never in Git, a browser variable, or Vercel.

## Deployment and current status

Artifact hashes and deployment order are in [docs/DEPLOYMENT.md](./docs/DEPLOYMENT.md).

1. Build and SHA-256 each contract and verifier.
2. Upload verifier WASM artifacts to OutLayer/FastFS.
3. Bind TURNSTILE_SECRET to the exact human-verifier hash through OutLayer protected secrets.
4. Upload global coin code, point the factory at its code hash, and configure public Vercel variables.
5. Launch fresh coins and exercise live wallet flows.

## What Auspice does not claim

- CAPTCHA is not proof of one unique person.
- An external event source is not incorruptible.
- A token chart is not a measure of its mechanic.
- A launched coin cannot be made safe by a hidden admin switch.

## License

[MIT](./LICENSE)

[nep-141]: https://github.com/near/NEPs/blob/master/neps/nep-0141.md
[nep-145]: https://github.com/near/NEPs/blob/master/neps/nep-0145.md
[nep-148]: https://github.com/near/NEPs/blob/master/neps/nep-0148.md
