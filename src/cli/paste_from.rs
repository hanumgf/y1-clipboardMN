
// src/cli/paste_from.rs

use crate::wayland;
use crate::core::constants::*;
use crate::cli::utils;
use std::io::{self, Write};

/// Fetch data directly from the system clipboard and output to stdout.
pub fn run(args: &[String]) {
    // Extract MIME type from positional arguments; ignore flags
    let mime = args.get(2)
        .filter(|s| !utils::is_option(s))
        .map(|s| s.as_str())
        .unwrap_or(DEFAULT_MIME);

    // Bypasses history storage to interact directly with the Wayland compositor
    let raw = wayland::paste_from_os(mime);

    if raw.is_empty() {
        eprintln!("{}no content available for MIME type: {}", LOG_ERROR, mime);
        eprintln!("  hint: ensure the clipboard is not empty or try a different format.");
        std::process::exit(1);
    }

    // Stream raw bytes to standard output
    let mut stdout = io::stdout();
    if let Err(e) = stdout.write_all(&raw) {
        eprintln!("{}io pipe failure: {}", LOG_ERROR, e);
        return;
    }
    let _ = stdout.flush();
}
