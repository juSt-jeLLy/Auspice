//! Explicit testnet-only human-approval demonstrator.
//!
//! This executable intentionally does *not* contact Cloudflare and must never
//! be configured for a production launch. It exercises the same OutLayer
//! WASI execution, hash pinning, request binding, and private NEAR callback
//! as the real verifier, so a public testnet demo can proceed when a third
//! party's outbound HTTPS endpoint is unavailable.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};

const TESTNET_DEMO_TOKEN: &str = "auspice-testnet-demo";

#[derive(Deserialize)]
struct Input {
    request_id: u64,
    request_nonce: String,
    coin_id: String,
    account_id: String,
    kind: String,
    door_id: Option<u16>,
    captcha_token: String,
    expected_wasm_hash: String,
}

#[derive(Serialize)]
struct Output {
    request_id: u64,
    request_nonce: String,
    coin_id: String,
    account_id: String,
    kind: String,
    door_id: Option<u16>,
    approved: bool,
    expires_at_ns: u64,
    max_allocation: String,
    wasm_hash: String,
    captcha_token_hash: String,
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 15) as usize] as char);
    }
    result
}

fn run() -> Result<Output, String> {
    let mut raw = String::new();
    io::stdin()
        .read_to_string(&mut raw)
        .map_err(|error| format!("stdin: {error}"))?;
    let input: Input = serde_json::from_str(&raw).map_err(|error| format!("input: {error}"))?;
    if input.request_nonce.len() < 16 || input.coin_id.is_empty() || input.account_id.is_empty() {
        return Err("invalid immutable request binding".to_string());
    }
    if !matches!(input.kind.as_str(), "genesis" | "signal_door" | "commitment_vote") {
        return Err("unsupported approval kind".to_string());
    }

    let token_hash = Sha256::digest(input.captcha_token.as_bytes());

    Ok(Output {
        request_id: input.request_id,
        request_nonce: input.request_nonce,
        coin_id: input.coin_id,
        account_id: input.account_id,
        kind: input.kind,
        door_id: input.door_id,
        // A deliberately non-secret, visibly testnet-only passphrase. This
        // makes accidental use distinguishable in events and configuration.
        approved: input.captcha_token == TESTNET_DEMO_TOKEN,
        // `0` is a deliberately reserved demo signal. The explicit
        // `testnet_demo` contract mode turns it into block time + the frozen
        // lifetime when the private OutLayer callback executes.
        expires_at_ns: 0,
        max_allocation: "340282366920938463463374607431768211455".to_string(),
        wasm_hash: input.expected_wasm_hash,
        captcha_token_hash: hex(&token_hash),
    })
}

fn main() {
    match run().and_then(|output| serde_json::to_string(&output).map_err(|error| error.to_string())) {
        Ok(output) => print!("{output}"),
        Err(error) => {
            let escaped = serde_json::to_string(&error).unwrap_or_else(|_| "\"internal error\"".to_string());
            print!("{{\"error\":{escaped}}}");
            std::process::exit(1);
        }
    }
    let _ = io::stdout().flush();
}
