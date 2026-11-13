# Auspice testnet deployment

All contracts and the Ship Signal proof launch use NEAR testnet only.

## Live testnet factory

- Factory account: `auspice-yagnesh.testnet`
- Factory owner: `auspice-yourname.testnet`
- Configured future-coin hash: `29dPdc4QYu21NqXz82obQKPFVRYXZwodn4enZkBrsGMm` (the matching global-code upload is pending)
- Verified factory state: `coin_count = 0`, minimum creation deposit `0.3 NEAR`

The first global upload from this recreated account failed on 2026-11-13 because the account had 24 NEAR while the protocol required 32.4004 NEAR for code storage. Fund the factory with at least 10 additional testnet NEAR before retrying. No coin was registered by the failed SHIP creation attempt.

## Published OutLayer verifier

- OutLayer project: `busymuseum5448.testnet/auspice`
- Build target: `wasm32-wasip2`
- Exact FastFS URL: `https://test.fastfs.io/busymuseum5448.testnet/outlayer.testnet/70fffbca1595684ebdee739ec2a7681b5ce1cf8f61567f656ad1a173aa60ab49.wasm`
- SHA-256: `70fffbca1595684ebdee739ec2a7681b5ce1cf8f61567f656ad1a173aa60ab49`

The URL returned HTTP 200 and served 414,073 bytes when verified on 2026-11-13. New launches pin this URL and hash directly; changing the project’s active version cannot alter an existing Door.

## Retired predecessor deployment

- Former factory account: `auspice-yagnesh.testnet`
- Former factory owner: `auspice-yagnesh.testnet`
- Pinned global coin code hash: `4Uab5k15uKw5fqSPyjFZccnPuF4HeqcQwY5knancJLKp`
- Global coin-code transaction: [HYk5GE22QL2QZm9qLvHuk8A5FbDAccpgfqT3dKQP1nw](https://explorer.testnet.near.org/transactions/HYk5GE22QL2QZm9qLvHuk8A5FbDAccpgfqT3dKQP1nw)
- Factory deployment/initialization: [5mHA4TDchJs9mHRfc6n9i9u73ZKJm9Z9A4rtyc2e4Xom](https://explorer.testnet.near.org/transactions/5mHA4TDchJs9mHRfc6n9i9u73ZKJm9Z9A4rtyc2e4Xom)

The factory had `coin_count = 0` and was deliberately deleted on 2026-11-13. Transaction [`DCvmPpzzLf6NVSPfbxFHonJEyun2Sj2QsLVJSmkibWEp`](https://explorer.testnet.near.org/transactions/DCvmPpzzLf6NVSPfbxFHonJEyun2Sj2QsLVJSmkibWEp) transferred the released balance/storage to `auspice-yourname.testnet`. Do not treat the listed code hash or account as a live deployment.

Install the NEAR deployment CLI once before using the deployment script:

```sh
cargo install near-cli-rs
```

## Required public accounts

- Factory: `auspice.testnet` (or the account selected before deployment)
- Ship Signal: `ship.<factory-account>`
- Fixtures: `alice`, `bob`, `carol`, `bot`, and `watcher` testnet accounts

Do not commit seed phrases, private keys, CAPTCHA secrets, or service credentials.

## Build and deploy

```sh
scripts/build-contracts.sh
AUSPICE_FACTORY_ID=auspice.testnet \
AUSPICE_FACTORY_OWNER_ID=<testnet-account-to-record-as-owner> \
scripts/deploy-testnet.sh
```

The replacement `auspice-yagnesh.testnet` is deployed and initialized. Its new factory owner is `auspice-yourname.testnet`.

The factory account itself must be funded and have its full-access signing key available to `near-cli-rs`. The deployment script stores the coin WASM as a NEAR global contract, then initializes the factory with the resulting immutable code hash. Coin creators subsequently call `create_coin` on the factory; no factory allowlist is involved.

## Before a live demo

- Fund the deployer and factory with testnet NEAR for state and gas.
- Generate a testnet-only Ed25519 signing key for human passes and put its PEM value only in `HUMAN_PASS_SIGNING_KEY_PEM`; put the matching raw 32-byte public key (Base64) in the immutable coin configuration.
- Set `TESTNET_DEMO_MODE=true`, `HUMAN_PASS_CHALLENGE_SECRET`, and `HUMAN_PASS_MAX_ALLOCATION` only for the clearly labeled testnet human-pass adapter.
- Build and upload `tee/auspice-verifier` to OutLayer. Use its returned FastFS URL and SHA-256 when creating a Door; each Door pins those exact values rather than trusting a mutable OutLayer project version.
- Record every deployed account, artifact hash, and demonstration transaction ID in this document.
