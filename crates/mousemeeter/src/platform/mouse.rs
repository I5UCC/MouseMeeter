//! Low-level mouse hook that drives `gestures::mouse` and swallows the events it decides to.

use std::cell::RefCell;

use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HHOOK, MSLLHOOKSTRUCT, SetWindowsHookExW, UnhookWindowsHookEx, WH_MOUSE_LL,
    WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN,
    WM_XBUTTONUP,
};

use crate::action::Action;
use crate::config::Outputs;
use crate::gestures::mouse::{MouseEvent, MouseGesture, XButton};
use crate::use_cases::window::WindowFilter;

pub struct Settings {
    pub enabled: bool,
    pub outputs: Outputs,
    pub bypass: WindowFilter,
}

fn translate(msg: u32, mouse_data: u32) -> Option<MouseEvent> {
    let high = (mouse_data >> 16) as u16;
    let xbutton = || match high {
        1 => Some(XButton::X1),
        2 => Some(XButton::X2),
        _ => None,
    };
    match msg {
        WM_XBUTTONDOWN => xbutton().map(MouseEvent::XDown),
        WM_XBUTTONUP => xbutton().map(MouseEvent::XUp),
        WM_MOUSEWHEEL => Some(MouseEvent::Wheel {
            up: high as i16 > 0,
        }),
        WM_LBUTTONDOWN => Some(MouseEvent::LeftDown),
        WM_RBUTTONDOWN => Some(MouseEvent::RightDown),
        WM_MBUTTONDOWN => Some(MouseEvent::MiddleDown),
        WM_RBUTTONUP => Some(MouseEvent::RightUp),
        _ => None,
    }
}

struct Hook {
    handle: HHOOK,
    gesture: MouseGesture,
    settings: Settings,
    on_action: Box<dyn Fn(Action)>,
}

impl Hook {
    /// Returns whether to swallow the event.
    fn on_event(&mut self, event: MouseEvent) -> bool {
        if !self.settings.enabled {
            self.gesture = MouseGesture::default();
            return false;
        }
        let settings = &self.settings;
        let (block, action) = self.gesture.on_event(event, &settings.outputs, || {
            settings.bypass.matches_foreground_window()
        });
        if let Some(action) = action {
            (self.on_action)(action);
        }
        block
    }
}

thread_local! {
    static HOOK: RefCell<Option<Hook>> = const { RefCell::new(None) };
}

/// Installs the hook on the calling thread, which must pump messages. `on_action` runs inside
/// the hook, so it must return quickly.
pub fn install(settings: Settings, on_action: Box<dyn Fn(Action)>) -> windows::core::Result<()> {
    let handle = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(hook_proc), None, 0)? };
    HOOK.with_borrow_mut(|h| {
        *h = Some(Hook {
            handle,
            gesture: MouseGesture::default(),
            settings,
            on_action,
        })
    });
    Ok(())
}

pub fn uninstall() {
    if let Some(hook) = HOOK.with_borrow_mut(Option::take) {
        let _ = unsafe { UnhookWindowsHookEx(hook.handle) };
    }
}

pub fn update_settings(settings: Settings) {
    HOOK.with_borrow_mut(|h| {
        if let Some(h) = h {
            h.settings = settings;
        }
    });
}

pub fn set_enabled(enabled: bool) {
    HOOK.with_borrow_mut(|h| {
        if let Some(h) = h {
            h.settings.enabled = enabled;
        }
    });
}

/// Side buttons currently held as part of a gesture: (Backward, Forward).
pub fn held_buttons() -> (bool, bool) {
    HOOK.with_borrow(|h| {
        h.as_ref()
            .filter(|h| h.settings.enabled)
            .map_or((false, false), |h| h.gesture.held())
    })
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let info = unsafe { &*(lparam.0 as *const MSLLHOOKSTRUCT) };
        if let Some(event) = translate(wparam.0 as u32, info.mouseData) {
            let block = HOOK.with(|hook| match hook.try_borrow_mut() {
                Ok(mut hook) => hook.as_mut().is_some_and(|h| h.on_event(event)),
                Err(_) => false,
            });
            if block {
                return LRESULT(1);
            }
        }
    }
    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use MouseEvent::*;
    use XButton::*;

    #[test]
    fn message_translation() {
        assert_eq!(translate(WM_XBUTTONDOWN, 1 << 16), Some(XDown(X1)));
        assert_eq!(translate(WM_XBUTTONUP, 2 << 16), Some(XUp(X2)));
        assert_eq!(translate(WM_XBUTTONUP, 3 << 16), None);
        assert_eq!(
            translate(WM_MOUSEWHEEL, 120 << 16),
            Some(Wheel { up: true })
        );
        assert_eq!(
            translate(WM_MOUSEWHEEL, (-120i16 as u16 as u32) << 16),
            Some(Wheel { up: false })
        );
    }
}
