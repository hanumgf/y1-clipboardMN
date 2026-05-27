
// src/daemon/mod.rs

use crate::storage::ClipboardDb;
use crate::wayland;
use crate::wayland::state::WaylandState;
use crate::core::constants::*;

/// Start the clipboard monitoring daemon.
/// Robustness: Restores the state from the database at startup to resume tracking seamlessly.
pub fn start_daemon(db: ClipboardDb, verbose: bool) {
    // 1. Establish the Wayland connection context
    let (conn, mut event_queue) = wayland::create_connection();
    let qh = event_queue.handle();
    let _registry = conn.display().get_registry(&qh, ());

    // 2. Prevent redundant storage cycles during initialization
    // Stability: Prefetches the most recently stored record from the database.
    // This avoids immediately duplicating the current system clipboard right after daemon startup.
    let last_stored = db.get_latest_data().unwrap_or_default();

    // 3. Initialize the core state architecture via the designated builder
    let mut state = WaylandState::new_daemon(db, verbose);
    state.last_data = last_stored;
    state.target_mime = DEFAULT_MIME.to_string();

    // 4. Perform initial protocol and seat alignment
    // Robustness: Synchronizes boundaries sequentially until all required interface managers are bound.
    for _ in 0..WAYLAND_SYNC_RETRIES {
        if event_queue.blocking_dispatch(&mut state).is_err() {
            break;
        }
        if state.manager.is_some() && state.seat.is_some() {
            break;
        }
    }

    // 5. Bind the physical clipboard data device interface
    if let (Some(manager), Some(seat)) = (&state.manager, &state.seat) {
        state.device = Some(manager.get_data_device(seat, &qh, ()));
        let _ = conn.flush();
    } else {
        eprintln!("{}critical: required protocols not supported by compositor.", LOG_ERROR);
        return;
    }

    // 6. Execute the primary event polling loop with signal awareness
    // Stability: Terminates gracefully when a system signal is caught or connection is lost.
    while !crate::core::is_exiting() {
        match event_queue.blocking_dispatch(&mut state) {
            Ok(_) => {
                // Background tasks are handled by Dispatch implementations.
            }
            Err(e) => {
                // Log error unless it was caused by a deliberate exit signal
                if !crate::core::is_exiting() {
                    eprintln!("{}wayland connection lost: {}", LOG_ERROR, e);
                }
                break; 
            }
        }
    }
    
    // Explicit cleanup happens here as 'state' and 'db' are dropped
    if verbose {
        println!("{}shutting down daemon safely.", LOG_INFO);
    }
}
