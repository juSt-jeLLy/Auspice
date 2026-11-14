//! A deliberately non-sensitive OutLayer diagnostic for the human verifier.
//!
//! It never reads, serializes, hashes, or sends `TURNSTILE_SECRET`.  It only
//! reports whether OutLayer injected that variable. A separate network probe
//! is intentionally omitted: this binary isolates secret injection from WASI
//! outbound networking.

use serde::Serialize;
use std::io::{self, Write};

#[derive(Serialize)]
struct Diagnostic {
    secret_present: bool,
}

fn main() {
    let result = Diagnostic {
        secret_present: std::env::var_os("TURNSTILE_SECRET").is_some(),
    };

    // Always return a JSON output with exit status zero.  This means OutLayer
    // records the result even when the network probe cannot connect.
    let mut stdout = io::stdout();
    serde_json::to_writer(&mut stdout, &result).expect("diagnostic JSON serialization");
    stdout.flush().expect("diagnostic stdout flush");
}
