
// src/cli/utils.rs

/// Categories for history range selection.
#[derive(Debug, PartialEq)]
pub enum RangeSelection {
    /// Specific single index.
    Single(usize),
    /// Bound range of indices (start, end).
    Range(usize, usize),
    /// N most recent entries.
    Latest(usize),
}

/// Validates if a string follows the strict option prefix format.
pub fn is_option(arg: &str) -> bool {
    arg.starts_with('-')
}

/// Matches an argument against a specific long and short option pair.
pub fn matches_option(arg: &str, long: &str, short: &str) -> bool {
    arg == long || arg == short
}

/// Parses a positional argument into a RangeSelection.
/// Returns Latest(default_limit) if the argument is an option or invalid.
pub fn parse_range(arg: Option<&String>, default_limit: usize) -> RangeSelection {
    let s = match arg {
        Some(s) if !s.trim().is_empty() && !is_option(s) => s.trim(),
        _ => return RangeSelection::Latest(default_limit),
    };

    if s.contains('-') {
        let parts: Vec<&str> = s.split('-').collect();
        let n1 = parts.first().and_then(|v| v.trim().parse::<usize>().ok());
        let n2 = parts.get(1).and_then(|v| v.trim().parse::<usize>().ok());

        return match (n1, n2) {
            (Some(start), Some(end)) => RangeSelection::Range(start.min(end), start.max(end)),
            (Some(start), None) => RangeSelection::Range(start, start + default_limit),
            (None, Some(end)) => RangeSelection::Range(0, end),
            _ => RangeSelection::Latest(default_limit),
        };
    }

    if let Ok(n) = s.parse::<usize>() {
        return RangeSelection::Single(n);
    }

    RangeSelection::Latest(default_limit)
}

/// Checks if a specific flag exists within the argument vector.
pub fn has_flag(args: &[String], long: &str, short: &str) -> bool {
    args.iter().any(|a| matches_option(a, long, short))
}
