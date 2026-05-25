
// src/cli/list.rs

use crate::storage::ClipboardDb;
use super::formatter;
use super::utils::{self, RangeSelection};
use crate::core::constants::*;

/// Entry point for the 'list' command.
pub fn run(args: &[String], db: ClipboardDb) {
    // Detect if raw output mode is requested for scripting (e.g., rofi integration)
    let is_raw = args.iter().any(|a| a == "--raw");
    
    // Detect if full history is requested
    let is_full = args.iter().any(|a| a == "--full" || a == "all");

    // Retrieve all metadata records up to the maximum permitted history size
    let all_items = db.fetch_metadata(MAX_HISTORY);
    let total_stored = db.get_total_count();
    let len = all_items.len();
    
    // Improved Extraction Logic: Clamps indices to prevent out-of-bounds panics
    let target_items: Vec<&(i64, i64, String, i64, Option<String>)> = if is_full {
        all_items.iter().collect()
    } else {
        let selection = utils::parse_range(args.get(2), 25);
        match selection {
            RangeSelection::Single(n) => {
                if n < len { vec![&all_items[n]] } else { vec![] }
            }
            RangeSelection::Range(start, end) => {
                // Robustness Fix: Clamp indices to the actual length of the available items
                if len == 0 {
                    vec![]
                } else {
                    let clamped_start = start.min(len - 1);
                    let clamped_end = end.min(len - 1);
                    if clamped_start <= clamped_end {
                        all_items[clamped_start..=clamped_end].iter().collect()
                    } else {
                        vec![]
                    }
                }
            }
            RangeSelection::Latest(limit) => {
                all_items.iter().take(limit).collect()
            }
        }
    };

    if target_items.is_empty() {
        if !is_raw { println!("{}no entries found matching the criteria.", LOG_INFO); }
        return;
    }

    // Pass the raw flag to the renderer to suppress decorations
    render_list("Clipboard History", &target_items, total_stored, is_raw);
}

/// Render the specified metadata items in a structured table layout.
pub fn render_list(title: &str, items: &Vec<&(i64, i64, String, i64, Option<String>)>, total_stored: usize, is_raw: bool) {
    // Calculate total display layout width (columns + separators + padding)
    let label_width = 6; 
    let total_width = WIDTH_ID + WIDTH_WHEN + WIDTH_SIZE + PREVIEW_WIDTH + label_width + (TABLE_SEP.len() * 3);

    // Conditional rendering: Skip headers if in raw mode
    if !is_raw {
        println!("\n--- {} ---", title);
        println!(
            "{:>wid_id$}{sep}{:>wid_when$}{sep}{:>wid_size$}{sep}{}",
            LIST_HEADER_ID,
            LIST_HEADER_WHEN,
            LIST_HEADER_SIZE,
            LIST_HEADER_CONTENT,
            wid_id = WIDTH_ID,
            wid_when = WIDTH_WHEN,
            wid_size = WIDTH_SIZE,
            sep = TABLE_SEP
        );
        println!("{}", TABLE_LINE_CHAR.repeat(total_width));
    }

    // Iterate and render each record row
    for (i, item) in items.iter().enumerate() {
        let (_real_id, ts, mime, size, preview) = item;
        let label = formatter::get_label(mime);
        
        let raw_preview = if mime.starts_with("image/") {
            format!("[{}] - {} bytes", mime.split('/').nth(1).unwrap_or(""), size)
        } else {
            preview.as_deref().unwrap_or("").to_string()
        };

        let formatted_preview = formatter::preview_content(&raw_preview);

        if is_raw {
            // Keep your preferred raw format exactly as it was
            println!(
                "{:>wid_id$} {} {}",
                i,
                label,
                formatted_preview,
                wid_id = WIDTH_ID / 2
            );
        } else {
            // Keep your preferred table format exactly as it was
            println!(
                "[{:>wid_id$}]{sep}{:>wid_when$}{sep}{:>wid_size$} B{sep}{} {}",
                i,
                formatter::format_time(*ts as u64),
                size,
                label,
                formatted_preview,
                wid_id = WIDTH_ID - 2,
                wid_when = WIDTH_WHEN,
                wid_size = WIDTH_SIZE - 2,
                sep = TABLE_SEP
            );
        }
    }
    
    // Conditional rendering: Skip footer if in raw mode
    if !is_raw {
        println!("{}", TABLE_LINE_CHAR.repeat(total_width));
        println!(
            "{}shown {} items | history: {} / {} entries", 
            LOG_INFO,
            items.len(),
            total_stored,
            MAX_HISTORY
        );
    }
}
