
// src/cli/show.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use std::io::{self, Write};

/// Inspect the content of an item with the specified ID or output it as raw binary data.
pub fn run(args: &[String], db: ClipboardDb) {
    // 1. Extract index from arguments
    let id_str = match args.get(2) {
        Some(s) => s,
        None => {
            eprintln!("{}no ID provided.", LOG_ERROR);
            println!("usage: y1-clip show <id> [--raw]");
            return;
        }
    };

    let idx = match id_str.parse::<usize>() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("{}'{}' is not a valid numerical ID.", LOG_ERROR, id_str);
            return;
        }
    };

    // 2. Identify the real database ID from metadata records
    let meta = db.fetch_metadata(MAX_HISTORY);
    let real_id = match meta.get(idx) {
        Some(&(id, _, _, _, _)) => id,
        None => {
            eprintln!("{}entry with ID [{}] not found.", LOG_ERROR, idx);
            return;
        }
    };

    // 3. Retrieve the full payload content (including large binaries) from the database
    let (mime, val) = match db.get_content_by_id(real_id) {
        Some(res) => res,
        None => {
            eprintln!("{}failed to retrieve content for ID [{}].", LOG_ERROR, idx);
            return;
        }
    };

    // 4. Evaluate output mode requested
    let is_raw = args.iter().any(|a| a == "--raw");

    if is_raw {
        // --- RAW Mode: Stream pure bytes to stdout without any decorators ---
        let mut stdout = io::stdout();
        if let Err(e) = stdout.write_all(&val) {
            eprintln!("{}failed to write raw data: {}", LOG_ERROR, e);
        }
        let _ = stdout.flush();
        // Terminate process flow directly to suppress appending extra trailing newlines
    } else {
        // --- HUMAN Mode: Render details in a highly structured terminal layout ---
        println!("--- DETAILS ---");
        println!("ID:       {}", idx);
        println!("MIME:     {}", mime);
        println!("SIZE:     {} bytes", val.len());
        println!("---------------");

        if mime.contains("text") || mime.contains("UTF8") {
            // Textual payload handling
            println!("{}", String::from_utf8_lossy(&val));
        } else {
            // Binary payload handling (e.g., images)
            println!("[Binary data cannot be displayed in terminal]");
            println!("Hint: Use 'y1-clip show {} --raw > file' to export.", idx);
        }
        println!("---------------");
    }
}
