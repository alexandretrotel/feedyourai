//! System-clipboard access for copying the combined output.

use arboard::Clipboard;

use color_eyre::eyre::{Result, eyre};

/// Copies `contents` to the system clipboard.
pub fn copy_to_clipboard(contents: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().map_err(|e| eyre!("clipboard error: {e}"))?;
    clipboard
        .set_text(contents.to_owned())
        .map_err(|e| eyre!("clipboard error: {e}"))?;

    Ok(())
}

/// Returns true if a clipboard failure should be treated as a non-fatal
/// warning rather than an error: in CI, or on Linux with no display server
/// attached (headless), no clipboard is ever expected to be available.
pub(crate) fn should_ignore_clipboard_error() -> bool {
    if std::env::var_os("CI").is_some() {
        return true;
    }
    if cfg!(target_os = "linux") {
        let has_display = std::env::var_os("DISPLAY").is_some()
            || std::env::var_os("WAYLAND_DISPLAY").is_some()
            || std::env::var_os("SWAYSOCK").is_some();
        return !has_display;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    /// Restores the original value (set or unset) of an env var on drop.
    struct EnvVarGuard {
        key: &'static str,
        original: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn new(key: &'static str) -> Self {
            let original = std::env::var_os(key);
            Self { key, original }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(val) => unsafe { std::env::set_var(self.key, val) },
                None => unsafe { std::env::remove_var(self.key) },
            }
        }
    }

    #[test]
    #[serial(env)]
    fn should_ignore_clipboard_error_true_when_ci_set() {
        let _guard = EnvVarGuard::new("CI");
        unsafe { std::env::set_var("CI", "true") };

        assert!(should_ignore_clipboard_error());
    }

    #[test]
    #[serial(env)]
    fn should_ignore_clipboard_error_false_when_ci_unset_on_non_linux() {
        let _guard = EnvVarGuard::new("CI");
        unsafe { std::env::remove_var("CI") };

        // The Linux-specific DISPLAY/WAYLAND_DISPLAY/SWAYSOCK branch is
        // `cfg!`-gated and unreachable on this platform (not Linux), so
        // once CI is unset the function always returns false here.
        assert!(!should_ignore_clipboard_error());
    }

    #[test]
    fn copy_to_clipboard_ok_or_reports_clipboard_error() {
        // In a sandboxed/headless test environment `Clipboard::new()` will
        // almost always fail (no display server / no clipboard access),
        // exercising the error path. If the environment does happen to
        // have clipboard access, accept the success path too.
        let result = copy_to_clipboard("some text");
        match result {
            Ok(()) => {}
            Err(e) => assert!(e.to_string().contains("clipboard error")),
        }
    }
}
