
// src/cli/cleaning.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use crate::cli::utils::ArgContext;
use std::io::{self, Write};

/// Remove a specific history record by its numerical index.
pub fn delete_run(args: &[String], db: ClipboardDb) {
    let ctx = ArgContext::parse(args);

    // Strict validation: 'delete' accepts NO flags and exactly 1 positional argument.
    if !ctx.unknown_flags.is_empty() || ctx.raw || ctx.full || ctx.force || ctx.verbose {
        eprintln!("{}command 'delete' does not support options.", LOG_ERROR);
        return;
    }

    if ctx.positionals.len() != 1 {
        eprintln!("{}command 'delete' requires exactly one index.", LOG_ERROR);
        println!("usage: y1-clip delete <id>");
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

    match db.delete_by_index(idx) {
        Ok(true) => println!("{}successfully removed entry [{}].", LOG_INFO, idx),
        Ok(false) => eprintln!("{}index [{}] is out of bounds.", LOG_ERROR, idx),
        Err(e) => eprintln!("{}storage transaction failure: {}", LOG_ERROR, e),
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
