
// src/cli/copy_to.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use crate::cli::utils::ArgContext;
use std::process::{Command, Stdio};
use std::env;

/// Re-broadcast an entry to the system clipboard using MRU index or database ID.
pub fn run(args: &[String], db: ClipboardDb) {
    let ctx = ArgContext::parse(args);

    if !ctx.unknown_flags.is_empty() || ctx.raw || ctx.full || ctx.force {
        eprintln!("{}command 'copy-to' does not support specified options.", LOG_ERROR);
        return;
    }

    let input_str = match ctx.positionals.first() {
        Some(s) => s,
        None => {
            eprintln!("{}missing required identifier.", LOG_ERROR);
            return;
        }
    };

    let val = match input_str.parse::<i64>() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("{}invalid numerical value: '{}'", LOG_ERROR, input_str);
            return;
        }
    };

    let real_id = if ctx.use_id {
        val
    } else {
        let meta = db.fetch_metadata(MAX_HISTORY);
        match meta.get(val as usize) {
            Some(&(id, ..)) => id,
            None => {
                eprintln!("{}index [{}] is out of bounds.", LOG_ERROR, val);
                return;
            }
        }
    };

    if let Err(e) = db.update_timestamp(real_id) {
        eprintln!("{}storage update failure: {}", LOG_ERROR, e);
        return;
    }

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
                    println!("{}", log_restore(real_id as usize));
                }
            } else {
                eprintln!("{}failed to spawn background synchronization worker.", LOG_ERROR);
            }
        }
        Err(e) => eprintln!("{}executable path resolution error: {}", LOG_ERROR, e),
    }
}
