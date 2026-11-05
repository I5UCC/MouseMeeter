use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use windows::Win32::System::SystemInformation::GetSystemTime;

/// Once the log reaches this size, its oldest lines are dropped.
const MAX_LOG_SIZE: usize = 2 * 1024 * 1024;
/// How much of the newest log is kept when it is trimmed. Trimming rewrites the file, so it
/// keeps well below the maximum to not have to do that again for a while.
const TRIMMED_LOG_SIZE: usize = 1024 * 1024;

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
    append(path, &line, MAX_LOG_SIZE, TRIMMED_LOG_SIZE);
}

fn append(path: &Path, line: &str, max_size: usize, trimmed_size: usize) {
    if std::fs::metadata(path).is_ok_and(|m| m.len() >= max_size as u64) {
        trim_front(path, trimmed_size);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
}

/// Drops the oldest lines so at most `keep` bytes remain, starting at a whole line.
fn trim_front(path: &Path, keep: usize) {
    let Ok(log) = std::fs::read(path) else { return };
    let tail = &log[log.len().saturating_sub(keep)..];
    let start = tail
        .iter()
        .position(|&b| b == b'\n')
        .map_or(tail.len(), |i| i + 1);
    let _ = std::fs::write(path, &tail[start..]);
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::platform::logger::write(&format!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_log_drops_its_oldest_lines() {
        let dir = std::env::temp_dir().join(format!("mousemeeter-log-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("log.txt");
        let read = || std::fs::read_to_string(&path).unwrap();

        for line in ["aaaa\n", "bbbb\n", "cccc\n", "dddd\n"] {
            append(&path, line, 20, 12);
        }
        assert_eq!(read(), "aaaa\nbbbb\ncccc\ndddd\n");

        // 20 bytes reached: the newest 12 bytes start mid-line, so only whole lines are kept.
        append(&path, "eeee\n", 20, 12);
        assert_eq!(read(), "cccc\ndddd\neeee\n");

        std::fs::remove_dir_all(&dir).unwrap();
    }
}
