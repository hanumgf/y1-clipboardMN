
// src/daemon/mod.rs

use crate::storage::ClipboardDb;
use crate::wayland;
use crate::wayland::state::WaylandState;
use crate::core::constants::*;
use std::os::unix::net::UnixListener;
use std::sync::mpsc;
use std::io::Read;
use std::fs;

pub fn start_daemon(db: ClipboardDb, verbose: bool) {
    let (tx, rx) = mpsc::channel::<i64>();
    let socket_path = crate::core::get_socket_path();
    let _ = fs::remove_file(&socket_path);

    // IPC Thread: Use blocking listener for stability
    let listener = UnixListener::bind(&socket_path).expect("failed to bind IPC socket");
    std::thread::spawn(move || {
        for mut s in listener.incoming().flatten() {
            let mut buf = String::new();
            if s.read_to_string(&mut buf).is_ok() {
                if let Ok(id) = buf.trim().parse::<i64>() {
                    let _ = tx.send(id);
                }
            }
        }
    });

    let (conn, mut event_queue) = wayland::create_connection();
    let qh = event_queue.handle();
    let _registry = conn.display().get_registry(&qh, ());

    let last_stored = db.get_latest_data().unwrap_or_default();
    let mut state = WaylandState::new_daemon(db, verbose);
    state.last_data = last_stored;
    state.target_mime = DEFAULT_MIME.to_string();

    // Initial Wayland Sync
    if event_queue.roundtrip(&mut state).is_err() { return; }
    if let (Some(manager), Some(seat)) = (&state.manager, &state.seat) {
        state.device = Some(manager.get_data_device(seat, &qh, ()));
        let _ = conn.flush();
    } else { return; }

    use std::os::fd::{AsFd, AsRawFd};
    while !crate::core::is_exiting() {
        // 1. Process IPC commands (Non-blocking check)
        // Drains the channel to handle rapid succession of 'store' or 'copy-to' commands.
        while let Ok(real_id) = rx.try_recv() {
            if let Some(ref mut db) = state.db {
                if let Some((mime, val)) = db.get_content_by_id(real_id) {
                    if let Some(ref manager) = state.manager {
                        let source = manager.create_data_source(&qh, ());
                        source.offer(mime.clone());
                        if mime.contains("text") {
                            for alt in TEXT_MIME_ALTS { source.offer(alt.to_string()); }
                        }
                        
                        state.target_mime = mime;
                        state.tx_buf = val;
                        state.is_provider = true;

                        if let Some(ref device) = state.device {
                            device.set_selection(Some(&source));
                            let _ = conn.flush();
                        }
                        state.current_source = Some(source);
                        
                        if state.verbose { 
                            println!("{}", log_restore(real_id as usize)); 
                        }
                    }
                }
            }
        }

        // 2. Refresh Wayland events with a balanced poll duration
        let _ = conn.flush();
        let mut poll_fds = [libc::pollfd { 
            fd: conn.as_fd().as_raw_fd(), 
            events: libc::POLLIN, 
            revents: 0 
        }];
        
        // Wait for 100ms for events, allowing the loop to check IPC regularly
        unsafe { libc::poll(poll_fds.as_mut_ptr(), 1, 100); }

        if let Some(guard) = event_queue.prepare_read() {
            let _ = guard.read();
        }
        let _ = event_queue.dispatch_pending(&mut state);
    }
    
    let _ = fs::remove_file(&socket_path);
}
