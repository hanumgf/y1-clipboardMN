
// src/main.rs

mod core;
mod storage;
mod wayland;
mod daemon;
mod cli;

use crate::core::constants::*;

fn main() {
    // 1. Initialize signal handler for graceful shutdown
    ctrlc::set_handler(move || {
        if crate::core::is_exiting() {
            // 🚨 Emergency Exit: Force terminate if Ctrl+C is pressed again
            eprintln!("\n{}forceful termination initiated.", LOG_ERROR);
            std::process::exit(1);
        }
        
        // Signal the primary loop to wrap up operations
        crate::core::request_exit();
        
        // Provide immediate feedback to the user
        eprintln!("\n{}termination signal received. closing storage safely...", LOG_INFO);
    }).expect("failed to set signal handler");

    let args: Vec<String> = std::env::args().collect();
    let db = storage::ClipboardDb::open();
    cli::handle_command(&args, db);
}
