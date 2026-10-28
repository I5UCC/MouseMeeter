//! The running app: hidden window and message loop, tray menu, and routing of actions to the
//! use cases.

use std::cell::RefCell;
use std::error::Error;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Diagnostics::Debug::MessageBeep;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, KillTimer,
    MB_OK, MSG, PostMessageW, PostQuitMessage, RegisterClassW, RegisterWindowMessageW, SetTimer,
    TranslateMessage, WINDOW_EX_STYLE, WM_APP, WM_LBUTTONDBLCLK, WM_RBUTTONUP, WM_TIMER, WNDCLASSW,
    WS_OVERLAPPED,
};
use windows::core::w;

use crate::action::{Action, AppAction};
use crate::config::Config;
use crate::gestures::hotkeys::{DOUBLE_PRESS, Hotkey, Hotkeys, Timer};
use crate::log;
use crate::platform::tray::{self, Tray};
use crate::platform::{keyboard, mouse, process};
use crate::settings_window;
use crate::use_cases::window::{self, WindowFilter};
use crate::use_cases::{MixerSettings, ProfileFiles, media};
use crate::voicemeeter::{self, Request, Startup};

const WM_APP_READY: u32 = WM_APP + 1;
const WM_APP_FAILED: u32 = WM_APP + 2;
const WM_APP_HOTKEY: u32 = WM_APP + 3;
const WM_APP_TRAY: u32 = WM_APP + 4;
const WM_APP_ACTION: u32 = WM_APP + 5;
const WM_APP_SETTINGS_SAVED: u32 = WM_APP + 6;

static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);

fn post(hwnd: usize, msg: u32, wparam: usize) {
    let hwnd = HWND(hwnd as *mut c_void);
    let _ = unsafe { PostMessageW(Some(hwnd), msg, WPARAM(wparam), LPARAM(0)) };
}

/// Sends each action to where it runs: Voicemeeter actions to the Voicemeeter thread, app
/// actions back to the main thread through its message queue. Safe to call from inside a hook.
#[derive(Clone)]
struct Dispatcher {
    voicemeeter: Sender<Request>,
    app: Sender<AppAction>,
    hwnd: usize,
}

impl Dispatcher {
    fn dispatch(&self, action: Action) {
        match action {
            Action::Mixer(action) => {
                let _ = self.voicemeeter.send(Request::Run(action));
            }
            Action::App(action) => {
                if self.app.send(action).is_ok() {
                    post(self.hwnd, WM_APP_ACTION, 0);
                }
            }
        }
    }
}

struct App {
    dir: PathBuf,
    hwnd: HWND,
    tray: Tray,
    config: Config,
    enabled: bool,
    hotkeys: Hotkeys,
    dispatcher: Dispatcher,
    app_actions: Receiver<AppAction>,
    settings_open: Arc<AtomicBool>,
    reload: bool,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

/// Runs `f` on the app unless it is already borrowed further up the stack, which can happen when
/// a call re-enters the window procedure.
fn with_app<R>(f: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|app| app.try_borrow_mut().ok()?.as_mut().map(f))
}

pub fn run(dir: PathBuf, config: Config) -> Result<(), Box<dyn Error>> {
    let instance: HINSTANCE = unsafe { GetModuleHandleW(None)? }.into();
    let hwnd = create_window(instance)?;
    let tray = Tray::new(hwnd, instance, WM_APP_TRAY)?;
    tray.show();

    let dll_dir = config
        .voicemeeter_path
        .clone()
        .unwrap_or_else(default_voicemeeter_dir);
    if !dll_dir.is_dir() {
        tray.remove();
        return Err(format!("Voicemeeter directory not found: {}", dll_dir.display()).into());
    }
    log!("Voicemeeter path: {}", dll_dir.display());

    let hwnd_addr = hwnd.0 as usize;
    let startup = Startup {
        dll_dir,
        raise_priority: config.set_affinity,
        crackling_fix: config.set_crackling_fix,
        reset_to_default: config.reset_on_startup,
        settings: mixer_settings(&config, &dir),
    };
    let (voicemeeter, worker) = voicemeeter::spawn(startup, move |ok| {
        post(hwnd_addr, if ok { WM_APP_READY } else { WM_APP_FAILED }, 0);
    });
    let (app_tx, app_actions) = mpsc::channel();

    APP.with_borrow_mut(|app| {
        *app = Some(App {
            dir,
            hwnd,
            tray,
            config,
            enabled: true,
            hotkeys: Hotkeys::default(),
            dispatcher: Dispatcher {
                voicemeeter,
                app: app_tx,
                hwnd: hwnd_addr,
            },
            app_actions,
            settings_open: Arc::default(),
            reload: false,
        })
    });

    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0) }.0 > 0 {
        unsafe {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }

    mouse::uninstall();
    keyboard::uninstall();
    let app = APP.with_borrow_mut(Option::take);
    if let Some(app) = &app {
        app.tray.remove();
        let _ = app.dispatcher.voicemeeter.send(Request::Shutdown);
    }
    join_with_timeout(worker, Duration::from_secs(2));
    unsafe {
        let _ = DestroyWindow(hwnd);
    }
    log!("Mousemeeter exited");

    if let Some(app) = app
        && app.reload
        && let Err(e) = std::process::Command::new(process::exe_path())
            .current_dir(&app.dir)
            .spawn()
    {
        log!("Failed to reload: {e}");
    }
    Ok(())
}

