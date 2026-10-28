//! Processes and windows: finding, tuning and killing processes, elevation, and paths.

use std::ffi::c_void;
use std::path::{Path, PathBuf};

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    BELOW_NORMAL_PRIORITY_CLASS, GetCurrentProcess, HIGH_PRIORITY_CLASS, OpenProcess,
    OpenProcessToken, PROCESS_ACCESS_RIGHTS, PROCESS_QUERY_LIMITED_INFORMATION,
    PROCESS_SET_INFORMATION, PROCESS_TERMINATE, SetPriorityClass, SetProcessAffinityMask,
    TerminateProcess,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId, SW_SHOWNORMAL,
    SetForegroundWindow,
};
use windows::core::{HSTRING, PCWSTR, w};

pub type Result<T> = windows::core::Result<T>;

pub fn exe_path() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("Mousemeeter.exe"))
}

/// Where config, profiles and the log live.
pub fn exe_dir() -> PathBuf {
    exe_path()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Ids of running processes whose executable matches one of `names` (without `.exe`).
pub fn find_pids(names: &[&str]) -> Vec<u32> {
    let mut pids = Vec::new();
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return pids;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut found = Process32FirstW(snapshot, &mut entry).is_ok();
        while found {
            let len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            let exe = String::from_utf16_lossy(&entry.szExeFile[..len]);
            if name_matches(&exe, names) {
                pids.push(entry.th32ProcessID);
            }
            found = Process32NextW(snapshot, &mut entry).is_ok();
        }
        let _ = CloseHandle(snapshot);
    }
    pids
}

fn name_matches(exe: &str, names: &[&str]) -> bool {
    let stem = match exe.len().checked_sub(4) {
        Some(i) if exe.is_char_boundary(i) && exe[i..].eq_ignore_ascii_case(".exe") => &exe[..i],
        _ => exe,
    };
    names.iter().any(|name| name.eq_ignore_ascii_case(stem))
}

fn with_process<T>(
    pid: u32,
    access: PROCESS_ACCESS_RIGHTS,
    f: impl FnOnce(HANDLE) -> Result<T>,
) -> Result<T> {
    unsafe {
        let handle = OpenProcess(access, false, pid)?;
        let result = f(handle);
        let _ = CloseHandle(handle);
        result
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Priority {
    High,
    BelowNormal,
}

impl Priority {
    fn class(self) -> windows::Win32::System::Threading::PROCESS_CREATION_FLAGS {
        match self {
            Priority::High => HIGH_PRIORITY_CLASS,
            Priority::BelowNormal => BELOW_NORMAL_PRIORITY_CLASS,
        }
    }
}

pub fn set_priority(pid: u32, priority: Priority) -> Result<()> {
    with_process(pid, PROCESS_SET_INFORMATION, |h| unsafe {
        SetPriorityClass(h, priority.class())
    })
}

pub fn set_own_priority(priority: Priority) -> Result<()> {
    unsafe { SetPriorityClass(GetCurrentProcess(), priority.class()) }
}

pub fn pin_to_first_core(pid: u32) -> Result<()> {
    with_process(
        pid,
        PROCESS_SET_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION,
        |h| unsafe { SetProcessAffinityMask(h, 1) },
    )
}

pub fn terminate(pid: u32) -> Result<()> {
    with_process(pid, PROCESS_TERMINATE, |h| unsafe {
        TerminateProcess(h, 1)
    })
}

pub fn foreground_process_id() -> Option<u32> {
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(GetForegroundWindow(), Some(&mut pid)) };
    (pid != 0).then_some(pid)
}

pub fn foreground_window_title() -> String {
    let mut buf = [0u16; 512];
    let len = unsafe { GetWindowTextW(GetForegroundWindow(), &mut buf) };
    String::from_utf16_lossy(&buf[..len.max(0) as usize])
}

/// Brings the top-level window with this title to the front.
pub fn focus_window(title: &str) {
    unsafe {
        if let Ok(hwnd) = FindWindowW(PCWSTR::null(), &HSTRING::from(title)) {
            let _ = SetForegroundWindow(hwnd);
        }
    }
}

pub fn is_elevated() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }
        let mut elevation = TOKEN_ELEVATION::default();
        let mut len = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut c_void),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}

/// Starts an elevated copy of this exe. Returns false if the UAC prompt was declined or failed.
pub fn relaunch_elevated() -> bool {
    let exe = HSTRING::from(exe_path().as_os_str());
    let dir = HSTRING::from(exe_dir().as_os_str());
    let result =
        unsafe { ShellExecuteW(None, w!("runas"), &exe, PCWSTR::null(), &dir, SW_SHOWNORMAL) };
    result.0 as isize > 32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_names_match_case_insensitively_without_extension() {
        let names = ["voicemeeter8x64", "audiodg"];
        assert!(name_matches("voicemeeter8x64.exe", &names));
        assert!(name_matches("VoiceMeeter8x64.EXE", &names));
        assert!(name_matches("audiodg", &names));
        assert!(!name_matches("voicemeeter8.exe", &names));
        assert!(!name_matches("exe", &names));
    }
}
