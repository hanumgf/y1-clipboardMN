
// src/cli/cleaning.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use crate::cli::utils::ArgContext;
use std::io::{self, Write};

/// Remove a specific history record by its numerical index.
pub fn delete_run(args: &[String], db: ClipboardDb) {
    let ctx = ArgContext::parse(args);

    if !ctx.unknown_flags.is_empty() || ctx.raw || ctx.full || ctx.force || ctx.verbose {
        eprintln!("{}command 'delete' does not support options.", LOG_ERROR);
        return;
    }

    let idx = match ctx.positionals.first() {
        Some(s) => match s.parse::<usize>() {
            Ok(i) => i,
            Err(_) => {
                eprintln!("{}invalid numerical index: '{}'", LOG_ERROR, s);
                return;
            }
        },
        None => {
            eprintln!("{}missing required history index.", LOG_ERROR);
            return;
        }
    };

    // 1. Fetch current metadata state to map index to real_id
    let meta = db.fetch_metadata(MAX_HISTORY);
    
    // 2. Safely resolve and execute deletion by persistent ID
    match meta.get(idx) {
        Some(&(real_id, ..)) => {
            match db.delete_by_id(real_id) {
                Ok(true) => println!("{}removed entry [{}].", LOG_INFO, idx),
                Ok(false) => eprintln!("{}failed to remove record with ID {}.", LOG_ERROR, real_id),
                Err(e) => eprintln!("{}storage transaction failure: {}", LOG_ERROR, e),
            }
        }
        None => {
            eprintln!("{}index [{}] is out of bounds.", LOG_ERROR, idx);
        }
    }
}

/// Purge the entire database and optimize file structure.
pub fn wipe_run(args: &[String], db: ClipboardDb) {
    let ctx = ArgContext::parse(args);

    // Strict validation: 'wipe' accepts ONLY --force/-f and 0 positional arguments.
    if !ctx.unknown_flags.is_empty() || ctx.raw || ctx.full || ctx.verbose {
        eprintln!("{}command 'wipe' does not support specified options.", LOG_ERROR);
        return;
    }

    if !ctx.positionals.is_empty() {
        eprintln!("{}command 'wipe' does not accept positional arguments.", LOG_ERROR);
        return;
    }

    if !ctx.force {
        print!("confirm database purge? [y/N]: ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("{}input stream read failure.", LOG_ERROR);
            return;
        }

        let res = input.trim().to_lowercase();
        if res != "y" && res != "yes" {
            println!("{}wipe operation aborted.", LOG_INFO);
            return;
        }
    }

    match db.wipe() {
        Ok(_) => println!("{}storage purged and optimized (VACUUM completed).", LOG_INFO),
        Err(e) => eprintln!("{}database reset failure: {}", LOG_ERROR, e),
    }
}
