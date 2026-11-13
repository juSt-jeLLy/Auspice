# Auspice

**Supply follows reality, not insiders.**

Auspice is a NEAR testnet protocol for memecoins with an equal-access Genesis allocation and future supply released only through immutable, attested Signal Doors.

It contains the NEAR contracts, Next.js application, testnet deployment scripts, and reproducible build artifacts for the Raincheck (`RAIN`) demonstration coin.

## Status

Implemented today:

- NEP-141 token rails, NEP-145 storage management, and NEP-148 metadata.
- Immutable Genesis allocations, caps, refund credit, permissionless settlement, and supply accounting.
- A permissionless keyless-account factory pinned to a global coin contract hash.
- A request-bound OutLayer Signal Door callback path; the browser cannot submit a proof or mint a token.
- Wallet connection/account switching, contract reads, transaction prompts, Genesis request flow, factory configuration, and verification views.
- A clearly labeled testnet-only human-pass adapter based on a one-use NEP-413 wallet challenge.

See [plan.md](./plan.md) for the complete product specification, threat model, and acceptance criteria.

## Local development

```sh
scripts/build-contracts.sh
cargo test --workspace

cd web
npm install
npm run dev
```

Copy `.env.example` to the deployment environment. For the testnet human-pass adapter, generate a development key with `node scripts/generate-human-pass-key.mjs /absolute/path/to/human-pass.pem`; never commit that private PEM. Put its public key in the immutable launch configuration.

## Deployment

`scripts/deploy-testnet.sh` deploys on NEAR testnet after `cargo install near-cli-rs`, a funded factory account with an available full-access key, and a factory owner account have been configured. Details and the non-secret environment checklist are in [docs/TESTNET.md](./docs/TESTNET.md).

The live testnet factory is [`auspice-yagnesh.testnet`](https://testnet.nearblocks.io/address/auspice-yagnesh.testnet), pinned to global coin code hash `4Uab5k15uKw5fqSPyjFZccnPuF4HeqcQwY5knancJLKp`. No demo coin has been created yet: its immutable OutLayer project and verifier hash are still required.

## Testnet only

Not audited. Do not use real funds. Human-pass and external-signal mechanisms reduce risks; they do not prove unique personhood or the objective truth of an upstream data source. `TESTNET_DEMO_MODE` is explicitly a demo adapter, not production Sybil resistance.

## License

MIT. See [LICENSE](./LICENSE).
