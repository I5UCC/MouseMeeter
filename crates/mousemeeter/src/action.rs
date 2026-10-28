//! Everything Mousemeeter can do, independent of the input that triggered it.
//!
//! Gestures (`crate::gestures`) turn mouse and keyboard input into an `Action`; the app routes it
//! to the matching use case (`crate::use_cases`).

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    /// Talks to Voicemeeter, so it runs on the Voicemeeter thread where slow calls can't stall
    /// the input hooks.
    Mixer(MixerAction),
    /// Handled by the app itself on the main thread.
    App(AppAction),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MixerAction {
    AdjustVolume { strip: i32, up: bool },
    ToggleMute { strip: i32 },
    SwitchProfile(Profile),
    RestartEngine,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    MediaKey(MediaKey),
    ToggleEnabled,
    KillForegroundApp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    Default,
    /// Switches back to the default profile if it is already loaded.
    Profile1,
    /// Switches back to the default profile if it is already loaded.
    Profile2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKey {
    Previous,
    Next,
    PlayPause,
}

impl From<MixerAction> for Action {
    fn from(action: MixerAction) -> Self {
        Action::Mixer(action)
    }
}

impl From<AppAction> for Action {
    fn from(action: AppAction) -> Self {
        Action::App(action)
    }
}
