
// src/cli/list.rs

use crate::storage::ClipboardDb;
use super::formatter;
use super::utils::{self, RangeSelection, ArgContext};
use crate::core::constants::*;

/// Entry point for the 'list' command.
pub fn run(args: &[String], db: ClipboardDb) {
    let ctx = ArgContext::parse(args);

    // Strict validation: check for unknown flags
    if !ctx.unknown_flags.is_empty() {
        eprintln!("{}unknown option detected: '{}'", LOG_ERROR, ctx.unknown_flags[0]);
        return;
    }

    // Arity enforcement: list accepts at most one positional argument (the range)
    if ctx.positionals.len() > 1 {
        eprintln!("{}command 'list' accepts only one range argument.", LOG_ERROR);
        return;
    }

    // Parse the positional range argument strictly
    let selection = match utils::parse_range(ctx.positionals.first(), 25) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}argument error: {}", LOG_ERROR, e);
            return;
        }
    };
    
    let all_items = db.fetch_metadata(MAX_HISTORY);
    let total_stored = db.get_total_count();
    let len = all_items.len();
    
    // Determine target items based on flags and range selection
    let target_items: Vec<&(i64, i64, String, i64, Option<String>)> = if ctx.full {
        all_items.iter().collect()
    } else {
        match selection {
            RangeSelection::Single(n) => {
                if n < len { vec![&all_items[n]] } else { vec![] }
            }
            RangeSelection::Range(start, end) => {
                if len == 0 {
                    vec![]
                } else {
                    let s = start.min(len - 1);
                    let e = end.min(len - 1);
                    if s <= e {
                        all_items[s..=e].iter().collect()
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
        if !ctx.raw {
            println!("{}no entries found matching the criteria.", LOG_INFO);
        }
        return;
    }

    // Execute rendering with the preferred visual balance
    render_list("Clipboard History", &target_items, total_stored, ctx.raw);
}

/// Render metadata items in a structured table layout.
pub fn render_list(title: &str, items: &Vec<&(i64, i64, String, i64, Option<String>)>, total_stored: usize, is_raw: bool) {
    let label_width = 6; 
    let total_width = WIDTH_ID + WIDTH_WHEN + WIDTH_SIZE + PREVIEW_WIDTH + label_width + (TABLE_SEP.len() * 3);

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
            println!(
                "[{:>wid_id$}] {} {}",
                i, label, formatted_preview,
                wid_id = WIDTH_ID - 2
            );
        } else {
            println!(
                "[{:>wid_id$}]{sep}{:>wid_when$}{sep}{:>wid_size$} B{sep}{} {}",
                i, formatter::format_time(*ts as u64), size, label, formatted_preview,
                wid_id = WIDTH_ID - 2,
                wid_when = WIDTH_WHEN,
                wid_size = WIDTH_SIZE - 2,
                sep = TABLE_SEP
            );
        }
    }
    
    if !is_raw {
        println!("{}", TABLE_LINE_CHAR.repeat(total_width));
        println!(
            "{}shown {} items | history: {} / {} entries", 
            LOG_INFO, items.len(), total_stored, MAX_HISTORY
        );
    }
}
