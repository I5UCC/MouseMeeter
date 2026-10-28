//! The Voicemeeter thread: waits for Voicemeeter, connects, and runs `MixerAction`s on a
//! `MixerSession`. Voicemeeter calls can block for seconds, so they never run on the thread that
//! owns the input hooks.

mod remote;

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::action::MixerAction;
use crate::log;
use crate::platform::process;
use crate::use_cases::{MixerSession, MixerSettings, tuning};
use remote::Voicemeeter;

const PROCESS_NAMES: [&str; 3] = ["voicemeeter8", "voicemeeter8x64", "voicemeeter"];
const STARTUP_TIMEOUT: Duration = Duration::from_secs(5 * 60);

pub enum Request {
    Run(MixerAction),
    Reconfigure(MixerSettings),
    Shutdown,
}

pub struct Startup {
    pub dll_dir: PathBuf,
    pub raise_priority: bool,
    pub crackling_fix: bool,
    pub reset_to_default: bool,
    pub settings: MixerSettings,
}

/// Starts the thread. `on_ready` is called with `true` once requests are handled, or `false` if
/// startup failed. It isn't called if `Shutdown` arrives during startup.
pub fn spawn(
    startup: Startup,
    on_ready: impl FnOnce(bool) + Send + 'static,
) -> (Sender<Request>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel();
    let handle = thread::Builder::new()
        .name("voicemeeter".into())
        .spawn(move || run(startup, rx, on_ready))
        .expect("failed to spawn Voicemeeter thread");
    (tx, handle)
}

fn run(startup: Startup, rx: Receiver<Request>, on_ready: impl FnOnce(bool)) {
    let mut settings = startup.settings;

    let pids = match wait_for_voicemeeter(&rx, &mut settings) {
        Wait::Found(pids) => pids,
        Wait::TimedOut => {
            log!("Voicemeeter did not start within the expected time.");
            return on_ready(false);
        }
        Wait::Shutdown => return,
    };

    let voicemeeter = match Voicemeeter::connect(&startup.dll_dir) {
        Ok(voicemeeter) => voicemeeter,
        Err(e) => {
            log!("Failed to load the Voicemeeter Remote API: {e}");
            return on_ready(false);
        }
    };
    tuning::apply(&pids, startup.raise_priority, startup.crackling_fix);

    let mut session = MixerSession::new(voicemeeter, settings);
    session.start(startup.reset_to_default);
    on_ready(true);

    while let Ok(request) = rx.recv() {
        match request {
            Request::Run(action) => session.run(action),
            Request::Reconfigure(settings) => session.reconfigure(settings),
            Request::Shutdown => break,
        }
    }
    session.into_mixer().disconnect();
}

enum Wait {
    Found(Vec<u32>),
    TimedOut,
    Shutdown,
}

fn wait_for_voicemeeter(rx: &Receiver<Request>, settings: &mut MixerSettings) -> Wait {
    log!("Waiting for Voicemeeter to start...");
    let deadline = Instant::now() + STARTUP_TIMEOUT;

    loop {
        let pids = process::find_pids(&PROCESS_NAMES);
        if !pids.is_empty() {
            log!("Found Voicemeeter process (PID: {pids:?})");
            // Give Voicemeeter a moment to finish starting.
            thread::sleep(Duration::from_secs(1));
            return Wait::Found(pids);
        }
        if Instant::now() >= deadline {
            return Wait::TimedOut;
        }

        // Sleep for a second, but stay responsive to requests sent from the tray.
        let until = Instant::now() + Duration::from_secs(1);
        loop {
            match rx.recv_timeout(until.saturating_duration_since(Instant::now())) {
                Ok(Request::Shutdown) | Err(RecvTimeoutError::Disconnected) => {
                    return Wait::Shutdown;
                }
                Ok(Request::Reconfigure(new)) => *settings = new,
                Ok(Request::Run(_)) => {}
                Err(RecvTimeoutError::Timeout) => break,
            }
        }
    }
}
