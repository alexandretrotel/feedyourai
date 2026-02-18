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
