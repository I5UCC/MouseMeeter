use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use windows::Win32::System::SystemInformation::GetSystemTime;

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
static LOCK: Mutex<()> = Mutex::new(());

/// Sets the log file and clears whatever the previous run left in it.
pub fn init(path: PathBuf) {
    let _ = std::fs::remove_file(&path);
    let _ = LOG_PATH.set(path);
}

pub fn write(message: &str) {
    let Some(path) = LOG_PATH.get() else { return };

    let t = unsafe { GetSystemTime() };
    let line = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03} {message}\r\n",
        t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond, t.wMilliseconds
    );

    let _guard = LOCK.lock().unwrap_or_else(|e| e.into_inner());
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::platform::logger::write(&format!($($arg)*))
    };
}
