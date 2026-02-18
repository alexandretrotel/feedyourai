use std::process;

pub(crate) fn command_error_details(output: &process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let details = if !stderr.trim().is_empty() {
        stderr.trim()
    } else {
        stdout.trim()
    };

    if details.is_empty() {
        "unknown error".to_string()
    } else {
        details.to_string()
    }
}
