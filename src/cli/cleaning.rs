
// src/cli/cleaning.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use crate::cli::utils;
use std::io::{self, Write};

/// Remove a specific history entry by its display index.
pub fn delete_run(args: &[String], db: ClipboardDb) {
    // Isolate positional index argument from flags
    let id_arg = args.get(2).filter(|s| !utils::is_option(s));

    let idx = match id_arg {
        Some(s) => match s.parse::<usize>() {
            Ok(i) => i,
            Err(_) => {
                eprintln!("{}'{}' is not a valid numerical index.", LOG_ERROR, s);
                return;
            }
        },
        None => {
            eprintln!("{}missing required entry index.", LOG_ERROR);
            println!("usage: y1-clip delete <id>");
            return;
        }
    };

    // Execute physical deletion from persistent storage
    match db.delete_by_index(idx) {
        Ok(true) => {
            println!("{}successfully removed entry at index [{}].", LOG_INFO, idx);
        }
        Ok(false) => {
            eprintln!("{}index [{}] is outside current history scope.", LOG_ERROR, idx);
        }
        Err(e) => {
            eprintln!("{}database operation failure: {}", LOG_ERROR, e);
        }
    }
}

/// Purge all history entries and perform database vacuuming.
pub fn wipe_run(db: ClipboardDb, args: &[String]) {
    // Check for force flag to bypass interactive confirmation
    let force = utils::has_flag(args, "--force", "-f");

    if !force {
        print!("purge all history and optimize storage? [y/N]: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("{}failed to read interactive input.", LOG_ERROR);
            return;
        }

        let response = input.trim().to_lowercase();
        if response != "y" && response != "yes" {
            println!("{}operation aborted by user.", LOG_INFO);
            return;
        }
    }

    // Execute full database reset and structural optimization
    match db.wipe() {
        Ok(_) => {
            println!("{}history purged and database optimized.", LOG_INFO);
        }
        Err(e) => {
            eprintln!("{}wipe operation failure: {}", LOG_ERROR, e);
        }
    }
}
