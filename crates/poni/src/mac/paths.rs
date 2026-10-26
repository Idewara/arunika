//! Where Arunika keeps its files on a Mac.
//!
//! - Settings + data: `~/Library/Application Support/Arunika/`
//! - Cache:           `~/Library/Caches/Arunika/`
//! - Log:             `~/Library/Logs/Arunika/arunika.log` (Console.app lists it)
//!
//! `ARUNIKA_HOME` overrides all three (used by `--render` and `--smoke` to stay
//! isolated).

use std::path::PathBuf;

fn home_override() -> Option<PathBuf> {
    std::env::var_os("ARUNIKA_HOME").map(PathBuf::from)
}

fn library() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")).join("Library")
}

pub fn data_dir() -> PathBuf {
    home_override().unwrap_or_else(|| library().join("Application Support").join("Arunika"))
}

pub fn cache_dir() -> PathBuf {
    home_override().map(|h| h.join("cache")).unwrap_or_else(|| library().join("Caches").join("Arunika"))
}

pub fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

pub fn store_path() -> PathBuf {
    data_dir().join("data.json")
}

pub fn log_path() -> PathBuf {
    home_override().map(|h| h.join("arunika.log")).unwrap_or_else(|| library().join("Logs").join("Arunika").join("arunika.log"))
}

/// Held while Arunika runs, so a second copy quits at start.
pub fn lock_path() -> PathBuf {
    cache_dir().join("arunika.lock")
}

/// Where files from a phone land (Phone drop): `~/Downloads/Arunika`. Shelf
/// items link to the files in it.
pub fn phone_inbox_dir() -> PathBuf {
    if let Some(h) = home_override() {
        return h.join("phone-drop");
    }
    let downloads = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")).join("Downloads");
    downloads.join("Arunika")
}

pub fn lyrics_cache_dir() -> PathBuf {
    cache_dir().join("lyrics")
}

/// Add-on files, one folder per add-on (`services::addons`), only there
/// while the add-on is switched on. A Mac that clears its caches downloads
/// them again.
pub fn addons_dir() -> PathBuf {
    cache_dir().join("addons")
}

/// The Oyenza sign-in (this Mac's key and the masked account), in a file
/// only this Mac account can read (`services::secret`). Signing out removes
/// it. With the settings, not the cache: macOS may empty caches by itself.
pub fn oyenza_link_path() -> PathBuf {
    data_dir().join("oyenza-link.bin")
}

/// A key signed out while Oyenza couldn't be reached, kept only to tell the
/// server once it can.
pub fn oyenza_revoke_path() -> PathBuf {
    data_dir().join("oyenza-revoke.bin")
}

/// The person's Apple Music token (owner-only file; uninstalling with the
/// cache folder removes it).
pub fn apple_music_token_path() -> PathBuf {
    cache_dir().join("apple-music-token.bin")
}

/// The Apple Music web player's (public) developer token, cached until it expires.
pub fn apple_music_web_token_path() -> PathBuf {
    cache_dir().join("apple-music-web.json")
}
