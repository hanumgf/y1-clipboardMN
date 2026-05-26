
// src/cli/mod.rs

mod daemon;
mod list;
mod show;
mod copy_to;
mod paste_from;
mod store;
mod search;
mod cleaning;
mod help;
pub mod formatter;
mod utils;

use crate::storage::ClipboardDb;
use crate::core::constants::*;

/// Primary command router responsible for lifecycle orchestration and resource isolation.
pub fn handle_command(args: &[String], db: ClipboardDb) {
    // 1. Immediate resolution of global requests and empty inputs
    if args.len() < 2 || utils::has_flag(args, "--help", "-h") {
        help::print_help();
        return;
    }

    if utils::has_flag(args, "--version", "-V") {
        help::print_version();
        return;
    }

    // 2. Syntax enforcement: ensure positional command precedes options
    let cmd = args[1].as_str();
    if utils::is_option(cmd) {
        eprintln!("{}invalid command format: '{}'", LOG_ERROR, cmd);
        println!("usage: y1-clip <command> [options]");
        std::process::exit(1);
    }

    // 3. Command Routing Matrix
    // Each subcommand manages its own internal validation context via ArgContext.
    match cmd {
        // --- System Operations ---
        "daemon"     => daemon::run(args, db),
        "list"       => list::run(args, db),
        "search"     => search::run(args, db),
        "show"       => show::run(args, db),
        "copy-to"    => copy_to::run(args, db),
        "store"      => store::run(args, db),

        // --- Database Management ---
        "delete"     => cleaning::delete_run(args, db),
        "wipe"       => cleaning::wipe_run(args, db),

        // --- Utility Access ---
        "paste-from" => paste_from::run(args),
        "help"       => help::print_help(),
        "version"    => help::print_version(),

        // --- Low-level Internal: Selection Service Worker ---
        // Decoupled background process specifically for serving Wayland data egress.
        "serve-internal" => {
            // Expected indices: [2] record ID (i64), [3] verbose flag (string bool)
            if args.len() >= 4 {
                let id_str = &args[2];
                let is_verbose = args[3] == "true";

                if let Ok(real_id) = id_str.parse::<i64>() {
                    // Cache record payload into memory before entering the blocking loop
                    if let Some((mime, val)) = db.get_content_by_id(real_id) {
                        // Resource Safeguard: Close database handles to release SQLite file locks.
                        // Prevents background workers from obstructing concurrent storage operations.
                        drop(db);

                        // Transfer control to the persistent Wayland egress serving loop
                        crate::wayland::copy_to_os(&mime, val, is_verbose);
                    }
                }
            }
            // Enforce process termination immediately upon service completion or sync failure.
            std::process::exit(0);
        }

        // --- Error State Handler ---
        _ => {
            eprintln!("{}unrecognized command identifier: '{}'", LOG_ERROR, cmd);
            println!("consult 'y1-clip help' for valid operations.");
            std::process::exit(1);
        }
    }
}
