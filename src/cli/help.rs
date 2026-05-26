
// src/cli/help.rs

/// Display the application version and summary.
pub fn print_version() {
    println!("y1-clipboard v1.0.0");
    println!("A robust, unified clipboard manager for Wayland.");
}

/// Print structured usage instructions, command definitions, and practical examples.
pub fn print_help() {
    print_version();

    println!("\nUSAGE:");
    println!("    y1-clip <COMMAND> [ARGS] [OPTIONS]");

    println!("\nCORE COMMANDS:");
    println!("    daemon             - Monitor clipboard changes in background and persist to DB.");
    println!("    list [range]       - Display history. Supports index ranges (e.g., 0-50).");
    println!("                         Flags: --raw (-R), --full (-A).");
    println!("    search <query>     - Keyword search across text history using SQLite indexing.");
    println!("                         Flags: --raw (-R).");
    println!("    copy-to <id>       - Restore a history entry to the system clipboard.");

    println!("\nDATA OPERATIONS:");
    println!("    show <id>          - Inspect entry content. Decodes text for terminal preview.");
    println!("                         Flags: --raw (-R) for exact binary output.");

    println!("    store [mime]       - Ingest stdin and save to database. Synchronizes to clipboard.");
    println!("    paste-from [mime]  - Access current system clipboard directly. Bypasses database.");

    println!("\nMANAGEMENT:");
    println!("    delete <id>        - Physically remove a specific record from storage.");
    println!("    wipe               - Clear all history. Runs VACUUM to optimize database size.");
    println!("                         Flags: --force (-f) to bypass confirmation.");

    println!("\nGLOBAL OPTIONS:");
    println!("    -v, --verbose      - Enable detailed event logging.");
    println!("    -h, --help         - Show this help information.");
    println!("    -V, --version      - Show version information.");


    println!("\nPRACTICAL EXAMPLES:");
    println!("    # 1. Interactive selection with fzf:");
    println!("    $ y1-clip list 0-100 --raw | fzf | awk '{{print $1}}' | xargs -r y1-clip copy-to");
    
    println!("\n    # 2. Exporting an image entry to a file:");
    println!("    $ y1-clip show 5 --raw > image_dump.png");
    
    println!("\n    # 3. Piping command output directly into history:");
    println!("    $ dmesg | tail -n 20 | y1-clip store");

    println!("\nNOTES:");
    println!("    - Persistent storage located at ~/.local/share/y1-clipboard/ (chmod 600).");
    println!("    - Non-blocking I/O is enforced via background synchronization workers.");
    println!();
}
