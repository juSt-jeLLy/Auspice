# Auspice deployment handoff

Deployment is deferred. The current coin contract is the only Auspice coin
contract; upload nothing until the team authorizes testnet deployment.

## Built artifacts

| Artifact | SHA-256 (hex) | Purpose |
| --- | --- | --- |
| `target/near/auspice_coin/auspice_coin.wasm` | `351aaf1622d160fc47d9714acc73a9c7dfb0f91ad2cd1ed5b3e17d674392983a` | NEP-141 coin with TEE approvals, all launch templates, Work-to-Mint, lazy decay, public probe receipts, and immutable locked-market trading |
| `target/near/auspice_factory/auspice_factory.wasm` | `b6a59efb6093d7c53f8c6052de1c331329c3110868d639446fcd6207f6e3af45` | Permissionless keyless coin factory |
| `target/wasm32-wasip2/release/auspice-verifier.wasm` | `7a776797cfb09893077d959258cc6ae443b89461b4d271a7c326d0fab4f2998f` | GitHub Release and Open-Meteo Rainfall verifier |
| `target/wasm32-wasip2/release/auspice-human-verifier.wasm` | `350b17d09fa390a28fbfdae49579206b8777ed48f07daef9db14a959782564ec` | Turnstile verifier with protected secret |

Rebuild and compare hashes immediately before every upload.

## Required order

1. Upload the event verifier to OutLayer and record its FastFS URL/hash.
2. Confirm the human verifier has `TURNSTILE_SECRET` in OutLayer protected
   secrets, limited to the pinned build.
3. Fund `auspice-yagnesh.testnet` before uploading coin global code. The
   458,331-byte module-complete coin WASM requires about 45.8331 NEAR of
   global-code storage stake plus transaction gas. Re-check the live balance
   and fund the account to at least 50 NEAR before using the example:

   ```sh
   near tokens send-near auspice-yourname.testnet to auspice-yagnesh.testnet '10 NEAR' network-config testnet sign-with-keychain send
   ```

4. Upload the coin global code only after that funding is confirmed.
5. Point the factory at the successful global-code hash.
6. Set Vercel public verifier URL/hash variables.
7. Create fresh Normal, Fortress, Work-to-Mint, Locked Market,
   circulation-decay, GitHub, Rainfall, Daily Streak, and Proofbound coins and
   run their full live flows, including nonce mining/claim, curve buy/sell,
   failed liquidity-lock transaction, decay preview/settlement, and
   allowed/blocked protection probe receipts.

## Public demo handoff

After the global coin WASM is uploaded and the factory has been pointed at its
new hash, create one **Fortress** coin and one **Locked Market** coin from the
factory. Use the Locked Market coin as the public Demo Arena target only after
its Genesis is finalized and it is `Live`.

Set this public Vercel value and redeploy the web project:

```sh
NEXT_PUBLIC_ARENA_COIN_ID=<live-locked-market-coin-account>.testnet
```

`/arena` then sends real wallet transactions to that account: recorded
max-transfer/max-wallet probes, and deliberately failing `LIQUIDITY_LOCK` and
`IMMUTABLE_RULES` attempts. The app is not complete as a public demo until
this live verification has been performed.

## Offline verification

```sh
cargo test --workspace
scripts/build-contracts.sh
cargo build -p auspice-verifier --target wasm32-wasip2 --release
cargo build -p auspice-human-verifier --target wasm32-wasip2 --release
cd web && npm run lint && npm run build && npm run test:e2e
```
