//! Deterministic, source-template verifier used inside an OutLayer WASI TEE.
//! It intentionally supports a small allowlisted vocabulary rather than an
//! arbitrary URL runner. GitHub Release and Open-Meteo Rainfall execute end-to-end.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SignalTemplate { GithubRelease, Rainfall, UsgsEarthquake, SportsScore, PublicAccountPost }

#[derive(Debug, Deserialize)]
pub struct Predicate {
    pub version: u16,
    pub required_tag: Option<String>,
    pub minimum_magnitude_x10: Option<u16>,
    pub expected_winner: Option<String>,
    pub required_text: Option<String>,
    pub minimum_rainfall_mm_x10: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct Input {
    pub request_id: u64,
    pub request_nonce: String,
    pub coin_id: String,
    pub door_id: u16,
    pub template: SignalTemplate,
    pub source_id: String,
    pub source_url: String,
    pub predicate: Predicate,
    pub predicate_version: u16,
    pub expected_wasm_hash: String,
}

#[derive(Debug, Serialize)]
pub struct Output {
    pub request_id: u64,
    pub request_nonce: String,
    pub coin_id: String,
    pub door_id: u16,
    pub template: String,
    pub source_id: String,
    pub source_url: String,
    pub predicate_version: u16,
    pub predicate_passed: bool,
    pub fetched_at_ns: u64,
    pub expires_at_ns: u64,
    /// Base64 is used so NEAR's Base64VecU8 can compare the pinned 32 bytes.
    pub wasm_hash: String,
    pub response_hash: String,
    pub event_facts: serde_json::Value,
}

#[derive(Deserialize)]
struct GithubRelease { tag_name: String, published_at: Option<String>, draft: bool, prerelease: bool }

pub fn parse_input(raw: &str) -> Result<Input, String> {
    let input: Input = serde_json::from_str(raw).map_err(|e| format!("invalid input: {e}"))?;
    if input.request_nonce.len() < 16 || input.predicate.version != input.predicate_version { return Err("invalid request binding".into()); }
    match input.template {
        SignalTemplate::GithubRelease => {
            if !input.source_url.starts_with("https://api.github.com/repos/") || !input.source_url.ends_with("/releases/latest") { return Err("source URL is not an allowlisted GitHub release endpoint".into()); }
            if !input.source_id.starts_with("github_release:") { return Err("source identifier is not GitHub release".into()); }
            Ok(input)
        }
        SignalTemplate::Rainfall => {
            if !input.source_url.starts_with("https://api.open-meteo.com/v1/forecast?") || !input.source_url.contains("current=rain") { return Err("source URL is not an allowlisted Open-Meteo current-rain endpoint".into()); }
            if !input.source_id.starts_with("open_meteo_rain:") || input.predicate.minimum_rainfall_mm_x10.is_none() { return Err("invalid rainfall source or threshold".into()); }
            Ok(input)
        }
        _ => Err("template is declared but not enabled in this verifier".into()),
    }
}

pub fn evaluate_github_release(input: Input, body: &str, now_ns: u64) -> Result<Output, String> {
    let release: GithubRelease = serde_json::from_str(body).map_err(|e| format!("invalid GitHub response: {e}"))?;
    let expected = input.predicate.required_tag.as_deref();
    let passed = !release.draft && !release.prerelease && expected.map(|tag| tag == release.tag_name).unwrap_or(true);
    let digest = Sha256::digest(body.as_bytes());
    let facts = serde_json::json!({"tag_name": release.tag_name, "published_at": release.published_at, "draft": release.draft, "prerelease": release.prerelease});
    Ok(Output { request_id: input.request_id, request_nonce: input.request_nonce, coin_id: input.coin_id, door_id: input.door_id, template: "GithubRelease".into(), source_id: input.source_id, source_url: input.source_url, predicate_version: input.predicate_version, predicate_passed: passed, fetched_at_ns: now_ns, expires_at_ns: now_ns.saturating_add(15 * 60 * 1_000_000_000), wasm_hash: input.expected_wasm_hash, response_hash: base64(&digest), event_facts: facts })
}

#[derive(Deserialize)]
struct OpenMeteoCurrent { rain: f64, time: Option<String> }
#[derive(Deserialize)]
struct OpenMeteoResponse { current: OpenMeteoCurrent }

/// The frozen URL names both coordinates and `current=rain`; a response hash
/// is returned so anyone can inspect the exact data the enclave used.
pub fn evaluate_rainfall(input: Input, body: &str, now_ns: u64) -> Result<Output, String> {
    let weather: OpenMeteoResponse = serde_json::from_str(body).map_err(|e| format!("invalid Open-Meteo response: {e}"))?;
    if !weather.current.rain.is_finite() || weather.current.rain < 0.0 { return Err("invalid rainfall reading".into()); }
    let minimum_x10 = input.predicate.minimum_rainfall_mm_x10.ok_or("missing rainfall threshold")?;
    let observed_x10 = (weather.current.rain * 10.0).round() as u16;
    let digest = Sha256::digest(body.as_bytes());
    let facts = serde_json::json!({"rainfall_mm": weather.current.rain, "rainfall_mm_x10": observed_x10, "minimum_rainfall_mm_x10": minimum_x10, "observed_at": weather.current.time});
    Ok(Output { request_id: input.request_id, request_nonce: input.request_nonce, coin_id: input.coin_id, door_id: input.door_id, template: "Rainfall".into(), source_id: input.source_id, source_url: input.source_url, predicate_version: input.predicate_version, predicate_passed: observed_x10 >= minimum_x10, fetched_at_ns: now_ns, expires_at_ns: now_ns.saturating_add(15 * 60 * 1_000_000_000), wasm_hash: input.expected_wasm_hash, response_hash: base64(&digest), event_facts: facts })
}

fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::new();
    for chunk in bytes.chunks(3) { let a = chunk[0]; let b = *chunk.get(1).unwrap_or(&0); let c = *chunk.get(2).unwrap_or(&0); output.push(TABLE[(a >> 2) as usize] as char); output.push(TABLE[((a & 3) << 4 | b >> 4) as usize] as char); output.push(if chunk.len() > 1 { TABLE[((b & 15) << 2 | c >> 6) as usize] as char } else { '=' }); output.push(if chunk.len() > 2 { TABLE[(c & 63) as usize] as char } else { '=' }); }
    output
}

