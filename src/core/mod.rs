
// src/core/mod.rs

pub mod constants;

use std::path::PathBuf;
use std::fs;
use crate::core::constants::{DB_DIR_NAME, DB_FILE_NAME};

/// Securely resolve the database path using the following priority layout:
/// 1. $XDG_DATA_HOME/y1-clipboard/y1_clipboard.sqlite
/// 2. ~/.local/share/y1-clipboard/... (Standard fallback destination)
/// 3. Current working directory (Ultimate fallback scenario)
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

    // Initialize target directory path dynamically if it does not already exist
    // Enforcing folder permissions equivalent to 700 (owner-only access) is recommended
    if !path.exists() {
        let _ = fs::create_dir_all(&path);
    }

    path.push(DB_FILE_NAME);
    
    path.to_str().unwrap_or(DB_FILE_NAME).to_string()
}
