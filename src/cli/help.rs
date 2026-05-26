
// src/cli/help.rs

/// Display the application version and primary description.
pub fn print_version() {
    println!("y1-clipboard v1.0.0");
    println!("A robust, unified clipboard manager for Wayland.");
}

/// Print high-density usage instructions, command definitions, and practical examples.
pub fn print_help() {
    print_version();

    println!("\nUSAGE:");
    println!("    y1-clip <COMMAND> [ARGS] [OPTIONS]");

    println!("\nCORE COMMANDS:");
    println!("    daemon             - Monitor clipboard changes in background and persist to storage.");
    println!("                         Flags: --verbose (-v).");
    
    println!("    list [range]       - Display history metadata. Supports index ranges (e.g., 0-50).");
    println!("                         Flags: --raw (-R), --full (-A).");
    
    println!("    search <query>     - Keyword scan across text history using SQLite indexing.");
    println!("                         Flags: --raw (-R).");
    
    println!("    copy-to <id>       - Restore a history entry to the system clipboard.");
    println!("                         Moves target entry to the top of the history (MRU).");

    println!("\nDATA OPERATIONS:");
    println!("    show <id>          - Inspect record content. Decodes text for terminal preview.");
    println!("                         Flags: --raw (-R) for unmodified binary output.");

    println!("    store [mime]       - Ingest stdin to database and synchronize with system clipboard.");
    println!("                         Defaults to text/plain if mime is omitted.");
    
    println!("    paste-from [mime]  - Stream current system clipboard directly to stdout.");
    println!("                         Bypasses database storage for immediate access.");

    println!("\nMANAGEMENT:");
    println!("    delete <id>        - Physically remove a specific record from persistent storage.");
    
    println!("    wipe               - Purge all history. Executes SQLite VACUUM for optimization.");
    println!("                         Flags: --force (-f) to bypass user confirmation.");

    println!("\nGLOBAL OPTIONS:");
    println!("    -h, --help         - Show this help information.");
    println!("    -V, --version      - Show version information.");
    println!("    -v, --verbose      - Enable detailed event and synchronization logging.");

    println!("\nPRACTICAL EXAMPLES:");
    println!("    # 1. Interactive selection with fzf:");
    println!("    $ y1-clip list 0-100 --raw | fzf | awk '{{print $1}}' | xargs -r y1-clip copy-to");
    
    println!("\n    # 2. Restoring binary data to a file:");
    println!("    $ y1-clip show 5 --raw > restored_image.png");
    
    println!("\n    # 3. Synchronizing command output to clipboard history:");
    println!("    $ dmesg | tail -n 20 | y1-clip store");

    println!("\nNOTES:");
    println!("    - Database: Secured at ~/.local/share/y1-clipboard/ (mode 600).");
    println!("    - Execution: Background processes (serve-internal) manage data egress.");
    println!();
}
