
// src/cli/copy_to.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use std::process::{Command, Stdio};
use std::env;

/// Restore an item with the specified ID from history to the OS clipboard.
pub fn run(args: &[String], db: ClipboardDb, verbose: bool) {
    // 1. Extract index from arguments
    let id_str = match args.get(2) {
        Some(s) => s,
        None => {
            eprintln!("{}no ID provided.", LOG_ERROR);
            println!("usage: y1-clip copy-to <id>");
            return;
        }
    };

    let idx = match id_str.parse::<usize>() {
        Ok(i) => i,
        Err(_) => {
            eprintln!("{}'{}' is not a valid numerical ID.", LOG_ERROR, id_str);
            return;
        }
    };

    // 2. Identify the real ID from metadata
    let meta = db.fetch_metadata(MAX_HISTORY);
    let item = match meta.get(idx) {
        Some(it) => it,
        None => {
            eprintln!("{}entry with ID [{}] not found.", LOG_ERROR, idx);
            return;
        }
    };

    let real_id = item.0; // id
    let _mime = &item.2;  // mime (retained for debugging purposes)

    // 3. Update the database timestamp (moves the item to the top of the history stack)
    if let Err(e) = db.update_timestamp(real_id) {
        eprintln!("{}failed to update record timestamp: {}", LOG_ERROR, e);
        return;
    }

    // 4. Restart itself as a background process (serve-internal mode)
    // Redirect stdout/stderr to null to ensure immunity from parent process termination.
    match env::current_exe() {
        Ok(exe) => {
            let res = Command::new(exe)
                .arg("serve-internal")
                .arg(real_id.to_string())
                .arg(verbose.to_string())
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            match res {
                Ok(_) => {
                    if verbose {
                        println!("{}", log_restore(idx));
                    }
                }
                Err(e) => {
                    eprintln!("{}failed to spawn background server: {}", LOG_ERROR, e);
                }
            }
        }
        Err(e) => {
            eprintln!("{}could not determine current executable path: {}", LOG_ERROR, e);
        }
    }
}
