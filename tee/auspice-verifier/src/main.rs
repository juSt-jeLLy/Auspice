use std::io::{self, Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use auspice_verifier::{evaluate_github_release, parse_input};

fn run() -> Result<String, String> {
    let mut raw = String::new(); io::stdin().read_to_string(&mut raw).map_err(|e| e.to_string())?;
    let input = parse_input(&raw)?;
    let url = input.source_url.clone();
    let (status, body) = http_get(&url)?;
    if !(200..300).contains(&status) { return Err(format!("GitHub HTTP {status}")); }
    let now_ns = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos().min(u64::MAX as u128) as u64;
    serde_json::to_string(&evaluate_github_release(input, &body, now_ns)?).map_err(|e| e.to_string())
}
fn main() { match run() { Ok(value) => print!("{value}"), Err(error) => { print!("{{\"error\":{}}}", serde_json::to_string(&error).unwrap()); std::process::exit(1) } }; let _ = io::stdout().flush(); }
#[cfg(target_os = "wasi")] fn http_get(url: &str) -> Result<(u16, String), String> { use std::time::Duration; use wasi_http_client::Client; let response = Client::new().get(url).connect_timeout(Duration::from_secs(10)).send().map_err(|e| e.to_string())?; let status = response.status(); let body = String::from_utf8(response.body().map_err(|e| e.to_string())?).map_err(|e| e.to_string())?; Ok((status, body)) }
#[cfg(not(target_os = "wasi"))] fn http_get(_: &str) -> Result<(u16, String), String> { Err("HTTP only available in WASI".into()) }
