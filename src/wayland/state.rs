
// src/wayland/state.rs

use crate::storage::ClipboardDb;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_protocols::ext::data_control::v1::client::{
    ext_data_control_device_v1::ExtDataControlDeviceV1,
    ext_data_control_manager_v1::ExtDataControlManagerV1,
};
use std::sync::{Arc, Mutex};

/// Container for MIME types offered by a specific selection.
/// Used to maintain isolation between concurrent data offers.
pub struct OfferData {
    pub mimes: Arc<Mutex<Vec<String>>>,
}

/// Core state architecture for Wayland protocol interaction.
pub struct WaylandState {
    pub manager: Option<ExtDataControlManagerV1>,
    pub seat: Option<WlSeat>,
    pub device: Option<ExtDataControlDeviceV1>,
    pub db: Option<ClipboardDb>,
    pub verbose: bool,
    pub target_mime: String,
    pub rx_buf: Vec<u8>,
    pub last_data: Vec<u8>,
    pub is_provider: bool,
    pub selection_received: bool,
}

impl WaylandState {
    pub fn new_daemon(db: ClipboardDb, verbose: bool) -> Self {
        Self {
            manager: None,
            seat: None,
            device: None,
            db: Some(db),
            verbose,
            target_mime: String::new(),
            rx_buf: Vec::new(),
            last_data: Vec::new(),
            is_provider: false,
            selection_received: false,
        }
    }

    pub fn new_action(target_mime: String, verbose: bool) -> Self {
        Self {
            manager: None,
            seat: None,
            device: None,
            db: None,
            verbose,
            target_mime,
            rx_buf: Vec::new(),
            last_data: Vec::new(),
            is_provider: false,
            selection_received: false,
        }
    }
}
