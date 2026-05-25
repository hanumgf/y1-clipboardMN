
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

/// Command Dispatcher: Safely routes all execution flows to their respective modules.
pub fn handle_command(args: &[String], db: ClipboardDb) {
    // 1. Evaluate argument presence and help requests with the highest priority
    if args.len() < 2 || args.iter().any(|a| a == "-h" || a == "--help") {
        help::print_help();
        return;
    }

    let cmd = args[1].as_str();
    
    // 2. Evaluate verbose logging flag presence
    let verbose = args.iter().any(|a| a == "--verbose" || a == "-v");

    // 3. Execute command routing matrix
    match cmd {
        // --- Public Commands ---
        "daemon"         => daemon::run(db, verbose),
        "list"           => list::run(args, db),
        "search"         => search::run(args, db),
        "show"           => show::run(args, db),
        "copy-to"        => copy_to::run(args, db, verbose),
        "store"          => store::run(args, db, verbose),
        "delete"         => cleaning::delete_run(args, db),
        "wipe"           => cleaning::wipe_run(db),
        "paste-from"     => paste_from::run(args),
        "version" | "-V" => help::print_version(),
        "help"           => help::print_help(),

        // --- Internal Only: Background Egress Serving Server (serve-internal) ---
        // Robustness Optimizations:
        //   - Strict argument pattern validations
        //   - Early drop of active database resources (Critical!)
        //   - Enforced process lifecycle termination paths
        "serve-internal" => {
            // Arguments Layout -> [2]: ID target string, [3]: Verbose boolean string
            if let (Some(id_str), Some(v_str)) = (args.get(2), args.get(3)) {
                let is_verbose = v_str == "true";
                
                if let Ok(real_id) = id_str.parse::<i64>() {
                    // Fetch and load target payload array into memory cache from DB
                    if let Some((mime, val)) = db.get_content_by_id(real_id) {
                        
                        // Resource Robustness:
                        // Disconnect the active SQLite connection before entering the blocking Wayland event loop.
                        // This eliminates the risk of background processes holding persistent storage file locks.
                        drop(db); 
                        
                        // Start serving the payload to the OS layers (Blocks indefinitely within this loop execution)
                        crate::wayland::copy_to_os(&mime, val, is_verbose);
                    } else if is_verbose {
                        eprintln!("{}serve-internal: record ID {} not found.", LOG_ERROR, real_id);
                    }
                }
            }
            
            // Explicitly terminate the active process layout as soon as the serving loop finishes.
            // This prevents background process leakage and zombie runtime states under any circumstance.
            std::process::exit(0);
        }

        // --- Error Handling: Unrecognized Commands ---
        _ => {
            eprintln!("{}'{}' is not a recognized command.", LOG_ERROR, cmd);
            println!("\nSee 'y1-clip --help' for a list of available commands.");
            std::process::exit(1);
        }
    }
}
