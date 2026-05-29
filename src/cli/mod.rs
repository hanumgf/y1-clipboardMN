
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

        // --- Error State Handler ---
        _ => {
            eprintln!("{}unrecognized command identifier: '{}'", LOG_ERROR, cmd);
            println!("consult 'y1-clip help' for valid operations.");
            std::process::exit(1);
        }
    }
}
