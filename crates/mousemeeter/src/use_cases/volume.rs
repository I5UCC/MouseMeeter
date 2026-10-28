//! Changing a strip's volume (side button + wheel) and mute (side button + F24).

use std::collections::HashMap;

use super::{Mixer, MixerSession};
use crate::log;

const MIN_GAIN: f32 = -60.0;
const MAX_GAIN: f32 = 12.0;

/// Last known strip values. Voicemeeter applies writes asynchronously, so reading a value back
/// right after writing it can return the old one; quick wheel steps build on this cache instead.
#[derive(Debug, Default)]
pub struct StripCache {
    gains: HashMap<i32, f32>,
    mutes: HashMap<i32, bool>,
}

impl<M: Mixer> MixerSession<M> {
    pub(super) fn adjust_volume(&mut self, strip: i32, up: bool) {
        let step = if up {
            self.settings.volume_step
        } else {
            -self.settings.volume_step
        };
        let gain = (self.gain(strip) + step).clamp(MIN_GAIN, MAX_GAIN);
        match self.mixer.set_gain(strip, gain) {
            Ok(()) => {
                self.strips.gains.insert(strip, gain);
                log!("Strip {strip} volume set to: {gain:.1} dB");
            }
            Err(e) => log!("Error setting volume of strip {strip}: {e}"),
        }
    }

    pub(super) fn toggle_mute(&mut self, strip: i32) {
        let muted = !self.is_muted(strip);
        match self.mixer.set_muted(strip, muted) {
            Ok(()) => {
                self.strips.mutes.insert(strip, muted);
                log!("Strip {strip} mute: {}", on_off(muted));
            }
            Err(e) => log!("Error toggling mute of strip {strip}: {e}"),
        }
    }

    /// Forgets the cached values and reads the configured strips from the mixer once it settled.
    pub(super) fn sync_strips(&mut self) {
        log!("Syncing Voicemeeter values...");
        self.strips = StripCache::default();
        self.mixer.wait_until_settled();
        for strip in self.settings.strips {
            self.read_strip(strip);
        }
    }

    fn read_strip(&mut self, strip: i32) {
        let gain = self.mixer.gain(strip);
        let muted = self.mixer.is_muted(strip);
        if let Some(gain) = gain {
            self.strips.gains.insert(strip, gain);
        }
        if let Some(muted) = muted {
            self.strips.mutes.insert(strip, muted);
        }
        match (gain, muted) {
            (Some(gain), Some(muted)) => log!(
                "Synced strip {strip}: Gain={gain:.1} dB, Mute={}",
                on_off(muted)
            ),
            _ => log!("Could not read strip {strip} from Voicemeeter"),
        }
    }

    fn gain(&mut self, strip: i32) -> f32 {
        if !self.strips.gains.contains_key(&strip) {
            self.read_strip(strip);
        }
        self.strips.gains.get(&strip).copied().unwrap_or(0.0)
    }

    fn is_muted(&mut self, strip: i32) -> bool {
        if !self.strips.mutes.contains_key(&strip) {
            self.read_strip(strip);
        }
        self.strips.mutes.get(&strip).copied().unwrap_or(false)
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "ON" } else { "OFF" }
}

#[cfg(test)]
mod tests {
    use super::super::MixerSettings;
    use super::super::fake::{FakeMixer, settings};
    use super::*;
    use crate::action::MixerAction;

    fn session(mixer: FakeMixer) -> MixerSession<FakeMixer> {
        MixerSession::new(mixer, settings())
    }

    fn volume(strip: i32, up: bool) -> MixerAction {
        MixerAction::AdjustVolume { strip, up }
    }

    #[test]
    fn steps_build_on_each_other_before_the_mixer_settles() {
        let mut mixer = FakeMixer::default();
        mixer.gains.insert(5, -10.0);
        let mut s = session(mixer);

        s.run(volume(5, true));
        s.run(volume(5, true));
        s.run(volume(5, true));
        s.run(volume(5, false));

        assert_eq!(s.into_mixer().settled_gain(5), Some(-9.0));
    }

    #[test]
    fn gain_is_clamped() {
        let mut mixer = FakeMixer::default();
        mixer.gains.insert(5, 11.8);
        mixer.gains.insert(6, -59.9);
        let mut s = session(mixer);

        s.run(volume(5, true));
        s.run(volume(6, false));

        let mixer = s.into_mixer();
        assert_eq!(mixer.settled_gain(5), Some(MAX_GAIN));
        assert_eq!(mixer.settled_gain(6), Some(MIN_GAIN));
    }

    #[test]
    fn unknown_strip_starts_at_zero() {
        let mut s = session(FakeMixer::default());
        s.run(volume(9, false));
        assert_eq!(s.into_mixer().settled_gain(9), Some(-0.5));
    }

    #[test]
    fn mute_toggles_from_the_mixer_value() {
        let mut mixer = FakeMixer::default();
        mixer.mutes.insert(6, true);
        let mut s = session(mixer);

        s.run(MixerAction::ToggleMute { strip: 6 });
        assert_eq!(s.mixer.settled_mute(6), Some(false));
        s.run(MixerAction::ToggleMute { strip: 6 });
        assert_eq!(s.into_mixer().settled_mute(6), Some(true));
    }

    #[test]
    fn failed_write_keeps_the_cached_value() {
        let mut mixer = FakeMixer::default();
        mixer.gains.insert(5, 0.0);
        let mut s = session(mixer);

        s.mixer.fail_writes = true;
        s.run(volume(5, true));
        s.mixer.fail_writes = false;
        s.run(volume(5, true));

        assert_eq!(s.into_mixer().settled_gain(5), Some(0.5));
    }

    #[test]
    fn step_size_follows_reconfigure() {
        let mut mixer = FakeMixer::default();
        mixer.gains.insert(5, 0.0);
        let mut s = session(mixer);

        s.reconfigure(MixerSettings {
            volume_step: 2.0,
            ..settings()
        });
        s.run(volume(5, true));

        assert_eq!(s.into_mixer().settled_gain(5), Some(2.0));
    }
}
