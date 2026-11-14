#!/usr/bin/env node

// Prints the metadata/config JSON accepted by `factory.create_coin`. This is
// intentionally a testnet-only, explicitly labelled launch: its human
// approval is a pinned OutLayer demo callback, not a CAPTCHA assertion.

const creatorId = process.env.AUSPICE_DEMO_CREATOR_ID ?? "busymuseum5448.testnet";
const symbol = (process.env.AUSPICE_DEMO_SYMBOL ?? "AUSD").toUpperCase();
const genesisDurationSeconds = BigInt(process.env.AUSPICE_DEMO_GENESIS_DURATION_SECONDS ?? "300");
const humanWasmUrl = "https://test.fastfs.io/busymuseum5448.testnet/outlayer.testnet/0edabc6944e69a3a17d6dc288565966916172289d7c72cf9ef8fc042a1a4a869.wasm";
const humanWasmHash = "0edabc6944e69a3a17d6dc288565966916172289d7c72cf9ef8fc042a1a4a869";
const ns = 1_000_000n;
const hour = 3_600n * 1_000_000_000n;
const now = BigInt(Date.now()) * ns;

function base64FromHex(hex) {
  return Buffer.from(hex, "hex").toString("base64");
}

const output = {
  metadata: {
    spec: "ft-1.0.0",
    name: "Auspice Testnet Approval Demo",
    symbol,
    decimals: 0,
  },
  config: {
    creator_id: creatorId,
    maximum_mintable_supply: "1000000",
    genesis_reserve: "700000",
    genesis_user_cap: "1000",
    genesis_price_yocto_near_per_token: "1",
    // Open immediately. Five minutes is enough for the repeatable testnet
    // walkthrough; callers may extend it with AUSPICE_DEMO_GENESIS_DURATION_SECONDS.
    genesis_open_at_ns: (now - hour).toString(),
    genesis_close_at_ns: (now + genesisDurationSeconds * 1_000_000_000n).toString(),
    human_verifier: {
      wasm_url: humanWasmUrl,
      wasm_hash: base64FromHex(humanWasmHash),
      wasm_build_target: "wasm32-wasip2",
      // Required by the shared config ABI, but intentionally not attached
      // when `testnet_demo` is true.
      secrets_profile: "testnet-demo",
      secrets_account_id: "busymuseum5448.testnet",
      expected_hostname: "web-tau-tan-79.vercel.app",
      genesis_action: "auspice_testnet_demo",
      signal_action: "auspice_testnet_demo",
      approval_lifetime_ns: "__AUSPICE_U64_600000000000__",
      testnet_demo: true,
    },
    outlayer_account_id: "outlayer.testnet",
    max_signal_age_ns: (15n * 60n * 1_000_000_000n).toString(),
    no_creator_genesis_allocation: true,
    doors: [],
    streak: null,
    proofbound: null,
    protection: null,
    // The remaining supply is held by the immutable market. Its reserve has
    // no creator/factory withdrawal path, which lets this launch power Arena.
    work_mint: null,
    circulation: null,
    bonding_curve: {
      token_reserve: "300000",
      virtual_near_reserve: "1000000000000000000000000",
      max_buy: "1000000000000000000000000",
      max_sell: "10000",
    },
    version: 1,
  },
};

process.stdout.write(JSON.stringify(output).replace(/"__AUSPICE_U64_(\d+)__"/g, "$1"));
