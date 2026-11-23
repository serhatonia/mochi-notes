//! Cross-platform "open in the system app" helpers.

use std::path::Path;
use std::process::Command;

/// Opens a URL in the user's default browser.
pub fn url(url: &str) {
    #[cfg(target_os = "windows")]
    let launched = Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .is_ok();

    #[cfg(target_os = "macos")]
    let launched = Command::new("open").arg(url).spawn().is_ok();

    #[cfg(all(unix, not(target_os = "macos")))]
    let launched = Command::new("xdg-open").arg(url).spawn().is_ok();

    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    let launched = false;

    if !launched {
        eprintln!("Could not open URL: {url}");
    }
}

/// Reveals a folder in the system file manager.
pub fn folder(path: &Path) {
    #[cfg(target_os = "windows")]
    let launched = Command::new("explorer").arg(path).spawn().is_ok();

    #[cfg(target_os = "macos")]
    let launched = Command::new("open").arg(path).spawn().is_ok();

    #[cfg(all(unix, not(target_os = "macos")))]
    let launched = Command::new("xdg-open").arg(path).spawn().is_ok();

    #[cfg(not(any(target_os = "windows", target_os = "macos", unix)))]
    let launched = false;

    if !launched {
        eprintln!("Could not open folder: {}", path.display());
    }
}
