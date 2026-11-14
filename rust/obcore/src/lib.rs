//! Shared code of the OpenBlade-Synapse daemon and GUI: data paths, key names,
//! mouse model database (devices/*.json) and translations (i18n/*.json).

pub mod devices;
pub mod i18n;
pub mod keys;

use std::path::{Path, PathBuf};

pub const VERSION: &str = "0.1.0";
pub const YEAR: u32 = 2026;
pub const AUTHORS: &str = "LT & Claude";
pub const APP_ID: &str = "openblade-synapse";

/// Directory with qml/, assets/, i18n/, devices/ (the project's `app` folder).
/// `$OPENBLADE_DATA` wins; otherwise walk up from the executable (rust/target/release -> project/app).
pub fn data_dir() -> PathBuf {
    if let Ok(d) = std::env::var("OPENBLADE_DATA") {
        return PathBuf::from(d);
    }
    if let Ok(exe) = std::env::current_exe() {
        let mut dir: Option<&Path> = exe.parent();
        while let Some(d) = dir {
            let cand = d.join("app");
            if cand.join("i18n").is_dir() && cand.join("devices").is_dir() {
                return cand;
            }
            if d.join("i18n").is_dir() && d.join("devices").is_dir() {
                return d.to_path_buf();
            }
            dir = d.parent();
        }
    }
    PathBuf::from("app")
}

fn xdg(var: &str, fallback: &str) -> PathBuf {
    std::env::var(var)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(fallback))
}

pub fn config_dir() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config").join(APP_ID)
}

pub fn cache_dir() -> PathBuf {
    xdg("XDG_CACHE_HOME", ".cache").join(APP_ID)
}

/// Unix socket the daemon listens on and the GUI connects to.
pub fn socket_path() -> PathBuf {
    let base = std::env::var("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(|_| std::env::temp_dir());
    base.join(format!("{APP_ID}.sock"))
}
