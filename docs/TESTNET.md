# Auspice testnet notes

The intended factory account is `auspice-yagnesh.testnet`, owned by
`auspice-yourname.testnet`. Deployment is currently paused by project choice;
do not treat any prior testnet artifact as the current Auspice contract.

The only supported contract flow is:

1. Turnstile token is sent in `request_human_check`.
2. The hash-pinned OutLayer human verifier checks it with its protected
   `TURNSTILE_SECRET`.
3. The private callback writes a short-lived wallet-bound approval.
4. The user consumes that approval in `request_genesis_allocation` or
   `claim_signal_door`.

The event verifier supports GitHub Release and Open-Meteo Rainfall Doors. Each
coin pins the exact FastFS URL and SHA-256 hash at creation. Daily Streak is a
separate fully on-chain template: holders check in once per immutable daily
window and anyone settles the target/mint or miss/burn result after it closes.

Proofbound needs no external event source. Its creator submits public evidence,
then settled Genesis participants receive one human-checked vote each. The
on-chain vote deterministically releases or permanently removes the immutable
creator and voter reward slices.

Normal reserves the post-Genesis allocation behind immutable creator vesting.
Fortress uses the same vesting rail and adds frozen max-wallet, max-transfer,
cooldown, and downward-only burn-fee decay rules. Work-to-Mint assigns the
post-Genesis reserve to wallet-bound proof-of-work claims. Its browser miner
searches locally and the contract independently verifies the submitted nonce.

Any launch may opt into daily circulation decay. Transfers settle sender and
receiver lazily; `settle_circulation` can settle an account explicitly and
`get_circulation_status` exposes its pending full days and projected burn.
Fortress transfer rules can be checked with `preview_transfer_protection` and
recorded without moving tokens through `probe_transfer_protection`; the public
`list_protection_probes` audit view preserves each decision and reason.

See [DEPLOYMENT.md](./DEPLOYMENT.md) for the current build hashes and
required deployment order.
