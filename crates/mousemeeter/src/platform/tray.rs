use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, POINT, WPARAM};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, GetCursorPos, GetSystemMetrics, HICON, HMENU, IDI_APPLICATION,
    IMAGE_ICON, LR_DEFAULTCOLOR, LoadIconW, LoadImageW, MF_BYCOMMAND, MF_SEPARATOR, MF_STRING,
    ModifyMenuW, PostMessageW, SM_CXSMICON, SM_CYSMICON, SetForegroundWindow, TPM_RETURNCMD,
    TPM_RIGHTBUTTON, TrackPopupMenu, WM_NULL,
};
use windows::core::{PCWSTR, w};

pub const ID_TOGGLE: u32 = 1;
pub const ID_RELOAD: u32 = 2;
pub const ID_REFRESH_CONFIG: u32 = 3;
pub const ID_OPEN_CONFIG: u32 = 4;
pub const ID_EXIT: u32 = 5;
pub const ID_SETTINGS: u32 = 6;

/// Matches the icon resource id in build.rs.
const ICON_RESOURCE_ID: u16 = 1;

#[derive(Clone, Copy)]
pub struct Tray {
    hwnd: HWND,
    icon: HICON,
    menu: HMENU,
    callback_message: u32,
}

impl Tray {
    pub fn new(
        hwnd: HWND,
        instance: HINSTANCE,
        callback_message: u32,
    ) -> windows::core::Result<Tray> {
        unsafe {
            let icon = LoadImageW(
                Some(instance),
                PCWSTR(ICON_RESOURCE_ID as usize as *const u16),
                IMAGE_ICON,
                GetSystemMetrics(SM_CXSMICON),
                GetSystemMetrics(SM_CYSMICON),
                LR_DEFAULTCOLOR,
            )
            .map(|h| HICON(h.0))
            .or_else(|_| LoadIconW(None, IDI_APPLICATION))?;

            let menu = CreatePopupMenu()?;
            AppendMenuW(menu, MF_STRING, ID_TOGGLE as usize, w!("Disable"))?;
            AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())?;
            AppendMenuW(menu, MF_STRING, ID_SETTINGS as usize, w!("Settings…"))?;
            AppendMenuW(menu, MF_STRING, ID_OPEN_CONFIG as usize, w!("Open Config"))?;
            AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())?;
            AppendMenuW(menu, MF_STRING, ID_RELOAD as usize, w!("Reload"))?;
            AppendMenuW(
                menu,
                MF_STRING,
                ID_REFRESH_CONFIG as usize,
                w!("Refresh Config"),
            )?;
            AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null())?;
            AppendMenuW(menu, MF_STRING, ID_EXIT as usize, w!("Exit"))?;

            Ok(Tray {
                hwnd,
                icon,
                menu,
                callback_message,
            })
        }
    }

    fn data(&self) -> NOTIFYICONDATAW {
        let mut data = NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: self.callback_message,
            hIcon: self.icon,
            ..Default::default()
        };
        for (dst, src) in data.szTip.iter_mut().zip("Mousemeeter".encode_utf16()) {
            *dst = src;
        }
        data
    }

    /// Also used to restore the icon after Explorer restarts.
    pub fn show(&self) -> bool {
        unsafe { Shell_NotifyIconW(NIM_ADD, &self.data()).as_bool() }
    }

    pub fn remove(&self) {
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.data());
        }
    }

    pub fn set_active(&self, active: bool) {
        let label = if active { w!("Disable") } else { w!("Enable") };
        unsafe {
            let _ = ModifyMenuW(
                self.menu,
                ID_TOGGLE,
                MF_BYCOMMAND | MF_STRING,
                ID_TOGGLE as usize,
                label,
            );
        }
    }

    /// Shows the context menu at the cursor and returns the chosen item id (0 if dismissed).
    /// Runs a modal loop, so the caller must not hold any `RefCell` borrows the window
    /// procedure or hooks need.
    pub fn show_menu(&self) -> u32 {
        unsafe {
            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);
            // Required so the menu closes when clicking elsewhere.
            let _ = SetForegroundWindow(self.hwnd);
            let id = TrackPopupMenu(
                self.menu,
                TPM_RETURNCMD | TPM_RIGHTBUTTON,
                pt.x,
                pt.y,
                None,
                self.hwnd,
                None,
            );
            let _ = PostMessageW(Some(self.hwnd), WM_NULL, WPARAM(0), LPARAM(0));
            id.0 as u32
        }
    }
}
