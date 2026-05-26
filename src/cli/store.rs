
// src/cli/store.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use crate::cli::utils::ArgContext;
use std::io::{self, Read};
use std::process::{Command, Stdio};
use std::env;

/// Ingest data from stdin, persist to database, and synchronize to system clipboard.
pub fn run(args: &[String], mut db: ClipboardDb) {
    let ctx = ArgContext::parse(args);

    // Strict validation: 'store' only permits --verbose/-v
    if !ctx.unknown_flags.is_empty() || ctx.raw || ctx.full || ctx.force {
        eprintln!("{}command 'store' does not support specified options.", LOG_ERROR);
        return;
    }

    // Arity enforcement: ensure no more than one positional (MIME) is provided
    if ctx.positionals.len() > 1 {
        eprintln!("{}command 'store' accepts at most one MIME type argument.", LOG_ERROR);
        return;
    }

    // Resolve target MIME from positional arguments or use system default
    let mime = ctx.positionals.first().map(|s| s.as_str()).unwrap_or(DEFAULT_MIME);

    // Read payload from standard input stream until EOF
    let mut buffer = Vec::new();
    if io::stdin().read_to_end(&mut buffer).is_err() {
        eprintln!("{}standard input stream read failure.", LOG_ERROR);
        return;
    }

    // Terminate processing for null or empty payloads
    if buffer.is_empty() {
        if ctx.verbose {
            println!("{}null payload detected; skipping storage.", LOG_INFO);
        }
        return;
    }

    // Execute atomic persistence with internal deduplication
    match db.insert_raw(mime, &buffer) {
        Ok(_) => {
            if ctx.verbose {
                println!("{}", log_save(mime, buffer.len()));
            }

            // Retrieve the record ID for the newly committed entry
            let meta = db.fetch_metadata(1);
            if let Some(&(real_id, _, _, _, _)) = meta.first() {
                
                // Spawn the background worker to claim Wayland selection ownership
                match env::current_exe() {
                    Ok(exe) => {
                        let status = Command::new(exe)
                            .arg("serve-internal")
                            .arg(real_id.to_string())
                            .arg(ctx.verbose.to_string())
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn();

                        if status.is_ok() {
                            if ctx.verbose {
                                println!("{}background synchronization initialized.", LOG_INFO);
                            }
                        } else {
                            eprintln!("{}failed to spawn background synchronization process.", LOG_ERROR);
                        }
                    }
                    Err(e) => eprintln!("{}binary path resolution error: {}", LOG_ERROR, e),
                }
            }
        }
        Err(e) => eprintln!("{}storage transaction failure: {}", LOG_ERROR, e),
    }
}
