# Auspice testnet deployment

All contracts and the Raincheck demo use NEAR testnet only.

## Required public accounts

- Factory: `auspice.testnet` (or the account selected before deployment)
- Raincheck: `rain.<factory-account>`
- Fixtures: `alice`, `bob`, `carol`, `bot`, and `watcher` testnet accounts

Do not commit seed phrases, private keys, CAPTCHA secrets, or service credentials.

## Build and deploy

```sh
scripts/build-contracts.sh
AUSPICE_FACTORY_ID=auspice.testnet \
AUSPICE_DEPLOYER_ID=<funded-testnet-account> \
scripts/deploy-testnet.sh
```

The deployment script stores the coin WASM as a NEAR global contract, then initializes the factory with the resulting immutable code hash. Coin creators subsequently call `create_coin` on the factory; no factory allowlist is involved.

## Before a live demo

- Fund the deployer and factory with testnet NEAR for state and gas.
- Generate separate testnet-only Ed25519 signing keys for human passes and signal proofs.
- Configure the human-pass adapter and OutLayer worker/proof path.
- Record every deployed account, artifact hash, and demonstration transaction ID in this document.
