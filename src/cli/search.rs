
// src/cli/search.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use crate::cli::utils;
use super::list;

/// Search for keyword matches within history and display filtered results.
pub fn run(args: &[String], db: ClipboardDb) {
    // Strictly isolate the search query from optional flags
    let query = match args.get(2).filter(|s| !utils::is_option(s)) {
        Some(q) if !q.trim().is_empty() => q.trim(),
        _ => {
            eprintln!("{}missing search keyword.", LOG_ERROR);
            println!("usage: y1-clip search <keyword> [--raw | -R]");
            return;
        }
    };

    // Execute metadata-level scan using SQLite indexing
    let results = db.search_metadata(query, MAX_HISTORY);
    let total_stored = db.get_total_count();

    if results.is_empty() {
        println!("{}no entries matching '{}' were found.", LOG_INFO, query);
        return;
    }

    // Check for script-mode flag
    let is_raw = utils::has_flag(args, "--raw", "-R");

    // Collect references to match render_list signature requirements
    let refs: Vec<_> = results.iter().collect();
    let title = format!("search: '{}' ({} hits)", query, results.len());
    
    // Delegate to unified list renderer
    list::render_list(&title, &refs, total_stored, is_raw);
}
