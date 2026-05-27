
// src/wayland/handlers/data_control.rs

use wayland_client::{Dispatch, Connection, QueueHandle, Proxy};
use wayland_protocols::ext::data_control::v1::client::{
    ext_data_control_device_v1::{self, ExtDataControlDeviceV1},
    ext_data_control_manager_v1::{self, ExtDataControlManagerV1},
    ext_data_control_offer_v1::{self, ExtDataControlOfferV1},
    ext_data_control_source_v1::{self, ExtDataControlSourceV1},
};
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::{Read, Write};
use std::os::fd::{AsFd, FromRawFd, OwnedFd, AsRawFd};
use crate::wayland::state::{WaylandState, OfferData};
use crate::core::constants::*;
use std::sync::{Arc, Mutex};

/// Evaluates if the requested MIME type is compatible with the target type.
fn mime_is_compatible(requested: &str, target: &str) -> bool {
    if requested == target { return true; }
    if requested.starts_with("text/") && target.starts_with("text/") { return true; }
    if requested.starts_with("image/") && target.starts_with("image/") { return true; }
    
    const TEXT_ALIASES: &[&str] = &[
        "text/plain", "text/plain;charset=utf-8", "text/plain;charset=UTF-8",
        "UTF8_STRING", "STRING", "TEXT", "COMPOUND_TEXT",
    ];
    let req_is_text_alias = TEXT_ALIASES.contains(&requested);
    let tgt_is_text_alias = TEXT_ALIASES.contains(&target) || target.starts_with("text/");
    
    req_is_text_alias && tgt_is_text_alias
}

/// Checks if offered MIME types contain sensitive keywords.
fn is_sensitive(mimes: &[String]) -> bool {
    SENSITIVE_MIME_HINTS.iter().any(|&hint| {
        mimes.iter().any(|m| m.to_lowercase().contains(hint))
    })
}

/// Creates a Unix pipe configured for blocking I/O with optional buffer expansion.
fn make_pipe(is_image: bool) -> Option<(std::fs::File, OwnedFd)> {
    let mut fds = [0i32; 2];
    if unsafe { libc::pipe(fds.as_mut_ptr()) } < 0 {
        return None;
    }

    unsafe {
        for &fd in &fds {
            let flags = libc::fcntl(fd, libc::F_GETFL, 0);
            if flags >= 0 {
                libc::fcntl(fd, libc::F_SETFL, flags & !libc::O_NONBLOCK);
            }
        }

        if is_image {
            libc::fcntl(fds[0], 1031, 4 * 1024 * 1024i32); // 4MB buffer
        }

        let read_file = std::fs::File::from_raw_fd(fds[0]);
        let write_fd  = OwnedFd::from_raw_fd(fds[1]);
        Some((read_file, write_fd))
    }
}

// --- ExtDataControlManagerV1 ---

