
// src/cli/copy_to.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use crate::cli::utils::ArgContext;
use std::process::{Command, Stdio};
use std::env;

/// Re-broadcast a history entry to the system clipboard and promote it to the top.
pub fn run(args: &[String], db: ClipboardDb) {
    let ctx = ArgContext::parse(args);

    // Strict validation: 'copy-to' accepts only --verbose/-v and exactly 1 positional index.
    if !ctx.unknown_flags.is_empty() || ctx.raw || ctx.full || ctx.force {
        eprintln!("{}command 'copy-to' does not support specified options.", LOG_ERROR);
        return;
    }

    // Arity enforcement: ensure only the target index is provided.
    if ctx.positionals.len() != 1 {
        eprintln!("{}command 'copy-to' requires exactly one history index.", LOG_ERROR);
        println!("usage: y1-clip copy-to <id>");
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

    // Map display index to persistent database identifier
    let meta = db.fetch_metadata(MAX_HISTORY);
    let real_id = match meta.get(idx) {
        Some(&(id, _, _, _, _)) => id,
        None => {
            eprintln!("{}index [{}] is out of bounds.", LOG_ERROR, idx);
            return;
        }
    };

    // Update entry timestamp to implement Most Recently Used (MRU) logic
    if let Err(e) = db.update_timestamp(real_id) {
        eprintln!("{}storage update failure: {}", LOG_ERROR, e);
        return;
    }

    // Spawn an independent background process to serve Wayland selection requests
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
                    println!("{}", log_restore(idx));
                }
            } else {
                eprintln!("{}failed to spawn background synchronization worker.", LOG_ERROR);
            }
        }
        Err(e) => eprintln!("{}executable path resolution error: {}", LOG_ERROR, e),
    }
}
