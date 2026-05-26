
// src/cli/copy_to.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use crate::cli::utils;
use std::process::{Command, Stdio};
use std::env;

/// Re-broadcast a history entry to the system clipboard and promote it to the top.
pub fn run(args: &[String], db: ClipboardDb, verbose: bool) {
    // Extract display index from positional arguments; ignore flags
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
            eprintln!("{}missing required history entry index.", LOG_ERROR);
            println!("usage: y1-clip copy-to <id>");
            return;
        }
    };

    // Resolve database identifier from the specified display index
    let meta = db.fetch_metadata(MAX_HISTORY);
    let real_id = match meta.get(idx) {
        Some(&(id, _, _, _, _)) => id,
        None => {
            eprintln!("{}index [{}] is outside current history bounds.", LOG_ERROR, idx);
            return;
        }
    };

    // Update entry timestamp to implement 'move to top' logic (MRU)
    if let Err(e) = db.update_timestamp(real_id) {
        eprintln!("{}storage timestamp update failure: {}", LOG_ERROR, e);
        return;
    }

    // Spawn a decoupled background worker to manage the Wayland selection
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

            match status {
                Ok(_) => {
                    if verbose {
                        println!("{}", log_restore(idx));
                    }
                }
                Err(e) => {
                    eprintln!("{}background server spawn failure: {}", LOG_ERROR, e);
                }
            }
        }
        Err(e) => {
            eprintln!("{}executable path resolution error: {}", LOG_ERROR, e);
        }
    }
}