impl Dispatch<ExtDataControlManagerV1, ()> for WaylandState {
    fn event(_: &mut Self, _: &ExtDataControlManagerV1, _: ext_data_control_manager_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
}

// --- ExtDataControlOfferV1 ---

impl Dispatch<ExtDataControlOfferV1, OfferData> for WaylandState {
    fn event(_: &mut Self, _: &ExtDataControlOfferV1, ev: ext_data_control_offer_v1::Event, data: &OfferData, _: &Connection, _: &QueueHandle<Self>) {
        if let ext_data_control_offer_v1::Event::Offer { mime_type } = ev {
            if let Ok(mut mimes) = data.mimes.lock() {
                if !mimes.contains(&mime_type) {
                    mimes.push(mime_type);
                }
            }
        }
    }
}

// --- ExtDataControlDeviceV1 ---

impl Dispatch<ExtDataControlDeviceV1, ()> for WaylandState {
    fn event(state: &mut Self, _: &ExtDataControlDeviceV1, ev: ext_data_control_device_v1::Event, _: &(), conn: &Connection, _: &QueueHandle<Self>) {
        if let ext_data_control_device_v1::Event::Selection { id } = ev {
            state.selection_received = true;

            let Some(offer) = id else { return };
            if state.is_provider { return; }

            let mimes: Vec<String> = offer
                .data::<OfferData>()
                .and_then(|d| d.mimes.lock().ok())
                .map(|g| g.clone())
                .unwrap_or_default();

            if mimes.is_empty() { return; }

            // Logic: Dynamic MIME selection prioritizing modern formats
            let priority: &[&str] = &[
                "image/webp",
                "image/png",
                "image/jpeg",
                "image/gif",
                MIME_URI_LIST,
                "text/plain;charset=utf-8",
                "text/plain",
            ];

            let mime_to_get = priority.iter()
                .find_map(|&p| {
                    mimes.iter()
                        .find(|&m| m == p || m.starts_with(&format!("{};", p)))
                        .map(|s| s.to_string()) // Explicitly convert &String to String
                })
                .or_else(|| {
                    mimes.iter()
                        .find(|m| m.starts_with("image/"))
                        .map(|s| s.to_string())
                })
                .or_else(|| {
                    mimes.iter()
                        .find(|m| m.starts_with("text/"))
                        .map(|s| s.to_string())
                })
                .or_else(|| mimes.first().map(|s| s.to_string()))
                .unwrap_or_else(|| DEFAULT_MIME.to_string());

            if state.verbose {
                println!("{}selected format: {}", LOG_INFO, mime_to_get);
            }

            let is_image = mime_to_get.starts_with("image/");
            let (read_file, write_fd) = match make_pipe(is_image) {
                Some(p) => p,
                None => return,
            };

            offer.receive(mime_to_get.clone(), write_fd.as_fd());
            drop(write_fd);
            offer.destroy();
            let _ = conn.flush();

            if let Some(db_path) = state.db.as_ref().map(|db| db.path.clone()) {
                let is_verbose = state.verbose;

                std::thread::spawn(move || {
                    let mut hash_context = md5::Context::new();
                    let mut chunk = vec![0u8; 65536];
                    let mut payload = Vec::with_capacity(1048576);
                    let mut reader = read_file.take(268435456); 

                    while let Ok(n) = reader.read(&mut chunk) {
                        if n == 0 { break; }
                        hash_context.consume(&chunk[..n]);
                        payload.extend_from_slice(&chunk[..n]);
                    }

                    if payload.is_empty() { return; }

                    let mut final_mime = mime_to_get;

                    // Conditional Promotion: URIs pointing to images are ingested as binary
                    if final_mime == MIME_URI_LIST {
                        let uri_content = String::from_utf8_lossy(&payload);
                        if let Some(line) = uri_content.lines().next() {
                            if line.starts_with("file://") {
                                let path_raw = line.trim_start_matches("file://");
                                if let Ok(decoded_path) = percent_encoding::percent_decode_str(path_raw).decode_utf8() {
                                    let path = std::path::Path::new(decoded_path.as_ref());
                                    
                                    if path.exists() && path.is_file() {
                                        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                                        let file_mime = match ext.to_lowercase().as_str() {
                                            "png" => Some("image/png"),
                                            "jpg" | "jpeg" => Some("image/jpeg"),
                                            "webp" => Some("image/webp"),
                                            "gif" => Some("image/gif"),
                                            _ => None,
                                        };

                                        if let Some(m) = file_mime {
                                            if let Ok(file_data) = std::fs::read(path) {
                                                payload = file_data;
                                                final_mime = m.to_string();
                                                let mut new_hash = md5::Context::new();
                                                new_hash.consume(&payload);
                                                hash_context = new_hash;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Ok(mut db_conn) = rusqlite::Connection::open(&db_path) {
                        db_conn.busy_timeout(std::time::Duration::from_millis(SQLITE_TIMEOUT_MS)).ok();
                        db_conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;").ok();

                        let hash = format!("{:x}", hash_context.finalize());
                        let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as i64;

                        let existing: Option<i64> = db_conn.query_row(
                            "SELECT id FROM clipboard WHERE hash = ?1 LIMIT 1",
                            rusqlite::params![hash], |row| row.get(0),
                        ).ok();

                        if let Some(id) = existing {
                            let _ = db_conn.execute("UPDATE clipboard SET timestamp = ?1 WHERE id = ?2", rusqlite::params![ts, id]);
                            return;
                        }

                        let preview = if final_mime.contains("text") || final_mime == MIME_URI_LIST {
                            let s = String::from_utf8_lossy(&payload);
                            Some(s.chars().take(PREVIEW_CHARS).collect::<String>().replace('\n', " "))
                        } else { None };

                        let res = db_conn.execute(
                            "INSERT INTO clipboard (timestamp, mime, size, preview, content, hash) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                            rusqlite::params![ts, final_mime.clone(), payload.len() as i64, preview, payload, hash],
                        );

                        if res.is_ok() && is_verbose {
                            println!("{}", log_save(&final_mime, payload.len()));
                        }
                        let _ = db_conn.execute("DELETE FROM clipboard WHERE id NOT IN (SELECT id FROM clipboard ORDER BY timestamp DESC LIMIT 256)", []);
                    }
                });
            } else {
                let mut buf = Vec::new();
                let mut reader = read_file.take(268435456);
                if reader.read_to_end(&mut buf).is_ok() {
                    state.rx_buf = buf;
                }
            }
        }
    }

    // Specialized child creation logic for asynchronous DataOffer instantiation
    wayland_client::event_created_child!(WaylandState, ExtDataControlDeviceV1, [
        ext_data_control_device_v1::EVT_DATA_OFFER_OPCODE => (ExtDataControlOfferV1, OfferData {
            mimes: Arc::new(Mutex::new(Vec::new()))
        })
    ]);
}

// --- ExtDataControlSourceV1 ---

impl Dispatch<ExtDataControlSourceV1, ()> for WaylandState {
    fn event(state: &mut Self, _: &ExtDataControlSourceV1, ev: ext_data_control_source_v1::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        match ev {
            ext_data_control_source_v1::Event::Send { mime_type, fd } => {
                if mime_is_compatible(&mime_type, &state.target_mime) {
                    unsafe {
                        let raw = fd.as_raw_fd();
                        let flags = libc::fcntl(raw, libc::F_GETFL, 0);
                        if flags >= 0 {
                            libc::fcntl(raw, libc::F_SETFL, flags & !libc::O_NONBLOCK);
                        }
                    }

                    let mut file = std::fs::File::from(fd);
                    let data = state.rx_buf.clone();

                    std::thread::spawn(move || {
                        if let Err(e) = file.write_all(&data) {
                            eprintln!("{}transmission failure: {}", LOG_ERROR, e);
                        }
                        let _ = file.flush();
                    });
                } else {
                    drop(std::fs::File::from(fd));
                }
            }
            ext_data_control_source_v1::Event::Cancelled => {
                std::process::exit(0);
            }
            _ => {}
        }
    }
}
