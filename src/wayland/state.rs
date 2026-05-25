
// src/wayland/state.rs

use crate::storage::ClipboardDb;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_protocols::ext::data_control::v1::client::{
    ext_data_control_device_v1::ExtDataControlDeviceV1,
    ext_data_control_manager_v1::ExtDataControlManagerV1,
};

/// Core state architecture holding all interaction contexts with the Wayland protocols.
pub struct WaylandState {
    pub manager: Option<ExtDataControlManagerV1>,
    pub seat: Option<WlSeat>,
    pub device: Option<ExtDataControlDeviceV1>,
    pub db: Option<ClipboardDb>,
    pub verbose: bool,
    pub target_mime: String,
    pub offered_mimes: Vec<String>,
    pub rx_buf: Vec<u8>,
    pub last_data: Vec<u8>,
    pub is_provider: bool,
}

impl WaylandState {
    /// Construct a dedicated persistent state architecture for the background monitoring daemon.
    pub fn new_daemon(db: ClipboardDb, verbose: bool) -> Self {
        Self {
            manager: None,
            seat: None,
            device: None,
            db: Some(db),
            verbose,
            target_mime: String::new(),
            offered_mimes: Vec::with_capacity(8),
            rx_buf: Vec::new(),
            last_data: Vec::new(),
            is_provider: false,
        }
    }

    /// Construct a lightweight, short-lived state context for direct CLI execution loops.
    pub fn new_action(target_mime: String, verbose: bool) -> Self {
        Self {
            manager: None,
            seat: None,
            device: None,
            db: None,
            verbose,
            target_mime,
            offered_mimes: Vec::new(),
            rx_buf: Vec::new(),
            last_data: Vec::new(),
            is_provider: false,
        }
    }
}
