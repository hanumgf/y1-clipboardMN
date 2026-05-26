
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

/// Primary entry point for routing CLI commands to specialized modules.
pub fn handle_command(args: &[String], db: ClipboardDb) {
    // 1. Intercept help or version requests and empty argument vectors
    if args.len() < 2 || utils::has_flag(args, "--help", "-h") {
        help::print_help();
        return;
    }
    
    if utils::has_flag(args, "--version", "-V") {
        help::print_version();
        return;
    }

    // 2. Extract command identifier; reject if formatted as an option (e.g., --list)
    let cmd = args[1].as_str();
    if utils::is_option(cmd) {
        eprintln!("{}invalid command format: '{}'", LOG_ERROR, cmd);
        println!("run 'y1-clip --help' for valid command syntax.");
        std::process::exit(1);
    }

    // 3. Evaluate global operational flags
    let verbose = utils::has_flag(args, "--verbose", "-v");

    // 4. Command routing matrix
    match cmd {
        "daemon"     => daemon::run(db, verbose),
        "list"       => list::run(args, db),
        "search"     => search::run(args, db),
        "show"       => show::run(args, db),
        "copy-to"    => copy_to::run(args, db, verbose),
        "store"      => store::run(args, db, verbose),
        "delete"     => cleaning::delete_run(args, db),
        "wipe"       => cleaning::wipe_run(db, args),
        "paste-from" => paste_from::run(args),
        "help"       => help::print_help(),
        "version"    => help::print_version(),

        // --- Internal: Background selection provider server ---
        "serve-internal" => {
            if let (Some(id_str), Some(v_str)) = (args.get(2), args.get(3)) {
                let is_verbose = v_str == "true";
                if let Ok(real_id) = id_str.parse::<i64>() {
                    if let Some((mime, val)) = db.get_content_by_id(real_id) {
                        // Drop DB handle to release file locks before entering blocking loop
                        drop(db); 
                        crate::wayland::copy_to_os(&mime, val, is_verbose);
                    }
                }
            }
            std::process::exit(0);
        }

        // --- Unknown Command Fallback ---
        _ => {
            eprintln!("{}unrecognized command: '{}'", LOG_ERROR, cmd);
            println!("run 'y1-clip --help' for a list of available commands.");
            std::process::exit(1);
        }
    }
}
