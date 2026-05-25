
// src/cli/utils.rs

/// Enum representing the range of history items requested by the user.
#[derive(Debug, PartialEq)]
pub enum RangeSelection {
    /// A single specific entry (e.g., "5")
    Single(usize),
    /// A defined span of entries (e.g., "5-10")
    Range(usize, usize),
    /// The latest N entries (used as default or fallback)
    Latest(usize),
}

/// Parse a string argument and return the corresponding RangeSelection.
pub fn parse_range(arg: Option<&String>, default_limit: usize) -> RangeSelection {
    // 1. Return the default limit if no valid argument is provided
    let arg = match arg {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => return RangeSelection::Latest(default_limit),
    };

    // 2. Parse range notation ("n-n1")
    if arg.contains('-') {
        let parts: Vec<&str> = arg.split('-').collect();
        
        // Extract and parse start and end values safely
        let n1 = parts.first().and_then(|s| s.trim().parse::<usize>().ok());
        let n2 = parts.get(1).and_then(|s| s.trim().parse::<usize>().ok());

        return match (n1, n2) {
            (Some(start), Some(end)) => {
                // Robustness optimization: automatically reorder bounds if specified
                // in reverse order (e.g., "10-5" maps correctly to start=5, end=10).
                RangeSelection::Range(start.min(end), start.max(end))
            }
            (Some(start), None) => {
                // Interpret open-ended ranges like "5-" as "from index 5 up to the default limit span"
                RangeSelection::Range(start, start + default_limit)
            }
            (None, Some(end)) => {
                // Interpret ranges like "-10" as "from index 0 up to index 10"
                RangeSelection::Range(0, end)
            }
            _ => RangeSelection::Latest(default_limit),
        };
    }

    // 3. Parse single index notation ("10")
    if let Ok(n) = arg.parse::<usize>() {
        return RangeSelection::Single(n);
    }

    // 4. Safe fallback to the latest entries if the input cannot be parsed
    RangeSelection::Latest(default_limit)
}
