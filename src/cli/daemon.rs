
// src/cli/daemon.rs

use crate::storage::ClipboardDb;
use crate::daemon;
use crate::core::constants::*;

/// Start the clipboard monitoring daemon.
pub fn run(db: ClipboardDb, verbose: bool) {
    // Explicitly notify the user that the daemon has started.
    // This is displayed regardless of the verbose flag to confirm the process is alive.
    println!("{}{}", LOG_INFO, MSG_DAEMON_START);
    
    if verbose {
        println!("{}verbose logging is enabled.", LOG_INFO);
    }

    // Hand over control to the core monitoring logic (src/daemon/mod.rs).
    // This function typically blocks and runs indefinitely until terminated via Ctrl+C.
    daemon::start_daemon(db, verbose);

    // Reaching this point implies an unexpected termination of the monitoring loop.
    // (Currently, this block handles cases where the event loop breaks)
    eprintln!("{}{}", LOG_ERROR, MSG_DAEMON_STOP);
}
