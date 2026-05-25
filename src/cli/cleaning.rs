
// src/cli/cleaning.rs

use crate::storage::ClipboardDb;
use std::io::{self, Write};

/// Delete the history entry with the specified ID.
pub fn delete_run(args: &[String], db: ClipboardDb) {
    // Validate argument presence
    let id_str = match args.get(2) {
        Some(s) => s,
        None => {
            eprintln!("error: no ID provided.");
            println!("usage: y1-clip delete <id>");
            return;
        }
    };

    // Parse the ID string to usize
    let idx = match id_str.parse::<usize>() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("error: '{}' is not a valid numerical ID.", id_str);
            return;
        }
    };

    // Perform database deletion
    match db.delete_by_index(idx) {
        Ok(true) => {
            println!("info: successfully deleted entry at ID [{}]", idx);
        }
        Ok(false) => {
            eprintln!("error: entry with ID [{}] not found.", idx);
        }
        Err(e) => {
            eprintln!("error: database failure: {}", e);
        }
    }
}

/// Wipe all history and optimize the database storage.
pub fn wipe_run(db: ClipboardDb) {
    // Prompt for user confirmation
    print!("Proceed to clear all history and optimize storage? [y/N]: ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        eprintln!("error: failed to read user input.");
        return;
    }

    // Check for positive confirmation (y or yes)
    let response = input.trim().to_lowercase();
    if response == "y" || response == "yes" {
        match db.wipe() {
            Ok(_) => {
                println!("info: all history has been wiped and database optimized.");
            }
            Err(e) => {
                eprintln!("error: failed to wipe database: {}", e);
            }
        }
    } else {
        println!("info: operation canceled.");
    }
}
