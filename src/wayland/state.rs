
// src/wayland/state.rs

use crate::storage::ClipboardDb;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_protocols::ext::data_control::v1::client::{
    ext_data_control_device_v1::ExtDataControlDeviceV1,
    ext_data_control_manager_v1::ExtDataControlManagerV1,
    ext_data_control_source_v1::ExtDataControlSourceV1,
};
use std::sync::{Arc, Mutex};

pub struct OfferData {
    pub mimes: Arc<Mutex<Vec<String>>>,
}

pub struct WaylandState {
    pub manager: Option<ExtDataControlManagerV1>,
    pub seat: Option<WlSeat>,
    pub device: Option<ExtDataControlDeviceV1>,
    pub db: Option<ClipboardDb>,
    pub verbose: bool,
    pub target_mime: String,
    pub rx_buf: Vec<u8>,
    pub tx_buf: Vec<u8>, 
    pub last_data: Vec<u8>,
    pub is_provider: bool,
    pub selection_received: bool,
    pub current_source: Option<ExtDataControlSourceV1>,
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
            tx_buf: Vec::new(),
            last_data: Vec::new(),
            is_provider: false,
            selection_received: false,
            current_source: None,
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
            tx_buf: Vec::new(),
            last_data: Vec::new(),
            is_provider: false,
            selection_received: false,
            current_source: None,
        }
    }
}