#[cfg(test)] mod tests {
    use super::*;
    fn input() -> Input { parse_input(r#"{"request_id":1,"request_nonce":"abcdefghijklmnop","coin_id":"ship.testnet","door_id":0,"template":"GithubRelease","source_id":"github_release:auspice/ship","source_url":"https://api.github.com/repos/auspice/ship/releases/latest","predicate":{"version":1,"required_tag":"v1.0.0","minimum_magnitude_x10":null,"expected_winner":null,"required_text":null},"predicate_version":1,"expected_wasm_hash":"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="}"#).unwrap() }
    #[test] fn accepts_matching_release() { let output = evaluate_github_release(input(), r#"{"tag_name":"v1.0.0","published_at":"2026-01-01T00:00:00Z","draft":false,"prerelease":false}"#, 100).unwrap(); assert!(output.predicate_passed); assert_eq!(output.response_hash.len(), 44); }
    #[test] fn rejects_non_allowlisted_url() { assert!(parse_input(r#"{"request_id":1,"request_nonce":"abcdefghijklmnop","coin_id":"ship.testnet","door_id":0,"template":"GithubRelease","source_id":"github_release:x/y","source_url":"https://example.com","predicate":{"version":1,"required_tag":null,"minimum_magnitude_x10":null,"expected_winner":null,"required_text":null},"predicate_version":1,"expected_wasm_hash":"x"}"#).is_err()); }
    #[test] fn evaluates_open_meteo_rainfall() {
        let rain = parse_input(r#"{"request_id":2,"request_nonce":"abcdefghijklmnop","coin_id":"rain.testnet","door_id":0,"template":"Rainfall","source_id":"open_meteo_rain:40.71,-74.01","source_url":"https://api.open-meteo.com/v1/forecast?latitude=40.71&longitude=-74.01&current=rain","predicate":{"version":1,"required_tag":null,"minimum_magnitude_x10":null,"expected_winner":null,"required_text":null,"minimum_rainfall_mm_x10":3},"predicate_version":1,"expected_wasm_hash":"hash"}"#).unwrap();
        let result = evaluate_rainfall(rain, r#"{"current":{"rain":0.4,"time":"2026-11-13T09:00"}}"#, 100).unwrap();
        assert!(result.predicate_passed);
        assert_eq!(result.template, "Rainfall");
    }
}
