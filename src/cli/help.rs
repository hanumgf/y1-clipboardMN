
// src/cli/help.rs

// src/cli/help.rs
/// Display the current application version and description.
pub fn print_version() {
    println!("y1-clipboard v1.0.0");
    println!("A robust, unified clipboard manager for Wayland.");
}

/// Print comprehensive usage instructions, commands, options, and practical examples.
pub fn print_help() {
    print_version();

    println!("\nUSAGE:");
    println!("    y1-clip <COMMAND> [ARGS] [OPTIONS]");

    println!("\nCORE COMMANDS:");
    println!("    daemon             - Background monitor. Watches system for changes and saves to DB.");
    println!("    list [n|n-n1]      - Display history. Defaults to latest 25. Supports ranges (e.g., 0-50).");
    println!("    search QUERY       - Keyword search. Rapidly scans text history using SQLite indexes.");
    
    println!("\nDATA OPERATIONS:");
    println!("    copy-to <id>       - Restore to system clipboard. Moves entry to top (ID 0).");
    println!("                         Backgrounds itself until replaced by a new system copy.");
    
    println!("    show <id>          - Inspect content. Decodes text for previewing in terminal.");
    println!("      --raw            - Dump exact bytes to stdout. Use for images or piping to files.");
    
    println!("    store [mime]       - Manual inject. Reads stdin into history. (e.g., echo '...' | y1-clip store)");
    println!("                         Default MIME is text/plain unless specified.");
    
    println!("    paste-from [mime]  - Direct OS access. Outputs current system clipboard to stdout.");
    println!("                         Bypasses database. Acts as a standalone 'wl-paste' replacement.");

    println!("\nMANAGEMENT:");
    println!("    delete <id>        - Remove a specific entry. Physical deletion from storage.");
    println!("    wipe               - Clear all history. Runs 'VACUUM' to minimize database file size.");

    println!("\nOPTIONS:");
    println!("    -v, --verbose      - Enable detailed logging for transfer and storage events.");
    println!("    -h, --help         - Show this help information.");
    println!("    -V, --version      - Show version.");

    println!("\nPRACTICAL EXAMPLES:");
    println!("    # 1. Selection menu (integrating with fzf):");
    println!("    $ y1-clip list 0-100 | fzf | awk '{{print $1}}' | xargs y1-clip copy-to");
    
    println!("\n    # 2. Working with images (Dumping binary):");
    println!("    $ y1-clip show 0 --raw > wallpaper.png");
    
    println!("\n    # 3. Piping command results directly to history:");
    println!("    $ journalctl -xe | tail -n 50 | y1-clip store");
    
    println!("\n    # 4. Save current clipboard to a file (without checking history):");
    println!("    $ y1-clip paste-from image/png > capture.png");

    println!("\nNOTES:");
    println!("    - Database is secured with 600 permissions at ~/.local/share/y1-clipboard/");
    println!("    - SQLite WAL mode ensures safe access even while the daemon is writing.");
    println!();
}
