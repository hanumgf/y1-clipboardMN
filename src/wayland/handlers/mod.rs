
// src/wayland/handlers/mod.rs

pub mod data_control;
pub mod seat;

use wayland_client::{protocol::wl_registry, Connection, Dispatch, QueueHandle};
use wayland_protocols::ext::data_control::v1::client::ext_data_control_manager_v1::ExtDataControlManagerV1;
use wayland_client::protocol::wl_seat::WlSeat;
use crate::wayland::state::WaylandState;
use crate::core::constants::*;

impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        reg: &wl_registry::WlRegistry,
        ev: wl_registry::Event,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { name, interface, version } = ev {
            // Bind clipboard management protocol
            // Our implementation targeting V1 explicitly handles bindings using a strict
            // protocol constraint to maintain type safety even if the compositor supports V2+.
            if interface == INTERFACE_MANAGER {
                let manager = reg.bind::<ExtDataControlManagerV1, _, _>(name, 1, qh, ());
                state.manager = Some(manager);
                
                if state.verbose {
                    println!("{}", log_protocol_bound(INTERFACE_MANAGER));
                }
            }

            // Bind seat interface (aggregation of input devices)
            // In environments with multiple seats, the first discovered seat instance
            // (typically seat0) is pinned and tracked. High backward compatibility is
            // expected; hence the reported interface version is honored.
            if interface == INTERFACE_SEAT && state.seat.is_none() {
                let seat = reg.bind::<WlSeat, _, _>(name, version, qh, ());
                state.seat = Some(seat);

                if state.verbose {
                    println!("{}", log_protocol_bound(INTERFACE_SEAT));
                }
            }
        }
    }
}
