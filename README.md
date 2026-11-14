# Auspice

**Supply follows reality, not insiders.**

Auspice is a NEAR testnet protocol for permissionless memecoin launches. Every
coin has a human-gated Genesis allocation and immutable real-world Signal Doors
that release later supply only after a pinned OutLayer TEE verifies the
configured event.

## What is implemented

- NEP-141 token rails, NEP-145 storage, and NEP-148 metadata.
- Keyless per-coin accounts created by a permissionless factory.
- Cloudflare Turnstile verified only inside a hash-pinned OutLayer TEE.
- One-use, wallet-bound, short-lived on-chain human approvals for both Genesis
  and Signal Door claims.
- GitHub Release and Open-Meteo Rainfall Signal Door templates.
- A fully on-chain Daily Streak template: holders check in once per day;
  the immutable target mints a fixed reward to all checkers, while a missed
  target permanently removes a fixed ritual-reserve amount and resets the
  streak. The current and longest streak are public contract state.
- A Proofbound Launch template: a creator’s public commitment, evidence
  requirement, deadline, quorum, vote threshold, creator reward, and voter
  reward are immutable. Settled Genesis participants use a fresh TEE human
  check to cast one equal vote; failure permanently removes both reward slices.
- Normal and Fortress templates with immutable creator vesting. Fortress adds
  max-wallet, max-transfer, cooldown, and burn-fee rules whose fee can only
  decay on its frozen schedule.
- Work-to-Mint launches with a fixed reserve, proof difficulty, one-use
  wallet-bound nonces, and a hard per-wallet claim cap. The coin dashboard can
  mine and submit a valid nonce locally without a mining backend.
- Optional circulation decay on every template. Settlement is lazy on token
  movement or explicitly callable; the public status view reports the exact
  pending days and burn amount for an account.
- Read-only protection previews plus durable public probe receipts recording
  the sender, receiver, amount, decision, reason, burn fee, and block time.
- Locked Market launches with an immutable contract-held curve. Buy deposits
  remain as locked liquidity and can leave only through the matching
  `sell_curve` path; neither a creator nor the factory has a withdrawal call.
- A Launch Passport on every coin page, shareable launch presets, and a public
  Demo Arena for real protection probes and named failed-rule transactions.
- Immutable source, predicate, verifier hash, caps, deadlines, settlement, and
  permanent unused-reserve removal.
- A Next.js launch form covering every template and optional decay, a
  factory-wide module-aware explorer, wallet flow, local nonce miner,
  circulation settlement, interactive protection probe/audit panel, signal
  queue, and all claim and close/burn controls.

The browser and Vercel server are never allocation authorities. They compose
transactions; the contract accepts only the private callback from the pinned
TEE worker.

## Local development

```sh
scripts/build-contracts.sh
cargo test --workspace
cargo build -p auspice-verifier --target wasm32-wasip2 --release
cargo build -p auspice-human-verifier --target wasm32-wasip2 --release
node scripts/simulate-streak.mjs

cd web
npm install
npm run lint
npm run build
npx playwright install chromium
npm run test:e2e
```

Copy `.env.example` to the relevant local or Vercel environment. The only
secret for human verification is `TURNSTILE_SECRET`, stored in OutLayer
protected secrets—never in this repository or Vercel.

## Deployment

Deployment is deliberately deferred. Read [docs/DEPLOYMENT.md](./docs/DEPLOYMENT.md)
for the built artifact hashes and required deployment order.

## Testnet only

Not audited. Do not use real funds. CAPTCHA reduces automation but does not
prove one human equals one person; a TEE proves which pinned program processed
an input, not that an upstream data source is objectively true. Streak
check-ins are holder-gated and capped per day; the Genesis CAPTCHA is only
demo-level Sybil resistance, so a determined attacker can still obtain or
split token holdings.

Proofbound protects immutable supply and creator-allocation rules. It does not
by itself lock DEX liquidity or eliminate trading risk; a liquidity lock needs
a separately integrated AMM/LP contract.

## License

MIT. See [LICENSE](./LICENSE).