/// A hidden top-level window rather than a message-only one, so the tray menu can take focus.
fn create_window(instance: HINSTANCE) -> windows::core::Result<HWND> {
    let class_name = w!("MousemeeterWindow");
    let class = WNDCLASSW {
        lpfnWndProc: Some(wndproc),
        hInstance: instance,
        lpszClassName: class_name,
        ..Default::default()
    };
    if unsafe { RegisterClassW(&class) } == 0 {
        return Err(windows::core::Error::from_thread());
    }
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class_name,
            w!("Mousemeeter"),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(instance),
            None,
        )?
    };
    TASKBAR_CREATED.store(
        unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) },
        Ordering::Relaxed,
    );
    Ok(hwnd)
}

fn join_with_timeout(handle: JoinHandle<()>, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    while !handle.is_finished() {
        if Instant::now() >= deadline {
            log!("Voicemeeter thread did not stop in time");
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let _ = handle.join();
}

fn default_voicemeeter_dir() -> PathBuf {
    std::env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"))
        .join("VB")
        .join("Voicemeeter")
}

fn mixer_settings(config: &Config, dir: &Path) -> MixerSettings {
    MixerSettings {
        strips: config.outputs.all(),
        volume_step: config.volume_change_amount,
        profiles: ProfileFiles {
            dir: dir.to_path_buf(),
            default: config.default_file.clone(),
            profile1: config.profile1_file.clone(),
            profile2: config.profile2_file.clone(),
        },
    }
}

fn mouse_settings(config: &Config, enabled: bool) -> mouse::Settings {
    mouse::Settings {
        enabled,
        outputs: config.outputs,
        bypass: WindowFilter::new(config),
    }
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_APP_READY => {
            with_app(App::on_ready);
        }
        WM_APP_FAILED => unsafe { PostQuitMessage(1) },
        WM_APP_ACTION => {
            with_app(App::run_app_actions);
        }
        WM_APP_HOTKEY => {
            if let Some(hotkey) = Hotkey::from_id(wparam.0) {
                with_app(|app| app.on_hotkey(hotkey));
            }
        }
        WM_TIMER => {
            with_app(|app| app.on_timer(wparam.0));
        }
        WM_APP_SETTINGS_SAVED => {
            with_app(App::refresh_config);
        }
        WM_APP_TRAY => match lparam.0 as u32 & 0xFFFF {
            WM_RBUTTONUP => {
                if let Some(tray) = with_app(|app| app.tray) {
                    // The menu runs a modal loop, so it's shown without the app borrowed.
                    let id = tray.show_menu();
                    with_app(|app| app.on_menu(id));
                }
            }
            WM_LBUTTONDBLCLK => {
                with_app(App::open_settings);
            }
            _ => {}
        },
        _ if msg != 0 && msg == TASKBAR_CREATED.load(Ordering::Relaxed) => {
            with_app(|app| app.tray.show());
        }
        _ => return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
    LRESULT(0)
}

impl App {
    fn on_ready(&mut self) {
        let dispatcher = self.dispatcher.clone();
        match mouse::install(
            mouse_settings(&self.config, self.enabled),
            Box::new(move |action| dispatcher.dispatch(action)),
        ) {
            Ok(()) => log!("Mouse hook installed successfully"),
            Err(e) => log!("Failed to install mouse hook: {e}"),
        }

        let hwnd = self.dispatcher.hwnd;
        match keyboard::install(Box::new(move |hotkey| {
            post(hwnd, WM_APP_HOTKEY, hotkey.id())
        })) {
            Ok(()) => log!("Keyboard hook installed successfully"),
            Err(e) => log!("Failed to install keyboard hook: {e}"),
        }
        log!("Mousemeeter started successfully");
    }

    fn run_app_actions(&mut self) {
        while let Ok(action) = self.app_actions.try_recv() {
            match action {
                AppAction::MediaKey(key) => media::press(key),
                AppAction::ToggleEnabled => self.toggle_enabled(),
                AppAction::KillForegroundApp => window::kill_foreground_app(),
            }
        }
    }

    fn on_hotkey(&mut self, hotkey: Hotkey) {
        let response = self
            .hotkeys
            .on_press(hotkey, mouse::held_buttons(), &self.config.outputs);
        match response.timer {
            Some(Timer::Start(hotkey)) => unsafe {
                SetTimer(
                    Some(self.hwnd),
                    hotkey.id(),
                    DOUBLE_PRESS.as_millis() as u32,
                    None,
                );
            },
            Some(Timer::Cancel(hotkey)) => unsafe {
                let _ = KillTimer(Some(self.hwnd), hotkey.id());
            },
            None => {}
        }
        if let Some(action) = response.action {
            self.dispatcher.dispatch(action);
        }
    }

    fn on_timer(&mut self, id: usize) {
        let _ = unsafe { KillTimer(Some(self.hwnd), id) };
        if let Some(action) = Hotkey::from_id(id).and_then(|h| self.hotkeys.on_timeout(h)) {
            self.dispatcher.dispatch(action);
        }
    }

    fn toggle_enabled(&mut self) {
        self.enabled = !self.enabled;
        mouse::set_enabled(self.enabled);
        self.tray.set_active(self.enabled);
        if !self.enabled {
            let _ = unsafe { MessageBeep(MB_OK) };
        }
        log!(
            "Mousemeeter {}",
            if self.enabled { "enabled" } else { "disabled" }
        );
    }

    fn on_menu(&mut self, id: u32) {
        match id {
            tray::ID_TOGGLE => self.toggle_enabled(),
            tray::ID_RELOAD => {
                self.reload = true;
                unsafe { PostQuitMessage(0) };
            }
            tray::ID_REFRESH_CONFIG => self.refresh_config(),
            tray::ID_SETTINGS => self.open_settings(),
            tray::ID_OPEN_CONFIG => self.open_config(),
            tray::ID_EXIT => unsafe { PostQuitMessage(0) },
            _ => {}
        }
    }

    fn refresh_config(&mut self) {
        self.config = Config::load(&self.dir.join("config.ini"));
        mouse::update_settings(mouse_settings(&self.config, self.enabled));
        let _ = self
            .dispatcher
            .voicemeeter
            .send(Request::Reconfigure(mixer_settings(
                &self.config,
                &self.dir,
            )));
        log!("Configuration refreshed");
    }

    /// Starts the settings window process, or brings it to the front if it is already open.
    /// The config is reloaded once it saves.
    fn open_settings(&mut self) {
        if self.settings_open.swap(true, Ordering::AcqRel) {
            process::focus_window(settings_window::TITLE);
            return;
        }
        let child = std::process::Command::new(process::exe_path())
            .arg(settings_window::ARG)
            .current_dir(&self.dir)
            .spawn();
        match child {
            Ok(mut child) => {
                let open = self.settings_open.clone();
                let hwnd = self.dispatcher.hwnd;
                std::thread::spawn(move || {
                    let saved = child.wait().is_ok_and(|status| status.success());
                    open.store(false, Ordering::Release);
                    if saved {
                        post(hwnd, WM_APP_SETTINGS_SAVED, 0);
                    }
                });
            }
            Err(e) => {
                self.settings_open.store(false, Ordering::Release);
                log!("Failed to open settings: {e}");
            }
        }
    }

    fn open_config(&self) {
        let path = self.dir.join("config.ini");
        if !path.is_file() {
            return;
        }
        if let Err(e) = std::process::Command::new("notepad.exe").arg(&path).spawn() {
            log!("Failed to open config: {e}");
        }
    }
}
