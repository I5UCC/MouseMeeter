//! Low-level keyboard hook that reports hotkey presses, and synthetic key taps.

use std::cell::RefCell;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT,
    KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, SendInput, VIRTUAL_KEY, VK_CONTROL, VK_F4, VK_F15,
    VK_F24, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE, VK_MEDIA_PREV_TRACK, VK_MENU, VK_R, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HHOOK, KBDLLHOOKSTRUCT, SetWindowsHookExW, UnhookWindowsHookEx, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::action::MediaKey;
use crate::gestures::hotkeys::Hotkey;

/// The key combination bound to each hotkey.
fn hotkey_for(vk: VIRTUAL_KEY, ctrl: bool, shift: bool, alt: bool) -> Option<Hotkey> {
    match vk {
        VK_F24 => Some(Hotkey::Profile),
        VK_F15 => Some(Hotkey::ToggleEnabled),
        VK_F4 if ctrl && alt => Some(Hotkey::KillForegroundApp),
        VK_R if ctrl && shift => Some(Hotkey::Restart),
        _ => None,
    }
}

struct Hook {
    handle: HHOOK,
    on_hotkey: Box<dyn Fn(Hotkey)>,
    down: [bool; 256],
}

thread_local! {
    static HOOK: RefCell<Option<Hook>> = const { RefCell::new(None) };
}

/// Installs the hook on the calling thread, which must pump messages. `on_hotkey` runs inside
/// the hook, so it must return quickly. Keys are never swallowed.
pub fn install(on_hotkey: Box<dyn Fn(Hotkey)>) -> windows::core::Result<()> {
    let handle = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), None, 0)? };
    HOOK.with_borrow_mut(|h| {
        *h = Some(Hook {
            handle,
            on_hotkey,
            down: [false; 256],
        })
    });
    Ok(())
}

pub fn uninstall() {
    if let Some(hook) = HOOK.with_borrow_mut(Option::take) {
        let _ = unsafe { UnhookWindowsHookEx(hook.handle) };
    }
}

fn is_pressed(vk: VIRTUAL_KEY) -> bool {
    unsafe { GetAsyncKeyState(vk.0 as i32) as u16 & 0x8000 != 0 }
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let info = unsafe { &*(lparam.0 as *const KBDLLHOOKSTRUCT) };
        let vk = (info.vkCode & 0xFF) as usize;

        HOOK.with(|hook| {
            let Ok(mut hook) = hook.try_borrow_mut() else {
                return;
            };
            let Some(hook) = hook.as_mut() else { return };
            match wparam.0 as u32 {
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    // Auto-repeat sends more key-downs; only the first one counts.
                    if std::mem::replace(&mut hook.down[vk], true) {
                        return;
                    }
                    let hotkey = hotkey_for(
                        VIRTUAL_KEY(vk as u16),
                        is_pressed(VK_CONTROL),
                        is_pressed(VK_SHIFT),
                        is_pressed(VK_MENU),
                    );
                    if let Some(hotkey) = hotkey {
                        (hook.on_hotkey)(hotkey);
                    }
                }
                WM_KEYUP | WM_SYSKEYUP => hook.down[vk] = false,
                _ => {}
            }
        });
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

/// Presses and releases a media key.
pub fn tap_media_key(key: MediaKey) {
    let vk = match key {
        MediaKey::Previous => VK_MEDIA_PREV_TRACK,
        MediaKey::Next => VK_MEDIA_NEXT_TRACK,
        MediaKey::PlayPause => VK_MEDIA_PLAY_PAUSE,
    };
    let input = |flags: KEYBD_EVENT_FLAGS| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let inputs = [
        input(KEYEVENTF_EXTENDEDKEY),
        input(KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP),
    ];
    unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_combinations() {
        assert_eq!(
            hotkey_for(VK_F24, false, false, false),
            Some(Hotkey::Profile)
        );
        assert_eq!(
            hotkey_for(VK_F15, true, true, true),
            Some(Hotkey::ToggleEnabled)
        );
        assert_eq!(
            hotkey_for(VK_F4, true, false, true),
            Some(Hotkey::KillForegroundApp)
        );
        assert_eq!(hotkey_for(VK_F4, false, false, true), None);
        assert_eq!(hotkey_for(VK_R, true, true, false), Some(Hotkey::Restart));
        assert_eq!(hotkey_for(VK_R, true, false, false), None);
        assert_eq!(hotkey_for(VK_R, false, false, false), None);
    }
}
