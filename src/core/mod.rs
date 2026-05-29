
// src/core/mod.rs

pub mod constants;

use std::path::PathBuf;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::core::constants::{DB_DIR_NAME, DB_FILE_NAME};
pub use self::constants::get_socket_path; 

/// Global atomic signal to coordinate graceful process termination across threads.
pub static SIG_EXIT: AtomicBool = AtomicBool::new(false);

/// Securely resolve the database path following XDG Data Home specifications.
/// 1. $XDG_DATA_HOME/y1-clipboard/y1_clipboard.sqlite
/// 2. ~/.local/share/y1-clipboard/... (Standard fallback)
/// 3. Current working directory (Absolute fallback)
pub fn get_db_path() -> String {
    let mut path = if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg_data)
    } else if let Ok(home) = std::env::var("HOME") {
        let mut p = PathBuf::from(home);
        p.push(".local");
        p.push("share");
        p
    } else {
        PathBuf::from(".")
    };

    path.push(DB_DIR_NAME);

    // Dynamically initialize the storage directory with 700-equivalent permissions if missing
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }

    path.push(DB_FILE_NAME);
    
    path.to_str().unwrap_or(DB_FILE_NAME).to_string()
}

/// Sets the global termination flag to true to initiate shutdown sequences.
pub fn request_exit() {
    SIG_EXIT.store(true, Ordering::SeqCst);
}

/// Checks the current state of the global termination signal.
pub fn is_exiting() -> bool {
    SIG_EXIT.load(Ordering::SeqCst)
}
