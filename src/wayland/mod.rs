
// src/wayland/mod.rs

pub mod state;
pub mod handlers;

use wayland_client::{Connection, EventQueue};
pub use self::state::WaylandState;
use crate::core::constants::*;

/// Establish a connection to the Wayland compositor.
pub fn create_connection() -> (Connection, EventQueue<WaylandState>) {
    let conn = Connection::connect_to_env()
        .expect(MSG_WAYLAND_CONN_FAIL);
    let event_queue = conn.new_event_queue();
    (conn, event_queue)
}

/// Extract data from the current OS clipboard (wl-paste equivalent).
/// Optimized to return immediately once data is acquired.
pub fn paste_from_os(mime: &str) -> Vec<u8> {
    let (conn, mut event_queue) = create_connection();
    let qh = event_queue.handle();
    let _registry = conn.display().get_registry(&qh, ());

    let mut state = WaylandState::new_action(mime.to_string(), false);

    // 1. Initial sync to bind protocols (Manager & Seat)
    let _ = event_queue.roundtrip(&mut state);

    if let (Some(manager), Some(seat)) = (&state.manager, &state.seat) {
        state.device = Some(manager.get_data_device(seat, &qh, ()));
        let _ = conn.flush();
    } else {
        return Vec::new();
    }

    // 2. Data acquisition loop
    // Robustness: Continue dispatching until rx_buf is populated or retries exhausted.
    let mut retry_count = 0;
    while !state.selection_received && retry_count < WAYLAND_SYNC_RETRIES {
        if event_queue.blocking_dispatch(&mut state).is_err() {
            break;
        }
        retry_count += 1;
    }

    state.rx_buf
}

/// Serve data to the system clipboard (wl-copy equivalent).
/// Optimized to minimize memory footprint during large data transfers.
pub fn copy_to_os(mime: &str, data: Vec<u8>, verbose: bool) {
    let (conn, mut event_queue) = create_connection();
    let qh = event_queue.handle();
    let _registry = conn.display().get_registry(&qh, ());

    // 🚀 Optimization: Move 'data' instead of cloning where possible
    let mut state = WaylandState::new_action(mime.to_string(), verbose);
    state.rx_buf = data; 
    state.is_provider = true; 

    // Protocol synchronization
    let _ = event_queue.roundtrip(&mut state);

    if let (Some(manager), Some(seat)) = (&state.manager, &state.seat) {
        let source = manager.create_data_source(&qh, ());
        
        // Advertise primary MIME
        source.offer(mime.to_string());
        
        // Re-introduce fallback Mimes for text to ensure legacy compatibility.
        if mime.contains("text") {
            for alt in TEXT_MIME_ALTS {
                if *alt != mime {
                    source.offer(alt.to_string());
                }
            }
        }

        let device = manager.get_data_device(seat, &qh, ());
        device.set_selection(Some(&source));
        
        // Ensure the selection claim is sent to the compositor immediately.
        let _ = conn.flush();

        // Main serving loop. Exits on 'Cancelled' event in handlers.
        loop {
            if event_queue.blocking_dispatch(&mut state).is_err() {
                break; 
            }
        }
    } else {
        eprintln!("{}failed to secure required Wayland protocols.", LOG_ERROR);
    }
}
