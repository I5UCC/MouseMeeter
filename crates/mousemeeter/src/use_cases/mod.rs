//! What Mousemeeter does, one file per feature.
//!
//! Voicemeeter features run on a `MixerSession` and only reach Voicemeeter through the `Mixer`
//! trait, so they can be tested with a fake. The other features are thin wrappers over
//! `crate::platform`.

pub mod media;
pub mod profiles;
pub mod tuning;
pub mod volume;
pub mod window;

use std::path::{Path, PathBuf};

use crate::action::MixerAction;
use volume::StripCache;

/// What the use cases need from the audio mixer. Implemented for Voicemeeter in
/// `crate::voicemeeter`.
pub trait Mixer {
    /// Waits until changes made inside the mixer (e.g. by loading a profile) can be read back.
    fn wait_until_settled(&mut self);
    fn gain(&mut self, strip: i32) -> Option<f32>;
    fn set_gain(&mut self, strip: i32, gain: f32) -> Result<(), String>;
    fn is_muted(&mut self, strip: i32) -> Option<bool>;
    fn set_muted(&mut self, strip: i32, muted: bool) -> Result<(), String>;
    fn load_profile(&mut self, path: &Path) -> Result<(), String>;
    fn restart_engine(&mut self) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixerSettings {
    /// Strips whose values are read back after loading a profile.
    pub strips: [i32; 3],
    /// dB per wheel step.
    pub volume_step: f32,
    pub profiles: ProfileFiles,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProfileFiles {
    pub dir: PathBuf,
    pub default: String,
    pub profile1: String,
    pub profile2: String,
}

/// The Voicemeeter features and the state they share. Lives on the Voicemeeter thread.
pub struct MixerSession<M> {
    mixer: M,
    settings: MixerSettings,
    strips: StripCache,
    current_profile: String,
}

impl<M: Mixer> MixerSession<M> {
    pub fn new(mixer: M, settings: MixerSettings) -> Self {
        MixerSession {
            current_profile: settings.profiles.default.clone(),
            mixer,
            settings,
            strips: StripCache::default(),
        }
    }

    /// Loads the default profile, or reads the current strip values from the mixer.
    pub fn start(&mut self, reset_to_default: bool) {
        if reset_to_default {
            self.load_profile(self.settings.profiles.default.clone());
        } else {
            self.sync_strips();
        }
    }

    pub fn run(&mut self, action: MixerAction) {
        match action {
            MixerAction::AdjustVolume { strip, up } => self.adjust_volume(strip, up),
            MixerAction::ToggleMute { strip } => self.toggle_mute(strip),
            MixerAction::SwitchProfile(profile) => self.switch_profile(profile),
            MixerAction::RestartEngine => self.restart_engine(),
        }
    }

    pub fn reconfigure(&mut self, settings: MixerSettings) {
        self.settings = settings;
    }

    pub fn into_mixer(self) -> M {
        self.mixer
    }
}

#[cfg(test)]
pub(crate) mod fake {
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    use super::{Mixer, MixerSettings, ProfileFiles};

    /// In-memory mixer. Writes land in `pending` and only become readable after
    /// `wait_until_settled`, like Voicemeeter applying changes asynchronously.
    #[derive(Debug, Default)]
    pub struct FakeMixer {
        pub gains: HashMap<i32, f32>,
        pub mutes: HashMap<i32, bool>,
        pending_gains: HashMap<i32, f32>,
        pending_mutes: HashMap<i32, bool>,
        /// Profile contents by file name: (strip, gain, muted).
        pub profiles: HashMap<String, Vec<(i32, f32, bool)>>,
        pub loaded: Vec<PathBuf>,
        pub restarts: usize,
        pub fail_writes: bool,
    }

    impl FakeMixer {
        /// Values as Voicemeeter will report them once settled.
        pub fn settled_gain(&self, strip: i32) -> Option<f32> {
            self.pending_gains
                .get(&strip)
                .or(self.gains.get(&strip))
                .copied()
        }

        pub fn settled_mute(&self, strip: i32) -> Option<bool> {
            self.pending_mutes
                .get(&strip)
                .or(self.mutes.get(&strip))
                .copied()
        }

        fn write(&self) -> Result<(), String> {
            if self.fail_writes {
                Err("write failed".into())
            } else {
                Ok(())
            }
        }
    }

    impl Mixer for FakeMixer {
        fn wait_until_settled(&mut self) {
            self.gains.extend(self.pending_gains.drain());
            self.mutes.extend(self.pending_mutes.drain());
        }

        fn gain(&mut self, strip: i32) -> Option<f32> {
            self.gains.get(&strip).copied()
        }

        fn set_gain(&mut self, strip: i32, gain: f32) -> Result<(), String> {
            self.write()?;
            self.pending_gains.insert(strip, gain);
            Ok(())
        }

        fn is_muted(&mut self, strip: i32) -> Option<bool> {
            self.mutes.get(&strip).copied()
        }

        fn set_muted(&mut self, strip: i32, muted: bool) -> Result<(), String> {
            self.write()?;
            self.pending_mutes.insert(strip, muted);
            Ok(())
        }

        fn load_profile(&mut self, path: &Path) -> Result<(), String> {
            self.write()?;
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            let strips = self.profiles.get(&name).ok_or("file not found")?.clone();
            for (strip, gain, muted) in strips {
                self.pending_gains.insert(strip, gain);
                self.pending_mutes.insert(strip, muted);
            }
            self.loaded.push(path.to_path_buf());
            Ok(())
        }

        fn restart_engine(&mut self) -> Result<(), String> {
            self.write()?;
            self.restarts += 1;
            Ok(())
        }
    }

    pub fn settings() -> MixerSettings {
        MixerSettings {
            strips: [5, 6, 7],
            volume_step: 0.5,
            profiles: ProfileFiles {
                dir: PathBuf::from("profiles"),
                default: "default.xml".into(),
                profile1: "profile1.xml".into(),
                profile2: "profile2.xml".into(),
            },
        }
    }
}
