#![windows_subsystem = "windows"]

//! Mousemeeter: mouse side buttons and hotkeys for Voicemeeter.
//!
//! - `action`: everything the app can do
//! - `gestures`: mouse and keyboard input → actions
//! - `use_cases`: what each action does
//! - `voicemeeter`: the Voicemeeter connection and its thread
//! - `platform`: Win32 hooks, tray, processes, logging
//! - `app`: window, message loop and wiring
//! - `settings_window`: the settings UI (`--settings`)

mod action;
mod app;
mod config;
mod gestures;
mod platform;
mod settings_window;
mod use_cases;
mod voicemeeter;

use config::Config;
use platform::{logger, process};

/// Application version (overridable at build time via `MOUSEMEETER_VERSION`).
const VERSION: &str = match option_env!("MOUSEMEETER_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};

fn main() {
    let dir = process::exe_dir();

    // Started from the tray; leaves the tray app's log alone. Exit code 0 tells it to reload
    // the config.
    if std::env::args().any(|arg| arg == settings_window::ARG) {
        let saved = settings_window::run(&dir);
        std::process::exit(if saved { 0 } else { 1 });
    }

    logger::init(dir.join("log.txt"));
    log!("Mousemeeter {VERSION}");
    let config = Config::load(&dir.join("config.ini"));

    if config.run_as_admin && !process::is_elevated() {
        if process::relaunch_elevated() {
            return;
        }
        log!("Could not restart with administrator rights; continuing without them");
    }

    if let Err(e) = app::run(dir, config) {
        log!("Failed to initialize application: {e}");
    }
}
