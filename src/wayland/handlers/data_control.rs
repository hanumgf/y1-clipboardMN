
// src/wayland/handlers/data_control.rs

use wayland_client::{Dispatch, Connection, QueueHandle};
use wayland_protocols::ext::data_control::v1::client::{
    ext_data_control_device_v1::{self, ExtDataControlDeviceV1},
    ext_data_control_manager_v1::{self, ExtDataControlManagerV1},
    ext_data_control_offer_v1::{self, ExtDataControlOfferV1},
    ext_data_control_source_v1::{self, ExtDataControlSourceV1},
};
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::{Read, Write};
use std::os::fd::{AsFd, FromRawFd, OwnedFd};
use crate::wayland::state::WaylandState;
use crate::core::constants::*;

impl Dispatch<ExtDataControlManagerV1, ()> for WaylandState {
    fn event(_: &mut Self, _: &ExtDataControlManagerV1, _: ext_data_control_manager_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

impl Dispatch<ExtDataControlOfferV1, ()> for WaylandState {
    fn event(state: &mut Self, _: &ExtDataControlOfferV1, ev: ext_data_control_offer_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        if let ext_data_control_offer_v1::Event::Offer { mime_type } = ev {
            if !state.offered_mimes.contains(&mime_type) {
                state.offered_mimes.push(mime_type);
            }
        }
    }
}

impl Dispatch<ExtDataControlDeviceV1, ()> for WaylandState {
    fn event(state: &mut Self, _: &ExtDataControlDeviceV1, ev: ext_data_control_device_v1::Event, _: &(), conn: &Connection, _: &QueueHandle<Self>) {
        if let ext_data_control_device_v1::Event::Selection { id: Some(offer) } = ev {
            if state.is_provider { return; }

            // Performance: Optimize MIME selection logic
            let priority = ["image/png", "image/jpeg", "text/plain;charset=utf-8", "text/plain"];
            let mime_to_get = priority.iter()
                .find(|&&p| state.offered_mimes.iter().any(|m| m == p))
                .map(|&s| s.to_string())
                .or_else(|| state.offered_mimes.first().cloned())
                .unwrap_or_else(|| DEFAULT_MIME.to_string());

            state.offered_mimes.clear();

            let mut fds = [0; 2];
            if unsafe { libc::pipe(fds.as_mut_ptr()) } < 0 { return; }
            let read_file = unsafe { std::fs::File::from_raw_fd(fds[0]) };
            let write_fd = unsafe { OwnedFd::from_raw_fd(fds[1]) };

            offer.receive(mime_to_get.clone(), write_fd.as_fd());
            drop(write_fd); 
            let _ = conn.flush();

            if let Some(db_path) = state.db.as_ref().map(|db| db.path.clone()) {
                let is_verbose = state.verbose;
                
                // Daemon Mode: Async ingestion
                std::thread::spawn(move || {
                    // Performance: Pre-allocate buffer to reduce re-allocations for typical payloads
                    let mut buf = Vec::with_capacity(4096); 
                    let mut reader = read_file.take(268435456); // 256MB limit
                    
                    if reader.read_to_end(&mut buf).is_err() || buf.is_empty() { return; }

                    if let Ok(db_conn) = rusqlite::Connection::open(&db_path) {
                        db_conn.busy_timeout(std::time::Duration::from_millis(SQLITE_TIMEOUT_MS)).ok();
                        
                        let hash = format!("{:x}", md5::compute(&buf));
                        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64;

                        let existing: Option<i64> = db_conn.query_row(
                            "SELECT id FROM clipboard WHERE hash = ?1 LIMIT 1",
                            rusqlite::params![hash], |row| row.get(0)
                        ).ok();

                        if let Some(id) = existing {
                            let _ = db_conn.execute("UPDATE clipboard SET timestamp = ?1 WHERE id = ?2", rusqlite::params![ts, id]);
                        } else {
                            let preview = if mime_to_get.contains("text") {
                                let s = String::from_utf8_lossy(&buf);
                                Some(s.chars().take(PREVIEW_CHARS).collect::<String>().replace('\n', " "))
                            } else { None };

                            let res = db_conn.execute(
                                "INSERT INTO clipboard (timestamp, mime, size, preview, content, hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                                rusqlite::params![ts, mime_to_get.clone(), buf.len() as i64, preview, buf, hash],
                            );
                            if res.is_ok() && is_verbose {
                                println!("{}", log_save(&mime_to_get, buf.len()));
                            }
                        }
                        let _ = db_conn.execute("DELETE FROM clipboard WHERE id NOT IN (SELECT id FROM clipboard ORDER BY timestamp DESC LIMIT 256)", []);
                    }
                });
            } else {
                // Action Mode: Direct read for immediate CLI output
                let mut buf = Vec::new();
                let mut reader = read_file.take(268435456);
                if reader.read_to_end(&mut buf).is_ok() {
                    state.rx_buf = buf;
                }
            }
        }
    }

    wayland_client::event_created_child!(WaylandState, ExtDataControlDeviceV1, [
        ext_data_control_device_v1::EVT_DATA_OFFER_OPCODE => (ExtDataControlOfferV1, ())
    ]);
}

impl Dispatch<ExtDataControlSourceV1, ()> for WaylandState {
    fn event(state: &mut Self, _: &ExtDataControlSourceV1, ev: ext_data_control_source_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        match ev {
            ext_data_control_source_v1::Event::Send { mime_type, fd } => {
                let is_match = mime_type == state.target_mime 
                    || state.target_mime.starts_with(&mime_type) 
                    || mime_type.starts_with(&state.target_mime);

                if is_match {
                    // Robustness: Clear O_NONBLOCK to prevent partial writes with large data
                    unsafe {
                        use std::os::fd::AsRawFd;
                        let raw_fd = fd.as_raw_fd();
                        let flags = libc::fcntl(raw_fd, libc::F_GETFL, 0);
                        if flags >= 0 {
                            libc::fcntl(raw_fd, libc::F_SETFL, flags & !libc::O_NONBLOCK);
                        }
                    }

                    let mut file = std::fs::File::from(fd);
                    
                    // Performance: Clone the buffer once and move it into the thread
                    // This avoids holding a reference to WaylandState and allows the loop to continue
                    let data = state.rx_buf.clone();

                    std::thread::spawn(move || {
                        if let Err(e) = file.write_all(&data) {
                            eprintln!("{}send error: {}", LOG_ERROR, e);
                        }
                        let _ = file.flush();
                    });
                }
            }
            ext_data_control_source_v1::Event::Cancelled => {
                std::process::exit(0);
            }
            _ => {}
        }
    }
}
