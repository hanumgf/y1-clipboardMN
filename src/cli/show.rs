
// src/cli/show.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use crate::cli::utils::ArgContext;
use std::io::{self, Write};

/// Detailed inspection of metadata and payload with strict argument validation.
pub fn run(args: &[String], db: ClipboardDb) {
    let ctx = ArgContext::parse(args);

    // Strict validation: 'show' only permits --raw/-R and --verbose/-v
    if !ctx.unknown_flags.is_empty() || ctx.full || ctx.force {
        eprintln!("{}command 'show' does not support specified options.", LOG_ERROR);
        return;
    }

    // Arity enforcement: ensure exactly one positional index is supplied
    if ctx.positionals.is_empty() {
        eprintln!("{}missing required history entry index.", LOG_ERROR);
        println!("usage: y1-clip show <id> [--raw | -R]");
        return;
    }

    if ctx.positionals.len() > 1 {
        eprintln!("{}command 'show' accepts only one index argument.", LOG_ERROR);
        return;
    }

    let idx_str = &ctx.positionals[0];
    let idx = match idx_str.parse::<usize>() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("{}invalid numerical index: '{}'", LOG_ERROR, idx_str);
            return;
        }
    };

    // Resolve internal database ID from the user-provided display index
    let meta = db.fetch_metadata(MAX_HISTORY);
    let real_id = match meta.get(idx) {
        Some(&(id, _, _, _, _)) => id,
        None => {
            eprintln!("{}index [{}] is outside current history bounds.", LOG_ERROR, idx);
            return;
        }
    };

    // Extract the full binary or text payload from storage
    let (mime, val) = match db.get_content_by_id(real_id) {
        Some(res) => res,
        None => {
            eprintln!("{}failed to fetch payload for record [{}].", LOG_ERROR, idx);
            return;
        }
    };

    if ctx.raw {
        // RAW mode: Directly stream unmodified bytes to stdout
        let mut stdout = io::stdout();
        let _ = stdout.write_all(&val);
        let _ = stdout.flush();
        return;
    }

    // HUMAN mode: Structure metadata and provide decoded string preview
    println!("--- DETAILS ---");
    println!("ID:       {}", idx);
    println!("MIME:     {}", mime);
    println!("SIZE:     {} bytes", val.len());
    println!("---------------");

    if mime.contains("text") || mime.contains("UTF8") {
        println!("{}", String::from_utf8_lossy(&val));
    } else {
        println!("[binary payload: terminal display suppressed]");
        println!("hint: utilize '--raw' to pipe data to a file.");
    }
    println!("---------------");
}
