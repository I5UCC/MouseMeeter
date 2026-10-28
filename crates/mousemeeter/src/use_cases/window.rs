//! Killing the foreground app (Ctrl+Alt+F4) and disabling the mouse gestures over the windows
//! listed in `[DeactivateOnWindow]`.

use crate::config::Config;
use crate::log;
use crate::platform::process;

pub fn kill_foreground_app() {
    let Some(pid) = process::foreground_process_id() else {
        return;
    };
    if pid == std::process::id() {
        return;
    }
    match process::terminate(pid) {
        Ok(()) => log!("Killed foreground process (PID: {pid})"),
        Err(e) => log!("Failed to kill active window: {e}"),
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct WindowFilter {
    titles: Vec<String>,
    match_mode: u8,
}

impl WindowFilter {
    pub fn new(config: &Config) -> WindowFilter {
        WindowFilter {
            titles: config.deactivate_on_window.clone(),
            match_mode: config.title_match_mode,
        }
    }

    pub fn matches(&self, title: &str) -> bool {
        self.titles
            .iter()
            .any(|pattern| title_matches(title, pattern, self.match_mode))
    }

    pub fn matches_foreground_window(&self) -> bool {
        !self.titles.is_empty() && self.matches(&process::foreground_window_title())
    }
}

/// AutoHotkey `SetTitleMatchMode` semantics: 1 = starts with, 2 = contains, 3 = exact.
fn title_matches(title: &str, pattern: &str, mode: u8) -> bool {
    if pattern.is_empty() {
        return false;
    }
    match mode {
        1 => title.starts_with(pattern),
        2 => title.contains(pattern),
        _ => title == pattern,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_match_modes() {
        assert!(title_matches("Game - Window", "Game", 1));
        assert!(!title_matches("My Game", "Game", 1));
        assert!(title_matches("My Game", "Game", 2));
        assert!(!title_matches("My Game", "game", 2));
        assert!(title_matches("Game", "Game", 3));
        assert!(!title_matches("Game ", "Game", 3));
        assert!(!title_matches("anything", "", 2));
    }

    #[test]
    fn filter_uses_config() {
        let config =
            Config::parse("[Settings]\nTitleMatchMode=2\n[DeactivateOnWindow]\n1=Game\n2=Editor\n");
        let filter = WindowFilter::new(&config);
        assert!(filter.matches("My Game"));
        assert!(filter.matches("Text Editor"));
        assert!(!filter.matches("Browser"));
        assert!(!WindowFilter::default().matches("Game"));
    }
}
