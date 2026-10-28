//! Side-button gestures: Backward/Forward + wheel for volume, Forward + click for media keys.

use crate::action::{Action, AppAction, MediaKey, MixerAction};
use crate::config::Outputs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XButton {
    /// Backward
    X1,
    /// Forward
    X2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEvent {
    XDown(XButton),
    XUp(XButton),
    Wheel { up: bool },
    LeftDown,
    RightDown,
    MiddleDown,
    RightUp,
}

/// Whether the hook should swallow the event, and what to do about it.
pub type Response = (bool, Option<Action>);

#[derive(Debug, Default)]
pub struct MouseGesture {
    x1: bool,
    x2: bool,
    /// An action fired during the current gesture, so the side-button release is swallowed too.
    action_fired: bool,
    /// The gesture started over a window from `DeactivateOnWindow` and passes through untouched.
    bypassed: bool,
}

impl MouseGesture {
    /// Side buttons held as part of an active gesture: (Backward, Forward).
    pub fn held(&self) -> (bool, bool) {
        if self.bypassed {
            (false, false)
        } else {
            (self.x1, self.x2)
        }
    }

    fn set(&mut self, button: XButton, pressed: bool) {
        match button {
            XButton::X1 => self.x1 = pressed,
            XButton::X2 => self.x2 = pressed,
        }
    }

    /// `is_bypassed_window` is only consulted when a gesture starts.
    pub fn on_event(
        &mut self,
        event: MouseEvent,
        outputs: &Outputs,
        is_bypassed_window: impl FnOnce() -> bool,
    ) -> Response {
        let any_held = self.x1 || self.x2;

        if let MouseEvent::XDown(_) = event
            && !any_held
        {
            self.bypassed = is_bypassed_window();
            self.action_fired = false;
        }

        if self.bypassed {
            match event {
                MouseEvent::XDown(b) => self.set(b, true),
                MouseEvent::XUp(b) => {
                    self.set(b, false);
                    self.bypassed = self.x1 || self.x2;
                }
                _ => {}
            }
            return (false, None);
        }

        match event {
            MouseEvent::XDown(b) => {
                self.set(b, true);
                (true, None)
            }
            MouseEvent::XUp(b) => {
                // Without an action the release goes through, so Backward/Forward still work.
                let block = self.action_fired;
                self.set(b, false);
                if !self.x1 && !self.x2 {
                    self.action_fired = false;
                }
                (block, None)
            }
            MouseEvent::Wheel { up } if any_held => {
                self.action_fired = true;
                let action = outputs
                    .for_buttons(self.x1, self.x2)
                    .map(|strip| MixerAction::AdjustVolume { strip, up }.into());
                (true, action)
            }
            MouseEvent::LeftDown | MouseEvent::RightDown | MouseEvent::MiddleDown if any_held => {
                // Clicks are swallowed while a side button is held; media keys need Forward alone.
                if !(self.x2 && !self.x1) {
                    return (true, None);
                }
                let key = match event {
                    MouseEvent::LeftDown => MediaKey::Previous,
                    MouseEvent::RightDown => MediaKey::Next,
                    _ => MediaKey::PlayPause,
                };
                self.action_fired = true;
                (true, Some(AppAction::MediaKey(key).into()))
            }
            // Stops the context menu from opening after a swallowed right click.
            MouseEvent::RightUp if any_held => (true, None),
            _ => (false, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use MouseEvent::*;
    use XButton::*;

    const OUTPUTS: Outputs = Outputs {
        output1: 5,
        output2: 6,
        output3: 7,
    };

    fn feed(gesture: &mut MouseGesture, event: MouseEvent) -> Response {
        gesture.on_event(event, &OUTPUTS, || false)
    }

    fn volume(strip: i32, up: bool) -> Option<Action> {
        Some(MixerAction::AdjustVolume { strip, up }.into())
    }

    fn media(key: MediaKey) -> Option<Action> {
        Some(AppAction::MediaKey(key).into())
    }

    #[test]
    fn side_button_without_action_releases_through() {
        let mut g = MouseGesture::default();
        assert_eq!(feed(&mut g, XDown(X1)), (true, None));
        assert_eq!(feed(&mut g, XUp(X1)), (false, None));
    }

    #[test]
    fn wheel_volume_per_button_combination() {
        let mut g = MouseGesture::default();
        feed(&mut g, XDown(X1));
        assert_eq!(feed(&mut g, Wheel { up: true }), (true, volume(5, true)));
        feed(&mut g, XDown(X2));
        assert_eq!(feed(&mut g, Wheel { up: false }), (true, volume(7, false)));
        feed(&mut g, XUp(X1));
        assert_eq!(feed(&mut g, Wheel { up: true }), (true, volume(6, true)));
    }

    #[test]
    fn release_after_action_is_swallowed_for_both_buttons() {
        let mut g = MouseGesture::default();
        feed(&mut g, XDown(X1));
        feed(&mut g, XDown(X2));
        feed(&mut g, Wheel { up: true });
        assert_eq!(feed(&mut g, XUp(X1)), (true, None));
        assert_eq!(feed(&mut g, XUp(X2)), (true, None));
        // The next gesture starts fresh.
        feed(&mut g, XDown(X2));
        assert_eq!(feed(&mut g, XUp(X2)), (false, None));
    }

    #[test]
    fn wheel_passes_through_without_side_button() {
        let mut g = MouseGesture::default();
        assert_eq!(feed(&mut g, Wheel { up: true }), (false, None));
        assert_eq!(feed(&mut g, LeftDown), (false, None));
        assert_eq!(feed(&mut g, RightUp), (false, None));
    }

    #[test]
    fn media_keys_need_forward_alone() {
        let mut g = MouseGesture::default();
        feed(&mut g, XDown(X2));
        assert_eq!(feed(&mut g, LeftDown), (true, media(MediaKey::Previous)));
        assert_eq!(feed(&mut g, RightDown), (true, media(MediaKey::Next)));
        assert_eq!(feed(&mut g, RightUp), (true, None));
        assert_eq!(feed(&mut g, MiddleDown), (true, media(MediaKey::PlayPause)));
        assert_eq!(feed(&mut g, XUp(X2)), (true, None));
    }

    #[test]
    fn clicks_with_backward_are_swallowed_without_action() {
        let mut g = MouseGesture::default();
        feed(&mut g, XDown(X1));
        assert_eq!(feed(&mut g, LeftDown), (true, None));
        assert_eq!(feed(&mut g, XUp(X1)), (false, None));
    }

    #[test]
    fn bypassed_window_passes_whole_gesture_through() {
        let mut g = MouseGesture::default();
        assert_eq!(g.on_event(XDown(X1), &OUTPUTS, || true), (false, None));
        assert_eq!(g.held(), (false, false));
        // Only checked when the gesture starts.
        assert_eq!(g.on_event(XDown(X2), &OUTPUTS, || panic!()), (false, None));
        assert_eq!(feed(&mut g, Wheel { up: true }), (false, None));
        assert_eq!(feed(&mut g, XUp(X1)), (false, None));
        assert_eq!(feed(&mut g, XUp(X2)), (false, None));
        assert_eq!(feed(&mut g, XDown(X1)), (true, None));
    }
}
