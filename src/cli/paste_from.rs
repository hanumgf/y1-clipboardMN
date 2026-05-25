
// src/cli/paste_from.rs

use crate::wayland;
use crate::core::constants::*;
use std::io::{self, Write};

pub fn run(args: &[String]) {
    // Determine target MIME type; fall back to default if not specified
    let mime = args.get(2).map(|s| s.as_str()).unwrap_or(DEFAULT_MIME);

    // Fetch directly from Wayland, bypassing the local database entirely
    let raw = wayland::paste_from_os(mime);

    if raw.is_empty() {
        // Troubleshoot root cause for empty data retrieval
        eprintln!("{}could not retrieve clipboard data.", LOG_ERROR);
        eprintln!("  hint: verify the clipboard is not empty and supports MIME: {}", mime);
        std::process::exit(1);
    }

    // Direct the raw binary bytes to stdout
    let mut stdout = io::stdout();
    if let Err(e) = stdout.write_all(&raw) {
        eprintln!("{}pipe error: {}", LOG_ERROR, e);
        return;
    }
    let _ = stdout.flush();
}
