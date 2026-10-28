//! Keyboard hotkeys: F24 (profiles / mute), F15 (enable/disable), Ctrl+Alt+F4 (kill app) and
//! Ctrl+Shift+R (restart engine / default profile).
//!
//! F24 and Ctrl+Shift+R have a different action on a double press, so their single-press action
//! is deferred until `DOUBLE_PRESS` passes without a second press. The caller owns the timer.

use std::collections::HashSet;
use std::time::Duration;

use crate::action::{Action, AppAction, MixerAction, Profile};
use crate::config::Outputs;

pub const DOUBLE_PRESS: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hotkey {
    /// F24: mute while a side button is held, otherwise profile 1 (single) / profile 2 (double).
    Profile = 1,
    /// F15
    ToggleEnabled = 2,
    /// Ctrl+Alt+F4
    KillForegroundApp = 3,
    /// Ctrl+Shift+R: restart the audio engine (single) / load the default profile (double).
    Restart = 4,
}

impl Hotkey {
    const ALL: [Hotkey; 4] = [
        Hotkey::Profile,
        Hotkey::ToggleEnabled,
        Hotkey::KillForegroundApp,
        Hotkey::Restart,
    ];

    /// Non-zero id, usable as a window message parameter or timer id.
    pub fn id(self) -> usize {
        self as usize
    }

    pub fn from_id(id: usize) -> Option<Hotkey> {
        Self::ALL.into_iter().find(|h| h.id() == id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Timer {
    Start(Hotkey),
    Cancel(Hotkey),
}

#[derive(Debug, Default, PartialEq)]
pub struct Response {
    pub action: Option<Action>,
    pub timer: Option<Timer>,
}

#[derive(Debug, Default)]
pub struct Hotkeys {
    /// Hotkeys pressed once whose double-press window is still open.
    pending: HashSet<Hotkey>,
}

impl Hotkeys {
    /// `held` are the side buttons currently held: (Backward, Forward).
    pub fn on_press(&mut self, hotkey: Hotkey, held: (bool, bool), outputs: &Outputs) -> Response {
        let now = |action: Action| Response {
            action: Some(action),
            timer: None,
        };
        match hotkey {
            Hotkey::ToggleEnabled => now(AppAction::ToggleEnabled.into()),
            Hotkey::KillForegroundApp => now(AppAction::KillForegroundApp.into()),
            Hotkey::Profile => match outputs.for_buttons(held.0, held.1) {
                Some(strip) => now(MixerAction::ToggleMute { strip }.into()),
                None => self.press(hotkey, MixerAction::SwitchProfile(Profile::Profile2)),
            },
            Hotkey::Restart => self.press(hotkey, MixerAction::SwitchProfile(Profile::Default)),
        }
    }

    fn press(&mut self, hotkey: Hotkey, on_double: MixerAction) -> Response {
        if self.pending.remove(&hotkey) {
            Response {
                action: Some(on_double.into()),
                timer: Some(Timer::Cancel(hotkey)),
            }
        } else {
            self.pending.insert(hotkey);
            Response {
                action: None,
                timer: Some(Timer::Start(hotkey)),
            }
        }
    }

    /// The double-press window closed without a second press: the single-press action.
    pub fn on_timeout(&mut self, hotkey: Hotkey) -> Option<Action> {
        if !self.pending.remove(&hotkey) {
            return None;
        }
        match hotkey {
            Hotkey::Profile => Some(MixerAction::SwitchProfile(Profile::Profile1).into()),
            Hotkey::Restart => Some(MixerAction::RestartEngine.into()),
            Hotkey::ToggleEnabled | Hotkey::KillForegroundApp => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OUTPUTS: Outputs = Outputs {
        output1: 5,
        output2: 6,
        output3: 7,
    };
    const NONE_HELD: (bool, bool) = (false, false);

    #[test]
    fn single_f24_loads_profile1_after_timeout() {
        let mut h = Hotkeys::default();
        let r = h.on_press(Hotkey::Profile, NONE_HELD, &OUTPUTS);
        assert_eq!(r.action, None);
        assert_eq!(r.timer, Some(Timer::Start(Hotkey::Profile)));
        assert_eq!(
            h.on_timeout(Hotkey::Profile),
            Some(MixerAction::SwitchProfile(Profile::Profile1).into())
        );
        assert_eq!(h.on_timeout(Hotkey::Profile), None);
    }

    #[test]
    fn double_f24_loads_profile2_and_cancels_timer() {
        let mut h = Hotkeys::default();
        h.on_press(Hotkey::Profile, NONE_HELD, &OUTPUTS);
        let r = h.on_press(Hotkey::Profile, NONE_HELD, &OUTPUTS);
        assert_eq!(
            r,
            Response {
                action: Some(MixerAction::SwitchProfile(Profile::Profile2).into()),
                timer: Some(Timer::Cancel(Hotkey::Profile)),
            }
        );
        // A late timer message does nothing.
        assert_eq!(h.on_timeout(Hotkey::Profile), None);
    }

    #[test]
    fn f24_with_side_button_mutes_immediately() {
        let mut h = Hotkeys::default();
        for (held, strip) in [((true, false), 5), ((false, true), 6), ((true, true), 7)] {
            let r = h.on_press(Hotkey::Profile, held, &OUTPUTS);
            assert_eq!(
                r,
                Response {
                    action: Some(MixerAction::ToggleMute { strip }.into()),
                    timer: None,
                }
            );
        }
    }

    #[test]
    fn restart_single_and_double() {
        let mut h = Hotkeys::default();
        h.on_press(Hotkey::Restart, NONE_HELD, &OUTPUTS);
        assert_eq!(
            h.on_timeout(Hotkey::Restart),
            Some(MixerAction::RestartEngine.into())
        );

        h.on_press(Hotkey::Restart, NONE_HELD, &OUTPUTS);
        let r = h.on_press(Hotkey::Restart, NONE_HELD, &OUTPUTS);
        assert_eq!(
            r.action,
            Some(MixerAction::SwitchProfile(Profile::Default).into())
        );
    }

    #[test]
    fn immediate_hotkeys() {
        let mut h = Hotkeys::default();
        let r = h.on_press(Hotkey::ToggleEnabled, (true, true), &OUTPUTS);
        assert_eq!(r.action, Some(AppAction::ToggleEnabled.into()));
        let r = h.on_press(Hotkey::KillForegroundApp, NONE_HELD, &OUTPUTS);
        assert_eq!(r.action, Some(AppAction::KillForegroundApp.into()));
    }

    #[test]
    fn id_round_trip() {
        for hotkey in Hotkey::ALL {
            assert_ne!(hotkey.id(), 0);
            assert_eq!(Hotkey::from_id(hotkey.id()), Some(hotkey));
        }
        assert_eq!(Hotkey::from_id(0), None);
    }
}
