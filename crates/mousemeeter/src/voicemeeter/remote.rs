//! VoicemeeterRemote64.dll, implementing `Mixer`.

use std::ffi::{CString, OsStr, c_char};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::thread;
use std::time::Duration;

use libloading::os::windows::{LOAD_WITH_ALTERED_SEARCH_PATH, Library};

use crate::log;
use crate::use_cases::Mixer;

type NoArgFn = unsafe extern "C" fn() -> i32;
type GetFloatFn = unsafe extern "C" fn(*const c_char, *mut f32) -> i32;
type SetFloatFn = unsafe extern "C" fn(*const c_char, f32) -> i32;
type SetStringWFn = unsafe extern "C" fn(*const c_char, *const u16) -> i32;

/// Function pointers into the DLL; `_lib` keeps it loaded.
struct RemoteApi {
    _lib: Library,
    login: NoArgFn,
    logout: NoArgFn,
    is_dirty: NoArgFn,
    get_float: GetFloatFn,
    set_float: SetFloatFn,
    set_string_w: SetStringWFn,
}

impl RemoteApi {
    fn load(dir: &Path) -> Result<Self, String> {
        let path = dir.join("VoicemeeterRemote64.dll");
        let lib = unsafe { Library::load_with_flags(&path, LOAD_WITH_ALTERED_SEARCH_PATH) }
            .map_err(|e| format!("{}: {e}", path.display()))?;

        unsafe {
            Ok(RemoteApi {
                login: *lib.get("VBVMR_Login").map_err(|e| e.to_string())?,
                logout: *lib.get("VBVMR_Logout").map_err(|e| e.to_string())?,
                is_dirty: *lib
                    .get("VBVMR_IsParametersDirty")
                    .map_err(|e| e.to_string())?,
                get_float: *lib
                    .get("VBVMR_GetParameterFloat")
                    .map_err(|e| e.to_string())?,
                set_float: *lib
                    .get("VBVMR_SetParameterFloat")
                    .map_err(|e| e.to_string())?,
                set_string_w: *lib
                    .get("VBVMR_SetParameterStringW")
                    .map_err(|e| e.to_string())?,
                _lib: lib,
            })
        }
    }

    /// Also makes Voicemeeter refresh the values `get_float` returns.
    fn is_dirty(&self) -> bool {
        unsafe { (self.is_dirty)() != 0 }
    }

    fn get_float(&self, name: &str) -> Option<f32> {
        let name = CString::new(name).ok()?;
        let mut value = 0.0;
        (unsafe { (self.get_float)(name.as_ptr(), &mut value) } == 0).then_some(value)
    }

    fn set_float(&self, name: &str, value: f32) -> Result<(), String> {
        let name = CString::new(name).map_err(|e| e.to_string())?;
        match unsafe { (self.set_float)(name.as_ptr(), value) } {
            0 => Ok(()),
            code => Err(format!("Voicemeeter returned {code}")),
        }
    }

    fn set_string(&self, name: &str, value: &OsStr) -> Result<(), String> {
        let name = CString::new(name).map_err(|e| e.to_string())?;
        let value: Vec<u16> = value.encode_wide().chain(Some(0)).collect();
        match unsafe { (self.set_string_w)(name.as_ptr(), value.as_ptr()) } {
            0 => Ok(()),
            code => Err(format!("Voicemeeter returned {code}")),
        }
    }
}

pub struct Voicemeeter {
    api: RemoteApi,
    connected: bool,
}

impl Voicemeeter {
    pub fn connect(dll_dir: &Path) -> Result<Voicemeeter, String> {
        let api = RemoteApi::load(dll_dir)?;
        let result = unsafe { (api.login)() };
        let connected = matches!(result, 0 | 1);
        if connected {
            log!("Connected to Voicemeeter");
        } else {
            log!("Failed to connect to Voicemeeter (login returned {result})");
        }
        Ok(Voicemeeter { api, connected })
    }

    pub fn disconnect(mut self) {
        if self.connected {
            unsafe { (self.api.logout)() };
            self.connected = false;
            log!("Disconnected from Voicemeeter");
        }
    }

    fn ensure_connected(&self) -> Result<(), String> {
        if self.connected {
            Ok(())
        } else {
            Err("not connected to Voicemeeter".into())
        }
    }

    /// Polls with exponential backoff until the dirty flag is `want` or the backoff passes
    /// `max_delay_ms`.
    fn wait_dirty_state(&self, want: bool, max_delay_ms: u64) {
        let mut delay = 10;
        while self.api.is_dirty() != want {
            if delay >= max_delay_ms {
                let state = if want { "dirty" } else { "clean" };
                log!("Timeout waiting for Voicemeeter parameters to become {state}.");
                return;
            }
            thread::sleep(Duration::from_millis(delay));
            delay *= 2;
        }
    }

    fn read(&self, name: &str) -> Option<f32> {
        if !self.connected {
            return None;
        }
        self.api.is_dirty();
        self.api.get_float(name)
    }
}

impl Mixer for Voicemeeter {
    fn wait_until_settled(&mut self) {
        if self.connected {
            self.wait_dirty_state(true, 2000);
            self.wait_dirty_state(false, 4000);
        }
    }

    fn gain(&mut self, strip: i32) -> Option<f32> {
        self.read(&format!("Strip[{strip}].Gain"))
    }

    fn set_gain(&mut self, strip: i32, gain: f32) -> Result<(), String> {
        self.ensure_connected()?;
        self.api.set_float(&format!("Strip[{strip}].Gain"), gain)
    }

    fn is_muted(&mut self, strip: i32) -> Option<bool> {
        self.read(&format!("Strip[{strip}].Mute")).map(|m| m != 0.0)
    }

    fn set_muted(&mut self, strip: i32, muted: bool) -> Result<(), String> {
        self.ensure_connected()?;
        let value = if muted { 1.0 } else { 0.0 };
        self.api.set_float(&format!("Strip[{strip}].Mute"), value)
    }

    fn load_profile(&mut self, path: &Path) -> Result<(), String> {
        self.ensure_connected()?;
        if !path.is_file() {
            return Err("file not found".into());
        }
        self.api.set_string("Command.Load", path.as_os_str())
    }

    fn restart_engine(&mut self) -> Result<(), String> {
        self.ensure_connected()?;
        self.api.set_string("Command.Restart", "1".as_ref())
    }
}
