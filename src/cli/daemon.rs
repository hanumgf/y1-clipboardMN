
// src/cli/daemon.rs

use crate::storage::ClipboardDb;
use crate::daemon;
use crate::core::constants::*;

/// Initialize and execute the background monitoring service.
pub fn run(db: ClipboardDb, verbose: bool) {
    // Notify the operator that the daemon initialization has commenced
    println!("{}{}", LOG_INFO, MSG_DAEMON_START);
    
    if verbose {
        println!("{}extended event logging is active.", LOG_INFO);
    }

    // Transfer execution to the core daemon logic (src/daemon/mod.rs)
    // This call is blocking and monitors Wayland events until an interrupt or error occurs.
    daemon::start_daemon(db, verbose);

    // Termination at this point indicates the internal event loop has collapsed
    eprintln!("{}{}", LOG_ERROR, MSG_DAEMON_STOP);
}
