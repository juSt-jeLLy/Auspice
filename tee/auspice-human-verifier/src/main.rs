use std::io::{self, Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use auspice_human_verifier::{evaluate, parse_input, TurnstileResponse};
#[cfg(target_os = "wasi")]
use auspice_human_verifier::TURNSTILE_SITEVERIFY_URL;

fn run() -> Result<String, String> {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw).map_err(|error| error.to_string())?;
    let input = parse_input(&raw)?;
    let secret = std::env::var("TURNSTILE_SECRET").map_err(|_| "TURNSTILE_SECRET is unavailable; this verifier must run with OutLayer protected secrets".to_string())?;
    let response = verify(&secret, &input.captcha_token)?;
    let now_ns = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|error| error.to_string())?.as_nanos().min(u64::MAX as u128) as u64;
    serde_json::to_string(&evaluate(input, response, now_ns)).map_err(|error| error.to_string())
}

#[cfg(target_os = "wasi")]
fn verify(secret: &str, token: &str) -> Result<TurnstileResponse, String> {
    use std::time::Duration;
    use wasi_http_client::Client;
    let response = Client::new().post(TURNSTILE_SITEVERIFY_URL).connect_timeout(Duration::from_secs(10)).form(&[("secret", secret), ("response", token)]).send().map_err(|error| format!("Turnstile Siteverify request failed: {error}"))?;
    if !(200..300).contains(&response.status()) { return Err(format!("Turnstile Siteverify HTTP {}", response.status())); }
    serde_json::from_slice(&response.body().map_err(|error| error.to_string())?).map_err(|error| format!("invalid Turnstile response: {error}"))
}

#[cfg(not(target_os = "wasi"))]
fn verify(_: &str, _: &str) -> Result<TurnstileResponse, String> { Err("Turnstile Siteverify is available only inside the WASI TEE".into()) }

fn main() {
    match run() { Ok(value) => print!("{value}"), Err(error) => { print!("{{\"error\":{}}}", serde_json::to_string(&error).unwrap()); std::process::exit(1) } }
    let _ = io::stdout().flush();
}
