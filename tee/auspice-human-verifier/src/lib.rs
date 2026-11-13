//! Hash-pinned Turnstile verifier intended to run only inside an OutLayer TEE.
//!
//! The browser token is deliberately never returned. This program returns a
//! narrowly scoped, short-lived allocation approval which the coin contract
//! binds to its outstanding request in a private OutLayer callback.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const TURNSTILE_SITEVERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";
const APPROVAL_LIFETIME_NS: u64 = 10 * 60 * 1_000_000_000;

#[derive(Clone, Debug, Deserialize)]
pub struct Input {
    pub request_id: u64,
    pub request_nonce: String,
    pub coin_id: String,
    pub account_id: String,
    pub kind: String,
    pub door_id: Option<u16>,
    pub captcha_token: String,
    pub expected_hostname: String,
    pub expected_action: String,
    pub max_allocation: String,
    pub expected_wasm_hash: String,
}

#[derive(Debug, Deserialize)]
pub struct TurnstileResponse {
    pub success: bool,
    pub hostname: Option<String>,
    pub action: Option<String>,
    pub challenge_ts: Option<String>,
    #[serde(default)]
    pub error_codes: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Output {
    pub request_id: u64,
    pub request_nonce: String,
    pub coin_id: String,
    pub account_id: String,
    pub kind: String,
    pub door_id: Option<u16>,
    pub approved: bool,
    pub expires_at_ns: u64,
    pub max_allocation: String,
    pub wasm_hash: String,
    pub captcha_token_hash: String,
    pub verification: VerificationFacts,
}

#[derive(Debug, Serialize)]
pub struct VerificationFacts {
    pub hostname: Option<String>,
    pub action: Option<String>,
    pub challenge_ts: Option<String>,
    pub error_codes: Vec<String>,
}

pub fn parse_input(raw: &str) -> Result<Input, String> {
    let input: Input = serde_json::from_str(raw).map_err(|error| format!("invalid input: {error}"))?;
    if input.request_nonce.len() < 16 || input.request_nonce.len() > 128 {
        return Err("invalid request nonce".into());
    }
    if input.coin_id.is_empty() || input.account_id.is_empty() || input.expected_hostname.is_empty() {
        return Err("missing immutable request binding".into());
    }
    if !matches!(input.kind.as_str(), "genesis" | "signal_door") {
        return Err("unsupported allocation action".into());
    }
    if input.kind == "genesis" && input.door_id.is_some() {
        return Err("genesis cannot name a Door".into());
    }
    if input.kind == "signal_door" && input.door_id.is_none() {
        return Err("signal Door approval requires a Door".into());
    }
    if input.expected_action.is_empty() || input.expected_action.len() > 32 || !input.expected_action.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-') {
        return Err("invalid Turnstile action".into());
    }
    if input.captcha_token.is_empty() || input.captcha_token.len() > 4096 {
        return Err("invalid CAPTCHA token".into());
    }
    if !input.max_allocation.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("invalid allocation cap".into());
    }
    Ok(input)
}

pub fn evaluate(input: Input, response: TurnstileResponse, now_ns: u64) -> Output {
    let hostname_matches = response.hostname.as_deref() == Some(input.expected_hostname.as_str());
    let action_matches = response.action.as_deref() == Some(input.expected_action.as_str());
    let approved = response.success && hostname_matches && action_matches;
    let token_hash = Sha256::digest(input.captcha_token.as_bytes());
    Output {
        request_id: input.request_id,
        request_nonce: input.request_nonce,
        coin_id: input.coin_id,
        account_id: input.account_id,
        kind: input.kind,
        door_id: input.door_id,
        approved,
        expires_at_ns: now_ns.saturating_add(APPROVAL_LIFETIME_NS),
        max_allocation: input.max_allocation,
        wasm_hash: input.expected_wasm_hash,
        captcha_token_hash: hex(&token_hash),
        verification: VerificationFacts {
            hostname: response.hostname,
            action: response.action,
            challenge_ts: response.challenge_ts,
            error_codes: response.error_codes,
        },
    }
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes { result.push(HEX[(byte >> 4) as usize] as char); result.push(HEX[(byte & 15) as usize] as char); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Input { parse_input(r#"{"request_id":4,"request_nonce":"1234567890abcdef","coin_id":"ship.testnet","account_id":"alice.testnet","kind":"genesis","door_id":null,"captcha_token":"one-use-token","expected_hostname":"web-tau-tan-79.vercel.app","expected_action":"auspice_genesis","max_allocation":"10000","expected_wasm_hash":"abc"}"#).unwrap() }

    #[test]
    fn accepts_matching_turnstile_facts() {
        let output = evaluate(input(), TurnstileResponse { success: true, hostname: Some("web-tau-tan-79.vercel.app".into()), action: Some("auspice_genesis".into()), challenge_ts: Some("2026-11-13T00:00:00Z".into()), error_codes: vec![] }, 100);
        assert!(output.approved);
        assert_eq!(output.expires_at_ns, 100 + APPROVAL_LIFETIME_NS);
        assert_eq!(output.captcha_token_hash.len(), 64);
    }

    #[test]
    fn fails_closed_on_wrong_host_or_action() {
        let bad_host = evaluate(input(), TurnstileResponse { success: true, hostname: Some("evil.example".into()), action: Some("auspice_genesis".into()), challenge_ts: None, error_codes: vec![] }, 100);
        assert!(!bad_host.approved);
        let bad_action = evaluate(input(), TurnstileResponse { success: true, hostname: Some("web-tau-tan-79.vercel.app".into()), action: Some("other".into()), challenge_ts: None, error_codes: vec![] }, 100);
        assert!(!bad_action.approved);
    }
}
