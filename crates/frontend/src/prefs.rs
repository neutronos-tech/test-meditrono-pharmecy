//! Per-device display preferences, kept in a small text file beside the local
//! database. Unreadable or missing preferences fall back to defaults.

use crate::theme::ThemeMode;
use std::{fs, path::Path};

pub fn load_theme(path: &Path) -> ThemeMode {
    match fs::read_to_string(path) {
        Ok(contents) if contents.lines().any(|line| line.trim() == "theme=dark") => ThemeMode::Dark,
        _ => ThemeMode::Light,
    }
}

pub fn save_theme(path: &Path, mode: ThemeMode) {
    let value = match mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    };
    // Losing a display preference is harmless, so a failed write is ignored.
    let _ = fs::write(path, format!("theme={value}\n"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_round_trips_and_defaults_to_light() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("preferences.txt");
        assert_eq!(load_theme(&path), ThemeMode::Light);
        save_theme(&path, ThemeMode::Dark);
        assert_eq!(load_theme(&path), ThemeMode::Dark);
        save_theme(&path, ThemeMode::Light);
        assert_eq!(load_theme(&path), ThemeMode::Light);
    }
}
