//! Small standalone helpers shared across the scanner submodules.

/// Returns true if `size` falls within the inclusive `[min, max]` bounds,
/// treating a missing bound as unconstrained.
pub(crate) fn size_allowed(size: u64, min: Option<u64>, max: Option<u64>) -> bool {
    if let Some(min) = min
        && size < min
    {
        return false;
    }
    if let Some(max) = max
        && size > max
    {
        return false;
    }
    true
}

/// Formats `bytes` as a human-readable size (`"512 B"`, `"1.2 KB"`, `"3.4
/// MB"`, ...), using 1024 as the unit step.
pub(crate) fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];

    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}
