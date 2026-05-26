
// src/cli/store.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use crate::cli::utils;
use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::env;

/// Ingest data from stdin, persist to database, and broadcast to system clipboard.
pub fn run(args: &[String], mut db: ClipboardDb, verbose: bool) {
    // Extract MIME type from positional arguments; ignore if it follows option format
    let mime = args.get(2)
        .filter(|s| !utils::is_option(s))
        .map(|s| s.as_str())
        .unwrap_or(DEFAULT_MIME);

    // Read payload from standard input stream
    let mut buffer = Vec::new();
    if io::stdin().read_to_end(&mut buffer).is_err() {
        eprintln!("{}failed to read payload from standard input.", LOG_ERROR);
        return;
    }

    // Skip processing for null/empty payloads
    if buffer.is_empty() {
        if verbose {
            println!("{}empty stream detected. skipping persistence.", LOG_INFO);
        }
        return;
    }

    // Execute atomic write to database with internal deduplication
    match db.insert_raw(mime, &buffer) {
        Ok(_) => {
            if verbose {
                println!("{}", log_save(mime, buffer.len()));
            }

            // Obtain the record identifier for the entry just processed
            let meta = db.fetch_metadata(1);
            if let Some(&(real_id, _, _, _, _)) = meta.first() {
                
                // Spawn the background server to claim Wayland clipboard ownership
                match env::current_exe() {
                    Ok(exe) => {
                        let status = Command::new(exe)
                            .arg("serve-internal")
                            .arg(real_id.to_string())
                            .arg(verbose.to_string())
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn();
                        
                        if status.is_ok() && verbose {
                            println!("{}system clipboard synchronization initialized.", LOG_INFO);
                        } else if status.is_err() {
                            eprintln!("{}failed to spawn background synchronization worker.", LOG_ERROR);
                        }
                    }
                    Err(e) => eprintln!("{}binary path resolution error: {}", LOG_ERROR, e),
                }
            }
        }
        Err(e) => {
            eprintln!("{}database transaction failure: {}", LOG_ERROR, e);
        }
    }
}
