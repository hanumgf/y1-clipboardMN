
// src/wayland/mod.rs

pub mod state;
pub mod handlers;

use wayland_client::{Connection, EventQueue};
pub use self::state::WaylandState;
use crate::core::constants::*;

pub fn create_connection() -> (Connection, EventQueue<WaylandState>) {
    let conn = Connection::connect_to_env().expect(MSG_WAYLAND_CONN_FAIL);
    let event_queue = conn.new_event_queue();
    (conn, event_queue)
}

pub fn paste_from_os(mime: &str) -> Vec<u8> {
    let (conn, mut event_queue) = create_connection();
    let qh = event_queue.handle();
    let _registry = conn.display().get_registry(&qh, ());

    let mut state = WaylandState::new_action(mime.to_string(), false);
    let _ = event_queue.roundtrip(&mut state);

    if let (Some(manager), Some(seat)) = (&state.manager, &state.seat) {
        state.device = Some(manager.get_data_device(seat, &qh, ()));
        let _ = conn.flush();
    } else {
        return Vec::new();
    }

    let mut retry_count = 0;
    while !state.selection_received && retry_count < WAYLAND_SYNC_RETRIES {
        if event_queue.blocking_dispatch(&mut state).is_err() { break; }
        retry_count += 1;
    }

    state.rx_buf
}
