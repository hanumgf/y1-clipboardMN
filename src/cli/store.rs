
// src/cli/store.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::env;

/// Read data from stdin, persist it to the database, and synchronize it to the OS clipboard.
pub fn run(args: &[String], mut db: ClipboardDb, verbose: bool) {
    // 1. Resolve target MIME type
    let mime = args.get(2).map(|s| s.as_str()).unwrap_or(DEFAULT_MIME);

    // 2. Ingest stream payload from standard input
    let mut buffer = Vec::new();
    let mut stdin = io::stdin();
    if let Err(e) = stdin.read_to_end(&mut buffer) {
        eprintln!("{}failed to read from stdin: {}", LOG_ERROR, e);
        return;
    }

    // Skip tracking to prevent empty payload ingestion
    if buffer.is_empty() {
        if verbose {
            println!("{}no data received from stdin. skipping store.", LOG_INFO);
        }
        return;
    }

    // 3. Execute atomic database persistence with deduplication filters
    match db.insert_raw(mime, &buffer) {
        Ok(_) => {
            if verbose {
                println!("{}", log_save(mime, buffer.len()));
            }

            // 4. Fetch the primary key identifier of the processed entry
            let meta = db.fetch_metadata(1);
            if let Some(&(real_id, _, _, _, _)) = meta.first() {
                
                // 5. Spawn a background server to immediately broadcast data to the Wayland selection context
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
                            println!("{}system clipboard synchronized.", LOG_INFO);
                        } else if status.is_err() {
                            eprintln!("{}failed to spawn background synchronization process.", LOG_ERROR);
                        }
                    }
                    Err(e) => eprintln!("{}executable path resolution error: {}", LOG_ERROR, e),
                }
            }
        }
        Err(e) => {
            eprintln!("{}failed to write to database: {}", LOG_ERROR, e);
        }
    }
}
