//! Best-effort cross-platform launcher used as the post-timer action.
//!
//! Two shapes:
//! - `app = None, args = Some(url)`: open the URL in the default browser
//! - `app = Some(path)`: launch the chosen executable, optionally passing
//!   `args` (split on whitespace, typically a URL for browsers)
//!
//! All errors are logged and swallowed — failing to launch a chosen app
//! must never crash the timer engine or the app.

use std::process::Command;

/// Split `args` into individual arguments by whitespace. Quoted/escaped
/// args are not supported; a single URL or simple flag list is enough for
/// the post-timer use case and keeps us free of `shell-words`.
fn split_args(args: Option<&str>) -> Vec<&str> {
    args.map(|s| s.split_whitespace().collect())
        .unwrap_or_default()
}

fn looks_like_url(s: &str) -> bool {
    s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("file://")
        || s.starts_with("mailto:")
}

pub fn launch(app_path: Option<&str>, args: Option<&str>) {
    let trimmed_args = args.map(str::trim).filter(|s| !s.is_empty());
    let trimmed_path = app_path.map(str::trim).filter(|s| !s.is_empty());

    let result = match (trimmed_path, trimmed_args) {
        (None, None) => {
            log::warn!("post-timer launch skipped: nothing configured");
            return;
        }
        (None, Some(url)) if looks_like_url(url) => open_url(url),
        (None, Some(other)) => {
            log::warn!(
                "post-timer launch skipped: no app path and target {:?} is not a URL",
                other
            );
            return;
        }
        (Some(path), args_opt) => open_app(path, args_opt),
    };

    if let Err(e) = result {
        log::warn!("post-timer launch failed: {}", e);
    }
}

#[cfg(target_os = "macos")]
fn open_url(url: &str) -> std::io::Result<()> {
    Command::new("open").arg(url).spawn().map(|_| ())
}

#[cfg(target_os = "macos")]
fn open_app(path: &str, args: Option<&str>) -> std::io::Result<()> {
    let mut cmd = Command::new("open");
    cmd.arg("-a").arg(path);
    let parts = split_args(args);
    if !parts.is_empty() {
        // `open -a App --args ...` forwards the trailing args to the app.
        cmd.arg("--args");
        cmd.args(parts);
    }
    cmd.spawn().map(|_| ())
}

#[cfg(target_os = "windows")]
fn open_url(url: &str) -> std::io::Result<()> {
    // The empty title argument keeps `start` from interpreting the URL as
    // a window title when the URL contains spaces.
    Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .map(|_| ())
}

#[cfg(target_os = "windows")]
fn open_app(path: &str, args: Option<&str>) -> std::io::Result<()> {
    let mut cmd = Command::new(path);
    cmd.args(split_args(args));
    cmd.spawn().map(|_| ())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn open_url(url: &str) -> std::io::Result<()> {
    Command::new("xdg-open").arg(url).spawn().map(|_| ())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn open_app(path: &str, args: Option<&str>) -> std::io::Result<()> {
    let mut cmd = Command::new(path);
    cmd.args(split_args(args));
    cmd.spawn().map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_args_handles_none_and_whitespace() {
        assert!(split_args(None).is_empty());
        assert!(split_args(Some("")).is_empty());
        assert!(split_args(Some("   ")).is_empty());
        assert_eq!(split_args(Some("a b  c")), vec!["a", "b", "c"]);
        assert_eq!(
            split_args(Some("--profile work https://example.com")),
            vec!["--profile", "work", "https://example.com"]
        );
    }

    #[test]
    fn url_detection_recognises_common_schemes() {
        assert!(looks_like_url("https://example.com"));
        assert!(looks_like_url("http://example.com"));
        assert!(looks_like_url("file:///etc/hosts"));
        assert!(looks_like_url("mailto:nobody@example.com"));
        assert!(!looks_like_url("example.com"));
        assert!(!looks_like_url("/Applications/Calculator.app"));
    }
}
