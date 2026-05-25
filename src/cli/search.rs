
// src/cli/search.rs

use crate::storage::ClipboardDb;
use crate::core::constants::*;
use super::list;

/// Search for text matching the specified keyword in history and display the results.
pub fn run(args: &[String], db: ClipboardDb) {
    // 1. Extract and validate the search query
    let query = match args.get(2) {
        Some(q) if !q.trim().is_empty() => q.trim(),
        _ => {
            eprintln!("{}no search keyword provided.", LOG_ERROR);
            println!("usage: y1-clip search <keyword>");
            return;
        }
    };

    // 2. Execute fast metadata search using SQLite index
    // Restricts the search scope within the MAX_HISTORY (256 entries) limit
    let results = db.search_metadata(query, MAX_HISTORY);
    
    // 🆕 Retrieve the total count of stored items for consistent footer rendering
    let total_stored = db.get_total_count();

    // 3. Verify hit count consistency
    if results.is_empty() {
        println!("{}no entries matching '{}' were found in the history.", LOG_INFO, query);
        return;
    }

    // 4. Render the search results
    // Convert references to match the signature required by list::render_list safely
    let refs: Vec<_> = results.iter().collect();
    
    let title = format!("search results for: '{}' ({} found)", query, results.len());
    
    // Pass the search results along with the global storage count to the renderer
    let is_raw = args.iter().any(|a| a == "--raw");
    list::render_list(&title, &refs, total_stored, is_raw);
}
