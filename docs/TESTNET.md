# Auspice testnet deployment

All contracts and the Raincheck demo use NEAR testnet only.

## Deployed factory

- Factory account: `auspice-yagnesh.testnet`
- Factory owner: `auspice-yagnesh.testnet`
- Pinned global coin code hash: `4Uab5k15uKw5fqSPyjFZccnPuF4HeqcQwY5knancJLKp`
- Global coin-code transaction: [HYk5GE22QL2QZm9qLvHuk8A5FbDAccpgfqT3dKQP1nw](https://explorer.testnet.near.org/transactions/HYk5GE22QL2QZm9qLvHuk8A5FbDAccpgfqT3dKQP1nw)
- Factory deployment/initialization: [5mHA4TDchJs9mHRfc6n9i9u73ZKJm9Z9A4rtyc2e4Xom](https://explorer.testnet.near.org/transactions/5mHA4TDchJs9mHRfc6n9i9u73ZKJm9Z9A4rtyc2e4Xom)

Factory views verified immediately after deployment: `coin_count = 0`, owner matches the listed account, and the stored hash matches the global coin code hash.

Install the NEAR deployment CLI once before using the deployment script:

```sh
cargo install near-cli-rs
```

## Required public accounts

- Factory: `auspice.testnet` (or the account selected before deployment)
- Raincheck: `rain.<factory-account>`
- Fixtures: `alice`, `bob`, `carol`, `bot`, and `watcher` testnet accounts

Do not commit seed phrases, private keys, CAPTCHA secrets, or service credentials.

## Build and deploy

```sh
scripts/build-contracts.sh
AUSPICE_FACTORY_ID=auspice.testnet \
AUSPICE_FACTORY_OWNER_ID=<testnet-account-to-record-as-owner> \
scripts/deploy-testnet.sh
```

The factory account itself must be funded and have its full-access signing key available to `near-cli-rs`. The deployment script stores the coin WASM as a NEAR global contract, then initializes the factory with the resulting immutable code hash. Coin creators subsequently call `create_coin` on the factory; no factory allowlist is involved.

## Before a live demo

- Fund the deployer and factory with testnet NEAR for state and gas.
- Generate a testnet-only Ed25519 signing key for human passes and put its PEM value only in `HUMAN_PASS_SIGNING_KEY_PEM`; put the matching raw 32-byte public key (Base64) in the immutable coin configuration.
- Set `TESTNET_DEMO_MODE=true`, `HUMAN_PASS_CHALLENGE_SECRET`, and `HUMAN_PASS_MAX_ALLOCATION` only for the clearly labeled testnet human-pass adapter.
- Configure the OutLayer project and verifier WASM hash used by the immutable callback path.
- Record every deployed account, artifact hash, and demonstration transaction ID in this document.
