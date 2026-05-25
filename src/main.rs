
// src/main.rs

mod core;
mod storage;
mod wayland;
mod daemon;
mod cli;

fn main() {
    // 1. Collect standard runtime command-line arguments
    let args: Vec<String> = std::env::args().collect();

    // 2. Initialize and open the database infrastructure securely
    // Robustness: Path resolution, filesystem permissions configuration (600), WAL mode,
    // index initialization, and busy timeout parameters are handled automatically within ClipboardDb::open().
    let db = storage::ClipboardDb::open();

    // 3. Delegate execution flow to the command router
    // Stability: Parses arguments and safely dispatches ownership contexts to individual
    // target subcommands (such as daemon, list, search, or copy-to).
    cli::handle_command(&args, db);
}
