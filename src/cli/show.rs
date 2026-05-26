
// src/cli/show.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use crate::cli::utils;
use std::io::{self, Write};

/// Inspect the content of a history entry by index or output raw bytes.
pub fn run(args: &[String], db: ClipboardDb) {
    // Positional argument extraction excluding options
    let id_arg = args.get(2).filter(|s| !utils::is_option(s));

    let idx = match id_arg {
        Some(s) => match s.parse::<usize>() {
            Ok(i) => i,
            Err(_) => {
                eprintln!("{}'{}' is not a valid numerical ID.", LOG_ERROR, s);
                return;
            }
        },
        None => {
            eprintln!("{}missing required entry ID.", LOG_ERROR);
            println!("usage: y1-clip show <id> [--raw | -R]");
            return;
        }
    };

    // Strict flag detection for binary stream output
    let is_raw = utils::has_flag(args, "--raw", "-R");

    let meta = db.fetch_metadata(MAX_HISTORY);
    let real_id = match meta.get(idx) {
        Some(&(id, _, _, _, _)) => id,
        None => {
            eprintln!("{}index [{}] exceeds available history scope.", LOG_ERROR, idx);
            return;
        }
    };

    let (mime, val) = match db.get_content_by_id(real_id) {
        Some(res) => res,
        None => {
            eprintln!("{}failed to fetch payload for entry [{}].", LOG_ERROR, idx);
            return;
        }
    };

    if is_raw {
        // RAW mode: Output pure binary payload to stdout without modification
        let mut stdout = io::stdout();
        let _ = stdout.write_all(&val);
        let _ = stdout.flush();
        return;
    }

    // HUMAN mode: Display structured metadata and decoded content
    println!("--- DETAILS ---");
    println!("ID:       {}", idx);
    println!("MIME:     {}", mime);
    println!("SIZE:     {} bytes", val.len());
    println!("---------------");

    if mime.contains("text") || mime.contains("UTF8") {
        println!("{}", String::from_utf8_lossy(&val));
    } else {
        println!("[binary payload: terminal output suppressed]");
        println!("hint: export data via 'y1-clip show {} --raw > output_file'", idx);
    }
    println!("---------------");
}
