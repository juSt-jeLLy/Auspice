//! A deliberately non-sensitive OutLayer diagnostic for the human verifier.
//!
//! It never reads, serializes, hashes, or sends `TURNSTILE_SECRET`.  It only
//! reports whether OutLayer injected that variable. A separate network probe
//! is intentionally omitted: this binary isolates secret injection from WASI
//! outbound networking.

use serde::Serialize;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct Diagnostic {
    secret_present: bool,
    clock_available: bool,
    https_status: Option<u16>,
    https_error: Option<String>,
}

#[cfg(target_os = "wasi")]
fn cloudflare_https_probe() -> Result<u16, String> {
    use std::time::Duration;
    use wasi_http_client::Client;

    // This makes a public GET to the exact host used by Turnstile Siteverify.
    // It deliberately sends neither a Turnstile token nor any protected secret.
    let response = Client::new()
        .get("https://challenges.cloudflare.com/cdn-cgi/trace")
        .connect_timeout(Duration::from_secs(10))
        .send()
        .map_err(|error| error.to_string())?;
    let status = response.status();
    let _ = response.body().map_err(|error| error.to_string())?;
    Ok(status)
}

#[cfg(not(target_os = "wasi"))]
fn cloudflare_https_probe() -> Result<u16, String> {
    Err("WASI HTTP is unavailable on this host build".to_string())
}

fn main() {
    let clock_available = SystemTime::now().duration_since(UNIX_EPOCH).is_ok();
    let (https_status, https_error) = match cloudflare_https_probe() {
        Ok(status) => (Some(status), None),
        Err(error) => (None, Some(error)),
    };
    let result = Diagnostic {
        secret_present: std::env::var_os("TURNSTILE_SECRET").is_some(),
        clock_available,
        https_status,
        https_error,
    };

    // Always return a JSON output with exit status zero.  This means OutLayer
    // records the result even when the network probe cannot connect.
    let mut stdout = io::stdout();
    serde_json::to_writer(&mut stdout, &result).expect("diagnostic JSON serialization");
    stdout.flush().expect("diagnostic stdout flush");
}
